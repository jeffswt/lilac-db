use crate::memtable::MemTable;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

/// A B-tree of key-value pairs, where each node may contain up to $M - 1$
/// keys and up to $M$ children (an M-order B-Tree).
///
/// The parameter `M` should be odd. All terms referred to in this
/// implementation respect the 1998 Knuth definition.
pub struct BTree<K: Ord, V, const M: usize> {
    root: *mut Node<K, V, M>,
}

struct Node<K: Ord, V, const M: usize> {
    keys: [Option<Box<K>>; M],
    values: [Option<Box<V>>; M],
    children: [*mut Node<K, V, M>; M],
    leaf: bool,
    parent: *mut Node<K, V, M>,
}

impl<K: Ord, V, const M: usize> Node<K, V, M> {
    pub fn layout() -> Layout {
        Layout::new::<Self>()
    }

    /// Creates a new node returning its mutable pointer (unsafe).
    pub unsafe fn new(key: K, value: V) -> *mut Self {
        let layout = Self::layout();
        let ptr = alloc(layout) as *mut Self;

        // the newly inserted node is temporarily colored red so that all paths
        // contain the same number of black nodes as before.
        for i in 0..M {
            (*ptr).keys[i] = None;
            (*ptr).values[i] = None;
            (*ptr).children[i] = ptr::null_mut();
        }
        (*ptr).leaf = false;
        (*ptr).parent = ptr::null_mut();

        return ptr;
    }

    /// Releases pointer.
    pub unsafe fn drop(ptr: *mut Self) -> () {
        for i in 0..M {
            match &(*ptr).keys[i] {
                Some(k) => drop(k),
                None => (),
            }
            match &(*ptr).values[i] {
                Some(v) => drop(v),
                None => (),
            }
        }
        dealloc(ptr as *mut u8, Self::layout());
    }
}

impl<K: Ord, V, const M: usize> BTree<K, V, M> {
    unsafe fn access(&mut self, key: &K) -> Option<&mut V> {
        let mut p = self.root;
        while p != ptr::null_mut() {
            // iterate all keys (separators)
            let mut i = 0;
            while i < M - 1 {
                match (*p).keys[i].as_ref() {
                    None => {
                        break;
                    }
                    // compare between separator and key
                    Some(sep_box) => {
                        let sep = sep_box.as_ref();
                        if key < sep {
                            // goto left-hand-side
                            p = (*p).children[i];
                            continue;
                        } else if key == sep {
                            // bingo
                            let val_box = (*p).values[i].as_mut().unwrap();
                            return Some(val_box);
                        }
                    }
                }
                i += 1;
            }
            // goto right-hand-side
            p = (*p).children[i];
        }
        None
    }
}
