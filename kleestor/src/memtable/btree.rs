use crate::memtable::MemTable;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

/// A B-tree of key-value pairs, where each node may contain up to $ORDER - 1$
/// key-value pairs and up to $ORDER$ children.
///
/// The parameter `ORDER` should be odd. All terms referred to in this
/// implementation respect the 1998 Knuth definition.
///
/// You should avoid making `ORDER` larger than 65535 (2^16 - 1). Doing this
/// also means a higher cache miss rate.
pub struct BTreeUnsafe<K: Ord + Eq, V, const ORDER: usize> {
    root: *mut Node<K, V, ORDER>,
}

impl<K: Ord + Eq, V, const ORDER: usize> MemTable<K, V> for BTreeUnsafe<K, V, ORDER> {
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
    keys: [*mut K; ORDER],
    children: [*mut Node<K, V, ORDER>; ORDER],
    values: [*mut V; ORDER],
}

/// Insert actions need to recursively return the (newly-created) key-value
/// pair, or returns 'Kept' if no new pairs are created.
///
/// The 'replaced or not' field, is not given if a split had happened -- which
/// is due to that a split implicitly means `key` does not exist in the tree.
enum InsertResult<K: Ord + Eq, V, const ORDER: usize> {
    Split {
        key: *mut K,
        value: *mut V,
        lchild: *mut Node<K, V, ORDER>,
        rchild: *mut Node<K, V, ORDER>,
    },
    Kept {
        replaced: bool,
    },
}

impl<K: Ord + Eq, V, const ORDER: usize> Node<K, V, ORDER> {
    pub fn layout() -> Layout {
        Layout::new::<Self>()
    }

    /// Creates a new node returning its mutable pointer (unsafe).
    pub unsafe fn new() -> *mut Self {
        let layout = Self::layout();
        let ptr = alloc(layout) as *mut Self;

        (*ptr).keys_cnt = 0;
        for i in 0..ORDER {
            (*ptr).keys[i] = ptr::null_mut();
            (*ptr).values[i] = ptr::null_mut();
            (*ptr).children[i] = ptr::null_mut();
        }
        ptr
    }

    /// Releases pointer.
    pub unsafe fn drop(ptr: *mut Self) -> () {
        for i in 0..ORDER {
            free_from_heap((*ptr).keys[i]);
            free_from_heap((*ptr).values[i]);
        }
        dealloc(ptr as *mut u8, Self::layout());
    }
}

unsafe fn as_u64<P>(p: *mut P) -> String {
    if p == ptr::null_mut() {
        String::from("--")
    } else {
        (*(p as *mut u64)).to_string()
    }
}

impl<K: Ord + Eq, V, const ORDER: usize> BTreeUnsafe<K, V, ORDER> {
    pub fn new() -> Self {
        // not having this won't have a big impact
        // but it'd better have one
        assert!(ORDER % 2 == 1);

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
            let k64 = as_u64((*p).keys[i]);
            let v64 = as_u64((*p).values[i]);
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
        'recurse: while p != ptr::null_mut() {
            // iterate all keys (separators)
            let mut i = 0;
            while i < ORDER - 1 {
                if (*p).keys[i] == ptr::null_mut() {
                    break;
                }
                // compare between separator and key
                let sep = &*(*p).keys[i];
                if key < sep {
                    // goto left-hand-side
                    p = (*p).children[i];
                    continue 'recurse;
                } else if key == sep {
                    // bingo
                    return Some(&mut *(*p).values[i]);
                }
                i += 1;
            }
            // goto THE right-hand-side
            p = (*p).children[i];
        }
        None
    }

    /// Insert key-value pair.
    unsafe fn insert_wrap(&mut self, k: K, v: V) -> Option<()> {
        let k = move_to_heap(k);
        let v = move_to_heap(v);

        match self.insert_r(self.root, k, v) {
            InsertResult::Kept { replaced } => match replaced {
                true => {
                    free_from_heap(k); // is not used in structure
                    Some(())
                },
                false => None,
            },
            InsertResult::Split {
                key,
                value,
                lchild,
                rchild,
            } => {
                let p = Node::new();
                (*p).keys[0] = key;
                (*p).values[0] = value;
                (*p).keys_cnt = 1;
                (*p).children[0] = lchild;
                (*p).children[1] = rchild;
                self.root = p;
                None
            }
        }
    }

    /// Insert recursively a key-value pair into a given node. Nodes are split
    /// on the way while backtracing.
    ///
    /// The return value is a custom result indicating if an additional
    /// key-value pair had been inserted to the parent node as a result of the
    /// direct child being split.
    unsafe fn insert_r(
        &mut self,
        p: *mut Node<K, V, ORDER>,
        mut key: *mut K,
        mut value: *mut V,
    ) -> InsertResult<K, V, ORDER> {
        // p.key[idx - 1] <= key < p.key[idx]
        // inserting new child at p.child[idx]
        let keys_cnt = (*p).keys_cnt;
        let mut idx: usize = 0;
        while idx < keys_cnt as usize {
            let k = &*(*p).keys[idx];
            if &*key < k {
                break;
            } else if &*key == k {
                // replacing value does not trigger a split
                free_from_heap((*p).values[idx]);
                (*p).values[idx] = value;
                return InsertResult::Kept { replaced: true };
            }
            idx += 1;
        }

        // insert to key-key range or recursively find a leaf
        let mut lchild: *mut Node<K, V, ORDER> = ptr::null_mut();
        let mut rchild: *mut Node<K, V, ORDER> = ptr::null_mut();
        if (*p).children[idx] != ptr::null_mut() {
            match self.insert_r((*p).children[idx], key, value) {
                InsertResult::Kept { replaced } => {
                    // the lower-layer does not require a split
                    return InsertResult::Kept { replaced };
                }
                InsertResult::Split {
                    key: k,
                    value: v,
                    lchild: lc,
                    rchild: rc,
                } => {
                    // we still need to insert the newly split median
                    key = k;
                    value = v;
                    lchild = lc;
                    rchild = rc;
                }
            }
        }

        // enough space, just insert and don't split
        if keys_cnt + 1 < ORDER as u16 {
            // shift stuff right
            for i in (idx..keys_cnt as usize).rev() {
                (*p).keys[i + 1] = (*p).keys[i];
                (*p).values[i + 1] = (*p).values[i];
                (*p).children[i + 1] = (*p).children[i];
            }
            // insert leaf
            (*p).keys[idx] = key;
            (*p).values[idx] = value;
            (*p).children[idx] = lchild;
            (*p).children[idx + 1] = rchild;
            (*p).keys_cnt += 1;
            return InsertResult::Kept { replaced: false };
        }

        // this node would be split into 3 (2 extra)
        let lc: *mut Node<K, V, ORDER> = Node::new();
        let rc: *mut Node<K, V, ORDER> = Node::new();
        let median_key: *mut K;
        let median_value: *mut V;

        if idx < ORDER / 2 {
            // insert new node at left child
            //    0 X 1   [2]   3 4 5
            //   0 L R 2       3 4 5 6
            for i in 0..idx {
                (*lc).keys[i] = (*p).keys[i];
                (*lc).values[i] = (*p).values[i];
                (*lc).children[i] = (*p).children[i];
            }

            (*lc).keys[idx] = key;
            (*lc).values[idx] = value;
            (*lc).children[idx] = lchild;
            (*lc).children[idx + 1] = rchild;

            for i in (idx + 1)..(ORDER / 2) {
                (*lc).keys[i] = (*p).keys[i];
                (*lc).values[i] = (*p).values[i];
                (*lc).children[i + 1] = (*p).children[i];
            }

            median_key = (*p).keys[ORDER / 2 - 1];
            median_value = (*p).values[ORDER / 2 - 1];

            for i in 0..(ORDER / 2) {
                (*rc).keys[i] = (*p).keys[i + ORDER / 2];
                (*rc).values[i] = (*p).values[i + ORDER / 2];
                (*rc).children[i] = (*p).children[i + ORDER / 2];
            }
            (*rc).children[ORDER / 2] = (*p).children[ORDER - 1];
        } else if idx == ORDER / 2 {
            // insert new node in between
            //    0 1 2   [X]   3 4 5
            //   0 1 2 L       R 4 5 6

            for i in 0..(ORDER / 2) {
                (*lc).keys[i] = (*p).keys[i];
                (*lc).values[i] = (*p).values[i];
                (*lc).children[i] = (*p).children[i];
            }
            (*lc).children[ORDER / 2] = lchild;

            median_key = key;
            median_value = value;

            (*rc).children[0] = rchild;
            for i in 0..(ORDER / 2) {
                (*rc).keys[i] = (*p).keys[i + ORDER / 2];
                (*rc).values[i] = (*p).values[i + ORDER / 2];
                (*rc).children[i + 1] = (*p).children[i + 1 + ORDER / 2];
            }
        } else {
            // insert at right child
            //    0 1 2   [3]   4 X 5
            //   0 1 2 3       4 L R 6

            for i in 0..(ORDER / 2) {
                (*lc).keys[i] = (*p).keys[i];
                (*lc).values[i] = (*p).values[i];
                (*lc).children[i] = (*p).children[i];
            }
            (*lc).children[ORDER / 2] = (*p).children[ORDER / 2];

            median_key = (*p).keys[ORDER / 2];
            median_value = (*p).values[ORDER / 2];

            for i in (ORDER / 2 + 1)..idx {
                (*rc).keys[i - ORDER / 2 - 1] = (*p).keys[i];
                (*rc).values[i - ORDER / 2 - 1] = (*p).values[i];
                (*rc).children[i - ORDER / 2 - 1] = (*p).children[i];
            }

            (*rc).keys[idx - ORDER / 2 - 1] = key;
            (*rc).values[idx - ORDER / 2 - 1] = value;
            (*rc).children[idx - ORDER / 2 - 1] = lchild;
            (*rc).children[idx - ORDER / 2] = rchild;

            for i in (idx + 1)..ORDER {
                (*rc).keys[i - 1 - ORDER / 2] = (*p).keys[i - 1];
                (*rc).values[i - 1 - ORDER / 2] = (*p).values[i - 1];
                (*rc).children[i - ORDER / 2] = (*p).children[i];
            }
        }

        // TODO: `p` is nof freed

        (*lc).keys_cnt = (ORDER / 2) as u16;
        (*rc).keys_cnt = (ORDER / 2) as u16;

        InsertResult::Split {
            key: median_key,
            value: median_value,
            lchild: lc,
            rchild: rc,
        }
    }
}

unsafe fn free_from_heap<T>(p: *mut T) -> () {
    if p != ptr::null_mut() {
        let _item = Box::from_raw(p);
    }
}

unsafe fn move_to_heap<T>(item: T) -> *mut T {
    Box::into_raw(Box::from(item))
}
