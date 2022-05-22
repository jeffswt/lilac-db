use crate::record::{KvDataRef, KvEntry};

/// Key-value iterator (pointer) interface.
pub trait KvPointer {
    /// Get key where iterator points to.
    ///
    /// It should be reminded that the resulting reference must live as long as
    /// the data structure does.
    fn key(&self) -> &[u8];

    /// Gets a reference to the pointing value.
    ///
    /// This reference should try to live as long as the data structure does,
    /// given that the value itself was not overwritten. Therefore additional
    /// measures (e.g. MVCC, locks) must be taken so that the value is not
    /// freed while it is in the hands of another iterator. 
    fn value(&self) -> KvDataRef;

    /// Gets a mutable reference to the pointing value.
    ///
    /// This exposes the underlying implementation. Expect reference to
    /// invalidate after pointer leaves scope.
    ///
    /// This function may be unimplemented since certain data structures are
    /// opened readonly.
    fn value_mut(&self) -> &mut KvEntry;
}
