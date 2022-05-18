use crate::lsmt::entry::{KvData, KvEntry};
use crate::record::ByteStream;
use crate::utils::const_as_mut;
use crate::utils::futures::{Mutex, Notify};
use std::cmp::max;
use std::collections::BTreeMap;
use std::mem;

/// A wrapped implementation of the transaction manager. Usage must guarantee
/// that earlier transactions get introduced to this manager before later
/// transactions.
///
/// Locks must follow the following order (lock granularity increases) to avoid
/// deadlocks:
///
///     KvEntry -> Transaction -> TransManager
pub struct TransactionMgrImpl {
    /// Unique lock on accessing transaction list.
    lock: Mutex<()>,

    /// List of ongoing transactions.
    ongoing_trans: BTreeMap<u64, Box<Transaction>>,
}

/// Data storing the transaction.
pub struct Transaction {
    /// Unique transaction timestamp.
    pub ts: u64,

    /// This lock must be acquired before accessing metadata relating to this
    /// specific transaction.
    pub lock: Mutex<()>,

    /// Redo log for written values (particularly writes).
    ///
    /// Contains the fields (entry reference, previous transaction id or
    /// timestamp, previous data). The third field is set to [`None`] if
    /// nothing was ever overwritten.
    ///
    /// The redo log must be reverted in reverse order.
    pub redo: Vec<(*mut KvEntry, u64, Option<KvData>)>,

    /// Transaction dependencies.
    pub deps: Vec<u64>,

    /// Transaction state.
    pub state: TransactionState,

    /// A notification utility to notify waiting transactions.
    pub await_finish: Notify,

    /// Transactions awaiting on myself.
    pub await_clients: Vec<u64>,
}

/// State of the transaction.
#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug)]
pub enum TransactionState {
    /// Locking resources in premature.
    Idle,
    /// Waiting for pending resources.
    Waiting,
    /// Transaction successfully completed.
    Committed,
    /// Transaction executed partially with errors and is pending a rollback.
    Aborting,
    /// Transaction has rolled back.
    Aborted,
}

impl TransactionMgrImpl {
    /// Creates a transaction.
    pub async unsafe fn create(&mut self, ts: u64) -> &mut Transaction {
        let mut trans = Box::from(Transaction {
            ts: ts,
            lock: Mutex::new(()),
            redo: Vec::new(),
            deps: Vec::new(),
            state: TransactionState::Idle,
            await_finish: Notify::new(),
            await_clients: Vec::new(),
        });
        // acquire lock for metadata access
        let _lock = self.lock.lock().await;
        // insert transaction and return reference
        let refer = trans.as_mut() as *mut Transaction;
        self.ongoing_trans.insert(ts, trans);
        &mut *refer
    }

    /// Acquires a lock for read-only operation over k-v entry.
    ///
    /// This operation is necessary before actually `read`ing any value.
    pub async unsafe fn read_lock(
        &mut self,
        trans: &mut Transaction,
        entry: &mut KvEntry,
    ) -> Result<(), ()> {
        let _lock_e = entry.lock.lock().await;
        let _lock_t = trans.lock.lock().await;

        // rules forbid reading an outdated value (entry locked by another
        // transaction which might be modifying this value), which ought
        // trigger a rollback.
        if entry.ts_write > trans.ts {
            return Err(());
        }
        // since the transaction timestamp is a forever-increasing ordered
        // value, and a value will always be loaded into the memtable (with an
        // RTS preceding current TS), there is always a guarantee that we will
        // discover at least 1 record preceding the current TS.

        // update dependencies
        trans.deps.push(entry.ts_write);
        // register read timestamp
        entry.ts_read = max(entry.ts_read, trans.ts);
        // this entry is now marked as locked
        Ok(())
    }

    /// Read data from entry. You must lock that value as read-only or
    /// read-write beforehand.
    pub unsafe fn read<'a>(
        &'a mut self,
        trans: &'a Transaction,
        entry: &'a KvEntry,
    ) -> Option<&'a ByteStream> {
        // entry has been locked by timestamp and therefore does not need
        // a mutex lock

        assert!(trans.ts < entry.ts_read);

        match &entry.record {
            KvData::Tombstone { .. } => None,
            KvData::Value { value, .. } => Some(&value),
        }
    }

    /// Write data to target entry after locking that resource. It is safe to
    /// call this in either write-only mode or read-write mode.
    ///
    /// Throws an error if an abort is required.
    pub async unsafe fn write(
        &mut self,
        trans: &mut Transaction,
        entry: &mut KvEntry,
        data: KvData,
    ) -> Result<(), ()> {
        let entry_ptr = entry as *mut KvEntry;
        let _lock_e = entry.lock.lock().await;
        let _lock_t = trans.lock.lock().await;

        // an existing transaction depends on this value and thus writing it
        // should trigger an abort
        if entry.ts_read > trans.ts {
            return Err(());
        }
        // if the value had already been overwritten (in a future transaction),
        // we simply skip this value. note that this entry does not depend on
        // himself, thus is safe to just skip this write. if the transaction
        // involves actions such as 'entry += 233', it should be locked as
        // read-write instead.
        if entry.ts_write > trans.ts {
            return Ok(());
        }
        // save the old version
        let old_data = mem::replace(&mut entry.record, data);
        trans.redo.push((entry_ptr, entry.ts_write, Some(old_data)));
        // register write timestamp
        entry.ts_write = trans.ts;
        // this entry is now written
        Ok(())
    }

    /// Acquire a lock for read-write operation over a k-v entry. This is
    /// especially suitable for actions like transfering some data from one to
    /// another, or alike, for consistency guarantees.
    ///
    /// This operation is necessary before actually `readwrite`ing any value.
    pub async unsafe fn readwrite_lock(
        &mut self,
        trans: &mut Transaction,
        entry: &mut KvEntry,
    ) -> Result<(), ()> {
        let entry_ptr = entry as *mut KvEntry;
        let _lock_e = entry.lock.lock().await;
        let _lock_t = trans.lock.lock().await;

        // an existing transaction depends on this value and thus writing it
        // should trigger an abort
        if entry.ts_read > trans.ts {
            return Err(());
        }
        // future transactions have already accessed this value and should not
        // depend upon
        if entry.ts_write > trans.ts {
            return Err(());
        }
        // update dependencies
        trans.deps.push(entry.ts_write);
        // save the old timestamp
        trans.redo.push((entry_ptr, entry.ts_write, None));
        // register read & write timestamp
        entry.ts_read = max(entry.ts_read, trans.ts);
        entry.ts_write = trans.ts;
        // this entry is now locked as rw
        Ok(())
    }

    /// Waits for dependent pending transactions to finish or abort. This is
    /// returned as a result.
    ///
    /// Each transaction should call this exactly once before actually reading
    /// or writing any value.
    pub async unsafe fn wait(&mut self, trans: &mut Transaction) -> Result<(), ()> {
        // a list of futures that the current transaction is dependent upon
        let mut notifiers = Vec::new();

        // add myself to all dependent transactions
        // lock is held only for this period
        '_add_waiter: {
            let _lock_t = trans.lock.lock().await;
            let _lock_m = self.lock.lock().await;

            for dep_id in &trans.deps {
                if let Some(dep) = self.ongoing_trans.get(&dep_id) {
                    let dep = const_as_mut(dep.as_ref());
                    dep.await_clients.push(trans.ts);
                    notifiers.push(dep.await_finish.notified());
                }
            }
        }

        // wait for all those notifiers to finish and check transaction status
        futures::future::join_all(notifiers).await;

        // we won't need a global sync since it's been held up by the notifiers
        let _lock_t = trans.lock.lock().await;
        if let TransactionState::Aborting = trans.state {
            return Err(());
        }
        Ok(())
    }

    /// Manually complete a transaction, notifying dependent clients.
    pub async unsafe fn commit(&mut self, trans: &mut Transaction) -> Result<(), ()> {
        let _lock_t = trans.lock.lock().await;

        trans.state = TransactionState::Committed;
        trans.await_finish.notify_waiters();
        Ok(())
    }

    /// Manually abandon a transaction, reverting all executed changes and
    /// notify dependent clients.
    ///
    /// Clients which depend on this transaction would have their status set to
    /// [`TransactionState::Aborting`].
    ///
    /// A memtable reference is passed to this, alongside with an [`RwLock`]
    /// that ensures read-exclusive access to that memtable.
    pub async unsafe fn abort(&mut self, trans: &mut Transaction) -> () {
        let _lock_t = trans.lock.lock().await;

        // you can't just abort a committed transaction!
        assert_ne!(trans.state, TransactionState::Committed);

        // redo log must be read in reverse order
        let mut redo = mem::take(&mut trans.redo);
        redo.reverse();

        for (entry, ts_write, data) in redo {
            // this is safe if and only if memtable is readonly
            let entry = &mut *entry;
            let _lock_e = entry.lock.lock().await;

            if let Some(data) = data {
                entry.record = data;
            }
            entry.ts_write = ts_write;
        }

        // mark dependent clients as aborting
        '_abort_clients: {
            let _lock_m = self.lock.lock().await;
            for dep in &trans.await_clients {
                if let Some(dep) = self.ongoing_trans.get_mut(dep) {
                    // you can't technically depend on yourself...
                    assert_ne!(dep.ts, trans.ts);

                    // a dependency lock is not required since we'll notify
                    // them through the notifier later
                    if let TransactionState::Waiting = dep.state {
                        dep.state = TransactionState::Aborting;
                    }
                }
            }
        }
        trans.state = TransactionState::Aborted;
        trans.await_finish.notify_waiters();
    }

    /// Transaction removed from the data structure, and references should be
    /// no longer considered valid.
    pub async unsafe fn remove_trans(&mut self, trans: &mut Transaction) -> () {
        let _lock_m = self.lock.lock().await;

        self.ongoing_trans.remove(&trans.ts);
    }
}
