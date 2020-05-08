//! 连续线性缓存空间  
//! 

use crate::alloc::{Alloc, DefaultAllocator};
use crate::gds::CapacityStrategy;
use std::ptr::NonNull;
use std::alloc::Layout;
use std::mem;

pub struct LinearBuf<T, A: Alloc = DefaultAllocator<T>> {
    buf: Option<NonNull<T>>,
    cap: usize,
    a: A,
    layout: Option<Layout>,
}

impl<T, A: Alloc<Item=T>> LinearBuf<T, A> {
    
    /// 分配`size = size_of::<T>() * want_size`字节的内存  
    pub fn new(want_size: usize) -> Self {
        let (size_, align) = (want_size * mem::size_of::<T>(), mem::align_of::<T>());
        let mut a = A::new();
        let (buf, size_, layout) = if size_ == 0 {
            (None, 0, None)
        } else {
            match Layout::from_size_align(size_, align) {
                Ok(layout) => (a.alloc(layout), layout.size() / mem::size_of::<T>(), Some(layout)),
                _ => (None, 0, None),
            }
        };

        let cap = if buf.is_some() { size_ } else { 0 };
        
        LinearBuf {
            buf,
            cap,
            a,
            layout,
        }
    }
    
    pub fn dealloc_buf(&mut self) {
        if self.is_valid() {
            self.a.dealloc(self.buf.unwrap(), self.layout.unwrap());
            self.cap = 0;
            self.layout = None;
            self.buf = None;
        }
    }
    
    /// want_size <= self.capacity(), 内存不会改变;  
    /// want_size > self.capacity(), 增加的容量会填充为0
    pub fn resize(&mut self, want_size: usize) {
        if want_size > self.capacity() {
            let (size_, align) = (mem::size_of::<T>() * want_size, mem::align_of::<T>());
            
            if self.is_valid() {
                match self.a.realloc(self.buf.unwrap(), self.layout.unwrap(), size_) {
                    Some(new_ptr) => {
                        let old_cap = self.capacity();
                        self.buf = Some(new_ptr);
                        self.cap = want_size;
                        self.write_bytes_from(0, old_cap, self.capacity() - old_cap);
                    },
                    _ => {},
                }
            } else {
                match Layout::from_size_align(size_, align) {
                    Ok(layout) => {
                        self.buf = self.a.alloc_zeroed(layout);
                        if self.buf.is_some() {
                            self.layout = Some(layout);
                            self.cap = layout.size() / mem::size_of::<T>();
                        }
                    },
                    _ => {},
                }
            }
        }
    }
    
    fn need_size(&self, additional_size: usize, cs: CapacityStrategy) -> usize {
        match cs { 
            CapacityStrategy::OnDemand => self.capacity() + additional_size,
            CapacityStrategy::DoubleOnDemand => self.capacity() + (additional_size << 1),
            CapacityStrategy::DoubleCapacity => (self.capacity() + additional_size) << 1,
        }
    }
    
    /// 扩容的容量至少大于additional_size  
    pub fn reserve(&mut self, additional_size: usize, cs: CapacityStrategy) {
        let size_ = self.need_size(additional_size, cs);

        self.resize(size_);
    }
    
    /// 缓冲能缓存T类型数据的个数
    pub fn capacity(&self) -> usize {
        self.cap
    }
    
    /// 获取指向该缓存的指针  
    fn as_ptr(&self) -> *mut T {
        self.buf.unwrap().as_ptr()
    }
    
    #[inline]
    fn is_valid(&self) -> bool {
        self.buf.is_some()
    }

    /// 读取缓存pos(pos < self.capacity())位置出的数据;  
    #[inline]
    pub fn read(&self, pos: usize) -> Option<T> {
        if self.is_valid() && pos < self.capacity() {
            unsafe {
                Some(self.as_ptr().add(pos).read())
            }
        } else {
            None
        }
    }
    
    #[inline]
    pub fn read_as_ref<'a>(&self, pos: usize) -> Option<&'a T> {
        if self.is_valid() && pos < self.capacity() {
            unsafe {
                self.as_ptr().add(pos).as_ref()
            }
        } else {
            None
        }
    }
    
    #[inline]
    pub fn read_as_mut<'a>(&mut self, pos: usize) -> Option<&'a mut T> {
        if self.is_valid() && pos < self.capacity() {
            unsafe {
                self.as_ptr().add(pos).as_mut()
            }
        } else {
            None
        }
    }
    
    /// 向缓存pos位置处写数据;  
    pub fn write(&self, pos: usize, val: T) -> Result<(), &str> {
        if self.capacity() <= pos || !self.is_valid() {
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
        if self.is_valid() {
            let count = if self.capacity() < count {self.capacity()} else { count };
            unsafe {
                self.as_ptr().write_bytes(val, count);
            }
        }
    }
    
    /// 从缓存的pos位置开始将count个位置处每个字节都写为val;  
    /// pos+count超出缓存容量, 不会越界操作, 仅写pos后的所有缓存;  
    pub fn write_bytes_from(&self, val: u8, pos: usize, count: usize) {
        if self.is_valid() {
            if pos < self.capacity() {
                let count = if self.capacity() < pos + count {
                    self.capacity() - pos
                } else {
                    count
                };

                unsafe {
                    self.as_ptr().add(pos).write_bytes(val, count);
                }
            }
        }
    }
    
    pub fn write_from_vec(&mut self, pos: usize, v: &Vec<T>) {
        if self.is_valid() {
            if pos < self.capacity() {
                let count = if self.capacity() < pos + v.len() {
                    self.capacity() - pos
                } else {
                    v.len()
                };

                unsafe {
                    v.as_ptr().copy_to_nonoverlapping(self.as_ptr().add(pos), count);
                }
            }
        }
    }
    
    /// 交换两个缓存的内存
    pub fn swap(&mut self, other: &mut Self) -> Result<(), &str> {
        let count = if self.capacity() < other.capacity() {
            self.reserve(other.capacity() - self.capacity(), CapacityStrategy::OnDemand);
            other.capacity()
        } else {
            other.reserve(self.capacity() - other.capacity(), CapacityStrategy::OnDemand);
            self.capacity()
        };
        
        let tmp = Self::new(count);
        if tmp.capacity() == count && self.capacity() == count && other.capacity() == count {
            unsafe {
                self.as_ptr().copy_to_nonoverlapping(tmp.as_ptr(), count);
                other.as_ptr().copy_to_nonoverlapping(self.as_ptr(), count);
                tmp.as_ptr().copy_to_nonoverlapping(other.as_ptr(), count);
            }
            Ok(())
        } else {
            Err("Extra memroy allocated failed!")
        }
    }
}

