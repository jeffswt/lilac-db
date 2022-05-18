pub mod reader;
pub mod writer;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum MetaBlockType {
    Index = 1,
    BloomFilter = 2,
}
