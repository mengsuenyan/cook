mod alloc;

pub use alloc::{Alloc, AllocErr};

#[derive(Copy, Clone)]
pub struct DefaultAllocator;

unsafe  impl Alloc for DefaultAllocator {
    #[inline]
    fn new() -> Self {
        DefaultAllocator {}
    }
}