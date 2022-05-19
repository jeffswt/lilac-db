use crate::record::{KvData, KvDataRef};

/// Key-value iterator (pointer) interface.
pub trait KvPointer {
    /// Get key where iterator points to.
    fn key(&self) -> &[u8];

    /// Gets a reference to the pointing value.
    ///
    /// You should expect this reference to invalidate as soon as the pointer
    /// left this region.
    fn value(&self) -> KvDataRef;

    /// Gets a mutable reference to the pointing value.
    ///
    /// This exposes the underlying implementation. Lifetime should be manually
    /// checked upon.
    fn value_mut(&self) -> &mut KvData;
}
