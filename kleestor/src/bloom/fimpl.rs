use crate::bloom::HashStrategy;
use crate::record::ByteStream;
use std::io::Write;

/// Common implementation for bloom filters on different hash sizes and hash
/// functions.
///
/// Parameters hold that 2^ML be the actual number of bloom filter bits, and K
/// be the number of slots hashed per entry.
pub struct BloomFilterImpl<Hasher, const ML: usize, const K: usize>
where
    [(); K]: Sized,
    [(); 1 << (ML - 3)]: Sized,
    Hasher: HashStrategy<ML, K>,
{
    data: Box<[u8; 1 << (ML - 3)]>,
}

/// The implementation is guaranteed to be endian-safe.
impl<Hasher, const ML: usize, const K: usize> BloomFilterImpl<Hasher, ML, K>
where
    [(); K]: Sized,
    [(); 1 << (ML - 3)]: Sized,
    Hasher: HashStrategy<ML, K>,
{
    /// Creates new empty bloom filter.
    pub fn new() -> Self {
        Self {
            data: Box::from([0u8; 1 << (ML - 3)]),
        }
    }

    /// Creates bloom filter from memory slice.
    pub fn from_slice(region: &[u8]) -> Self {
        let region: [u8; 1 << (ML - 3)] = region.try_into().expect("incorrect slice length");
        Self {
            data: Box::from(region),
        }
    }

    /// Inserts key into bloom filter.
    pub fn insert(&mut self, message: &[u8]) -> () {
        let positions = Self::hash(message);
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
    pub fn query(&self, message: &[u8]) -> bool {
        let positions = Self::hash(message);
        // simd not required as like above
        for position in positions {
            let mask = 1u8 << (position & 0x07);
            if self.data.as_ref()[(position >> 3) as usize] & mask == 0u8 {
                return false;
            }
        }
        return true;
    }

    /// Query actual size in bytes.
    pub fn size(&self) -> usize {
        1_usize << (ML - 3)
    }

    /// Get default size in bytes.
    pub fn default_size() -> usize {
        1_usize << (ML - 3)
    }

    /// Write bytes to disk.
    pub fn write(&self, file: &mut std::fs::File) -> std::io::Result<usize> {
        file.write(self.data.as_slice())
    }

    /// Shortcut for performing a hash action.
    ///
    /// Usage: `Self::hash(...)`.
    #[inline]
    fn hash(message: &[u8]) -> [u32; K] {
        Hasher::hash(message)
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
        let mut bf = BloomFilterImpl::<SipHash<20, 2>, 20, 2>::new();
        for i in 0..98765 {
            let s = format!("test-string-{i}");
            let s = ByteStream::from_slice(s.as_bytes());
            bf.insert(s.as_ref());
        }
        // test that there are no false negatives
        for i in 0..98765 {
            let s = format!("test-string-{i}");
            let s = ByteStream::from_slice(s.as_bytes());
            let result = bf.query(s.as_ref());
            assert_ne!(result, false);
        }
    }
}
