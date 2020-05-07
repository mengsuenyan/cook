//! 连续线性缓存空间  
//! 

use crate::alloc::{Alloc, DefaultAllocator};
use std::ptr::NonNull;
use std::alloc::Layout;
use std::mem;

pub struct LinearBuf<T, A: Alloc = DefaultAllocator> {
    ptr: NonNull<T>,
    cap: usize,
    a: A,
}

impl<T, A: Alloc> LinearBuf<T, A> {
    
    /// 分配`size = size_of::<T>() * want_size`字节的内存  
    /// 分配内存失败会panic  
    /// 分配的内存至少足够保存want_size个数据, 但不一定等于want_size.  
    pub fn new(want_size: usize) -> Self {
        let size = if want_size == 0 { want_size + 1 } else { want_size };
        
        let layout = Layout::from_size_align(size * mem::size_of::<T>(), mem::align_of::<T>()).unwrap();
        let mut a = A::new();
        let ptr = unsafe {
            a.alloc(layout).unwrap().cast::<T>()
        };
        
        LinearBuf {
            ptr,
            cap: size,
            a
        }
    }
    
    /// 改变缓存的容量;  
    /// 不保证缓存还是在原来的位置, 也不保证缓存的容量是want_size, 但是能够保证
    /// 缓存的容量至少能存储want_size个T数据;  
    pub fn resize(&mut self, want_size: usize) {
        let size = if want_size == 0 { want_size + 1} else {want_size};
        let size= size * mem::size_of::<T>();
        let layout = Layout::from_size_align(size, mem::align_of::<T>()).unwrap();
        let new_ptr = unsafe {
            self.a.realloc(self.ptr.cast::<u8>(), layout, size).unwrap().cast::<T>()
        };
        
        self.ptr = new_ptr;
    }
    
    /// 缓冲能缓存T类型数据的个数
    pub fn capacity(&self) -> usize {
        self.cap
    }
    
    /// 获取指向该缓存的指针  
    fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    
    /// 读取缓存pos位置出的数据;  
    /// 如果该位置从来没有写过数据, 那么读出来的可能是随机数据;  
    pub fn read(&self, pos: usize) -> Option<T> {
        if self.capacity() <= pos {
            None
        } else {
            unsafe  {
                Some(self.as_ptr().add(pos).read())
            }
        }
    }
    
    /// 向缓存pos位置处写数据;  
    pub fn write(&self, pos: usize, val: T) -> Result<(), &str> {
        if self.capacity() <= pos {
            Err("Write beyond boundary.")
        } else {
            unsafe {
                self.as_ptr().add(pos).write(val)
            }
            Ok(())
        }
    }
    
    /// 将缓存前count个数据位置都填写为val;  
    /// 若count超过缓存容量, 仅会将整个缓存每个字节写为val, 不会越界操作;  
    pub fn write_bytes(&self, val: u8, count: usize) {
        let count = if self.capacity() < count {self.capacity()} else { count };
        unsafe {
            self.as_ptr().write_bytes(val, count);
        }
    }
    
    /// 从缓存的pos位置开始将count个位置处每个字节都写为val;  
    /// pos+count超出缓存容量, 不会越界操作, 仅写pos后的所有缓存;  
    pub fn write_bytes_from(&self, val: u8, pos: usize, count: usize) {
        if pos < self.capacity() {
            let count = if self.capacity() < pos + count {
                self.capacity() - pos
            } else {
                count
            };
            
            unsafe  {
                self.as_ptr().add(pos).write_bytes(val, count);
            }
        }
    }
}

impl<T, A: Alloc> Clone for LinearBuf<T, A> {
    
    fn clone(&self) -> Self {
        let lb = LinearBuf::new(self.capacity());
        unsafe {
            self.as_ptr().copy_to(lb.as_ptr(), lb.capacity());
        }
        
        lb
    }
}

impl<T> Copy for LinearBuf<T, DefaultAllocator> {}
