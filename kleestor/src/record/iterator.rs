use super::ByteStream;

/// Key-value iterator (pointer) interface.
pub trait KvPointer {
    /// Get key where iterator points to.
    fn key(&self) -> &ByteStream;

    /// Gets a static reference to the pointing value.
    fn value(&self) -> &ByteStream;

    /// Gets a mutable reference to the pointing value.
    fn value_mut(&self) -> &mut ByteStream;
}
