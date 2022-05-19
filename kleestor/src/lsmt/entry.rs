use crate::record::KvData;
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
