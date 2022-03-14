use crate::memtable::MemTable;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

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

impl<K: Ord + Sized, V: Sized> Node<K, V> {
    /// Returns a constant layout of the node itself.
    pub fn layout() -> Layout {
        Layout::new::<Self>()
    }

    /// Creates a new node returning its mutable pointer (unsafe).
    ///
    /// Color and references are not expected to be set on default.
    pub unsafe fn new(key: K, value: V) -> *mut Self {
        let layout = Self::layout();
        let ptr = alloc(layout) as *mut Self;

        (*ptr).color = Color::Black;
        (*ptr).key = key;
        (*ptr).value = value;
        (*ptr).parent = ptr::null_mut();
        (*ptr).child[0] = ptr::null_mut();
        (*ptr).child[1] = ptr::null_mut();

        return ptr;
    }

    /// Releases pointer.
    pub unsafe fn drop(ptr: *mut Self) -> () {
        drop(Box::from_raw(&mut (*ptr).key));
        drop(Box::from_raw(&mut (*ptr).value));

        dealloc(ptr as *mut u8, Self::layout());
    }
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

    /// Get side of node relative to its parent.
    ///
    /// `p` must have a parent (`g`).
    unsafe fn get_side(&mut self, p: *mut Node<K, V>, g: *mut Node<K, V>) -> usize {
        if p == (*g).child[1] {
            1
        } else {
            0
        }
    }

    /// Rotate tree to desired direction (left: 0, right: 1).
    unsafe fn rotate(&mut self, p: *mut Node<K, V>, side: usize) -> () {
        let g = (*p).parent; // grandparent
        let s = (*p).child[1 - side]; // the child that will replace p's position
        let c = (*s).child[side]; // the grandson one that would become p's child

        // attach the grandson on other side
        (*p).child[1 - side] = c;
        if c != ptr::null_mut() {
            (*c).parent = p;
        }

        // reconnect p and its son
        (*s).child[side] = p;
        (*p).parent = s;

        // connect son and parent or set as root
        (*s).parent = g;
        if g != ptr::null_mut() {
            (*g).child[self.get_side(p, g)] = s;
        } else {
            self.root = s;
        }
    }

    /// Insert key-value pair into tree. `Some` is returned with when an
    /// existing value is overwritten. `None` is returned if no such value
    /// existed with `key`.
    unsafe fn insert_cases(
        &mut self,
        mut n: *mut Node<K, V>,
        mut p: *mut Node<K, V>,
        mut side: usize,
    ) -> () {
        (*n).color = Color::Red;
        (*n).child[0] = ptr::null_mut();
        (*n).child[1] = ptr::null_mut();
        (*n).parent = p;

        let g: *mut Node<K, V> = ptr::null_mut(); // grandparent of `n`
        let u: *mut Node<K, V> = ptr::null_mut(); // uncle of `n`

        if p == ptr::null_mut() {
            self.root = n;
            return;
        } else {
            (*p).child[side] = n;
        }

        loop {
            // Insert case 1:
            // The current node’s parent `p` is black.
            if let Color::Black = (*p).color {
                // a red node does not have a red child
                return;
            }
            // `p` is now red
            let g = (*p).parent;
            if g == ptr::null_mut() {
                // Insert case 4:
                // The parent `p` is red and the root. Because `n` is also red,
                // requirement 3 is violated. But after switching `p`’s color
                // the tree is in RB-shape. The black height of the tree
                // increases by 1.
                (*p).color = Color::Black;
                return;
            }
            // `p` is red and `g` is non-null
            side = self.get_side(p, g);
            let u = (*g).child[1 - side]; // uncle of p
            if u == ptr::null_mut() {
                return self.insert_case_5_6(n, p, g, side);
            } else if let Color::Black = (*u).color {
                return self.insert_case_5_6(n, p, g, side);
            }

            // Insert case 2:
            // If both the parent `p` and the uncle `u` are red, then both of
            // them can be repainted black and the grandparent `g` becomes red.
            (*p).color = Color::Black;
            (*u).color = Color::Black;
            (*g).color = Color::Red;
            // trace up the tree for 1 black level (2 tree levels)
            n = g;
            p = (*n).parent;
            if p == ptr::null_mut() {
                break;
            }
        }

        // Insert case 3:
        // Case 2 has been executed for $(h-1)/2$ times and the total height of
        // the tree has increased by 1, now being $h$. The current node N is
        // the (red) root of the tree, and all RB-properties are satisfied.
        return;
    }

    unsafe fn insert_case_5_6(
        &mut self,
        n: *mut Node<K, V>,
        mut p: *mut Node<K, V>,
        g: *mut Node<K, V>,
        side: usize,
    ) -> () {
        // Insert case 5:
        // The parent `p` is red but the uncle `u` is black.
        if n == (*p).child[1 - side] {
            self.rotate(p, side);
            // n = p;
            p = (*g).child[side];
        }

        // Insert case 6:
        // The current node `n` is now certain to be an outer grandchild of `g`.
        // if g != self.root {
        //     self.rotate(g, 1 - side);
        // }
        self.rotate(g, 1 - side);
        (*p).color = Color::Black;
        (*g).color = Color::Red;

        return;
    }
}
