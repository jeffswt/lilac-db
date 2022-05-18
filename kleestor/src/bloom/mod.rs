pub mod fimpl;
pub mod strategies;

use crate::record::ByteStream;
use self::{fimpl::BloomFilterImpl, strategies::SfHash64};

/// A hash strategy that produces K positions for a bloom filter on a span of
/// 2^ML slots in total.
pub trait HashStrategy<const ML: usize, const K: usize>
where
    [(); K]: Sized,
{
    fn hash(message: &[u8]) -> [u32; K];
}

/// Default bloom filter constructor.
pub type BloomFilter = BloomFilterImpl<SfHash64<24, 2>, 24, 2>;
