pub mod futures;
pub mod varint;

use std::mem;

#[inline]
pub unsafe fn const_as_mut<T>(item: &T) -> &mut T {
    &mut *(item as *const T as *mut T)
}

#[inline]
pub unsafe fn reborrow<T>(item: &T) -> &T {
    &*(mem::transmute::<*const T, *const T>(item as *const T))
} 

#[inline]
pub unsafe fn reborrow_arr<T>(item: &[T]) -> &'static [T] {
    &*(mem::transmute::<*const [T], *const [T]>(item as *const [T]))
} 

#[inline]
pub unsafe fn reborrow_mut<T>(item: &mut T) -> &mut T {
    &mut *(mem::transmute::<*mut T, *mut T>(item as *mut T))
} 
