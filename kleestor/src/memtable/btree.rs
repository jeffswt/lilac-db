use crate::memtable::MemTable;
use std::mem;
use std::mem::MaybeUninit;
use std::ptr;

/// 15-order B-Trees have the best performance.
pub type BTree<K, V> = BTreeImpl<K, V, 7>;

/// A B-tree with at most $2n$ key-value pairs per node and at most $2n + 1$
/// children per node.
///
/// The parameter $N$ may be any positive integer. Giving it a larger value
/// might increase the probability of cache misses.
///
/// According to the 1998 Knuth definition, this is an $2n + 1$-order B-tree.
pub struct BTreeImpl<K: Ord + Eq, V, const N: usize>
where
    [(); N * 2]: Sized,
    [(); N * 2 + 1]: Sized,
{
    /// Reference to root node.
    root: Option<Box<Node<K, V, N>>>,
    /// Number of items.
    length: usize,
}

impl<K: Ord + Eq, V, const N: usize> MemTable<K, V> for BTreeImpl<K, V, N>
where
    [(); N * 2]: Sized,
    [(); N * 2 + 1]: Sized,
{
    fn get(&mut self, key: &K) -> Option<&mut V> {
        unsafe { self.access(&key) }
    }

    fn insert(&mut self, key: K, value: V) -> Option<()> {
        unsafe { self.insert_wrap(key, value) }
    }

    fn remove(&mut self, _key: &K) -> Result<(), ()> {
        Err(())
    }
}

struct Node<K: Ord + Eq, V, const N: usize>
where
    [(); N * 2]: Sized,
    [(); N * 2 + 1]: Sized,
{
    /// How many valid key-value pairs are contained in this node.
    count: u16,
    /// The keys [0..count] are initialized.
    keys: [mem::MaybeUninit<K>; N * 2],
    /// The values [0..count] are initialized.
    values: [mem::MaybeUninit<V>; N * 2],
    /// Child references to [count..] are invalid.
    children: [Option<Box<Node<K, V, N>>>; N * 2 + 1],
}

impl<K: Ord + Eq, V, const N: usize> Node<K, V, N>
where
    [(); N * 2]: Sized,
    [(); N * 2 + 1]: Sized,
{
    /// Resets all fields to their should-be-initial state. Old values won't be
    /// read or dropped or destructed so using this must be cautious.
    unsafe fn init(this: *mut Self) {
        ptr::addr_of_mut!((*this).count).write(0);
        for i in 0..=N * 2 {
            ptr::addr_of_mut!((*this).children[i]).write(None);
        }
    }

    /// Creates new uninitialized node in a `Box`.
    pub unsafe fn new() -> Box<Self> {
        let mut p = Box::new_uninit();
        Self::init(p.as_mut_ptr());
        p.assume_init()
    }
}

impl<K: Ord + Eq, V, const N: usize> Drop for Node<K, V, N>
where
    [(); N * 2]: Sized,
    [(); N * 2 + 1]: Sized,
{
    fn drop(&mut self) -> () {
        unsafe {
            for i in 0..(self.count as usize) {
                ptr::drop_in_place(self.keys[i].as_mut_ptr());
                ptr::drop_in_place(self.values[i].as_mut_ptr());
            }
        }
    }
}

/// Insert actions need to recursively return the (newly-created) key-value
/// pair if the node was split into half (across its median).
///
/// If a replace happened upon the key, `Replaced` is returned.
///
/// Otherwise a replace did not happen at the leaf, and splits are no longer
/// required, then a `Finished` is returned.
enum InsertResult<K: Ord + Eq, V, const N: usize>
where
    [(); N * 2]: Sized,
    [(); N * 2 + 1]: Sized,
{
    Split {
        key: K,
        value: V,
        /// The left child created during a split is always the original node
        /// with half of its data preserved. Therefore only the (newly created)
        /// right child is to provided.
        rchild: Option<Box<Node<K, V, N>>>,
    },
    Replaced,
    Finished,
}

impl<K: Ord + Eq, V, const N: usize> BTreeImpl<K, V, N>
where
    [(); N * 2]: Sized,
    [(); N * 2 + 1]: Sized,
{
    pub fn new() -> Self {
        // not having this won't have a big impact
        // but it'd better have one
        assert!(N >= 2 && N <= 16384);

        unsafe {
            Self {
                root: Some(Node::new()),
                length: 0,
            }
        }
    }

    #[allow(dead_code)]
    pub fn debug(&mut self) -> () {
        let p64 = self.root.as_ref().unwrap().as_ref() as *const Node<K, V, N> as usize as u64;
        println!("Root {p64:x};");
        unsafe {
            Self::debug_r(self.root.as_mut().unwrap());
        }
        println!("");
    }

    unsafe fn debug_r(p: &mut Box<Node<K, V, N>>) -> () {
        let p64 = p.as_ref() as *const Node<K, V, N> as usize as u64;
        let cc = p.count as usize;
        println!("    Node {p64:x}, keys={cc}");
        for i in 0..=cc {
            let mut k64 = String::from("--");
            let mut v64 = String::from("--");
            if i < cc {
                k64 = (*(p.keys[i].assume_init_ref() as *const K as *const u64)).to_string();
                v64 = (*(p.values[i].assume_init_ref() as *const V as *const u64)).to_string();
            }
            let c64 = match &p.children[i] {
                Some(r) => r.as_ref() as *const Node<K, V, N> as usize as u64,
                None => 0u64,
            };
            println!("      - [K{i}] {k64} [V{i}] {v64} [ch{i}] {c64:x}");
        }

        for i in 0..=cc {
            if let Some(ch) = &mut p.children[i] {
                Self::debug_r(ch);
            }
        }
    }

    /// Accesses mutable reference to value with key.
    ///
    /// The given value is not guaranteed to be safe across threads. We suggest
    /// manual safety implications.
    unsafe fn access(&mut self, key: &K) -> Option<&mut V> {
        let mut p = self.root.as_mut().unwrap().as_mut();
        'recurse: loop {
            let count = p.count as usize;
            // iterate all keys (as separators)
            for i in 0..count {
                let sep = p.keys[i].assume_init_ref();
                if key == sep {
                    // match key
                    return Some(p.values[i].assume_init_mut());
                } else if key < sep {
                    // goto left-hand-side
                    match &mut p.children[i] {
                        None => return None,
                        Some(child) => p = child.as_mut(),
                    }
                    continue 'recurse;
                }
            }
            // key is greater than all separators, goto THE right-hand-side
            match &mut p.children[count] {
                None => return None,
                Some(child) => p = child.as_mut(),
            }
        }
        // technically unreachable
    }

    /// Insert key-value pair.
    unsafe fn insert_wrap(&mut self, k: K, v: V) -> Option<()> {
        match Self::insert_r(&mut self.root.as_mut().unwrap(), k, v) {
            InsertResult::Split { key, value, rchild } => {
                let mut p = Node::new();
                p.count = 1;
                p.keys[0].write(key);
                p.values[0].write(value);
                p.children[0] = mem::take(&mut self.root);
                p.children[1] = rchild;
                self.root = Some(p);
                None
            }
            InsertResult::Replaced => Some(()),
            InsertResult::Finished => None,
        }
    }

    /// Insert recursively a key-value pair into a given node. Nodes are split
    /// on the way while backtracing.
    ///
    /// The return value is a custom result indicating if an additional
    /// key-value pair had been inserted to the parent node as a result of the
    /// direct child being split.
    unsafe fn insert_r(p: &mut Node<K, V, N>, mut key: K, mut value: V) -> InsertResult<K, V, N> {
        // find the index at where to insert:
        //     p.key[idx - 1] < key < p.key[idx]
        //     inserting new child at p.child[idx]
        let mut idx: usize = 0;
        let count = p.count as usize;
        while idx < count {
            let k = p.keys[idx].assume_init_ref();
            if &key < k {
                break;
            } else if &key == k {
                let _drop_value = mem::replace(p.values[idx].assume_init_mut(), value);
                return InsertResult::Replaced;
            }
            idx += 1;
        }

        // insert to between 2 keys or recursively split a leaf
        let mut rchild: Option<Box<Node<K, V, N>>> = None;
        if let Some(child) = &mut p.children[idx] {
            match Self::insert_r(child, key, value) {
                InsertResult::Split {
                    key: k,
                    value: v,
                    rchild: rc,
                } => {
                    // we still need to insert the newly split median
                    key = k;
                    value = v;
                    rchild = rc;
                }
                InsertResult::Replaced => return InsertResult::Replaced,
                InsertResult::Finished => return InsertResult::Finished,
            }
        }

        // space is enough, insert only, no splitting required
        if count + 1 <= N * 2 {
            // shift stuff right
            Self::slice_shr(&mut p.keys[idx..=count], 1);
            Self::slice_shr(&mut p.values[idx..=count], 1);
            Self::slice_shr_with(&mut p.children[idx + 1..=count + 1], 1, None);
            // insert leaf
            p.keys[idx].write(key);
            p.values[idx].write(value);
            p.children[idx + 1] = rchild;

            p.count += 1;
            return InsertResult::Finished;
        }

        // new left node: `p` (in-place); new right node: `q` (created)
        let mut q_box = Node::<K, V, N>::new();
        let q = &mut q_box;
        let mut median_key = MaybeUninit::<K>::uninit();
        let mut median_value = MaybeUninit::<V>::uninit();

        // perform node split depending on insert location
        if idx < N {
            // insert new node at left child
            //
            //      0 1 2 3 4 5 6 7 8 9    [insert X @ idx = 1]
            //     0 1 2 3 4 5 6 7 8 9 a
            //
            //  0 X 1 2 3   [4]   5 6 7 8 9
            // 0(1)R 2 3 4       5 6 7 8 9 a
            Self::slice_copy(&mut p.keys[N..N * 2], &mut q.keys[0..N]);
            Self::slice_copy(&mut p.values[N..N * 2], &mut q.values[0..N]);
            Self::slice_copy(&mut p.children[N..=N * 2], &mut q.children[0..=N]);

            median_key = mem::replace(&mut p.keys[N - 1], mem::MaybeUninit::uninit());
            median_value = mem::replace(&mut p.values[N - 1], mem::MaybeUninit::uninit());

            Self::slice_shr(&mut p.keys[idx..N], 1);
            Self::slice_shr(&mut p.values[idx..N], 1);
            Self::slice_shr_with(&mut p.children[idx + 1..=N], 1, rchild);

            p.keys[idx].write(key);
            p.values[idx].write(value);
        } else if idx == N {
            // insert node at median position
            //
            //      0 1 2 3 4 5 6 7 8 9    [insert X @ idx = 5]
            //     0 1 2 3 4 5 6 7 8 9 a
            //
            //  0 1 2 3 4   [X]   5 6 7 8 9
            // 0 1 2 3 4(5)      R 6 7 8 9 a
            Self::slice_copy(&mut p.keys[N..N * 2], &mut q.keys[0..N]);
            Self::slice_copy(&mut p.values[N..N * 2], &mut q.values[0..N]);
            Self::slice_copy(&mut p.children[N + 1..=N * 2], &mut q.children[1..=N]);

            median_key.write(key);
            median_value.write(value);

            q.children[0] = rchild;
        } else {
            // insert new node at right child
            //
            //      0 1 2 3 4 5 6 7 8 9    [insert X @ idx = 8]
            //     0 1 2 3 4 5 6 7 8 9 a
            //
            //  0 1 2 3 4   [5]   6 7 X 8 9
            // 0 1 2 3 4 5       6 7(8)R 9 a
            median_key = mem::replace(&mut p.keys[N], mem::MaybeUninit::uninit());
            median_value = mem::replace(&mut p.values[N], mem::MaybeUninit::uninit());

            Self::slice_copy(&mut p.keys[N + 1..idx], &mut q.keys[0..idx - N - 1]);
            Self::slice_copy(&mut p.values[N + 1..idx], &mut q.values[0..idx - N - 1]);
            Self::slice_copy(
                &mut p.children[N + 1..=idx],
                &mut q.children[0..=idx - N - 1],
            );

            q.keys[idx - N - 1].write(key);
            q.values[idx - N - 1].write(value);
            q.children[idx - N] = rchild;

            Self::slice_copy(&mut p.keys[idx..N * 2], &mut q.keys[idx - N..N]);
            Self::slice_copy(&mut p.values[idx..N * 2], &mut q.values[idx - N..N]);
            Self::slice_copy(
                &mut p.children[idx + 1..=N * 2],
                &mut q.children[idx - N + 1..=N],
            );
        }

        // cleanup old children references and reset counters
        for i in N + 1..=count {
            ptr::write(&mut p.children[i] as *mut Option<Box<Node<K, V, N>>>, None);
        }
        p.count = N as u16;
        q.count = N as u16;

        InsertResult::Split {
            key: median_key.assume_init(),
            value: median_value.assume_init(),
            rchild: Some(q_box),
        }
    }

    /// Shift every item on `slice` right `offset` indices in-place. Items
    /// exceeding this area will not be overwritten.
    ///
    /// If the shift guarantees non-overlapping, `slice_move` should be used
    /// instead for performance implications.
    #[inline]
    unsafe fn slice_shr<T>(slice: &mut [MaybeUninit<T>], offset: usize) -> () {
        if slice.len() < offset {
            return;
        }

        let slice_ptr = slice.as_mut_ptr();
        ptr::copy(slice_ptr, slice_ptr.add(offset), slice.len() - offset);
    }

    /// Shift every item on `slice` right `offset` indices in-place. Items
    /// exceeding this area will not be overwritten.
    ///
    /// The leftmost item is overwritten with an additional value `overwrite`.
    /// The last item is not dropped.
    ///
    /// If the shift guarantees non-overlapping, `slice_move` should be used
    /// instead for performance implications.
    #[inline]
    unsafe fn slice_shr_with<T>(slice: &mut [T], offset: usize, overwrite: T) -> () {
        if slice.len() < offset {
            return;
        }

        let slice_ptr = slice.as_mut_ptr();
        ptr::copy(slice_ptr, slice_ptr.add(offset), slice.len() - offset);
        ptr::write(slice_ptr, overwrite);
    }

    /// Copies data on `src` to `dest`. The two slices must be manually
    /// guaranteed to be non-overlapping.
    ///
    /// Undefined behaviour might occur if this limit is broken.
    #[inline]
    unsafe fn slice_copy<T>(src: &mut [T], dest: &mut [T]) -> () {
        assert_eq!(src.len(), dest.len());

        ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr(), src.len());
    }
}

#[cfg(test)]
pub mod tests {
    use super::BTreeImpl;
    use crate::memtable::MemTable;
    use std::collections::BTreeMap;

    fn expect_ok<const N: usize>(at: u64)
    where
        [(); N * 2]: Sized,
        [(); N * 2 + 1]: Sized,
    {
        let n = N as u64;
        let mut mp = BTreeImpl::<u64, u64, N>::new();
        for i in 0u64..=n * 2 {
            if i != at {
                mp.insert(i, i * 233 + 2333);
            }
        }
        mp.insert(at as u64, at * 233 + 2333 as u64);
        for key in 0u64..=n * 2 {
            let value = match mp.get(&key) {
                Some(&mut x) => x,
                None => 11,
            };
            assert_eq!(value, key * 233 + 2333);
        }
    }

    #[test]
    fn left_insert() {
        for i in 0..=6 {
            expect_ok::<7>(i);
        }
    }

    #[test]
    fn median_insert() {
        expect_ok::<7>(7);
    }

    #[test]
    fn right_insert() {
        for i in 8..=14 {
            expect_ok::<7>(i);
        }
    }

    /// Memory leak testing utilities
    #[derive(PartialOrd, Ord, PartialEq, Eq)]
    struct DroppableI64 {
        value: i64,
        counter: *mut i64,
    }

    impl Drop for DroppableI64 {
        fn drop(&mut self) -> () {
            unsafe { *self.counter += self.value }
            let v = self.value;
            println!("dropping {v}");
        }
    }

    impl DroppableI64 {
        fn from(value: i64, counter_ref: *mut i64) -> Self {
            Self {
                value: value,
                counter: counter_ref,
            }
        }
    }

    #[test]
    pub fn no_memory_leak() {
        // counter for drop references
        let mut drop_counter = 0i64;
        let ctr_ref = &mut drop_counter as *mut i64;

        // ensure that the drop counter is working properly
        {
            let mut mp = BTreeMap::<u64, DroppableI64>::new();
            mp.insert(10, DroppableI64::from(1, ctr_ref));
            mp.insert(20, DroppableI64::from(3, ctr_ref));
            mp.insert(30, DroppableI64::from(7, ctr_ref));
            mp.insert(40, DroppableI64::from(15, ctr_ref));
            mp.insert(50, DroppableI64::from(31, ctr_ref));
        }
        assert_eq!(drop_counter, 57);

        // check validity on tree
        {
            let mut mp = BTreeImpl::<u64, DroppableI64, 2>::new();
            mp.insert(10, DroppableI64::from(1000, ctr_ref));
            mp.insert(20, DroppableI64::from(3000, ctr_ref));
            mp.insert(30, DroppableI64::from(7000, ctr_ref));
            mp.insert(40, DroppableI64::from(15000, ctr_ref));
            mp.insert(50, DroppableI64::from(31000, ctr_ref));
        }
        assert_eq!(drop_counter, 57057);
    }

    #[test]
    fn stress_test() {
        // ensure it works on big data
        let loops: u64 = 23333;
        let mut mp = BTreeImpl::<u64, u64, 5>::new();
        for key in 1..=loops {
            mp.insert(key, key * 2 + 1);
        }
        for key in 1..=loops {
            let value = match mp.get(&key) {
                Some(&mut x) => x,
                None => 0,
            };
            assert_eq!(value, key * 2 + 1);
        }
    }
}
