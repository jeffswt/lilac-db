use crate::memtable::MemTable;
use std::collections::BTreeMap;

/// Wrapper of built-in B-Tree map implementation.
pub struct BTreeBuiltin<K: Ord + Sized, V: Sized> {
    proxy: BTreeMap<K, V>,
}

impl<K: Ord + Sized, V: Sized> MemTable<K, V> for BTreeBuiltin<K, V> {
    fn get(&mut self, key: &K) -> Option<&mut V> {
        match self.proxy.get(key) {
            None => None,
            Some(value) => {
                let r = value as *const V;
                let mut_r = (r as usize) as *mut V;
                unsafe {
                    let value_mut = &mut (*mut_r);
                    Some(value_mut)
                }
            }
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<()> {
        self.proxy.insert(key, value);
        Some(())
    }

    fn remove(&mut self, key: &K) -> Result<(), ()> {
        Err(())
    }
}

impl<K: Ord + Sized, V: Sized> BTreeBuiltin<K, V> {
    pub fn new() -> Self {
        Self {
            proxy: BTreeMap::new(),
        }
    }
}
