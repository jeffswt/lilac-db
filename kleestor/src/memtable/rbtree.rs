use crate::memtable::MemTable;
use std::alloc::{alloc, dealloc, Layout};
use std::marker::PhantomData;
use std::ptr;
use std::rc::{Rc, Weak};

/// A thread-safe implementation of B tree, which utilizes mutex locks on the
/// nodes to provide thread-safety.
///
/// It takes a parameter `K` as Key type and parameter `V` as Value type. The
/// number of children per tree node is given by the parameter `M`.
pub struct RBTree<K: Ord + Sized, V: Sized> {
    /// Pointer to tree root.
    root: *mut Node<K, V>,
    /// A total of `length` nodes are in this tree.
    length: usize,
}

/// Access points for universal trait.
impl<K: Ord + Sized, V: Sized> MemTable<K, V> for RBTree<K, V> {
    fn get(&mut self, key: &K) -> Option<&V> {
        None
    }

    fn insert(&mut self, key: K, value: V) -> Option<()> {
        None
    }

    fn remove(&mut self, key: &K) -> Result<(), ()> {
        Err(())
    }
}

/// A red-black tree node.
struct Node<K: Ord + Sized, V: Sized> {
    pub color: Color,
    pub key: K,
    pub value: V,
    pub parent: *mut Node<K, V>,
    pub child: [*mut Node<K, V>; 2], // left: [0], right: [1]
}

enum Color {
    Red,
    Black,
}

/// Implementations for fundamental tree algorithms.
impl<K: Ord + Sized, V: Sized> RBTree<K, V> {
    /// Access node with key in red-black tree.
    unsafe fn access(&self, key: &K) -> *mut Node<K, V> {
        if self.length <= 0 {
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
        return p;
    }

    /// Rotate tree to desired direction (left: 0, right: 1).
    unsafe fn rotate(&mut self, p: *mut Node<K, V>, dir: usize) -> () {
        let g = (*p).parent; // grandparent
        let s = (*p).child[1 - dir]; // the child that will replace p's position
        let c = (*s).child[dir]; // the grandson one that would become p's child

        // attach the grandson on other side
        (*p).child[1 - dir] = c;
        if c != ptr::null_mut() {
            (*c).parent = p;
        }

        // reconnect p and its son
        (*s).child[dir] = p;
        (*p).parent = s;

        // connect son and parent or set as root
        (*s).parent = g;
        if g != ptr::null_mut() {
            (*g).child[if p == (*g).child[1] { 1 } else { 0 }] = s;
        } else {
            self.root = s;
        }
    }

    /// Insert key-value pair into tree. `Some(())` is returned when an
    /// existing value is overwritten. `None` is returned if no such value
    /// existed with `key`.
    fn insert(&mut self, key: K, value: V) -> Option<()> {
        None
    }
}
