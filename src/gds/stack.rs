//! 
//! 

use crate::gds::{LinearBuf, CapacityStrategy};
use crate::alloc::DefaultAllocator;
use std::{fmt, mem, cmp};

const CAPACITY_STRATEGY: CapacityStrategy = CapacityStrategy::DoubleCapacity;

pub struct Stack<T> {
    buf: LinearBuf<T, DefaultAllocator<T>>,
    len: usize,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack::with_capacity(0)
    }
    
    pub fn with_capacity(size: usize) -> Self {
        let buf = LinearBuf::new(size);
        Stack {
            buf,
            len: 0,
        }
    }
    
    #[inline]
    pub fn size(&self) -> usize {
        self.len
    }
    
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }
    
    pub fn top(&self) -> Option<&T> {
        self.buf.read_as_ref(self.size())
    }
    
    #[inline]
    fn reserve(&mut self, additional_size: usize) {
        self.buf.reserve(additional_size, CAPACITY_STRATEGY);
    }
    
    /// 入栈失败会panic;  
    pub fn push(&mut self, val: T) {
        if self.size() >= self.buf.capacity() {
            self.reserve(1);
        }

        self.buf.write(self.size(), val).unwrap();
        self.len += 1;
    }
    
    pub fn pop(&mut self) -> Option<T> {
        if self.size() > 0 {
            self.len -= 1;
            self.buf.read(self.size())
        } else {
            None
        }
    }
    
    /// 交换失败会panic;  
    pub fn swap(&mut self, other: &mut Self) {
        self.buf.swap(&mut other.buf).unwrap()
    }
    
    pub fn from_vec(v: &Vec<T>) -> Self {
        let mut s = Self::with_capacity(v.len());
        s.buf.write_from_vec(0, v);
        s.len = v.len();
        s
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        self.buf.dealloc_buf();
    }
}

impl<T: cmp::PartialEq> cmp::PartialEq for Stack<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.size() == other.size() {
            for i in 0..self.size() {
                if self.buf.read_as_ref(i) != other.buf.read_as_ref(i) {
                    return false
                }
            }
            true
        } else {
            false
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::with_capacity(self.size() * mem::size_of::<T>());
        s.push('|');
        for i in 0 .. self.size() {
            let ele = self.buf.read_as_ref(i);
            let subs = format!("{:?}|", ele.unwrap());
            s.push_str(subs.as_str());
        }
        if self.size() == 0 {
            s.push('|');
        }
        f.write_str(s.as_str())
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

mod tests {
    #[test]
    fn stack() {
        let mut s = super::Stack::new();
        let mut v = Vec::new();
        for i in 0..100usize {
            s.push(i);
            v.push(i);
            assert_eq!(s, super::Stack::from_vec(&v));
        }
        
        for _ in 0..100usize {
            s.pop();
            v.pop();
            assert_eq!(s, super::Stack::from_vec(&v));
        }
    }
}
