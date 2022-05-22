use crate::record::KvDataRef;

use super::KvEntry;

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
    /// ensured.
    fn value_mut(&self) -> &mut KvEntry;
}
