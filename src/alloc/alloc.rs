//! Rust 1.42.0版本Alloc是nightly-only API, 这里包装一层已待将来使用.
//! 

use std::alloc::{Layout};
use std::fmt;
use std::ptr::NonNull;
use std::fmt::Formatter;

pub struct AllocErr;

impl fmt::Display for AllocErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("memory allocation failed")
    }
}

impl fmt::Debug for AllocErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("memory allocation failed")
    }
}

pub trait Alloc {
    type Item;
    
    fn new() -> Self;
    
    #[inline]
    fn alloc(&mut self, layout: Layout) -> Option<NonNull<Self::Item>> {
        unsafe {
            NonNull::new(std::alloc::alloc(layout).cast())
        }
    }
    
    #[inline]
    fn dealloc(&mut self, ptr: NonNull<Self::Item>, layout: Layout) {
        unsafe {
            std::alloc::dealloc(ptr.as_ptr().cast(), layout);
        }
    }

    #[inline]
    fn alloc_zeroed(&mut self, layout: Layout) -> Option<NonNull<Self::Item>> {
        unsafe {
            NonNull::new(std::alloc::alloc_zeroed(layout).cast())
        }
    }
    
    #[inline]
    fn realloc(&mut self, ptr: NonNull<u8>, layout: Layout, new_size: usize) -> Option<NonNull<Self::Item>> {
        unsafe {
            NonNull::new(std::alloc::realloc(ptr.as_ptr(), layout, new_size))
        }
    }
}
