use crate::record::{ByteStream, KvDataRef, KvEntry, KvPointer};
use crate::utils;
use std::cmp::Ordering;

/// Joins a list of [`Iterator<KvPointer>`] with priority. Earlier items have
/// higher priority and will override all latter items with the same key.
///
/// It must be guaranteed that keys are unique in every iterator.
///
/// Writing is banned in this iterator.
pub struct KvMergeIterator<'a, Pointer, Iter>
where
    Pointer: KvPointer,
    Iter: Iterator<Item = Pointer> + 'a,
{
    /// A list of all iterators to merge.
    iterators: Vec<Iter>,
    /// Buffer that is used to compare and store new (key, value, index) pairs.
    buffer: Vec<(&'a [u8], Pointer, usize)>,
}

impl<'a, Pointer, Iter> KvMergeIterator<'a, Pointer, Iter>
where
    Pointer: KvPointer,
    Iter: Iterator<Item = Pointer> + 'a,
{
    /// Create merged iterator.
    pub fn new(iters: Vec<Iter>) -> Self {
        let mut iter = Self {
            iterators: iters,
            buffer: vec![],
        };
        for index in 0..iter.iterators.len() {
            iter.binary_insert(index);
        }
        iter
    }

    /// Insert a next item from the [`index`]-th iterator, until the new / next
    /// item is no longer equal to any of the existing items in [`buffer`], or
    /// no next items exist anymore.
    ///
    /// It is required that no items belonging to the [`index`]-th iterator
    /// still persist in the buffer.
    fn binary_insert(&mut self, index: usize) -> () {
        let iter = unsafe { utils::const_as_mut(&self.iterators[index]) };
        loop {
            let item = match iter.next() {
                None => break,
                Some(it) => it,
            };
            let key = unsafe { utils::reborrow_slice(item.key()) };

            match self.binary_search(key) {
                FoundIndex::Equal(insert_at) => {
                    // higher priority
                    if index < self.buffer[insert_at].2 {
                        self.buffer[insert_at] = (key, item, index);
                    } else {
                        continue;
                    }
                }
                FoundIndex::Less(insert_at) => {
                    self.buffer.insert(insert_at, (key, item, index));
                }
            };
            break;
        }
    }

    /// Binary search for a location where we could insert a new item.
    fn binary_search(&self, key: &[u8]) -> FoundIndex {
        let len = self.buffer.len();
        if len == 0 {
            return FoundIndex::Less(0); // no keys
        }
        let mut left = 0_usize;
        let mut right = len;

        while left < right {
            let mid = (left + right) >> 1;
            if mid >= len {
                return FoundIndex::Less(len); // key > max(all_keys)
            }
            match ByteStream::ref_2_partial_cmp(self.buffer[mid].0, key) {
                Some(Ordering::Less) => left = mid + 1,
                Some(Ordering::Greater) => right = mid,
                Some(Ordering::Equal) => return FoundIndex::Equal(mid),
                _ => panic!("expect ordering to return comparison"),
            }
        }
        FoundIndex::Less(left) // it is magically never equal
    }
}

impl<'a, Pointer, Iter> Iterator for KvMergeIterator<'a, Pointer, Iter>
where
    Pointer: KvPointer,
    Iter: Iterator<Item = Pointer> + 'a,
{
    type Item = KvMergePointer<Pointer>;

    fn next(&mut self) -> Option<Self::Item> {
        // really nothing to take or else
        let (_key, item, index) = match self.buffer.len() {
            0 => return None,
            _ => self.buffer.remove(0),
        };

        // fill the next in and leave
        self.binary_insert(index);
        Some(KvMergePointer { _item: item })
    }
}

/// Internal pointer implementation for joiner iterator.
pub struct KvMergePointer<Pointer: KvPointer> {
    _item: Pointer,
}

impl<Pointer: KvPointer> KvPointer for KvMergePointer<Pointer> {
    fn key(&self) -> &[u8] {
        self._item.key()
    }

    /// Gets a reference to the pointing value.
    ///
    /// You should expect this reference to invalidate as soon as the pointer
    /// left this region.
    fn value(&self) -> KvDataRef {
        self._item.value().clone()
    }

    /// Gets a mutable reference to the pointing value.
    ///
    /// This exposes the underlying implementation. Lifetime should be manually
    /// ensured.
    fn value_mut(&self) -> &mut KvEntry {
        unimplemented!("kvmerge may only be used read-only");
    }
}

/// Just an internal representation used for binary search within
/// [`KvMergeIterator`].
enum FoundIndex {
    /// The discovered [`vec[index]`] is less than the queried key.
    Less(usize),
    /// The discovered [`vec[index]`] is equal to the queried key.
    Equal(usize),
}

#[cfg(test)]
mod tests {
    use crate::memtable::rbtree::RBTree;
    use crate::memtable::MemTable;
    use crate::record::{ByteStream, KvData, KvEntry, KvPointer};
    use crate::sstable::reader::SSTableReader;
    use crate::sstable::writer::SSTableWriter;
    use std::io::Result;
    use std::path::PathBuf;

    use super::KvMergeIterator;

    #[test]
    fn merge_ok() -> () {
        // write 1000 - 9999 in 4 files to modulo separately
        create_run(0, 1000, 9999, 4).unwrap();
        create_run(1, 1001, 9999, 4).unwrap();
        create_run(2, 1002, 9999, 4).unwrap();
        create_run(3, 1003, 9999, 4).unwrap();
        // load runs
        let runs = vec![read_run(0), read_run(1), read_run(2), read_run(3)];
        let iters = runs.iter().map(|run| run.iter()).collect();
        // read merged items
        let merger = KvMergeIterator::new(iters);
        for item in merger {
            let z = item._to_string();
            println!("found -- {z}");
        }
        // clean objects
        for id in 0..=3 {
            let _ = std::fs::remove_file(&get_file_path(id));
        }
    }

    fn get_file_path(id: u32) -> PathBuf {
        let mut tmp_dir = std::env::temp_dir();
        tmp_dir.push(format!("_kleestor_record_kvmerge_run_{id}.db"));
        tmp_dir
    }

    /// Write run #id to disk.
    fn create_run(id: u32, begin: i64, end: i64, skip: usize) -> Result<()> {
        // create input memtable
        let mut map = RBTree::<ByteStream, KvEntry>::new();
        for i in (begin..=end).step_by(skip) {
            let key = format!("sample-key-{i}");
            let value = format!("value-{i}-{i}-{i}-{i}-run-{id}");
            map.insert(
                ByteStream::from_slice(key.as_bytes()),
                KvEntry::new(KvData::Value {
                    cached: false,
                    value: ByteStream::from_slice(value.as_bytes()),
                }),
            );
        }
        // write data to disk
        let file = std::fs::File::create(&get_file_path(id))?;
        let mut table = SSTableWriter::new(file);
        table.write(map.iter_mut())?;
        Ok(())
    }

    /// Read run from disk.
    fn read_run(id: u32) -> SSTableReader {
        let file = std::fs::File::open(&get_file_path(id)).unwrap();
        SSTableReader::new(file).unwrap()
    }
}
