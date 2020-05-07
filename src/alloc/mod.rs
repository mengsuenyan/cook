mod alloc;

pub use alloc::{Alloc, AllocErr};

pub struct GlobalAllocator;

unsafe  impl Alloc for GlobalAllocator {
    #[inline]
    fn new() -> Self {
        GlobalAllocator {}
    }
}