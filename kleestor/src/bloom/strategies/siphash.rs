use crate::bloom::HashStrategy;
use crate::record::ByteStream;
#[allow(deprecated)]
use std::hash::{Hasher, SipHasher};

/// SipHash produces hashes of up to 64 bits.
///
/// The naive modulus strategy suggests that you ensure M == 2^ML for best
/// collision avoidance. Otherwise the distribution between slots would be quite
/// uneven, causing waste on bits.
///
/// It is also required that ML * K be less than 64 bits.
pub struct SipHash<const M: usize, const ML: usize, const K: usize>
where
    [(); K]: Sized, {}

impl<const M: usize, const ML: usize, const K: usize> HashStrategy<M, ML, K> for SipHash<M, ML, K> {
    #[inline]
    #[allow(deprecated)]
    fn hash(message: &ByteStream) -> [u32; K] {
        assert!(
            M * 2 >= (1 << ML) && M <= (1 << ML),
            "2^(ML-1) <= M <= 2^ML"
        );
        assert!(ML * K <= 64, "sufficient bits for hashing");

        let mut hasher = SipHasher::new();
        hasher.write(message.as_ref());
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
