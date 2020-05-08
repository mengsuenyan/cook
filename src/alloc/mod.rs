mod alloc;

pub use alloc::{Alloc};
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct DefaultAllocator<T> {
    phantom: PhantomData<T>
}

impl<T> Alloc for DefaultAllocator<T> {
    type Item = T;
    
    #[inline]
    fn new() -> Self {
        DefaultAllocator {
            phantom: PhantomData
        }
    }
}