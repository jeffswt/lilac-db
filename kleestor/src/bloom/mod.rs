mod fimpl;
mod strategies;

use crate::record::ByteStream;

/// A hash strategy that produces K positions for a bloom filter on a span of
/// M slots in total (it should hold that 2^ML >= M).
trait HashStrategy<const M: usize, const ML: usize, const K: usize>
where
    [(); K]: Sized,
{
    fn hash(message: &ByteStream) -> [u32; K];
}
