use crate::lsmt::entry::{KvData, KvEntry};
use crate::record::ByteStream;
use std::cmp::max;
use std::collections::BTreeMap;
use std::mem;
use std::sync::Mutex;

/// A wrapped implementation of the transaction manager. Usage must guarantee
/// that earlier transactions get introduced to this manager before later
/// transactions.
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

    /// Redo log for written values (particularly writes).
    pub redo: Vec<(*mut KvEntry, u64, Option<KvData>)>,

    /// Transaction dependencies.
    pub deps: Vec<u64>,
}

impl TransactionMgrImpl {
    /// Creates a transaction.
    pub unsafe fn create(&mut self, ts: u64) -> &mut Transaction {
        let mut trans = Box::from(Transaction {
            ts: ts,
            redo: vec![],
            deps: vec![],
        });
        // acquire lock for metadata access
        let _lock = match self.lock.lock() {
            Err(_) => panic!("fail to access transaction manager lock"),
            Ok(l) => l,
        };
        // insert transaction and return reference
        let refer = trans.as_mut() as *mut Transaction;
        self.ongoing_trans.insert(ts, trans);
        &mut *refer
    }

    /// Acquires a lock for read-only operation over k-v entry.
    ///
    /// This operation is necessary before actually `read`ing any value.
    pub unsafe fn read_lock(
        &mut self,
        trans: &mut Transaction,
        entry: &mut KvEntry,
    ) -> Result<(), ()> {
        // acquire lock over entry metadata access
        let _lock = match entry.lock.lock() {
            Err(_) => return Err(()),
            Ok(l) => l,
        };
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
        assert!(trans.ts < entry.ts_read);

        match &entry.record {
            KvData::Tombstone { .. } => None,
            KvData::Value { value, .. } => Some(&value),
        }
    }

    /// Write data to target entry.
    ///
    /// Throws an error if an abort is required.
    pub unsafe fn write(
        &mut self,
        trans: &mut Transaction,
        entry: &mut KvEntry,
        data: KvData,
    ) -> Result<(), ()> {
        // acquire lock over entry metadata access
        let entry_ptr = entry as *mut KvEntry;
        let _lock = match entry.lock.lock() {
            Err(_) => return Err(()),
            Ok(l) => l,
        };
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
    pub unsafe fn readwrite_lock(
        &mut self,
        trans: &mut Transaction,
        entry: &mut KvEntry,
    ) -> Result<(), ()> {
        // acquire lock over entry metadata access
        let entry_ptr = entry as *mut KvEntry;
        let _lock = match entry.lock.lock() {
            Err(_) => return Err(()),
            Ok(l) => l,
        };
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
}
