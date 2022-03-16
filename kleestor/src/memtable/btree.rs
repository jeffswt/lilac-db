use crate::memtable::MemTable;
use std::alloc::{alloc, dealloc, Layout};
use std::mem;
use std::ptr;

/// A B-tree of key-value pairs, where each node may contain up to $ORDER - 1$
/// key-value pairs and up to $ORDER$ children.
///
/// The parameter `ORDER` should be odd. All terms referred to in this
/// implementation respect the 1998 Knuth definition.
///
/// You should avoid making `ORDER` larger than 65535 (2^16 - 1). Doing this
/// also means a higher cache miss rate.
pub struct BTree<K: Ord + Eq, V, const ORDER: usize> {
    root: *mut Node<K, V, ORDER>,
}

impl<K: Ord + Eq, V, const ORDER: usize> MemTable<K, V> for BTree<K, V, ORDER> {
    fn get(&mut self, key: &K) -> Option<&mut V> {
        unsafe { self.access(&key) }
    }

    fn insert(&mut self, key: K, value: V) -> Option<()> {
        unsafe { self.insert_wrap(key, value) }
    }

    fn remove(&mut self, key: &K) -> Result<(), ()> {
        Err(())
    }
}

struct Node<K: Ord + Eq, V, const ORDER: usize> {
    keys_cnt: u16,
    keys: [Option<Box<K>>; ORDER],
    children: [*mut Node<K, V, ORDER>; ORDER],
    values: [Option<Box<V>>; ORDER],
}

enum InsertResult<K: Ord + Eq, V, const ORDER: usize> {
    Split {
        key: K,
        value: V,
        lchild: *mut Node<K, V, ORDER>,
        rchild: *mut Node<K, V, ORDER>,
    },
    Kept,
}

impl<K: Ord + Eq, V, const ORDER: usize> Node<K, V, ORDER> {
    pub fn layout() -> Layout {
        Layout::new::<Self>()
    }

    /// Creates a new node returning its mutable pointer (unsafe).
    pub unsafe fn new() -> *mut Self {
        let layout = Self::layout();
        let ptr = alloc(layout) as *mut Self;

        // the newly inserted node is temporarily colored red so that all paths
        // contain the same number of black nodes as before.
        (*ptr).keys_cnt = 0;
        for i in 0..ORDER {
            (*ptr).keys[i] = None;
            (*ptr).values[i] = None;
            (*ptr).children[i] = ptr::null_mut();
        }

        return ptr;
    }

    /// Releases pointer.
    pub unsafe fn drop(ptr: *mut Self) -> () {
        for i in 0..ORDER {
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

unsafe fn as_u64<P>(p: &Option<Box<P>>) -> String {
    match p {
        None => String::from("--"),
        Some(x) => {
            let p64 = *(x.as_ref() as *const P as *const u64);
            p64.to_string()
        }
    }
}

impl<K: Ord + Eq, V, const ORDER: usize> BTree<K, V, ORDER> {
    pub fn new() -> Self {
        unsafe { Self { root: Node::new() } }
    }

    pub fn debug(&mut self) -> () {
        let p64 = self.root as usize as u64;
        println!("Root {p64:x};");
        unsafe {
            self.debug_r(self.root);
        }
        println!("");
    }

    unsafe fn debug_r(&mut self, p: *mut Node<K, V, ORDER>) -> () {
        let p64 = p as usize as u64;
        let cc = (*p).keys_cnt as usize;
        println!("    Node {p64:x}, keys={cc}");
        for i in 0..=cc {
            let k64 = as_u64(&(*p).keys[i]);
            let v64 = as_u64(&(*p).values[i]);
            let c64 = (*p).children[i] as usize as u64;
            println!("      - [K{i}] {k64} [V{i}] {v64} [ch{i}] {c64:x}");
        }

        for i in 0..=cc {
            let ch = (*p).children[i];
            if ch != ptr::null_mut() {
                self.debug_r(ch);
            }
        }
    }

    unsafe fn access(&mut self, key: &K) -> Option<&mut V> {
        let mut p = self.root;
        while p != ptr::null_mut() {
            // iterate all keys (separators)
            let mut i = 0;
            while i < ORDER - 1 {
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

    /// Insert key-value pair.
    unsafe fn insert_wrap(&mut self, key: K, value: V) -> Option<()> {
        match self.insert_r(self.root, key, value) {
            None => (),
            Some((k, v)) => {
                let p = Node::new();
                (*p).keys[0] = Some(Box::from(k));
                (*p).values[0] = Some(Box::from(v));
                (*p).keys_cnt = 1;
                self.root = p;
            }
        }
        None
    }

    /// Insert recursively a key-value pair into a given node. Nodes are split
    /// on the way while backtracing.
    ///
    /// The return value is an option indicating if an additional key-value
    /// pair had been inserted to the parent node as a result of the direct
    /// child being split.
    unsafe fn insert_r(
        &mut self,
        p: *mut Node<K, V, ORDER>,
        mut key: K,
        mut value: V,
    ) -> Option<(K, V)> {
        // p.key[idx - 1] <= key < p.key[idx]
        // inserting new child at p.child[idx]
        let keys_cnt = (*p).keys_cnt;
        let mut idx: usize = 0;
        while idx < keys_cnt as usize {
            let k = (*p).keys[idx].as_ref().unwrap();
            if key < **k {
                break;
            } else if key == **k {
                // replacing value does not trigger a split
                (*p).values[idx] = Some(Box::from(value));
                return None;
            }
            idx += 1;
        }

        // insert to key-key range or recursively find a leaf
        if (*p).children[idx] != ptr::null_mut() {
            match self.insert_r((*p).children[idx], key, value) {
                None => {
                    // the lower-layer does not require a split
                    return None;
                }
                Some((k, v)) => {
                    // we still need to insert the newly split median
                    key = k;
                    value = v;
                }
            }
        }

        // enough space, just insert and don't split
        if keys_cnt + 1 < ORDER as u16 {
            // shift stuff right
            for i in (idx..keys_cnt as usize).rev() {
                (*p).keys[i + 1] = mem::take(&mut (*p).keys[i]);
                (*p).values[i + 1] = mem::take(&mut (*p).values[i]);
                (*p).children[i + 1] = (*p).children[i];
            }
            // insert leaf
            (*p).keys[idx] = Some(Box::from(key));
            (*p).values[idx] = Some(Box::from(value));
            (*p).children[idx] = ptr::null_mut();
            (*p).keys_cnt += 1;
            return None;
        }

        // this node would be split into 3 (2 extra)
        unimplemented!();
        None
    }
}
