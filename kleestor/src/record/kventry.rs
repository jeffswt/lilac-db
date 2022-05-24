use crate::record::ByteStream;
use crate::utils::futures::Mutex;

/// A level-0 data record that is stored within memory.
///
/// The key is stored outside the entry.
pub struct KvEntry {
    /// Isolate threads from each other while tampering with metadata.
    pub lock: Mutex<()>,

    /// Read timestamp, as defined in the TS-based MVCC.
    pub ts_read: u64,

    /// Write timestamp, as defined in the TS-based MVCC.
    pub ts_write: u64,

    /// Content of the entry.
    pub record: KvData,
}

impl KvEntry {
    pub fn new(record: KvData) -> Self {
        Self {
            lock: Mutex::<()>::new(()),
            ts_read: 0_u64,
            ts_write: 0_u64,
            record,
        }
    }
}

/// A record is either deleted or re-applied to that value.
pub enum KvData {
    /// The key-value pair is marked as deleted at this record.
    Tombstone { cached: bool },

    /// The record contains a key-value pair.
    Value { cached: bool, value: ByteStream },
}

impl Clone for KvData {
    fn clone(&self) -> Self {
        match self {
            KvData::Tombstone { cached } => KvData::Tombstone { cached: *cached },
            KvData::Value { cached, value } => KvData::Value {
                cached: *cached,
                value: ByteStream::from(value),
            },
        }
    }
}

impl From<&KvDataRef> for KvData {
    fn from(other: &KvDataRef) -> Self {
        match other {
            KvDataRef::Tombstone { cached } => KvData::Tombstone { cached: *cached },
            KvDataRef::Value { cached, value } => KvData::Value {
                cached: *cached,
                value: ByteStream::from(*value),
            },
        }
    }
}

/// Reference to `KvData`.
pub enum KvDataRef {
    Tombstone { cached: bool },
    Value { cached: bool, value: &'static [u8] },
}

impl Clone for KvDataRef {
    fn clone(&self) -> Self {
        match self {
            KvDataRef::Tombstone { cached } => KvDataRef::Tombstone { cached: *cached },
            KvDataRef::Value { cached, value } => KvDataRef::Value {
                cached: *cached,
                value,
            },
        }
    }
}
