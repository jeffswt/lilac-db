pub mod futures;

#[inline]
pub unsafe fn const_as_mut<T>(item: &T) -> &mut T {
    &mut *(item as *const T as *mut T)
}
