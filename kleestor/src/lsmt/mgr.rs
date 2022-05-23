use crate::lsmt::transimpl::{Transaction, TransactionMgrImpl};
use crate::memtable::rbtree::RBTree;
use crate::memtable::MemTable;
use crate::record::{ByteStream, KvData, KvEntry};
use crate::sstable::reader::SSTableReader;
use crate::utils;
use crate::utils::futures::RwLock;
use std::cmp::Ordering;

pub struct LsmTree {
    /// Transaction manager.
    trans: TransactionMgrImpl,

    /// Level 0 of the LSM tree, a read-write mapping.
    lv0: RBTree<ByteStream, KvEntry>,

    /// A read-write lock denying conflict access to lv0 structure.
    lv0_lock: RwLock<()>,

    /// Level 1 contains a series of red-black trees pending flush to level 2.
    /// New trees must be pushed to the front (i.e. lower index means newer
    /// data).
    lv1: Vec<RBTree<ByteStream, KvEntry>>,

    /// Removal (or insertion) of level 1 structures should be exclusive. The
    /// granularity may be arbitrarily large, as long as it does not block
    /// access to the merger.
    lv1_lock: RwLock<()>,

    /// More levels incoming.
    lvrest: Vec<(SSLoc, SSTableReader)>,

    /// Also need to lock lvrest when merging.
    lvrest_lock: RwLock<()>,
}

impl LsmTree {
    /// Create transaction.
    pub async fn tr_create(&mut self, ts: u64) -> TransactionToken {
        unsafe {
            let trans = self.trans.create(ts).await;
            TransactionToken { _trans: trans }
        }
    }

    /// Locks transaction resource as read-only at given key.
    pub async fn tr_lock_ro(
        &mut self,
        token: &TransactionToken,
        key: &ByteStream,
    ) -> Result<(), ()> {
        unsafe {
            // access entry in tree
            let entry = {
                let _lock = self.lv0_lock.read().await;
                match self.lv0.get(key) {
                    None => return Ok(()), // doesn't even exist in l0, safely read
                    Some(ent) => ent,
                }
            };
            let trans = &mut *token._trans;
            self.trans.read_lock(trans, entry).await
        }
    }

    /// Locks transaction resource as read-write at given key. It is strongly
    /// advised that you lock r/w even if the resource is accessed as write
    pub async fn tr_lock_rw(
        &mut self,
        token: &TransactionToken,
        key: &ByteStream,
    ) -> Result<(), ()> {
        unsafe {
            // access entry in tree
            let entry = {
                let _lock = self.lv0_lock.read().await;
                // try and find existing pair
                match self.lv0.get(key) {
                    Some(ent) => ent,
                    None => {
                        // insert new pair
                        self.lv0.insert(
                            ByteStream::from(key),
                            KvEntry::new(KvData::Value {
                                cached: false,
                                value: ByteStream::new(),
                            }),
                        );
                        // and return the inserted
                        self.lv0.get(key).unwrap()
                    }
                }
            };
            let trans = &mut *token._trans;
            self.trans.readwrite_lock(trans, entry).await
        }
    }

    /// Wait for pending resources to complete. Abort is required upon failure.
    pub async fn tr_wait(&mut self, token: &TransactionToken) -> Result<(), ()> {
        unsafe {
            let trans = &mut *token._trans;
            self.trans.wait(trans).await
        }
    }

    /// Commit transaction. You should no longer be holding anything related to
    /// this transaction anymore (which explains why it's been consumed).
    pub async fn tr_commit(&mut self, token: TransactionToken) -> () {
        unsafe {
            let trans = &mut *token._trans;
            self.trans.commit(trans).await;
            self.trans.remove_trans(trans).await;
        }
    }

    /// Abort transaction. You should no longer be holding anything related to
    /// this transaction anymore (which explains why it's been consumed).
    pub async fn tr_abort(&mut self, token: TransactionToken) -> () {
        unsafe {
            let trans = &mut *token._trans;
            self.trans.abort(trans).await;
            self.trans.remove_trans(trans).await;
        }
    }

    /// Access a value outside a transaction.
    pub async fn raw_get(&mut self, key: &[u8]) -> Option<ByteStream> {
        // crappy design of memtables...
        let key_bs = ByteStream::from(key);

        // lookup lv0
        '_lv0: {
            let _lock = self.lv0_lock.read().await;
            match self.lv0.get(&key_bs) {
                Some(entry) => match &entry.record {
                    KvData::Tombstone { .. } => return None,
                    KvData::Value { value, .. } => return Some(ByteStream::from(value)),
                },
                None => (),
            }
        }
        // lookup lv1
        '_lv1: {
            let _lock = self.lv1_lock.read().await;
            for table in &self.lv1 {
                // rbtree actually needs const ref only
                match unsafe { utils::const_as_mut(table) }.get(&key_bs) {
                    Some(entry) => match &entry.record {
                        KvData::Tombstone { .. } => return None,
                        KvData::Value { value, .. } => return Some(ByteStream::from(value)),
                    },
                    None => (),
                };
            }
        }
        // lookup sstables
        '_lvrest: {
            let _lock = self.lvrest_lock.read().await;
            for (_loc, ss) in &self.lvrest {
                match ss.get(key) {
                    Some(crate::record::KvData::Tombstone { .. }) => return None,
                    Some(crate::record::KvData::Value { value, .. }) => return Some(value),
                    None => (),
                };
            }
        }
        None
    }

    /// Modify value outside a transaction. This will break existing references
    /// to this value.
    pub async fn raw_insert(&mut self, key: ByteStream, value: ByteStream) -> () {
        let record = KvData::Value {
            cached: false,
            value,
        };
        let _lock = self.lv0_lock.write().await;
        self.lv0.insert_internal(key, record);
    }
}

/// Friendly RAII token for holding a transaction object.
pub struct TransactionToken {
    _trans: *mut Transaction,
}

/// Location of an SSTable. When comparing [`SSLoc`]s, the smaller one is
/// always the newer one.
#[derive(PartialEq, Eq, Ord)]
struct SSLoc {
    /// Tier. The larger it gets, the older it is.
    tier: u32,

    /// Run. In one tier, the larger the run is, the newer it is.
    run: u32,
}

impl PartialOrd for SSLoc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let result = match (
            self.tier.partial_cmp(&other.tier),
            self.run.partial_cmp(&other.run),
        ) {
            (Some(Ordering::Less), _) => Ordering::Less,
            (Some(Ordering::Equal), Some(Ordering::Greater)) => Ordering::Less,
            (Some(Ordering::Equal), Some(Ordering::Equal)) => Ordering::Equal,
            (Some(Ordering::Equal), Some(Ordering::Less)) => Ordering::Greater,
            (Some(Ordering::Greater), _) => Ordering::Greater,
            _ => unreachable!(),
        };
        Some(result)
    }
}
