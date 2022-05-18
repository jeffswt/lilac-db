use super::ByteStream;

/// Key-value iterator (pointer) interface.
pub trait KvPointer {
    /// Get key where iterator points to.
    fn key(&self) -> &[u8];

    /// Gets a static reference to the pointing value.
    fn value(&self) -> &[u8];

    /// Gets a mutable reference to the pointing value.
    fn value_mut(&self) -> &mut ByteStream;
}
