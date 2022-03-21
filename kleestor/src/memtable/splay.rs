use crate::memtable::MemTable;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

/// Splay tree.
pub struct SplayTree<K: Ord + Eq, V> {
    /// Pointer to tree root.
    root: *mut Node<K, V>,
}

/// Access points for universal trait.
impl<K: Ord + Eq, V> MemTable<K, V> for SplayTree<K, V> {
    fn get(&mut self, key: &K) -> Option<&mut V> {
        unsafe {
            let ptr = self.access(key);
            if ptr == ptr::null_mut() {
                return None;
            }
            let value = &mut (*ptr).value;
            return Some(value);
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<()> {
        unsafe { self.insert_wrap(key, value) }
    }

    fn remove(&mut self, _key: &K) -> Result<(), ()> {
        Err(())
    }
}

struct Node<K: Ord + Eq, V> {
    pub key: K,
    pub value: V,
    pub parent: *mut Node<K, V>,
    pub child: [*mut Node<K, V>; 2], // left: [0], right: [1]
}

impl<K: Ord + Eq, V> Node<K, V> {
    /// Returns a constant layout of the node itself.
    pub fn layout() -> Layout {
        Layout::new::<Self>()
    }

    /// Creates a new node returning its mutable pointer (unsafe).
    pub unsafe fn new(key: K, value: V) -> *mut Self {
        let layout = Self::layout();
        let ptr = alloc(layout) as *mut Self;

        (*ptr).key = key;
        (*ptr).value = value;
        (*ptr).parent = ptr::null_mut();
        (*ptr).child[0] = ptr::null_mut();
        (*ptr).child[1] = ptr::null_mut();

        return ptr;
    }

    /// Releases pointer.
    #[allow(dead_code)]
    pub unsafe fn drop(ptr: *mut Self) -> () {
        drop(Box::from_raw(&mut (*ptr).key));
        drop(Box::from_raw(&mut (*ptr).value));

        dealloc(ptr as *mut u8, Self::layout());
    }
}

/// Implementations for fundamental tree algorithms.
impl<K: Ord + Eq, V> SplayTree<K, V> {
    /// Creates new instance.
    pub fn new() -> Self {
        Self {
            root: ptr::null_mut(),
        }
    }

    /// Access node with key in binary search tree.
    unsafe fn access(&mut self, key: &K) -> *mut Node<K, V> {
        if self.root == ptr::null_mut() {
            return ptr::null_mut();
        }
        // start from root
        let mut p = self.root;
        while p != ptr::null_mut() {
            if *key == (*p).key {
                break;
            } else if *key < (*p).key {
                p = (*p).child[0];
            } else {
                p = (*p).child[1];
            }
        }
        // if `key` is not found `p` would definitely be null
        if p != ptr::null_mut() {
            self.splay(p, ptr::null_mut());
        }
        return p;
    }

    /// Rotate node up 1 level (must not be root).
    unsafe fn rotate(&mut self, p: *mut Node<K, V>) -> () {
        let q = (*p).parent;
        let g = (*q).parent;
        let side: usize = if (*q).child[0] == p { 0 } else { 1 };

        // attach the grandson on other side
        let c = (*p).child[1 - side];
        (*q).child[side] = c;
        if c != ptr::null_mut() {
            (*c).parent = q;
        }

        // connect to grandparent
        if g != ptr::null_mut() {
            let qside: usize = if (*g).child[0] == q { 0 } else { 1 };
            (*g).child[qside] = p;
        }
        (*p).parent = g;

        // reconnect q and its son p
        (*p).child[1 - side] = q;
        (*q).parent = p;
    }

    /// Elevate given node `p` to become a direct child of `t` (or become root
    /// if `t` is null).
    #[allow(unused_assignments)]
    unsafe fn splay(&mut self, p: *mut Node<K, V>, t: *mut Node<K, V>) -> () {
        let mut q: *mut Node<K, V> = ptr::null_mut();
        loop {
            q = (*p).parent;
            if q == ptr::null_mut() || q == t {
                break;
            }
            let g = (*q).parent;
            if g != ptr::null_mut() && g != t {
                let pside = if p == (*q).child[0] { 0 } else { 1 };
                let qside = if q == (*g).child[0] { 0 } else { 1 };
                if pside == qside {
                    self.rotate(q);
                } else {
                    self.rotate(p);
                }
            }
            self.rotate(p);
        }
        if t == ptr::null_mut() {
            self.root = p;
        }
    }

    /// Insert key-value pair into tree. `Some` is returned with when an
    /// existing value is overwritten. `None` is returned if no such value
    /// existed with `key`.
    unsafe fn insert_wrap(&mut self, key: K, value: V) -> Option<()> {
        let mut p = self.root; // the to-be parent of `n`
        while p != ptr::null_mut() {
            if key == (*p).key {
                // changing value and just leaving
                (*p).value = value;
                self.splay(p, ptr::null_mut());
                return Some(());
            } else if key < (*p).key {
                if (*p).child[0] == ptr::null_mut() {
                    let ch = Node::new(key, value);
                    (*p).child[0] = ch;
                    (*ch).parent = p;
                    self.splay(ch, ptr::null_mut());
                    return None;
                }
                p = (*p).child[0];
            } else {
                if (*p).child[1] == ptr::null_mut() {
                    let ch = Node::new(key, value);
                    (*p).child[1] = ch;
                    (*ch).parent = p;
                    self.splay(ch, ptr::null_mut());
                    return None;
                }
                p = (*p).child[1];
            }
        }
        // should insert to root
        self.root = Node::new(key, value);
        None
    }
}
