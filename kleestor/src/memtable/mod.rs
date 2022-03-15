pub mod btree;
pub mod btree_builtin;
pub mod rbtree;
pub mod splay;

/// Basic MemTable implementation interface.
pub trait MemTable<K: Ord + Sized, V: Sized> {
    /// Accesses `table[key] -> value`.
    fn get(&mut self, key: &K) -> Option<&mut V>;

    /// Sets `table[key] = value` and returns `Some(())`. When no such
    /// key-value pair exists, one is created and `None` is returned instead.
    fn insert(&mut self, key: K, value: V) -> Option<()>;

    /// Removes key from table. Returns `Ok(())` for deletion success, or
    /// `Err(())` when no such key exists.
    fn remove(&mut self, key: &K) -> Result<(), ()>;
}
