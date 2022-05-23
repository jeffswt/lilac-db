use crate::record::{KvDataRef, KvEntry};
use std::fmt::Formatter;

/// Key-value iterator (pointer) interface.
pub trait KvPointer {
    /// Get key where iterator points to.
    ///
    /// This reference MAY invalidate after Pointer de-initialization, unlike
    /// that values SHOULD live as long as the data structure does.
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

    /// You may wrap a custom 'Display' trait over this function.
    fn _fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        let key = self.key();
        let ks = String::from_utf8(Vec::from(key)).unwrap_or(String::from("????"));

        let value = self.value();
        match value {
            KvDataRef::Tombstone { cached: false } => {
                fmt.write_fmt(format_args!("'{}' -> removed", ks))
            }
            KvDataRef::Tombstone { cached: true } => {
                fmt.write_fmt(format_args!("'{}' -> removed [cached]", ks))
            }
            KvDataRef::Value {
                cached: false,
                value,
            } => {
                let vs = String::from_utf8(Vec::from(value)).unwrap_or(String::from("????"));
                fmt.write_fmt(format_args!("'{}' -> '{}'", ks, vs))
            }
            KvDataRef::Value {
                cached: true,
                value,
            } => {
                let vs = String::from_utf8(Vec::from(value)).unwrap_or(String::from("????"));
                fmt.write_fmt(format_args!("'{}' -> '{}' [cached]", ks, vs))
            }
        }
    }

    /// Default ToString implementation.
    fn _to_string(&self) -> String {
        let mut buf = String::new();
        let mut formatter = Formatter::new(&mut buf);
        self._fmt(&mut formatter).unwrap();
        buf
    }
}
