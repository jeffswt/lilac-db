use crate::memtable::MemTable;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

/// A lock-free implementation of B+ Tree, which takes a parameter `K` as Key
/// type and parameter `V` as Value type. The number of children per tree node
/// is given by the parameter `M`, while each leaf data chunk contains at most
/// `U` data entries.
pub struct BPTreeInternal<K: Ord + Sized, V: Sized, const M: usize, const U: usize> {
    root: Rc<Node<K, V, M>>,

    _marker_1: PhantomData<K>,
    _marker_2: PhantomData<V>,
}

/// Access points for universal trait.
impl<K: Ord + Sized, V: Sized, const M: usize, const U: usize> MemTable<K, V>
    for BPTreeInternal<K, V, M, U>
{
    fn query(&mut self, key: &K) -> Option<V> {
        self.search_in_btree(key)
    }

    fn insert(&mut self, key: K, value: V) -> Option<()> {
        self.insert_to_btree(key, value)
    }

    fn remove(&mut self, key: &K) -> Result<(), ()> {
        self.delete_from_btree(key)
    }
}

/// B+ tree node contains no data, only keys and pointers to nodes or data.
struct Node<K: Ord + Sized, V: Sized, const M: usize> {
    parent: Weak<Node<K, V, M>>,
    children: [Rc<Node<K, V, M>>; M],
    chunk: u32,
    location: u32,
    freeze_state: FreezeState,

    _marker_1: PhantomData<K>,
    _marker_2: PhantomData<V>,
}

/// Data chunks are used to store (references to) actual data.
struct Chunk<V: Sized, const U: usize> {
    _marker: PhantomData<V>,
}

/// The magic state used for the lock-free algorithm to keep operations
/// synchronized.
enum FreezeState {
    Infant,
    Normal,
    Freeze,
    Copy,
    Split,
}

/// Implementations for fundamental tree algorithms.
impl<K: Ord + Sized, V: Sized, const M: usize, const U: usize> BPTreeInternal<K, V, M, U> {
    ///////////////////////////////////////////////////////////////////////////
    /// Appendix B. B+ tree supporting methods

    /// Algorithm 3. (a) Search - High Level Methods
    fn search_in_btree(&mut self, key: &K) -> Option<V> {
        let node = self.find_leaf(key);
        let chunk = self.node_to_chunk(node);
        self.search_in_chunk(chunk, key)
    }

    /// Algorithm 3. (b) Insert - High Level Methods
    ///
    /// Returns `Result(())` when key had already existed prior to this insert,
    /// and `None` if creating a new key-value entry.
    fn insert_to_btree(&mut self, key: K, value: V) -> Option<()> {
        let node = self.find_leaf(key);
        if let FreezeState::Infant = node.freeze_state {
            self.help_infant(node);
        }
        let chunk = self.node_to_chunk(node);
        self.insert_to_chunk(chunk, key, value)
    }

    /// Algorithm 3. (c) Delete - High Level Methods
    ///
    /// Returns `Ok(())` when deletion succeeds. Will return an `Err(())` when
    /// no such key exists.
    fn delete_from_btree(&mut self, key: &K) -> Result<(), ()> {
        let node = self.find_leaf(key);
        if let FreezeState::Infant = node.freeze_state {
            self.help_infant(node);
        }
        let chunk = self.node_to_chunk(node);
        self.delete_from_chunk(chunk, key)
    }

    fn find_leaf(&self, key: K) -> Rc<Node<K, V, M>> {}
}
