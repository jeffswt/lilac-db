use crate::record::ByteStream;

/// A record is either deleted or re-applied to that value.
pub enum KvData {
    /// The key-value pair is marked as deleted at this record.
    Tombstone { cached: bool },

    /// The record contains a key-value pair.
    Value { cached: bool, value: ByteStream },
}

/// Reference to `KvData`.
pub enum KvDataRef<'a> {
    Tombstone { cached: bool },
    Value { cached: bool, value: &'a [u8] },
}
