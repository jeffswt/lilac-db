use crate::bloom::HashStrategy;
use std::intrinsics::rotate_right;
use std::num::Wrapping;
use std::ops::Shr;
use std::simd::Simd;

/// sfHash64 produces hashes of up to 64 bits.
///
/// It is required that ML * K be less than 64 bits.
pub struct SfHash64<const ML: usize, const K: usize>
where
    [(); K]: Sized, {}

impl<const ML: usize, const K: usize> HashStrategy<ML, K> for SfHash64<ML, K> {
    #[inline]
    fn hash(message: &[u8]) -> [u32; K] {
        assert!(ML * K <= 64, "sufficient bits for hashing");

        let mut finished = unsafe { sfhash64(message, message.len() as u64) };

        let mut result = [0u32; K];
        let mask: u64 = (1 << ML) - 1;
        for i in 0..K {
            result[i] = (finished & mask) as u32;
            finished >>= ML;
        }
        result
    }
}

// Constants for hashing
const MAGIC_SEED: u64 = 0xbc4a78eb0e083fb5_u64;
const MAGIC_SHIFT_1: u64 = 0xc2d4f379500c363f_u64;
const MAGIC_SHIFT_2: u64 = 0xa696a85adffcf585_u64;
const MAGIC_SHIFT_3: u64 = 0xfcb5791894673fd3_u64;
const MAGIC_SHIFT_4: u64 = 0xb828e5548ad84c69_u64;
const MAGIC_OFFSET_1: u64 = 0xff43a9d0c1c914cd_u64;
const MAGIC_OFFSET_2: u64 = 0xf049ed58f79e6153_u64;
const MAGIC_MIX: u64 = 0xed27a0e9f72a6d47_u64;

/// Unchecked in-place multiply.
macro_rules! wrapped_mul_inpl {
    ($dest:ident, $src:ident) => {
        $dest = (Wrapping($dest) * Wrapping($src)).0;
    };
}

#[inline]
fn mix(mut v: u64) -> u64 {
    v ^= v >> 23;
    wrapped_mul_inpl!(v, MAGIC_MIX);
    v ^ (v >> 47)
}

/// Expands to 1 of the 7 switch-cases in the sfhash terminator.
macro_rules! match_byte {
    ($dest:ident, $src:ident, $cmp:ident, $offset:literal, $shift:literal) => {
        if $cmp > $offset {
            $dest ^= (*$src.offset($offset) as u64) << $shift;
        }
    };
}

/// Declares a sfhash terminator (7 bytes max).
macro_rules! match_bytes {
    ($dest:ident, $src:ident, $cmp:ident) => {
        match_byte!($dest, $src, $cmp, 6, 48);
        match_byte!($dest, $src, $cmp, 5, 40);
        match_byte!($dest, $src, $cmp, 4, 32);
        match_byte!($dest, $src, $cmp, 3, 24);
        match_byte!($dest, $src, $cmp, 2, 16);
        match_byte!($dest, $src, $cmp, 1, 8);
        if $cmp > 0 {
            $dest ^= *$src as u64;
        }
    };
}

/// An sfHash64 implementation without seed intervention.
///
/// This algorithm is not portable between machines of different endiannesses.
#[inline(always)]
unsafe fn sfhash64(buffer: &[u8], len: u64) -> u64 {
    let mut ptr = buffer.as_ptr() as *const u64;
    let end2 = ptr.offset((len as isize) >> 3); // 64-bit alignment
    let mut h3: u64 = MAGIC_SEED ^ (Wrapping(len) * Wrapping(MAGIC_SHIFT_1)).0;
    let mut v: u64;

    // small key hashes (< 256 bits) should be dealt with with priority
    if len < 32 {
        while ptr != end2 {
            v = *ptr;
            h3 ^= mix(v);
            wrapped_mul_inpl!(h3, MAGIC_SHIFT_1);
            ptr = ptr.offset(1);
        }

        let ptr2 = ptr as *const u8;
        let len = len & 7;
        v = 0;
        match_bytes!(v, ptr2, len);

        h3 ^= mix(v);
        wrapped_mul_inpl!(h3, MAGIC_SHIFT_4);
        return mix(h3);
    }

    // batch hash 32 bytes at a time
    let end1 = ptr.offset(((len as isize) >> 5) << 2); // 256-bit alignment
    let mut h: u64;
    if ptr != end1 {
        let mut hv = Simd::<u64, 4>::from([
            (Wrapping(h3) + Wrapping(MAGIC_OFFSET_1) + Wrapping(MAGIC_OFFSET_2)).0,
            (Wrapping(h3) + Wrapping(MAGIC_OFFSET_1)).0,
            h3,
            (Wrapping(h3) - Wrapping(MAGIC_OFFSET_2)).0,
        ]);
        let vec_shr_23 = Simd::from([23, 23, 23, 23]);
        let vec_shr_47 = Simd::from([47, 47, 47, 47]);
        let vec_mul = Simd::from([MAGIC_SHIFT_1, MAGIC_SHIFT_2, MAGIC_SHIFT_3, MAGIC_SHIFT_4]);

        while ptr != end1 {
            let mut vv = Simd::from([*ptr, *ptr.offset(1), *ptr.offset(2), *ptr.offset(3)]);
            vv ^= vv.shr(vec_shr_23);
            hv ^= vv ^ vv.shr(vec_shr_47);
            hv *= vec_mul;
            ptr = ptr.offset(4);
        }
        let ha = hv.as_array();
        h = rotate_right(ha[0], 1)
            ^ rotate_right(ha[1], 3)
            ^ rotate_right(ha[2], 6)
            ^ rotate_right(ha[3], 11);
    } else {
        h = h3;
    }

    // batch hash 8 bytes at a time, up to 24 bytes
    while ptr != end2 {
        v = *ptr;
        h ^= mix(v);
        wrapped_mul_inpl!(h, MAGIC_SHIFT_1);
        ptr = ptr.offset(1);
    }

    // hash the last 7 bytes
    let ptr2 = ptr as *const u8;
    let len = len & 7;
    v = 0;
    match_bytes!(v, ptr2, len);

    // mix and leave
    h ^= mix(v);
    wrapped_mul_inpl!(h, MAGIC_SHIFT_4);
    return mix(h);
}

#[cfg(test)]
mod tests {
    use super::sfhash64;

    fn sfhash64_signature() -> u32 {
        // hash keys of the form {}, {0}, {0,1}, {0,1,2}, ..., {0,1,2,...,254}
        let mut digest_bytes = Vec::<u8>::new();
        let mut message = Vec::<u8>::new();
        for n in 0..=255 {
            let digest = unsafe { sfhash64(&message, n) };
            message.push(n as u8);
            // push data in little-endian
            digest_bytes.push(((digest >> 0) & 0xff) as u8);
            digest_bytes.push(((digest >> 8) & 0xff) as u8);
            digest_bytes.push(((digest >> 16) & 0xff) as u8);
            digest_bytes.push(((digest >> 24) & 0xff) as u8);
            digest_bytes.push(((digest >> 32) & 0xff) as u8);
            digest_bytes.push(((digest >> 40) & 0xff) as u8);
            digest_bytes.push(((digest >> 48) & 0xff) as u8);
            digest_bytes.push(((digest >> 56) & 0xff) as u8);
        }
        // hash the generated 2048 bytes into another digest
        let digest = unsafe { sfhash64(&digest_bytes, 2048) };
        digest as u32
    }

    #[test]
    fn sfhash64_signature_valid() {
        let sig = sfhash64_signature();
        assert_eq!(sig, 0xf55ec779);
    }
}
