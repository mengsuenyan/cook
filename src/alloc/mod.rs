mod alloc;

pub use alloc::{Alloc};

#[derive(Copy, Clone)]
pub struct DefaultAllocator<T>;

impl Alloc<T> for DefaultAllocator<T> {
    type Item = T;
    
    #[inline]
    fn new() -> Self {
        DefaultAllocator {}
    }
}