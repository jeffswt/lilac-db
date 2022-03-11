mod bptree;

/// Basic MemTable implementation interface.
pub trait MemTable<K: Ord + Sized, V: Sized> {
    /// Accesses `table[key] -> value`.
    fn query(&mut self, key: &K) -> Option<V>;

    /// Sets `table[key] = value` and returns `Some(())`. When no such
    /// key-value pair exists, one is created and `None` is returned instead.
    fn insert(&mut self, key: K, value: V) -> Option<()>;

    /// Removes key from table. Returns `Ok(())` for deletion success, or
    /// `Err(())` when no such key exists.
    fn remove(&mut self, key: &K) -> Result<(), ()>;
}
