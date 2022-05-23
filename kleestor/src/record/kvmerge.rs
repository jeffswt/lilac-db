use crate::record::{ByteStream, KvDataRef, KvEntry, KvPointer};
use crate::utils::{self, reborrow};
use std::cmp::Ordering;
use std::mem;

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

            let item_s = item._to_string();
            println!("binary insert -- {item_s}");

            let insert_at = match self.binary_search(key) {
                FoundIndex::Equal(_) => continue,
                FoundIndex::Less(idx) => idx,
            };
            self.buffer.insert(insert_at, (key, item, index));
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
