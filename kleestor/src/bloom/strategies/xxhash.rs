use crate::bloom::HashStrategy;
use fasthash::{xx::Hasher64, FastHasher};
use std::hash::Hasher;

/// XxHash produces hashes of up to 64 bits.
///
/// It is required that ML * K be less than 64 bits.
pub struct XxHash<const ML: usize, const K: usize>
where
    [(); K]: Sized, {}

impl<const ML: usize, const K: usize> HashStrategy<ML, K> for XxHash<ML, K> {
    #[inline]
    #[allow(deprecated)]
    fn hash(message: &[u8]) -> [u32; K] {
        assert!(ML * K <= 64, "sufficient bits for hashing");

        let mut hasher = Hasher64::new();
        hasher.write(message);
        let mut finished = hasher.finish();

        let mut result = [0u32; K];
        let mask: u64 = (1 << ML) - 1;
        for i in 0..K {
            result[i] = (finished & mask) as u32;
            finished >>= ML;
        }
        result
    }
}
