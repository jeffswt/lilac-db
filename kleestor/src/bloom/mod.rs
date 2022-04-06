pub mod fimpl;
pub mod strategies;

use crate::record::ByteStream;

/// A hash strategy that produces K positions for a bloom filter on a span of
/// 2^ML slots in total.
pub trait HashStrategy<const ML: usize, const K: usize>
where
    [(); K]: Sized,
{
    fn hash(message: &ByteStream) -> [u32; K];
}
