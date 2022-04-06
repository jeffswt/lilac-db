use crate::record::ByteStream;

use super::HashStrategy;

/// Common implementation for bloom filters on different hash sizes and hash
/// functions.
///
/// Parameters hold that M be the actual number of bloom filter bits, and K
/// be the number of slots hashed per entry.
///
/// M is required to be a power of 2 while being larger than 8.
struct BloomFilterImpl<Hasher, const M: usize, const ML: usize, const K: usize>
where
    [(); K]: Sized,
    [(); M >> 3]: Sized,
    Hasher: HashStrategy<M, ML, K>,
{
    data: Box<[u8; M >> 3]>,
}

/// The implementation is guaranteed to be endian-safe.
impl<Hasher, const M: usize, const ML: usize, const K: usize> BloomFilterImpl<Hasher, M, ML, K>
where
    [(); K]: Sized,
    [(); M >> 3]: Sized,
    Hasher: HashStrategy<M, ML, K>,
{
    /// Creates new empty bloom filter.
    pub fn new() -> Self {
        Self {
            data: Box::from([0u8; M >> 3]),
        }
    }

    /// Inserts key into bloom filter.
    pub fn insert(&mut self, message: &ByteStream) -> () {
        let positions = Self::hash(&message);
        // a simd optimization is not necessary here since memory fetch (we can
        // almost assure that `self.data[...]` misses) is far slower than
        // unwrapping that loop
        for position in positions {
            let mask = 1u8 << (position & 0x07);
            self.data.as_mut()[(position >> 3) as usize] |= mask;
        }
    }

    /// Queries if a key exists in this bloom filter.
    ///
    /// A `false` result indicates a definite 'not exist', while a `true` might
    /// inflict a false positive.
    pub fn query(&self, message: &ByteStream) -> bool {
        let positions = Self::hash(&message);
        // simd not required as like above
        for position in positions {
            let mask = 1u8 << (position & 0x07);
            if self.data.as_ref()[(position >> 3) as usize] & mask == 0u8 {
                return false;
            }
        }
        return true;
    }

    /// Shortcut for performing a hash action.
    ///
    /// Usage: `Self::hash(...)`.
    #[inline]
    fn hash(message: &ByteStream) -> [u32; K] {
        Hasher::hash(&message)
    }
}

#[cfg(test)]
mod tests {
    use super::BloomFilterImpl;
    use crate::bloom::strategies::SipHash;
    use crate::record::ByteStream;

    #[test]
    fn no_false_negatives() {
        // uses the default siphash for testing
        let mut bf = BloomFilterImpl::<SipHash<{1 << 20}, 20, 2>, {1 << 20}, 20, 2>::new();
        for i in 0..98765 {
            let s = format!("test-string-{i}");
            let s = ByteStream::from_slice(s.as_bytes());
            bf.insert(&s);
        }
        // test that there are no false negatives
        for i in 0..98765 {
            let s = format!("test-string-{i}");
            let s = ByteStream::from_slice(s.as_bytes());
            let result = bf.query(&s);
            assert_ne!(result, false);
        }
    }
}
