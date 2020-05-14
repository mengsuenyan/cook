//! 二叉堆  
//! 

enum BHeapType<T> {
    MaxHeap(Vec<T>),
    MinHeap(Vec<T>),
}

use BHeapType::{MaxHeap, MinHeap};

impl<T> BHeapType<T> {
    fn is_max_heap(&self) -> bool {
        match self {
            MaxHeap(_) => true,
            _ => false,
        }
    }
    
    #[inline]
    fn len(&self) -> usize {
        self.as_vec_ref().len()
    }
    
    fn capacity(&self) -> usize {
        self.as_vec_ref().capacity()
    }
    
    fn into_vec(self) -> Vec<T> {
        match self {
            MaxHeap(x) => x,
            MinHeap(x) => x,
        }
    }
    
    #[inline]
    fn as_vec_ref(&self) -> &Vec<T> {
        match self {
            MaxHeap(x) => x,
            MinHeap(x) => x,
        }
    }
    
    #[inline]
    fn as_vec_mut(&mut self) -> &mut Vec<T> {
        match self {
            MaxHeap(x) => x,
            MinHeap(x) => x,
        }
    }
    
    #[inline]
    fn as_slice_mut(&mut self) -> &mut [T] {
        match self {
            MaxHeap(x) => x.as_mut_slice(),
            MinHeap(x) => x.as_mut_slice(),
        }
    }

    fn with_capacity(cap: usize, is_max_heap: bool) -> Self {
        if is_max_heap {
            MaxHeap(Vec::with_capacity(cap))
        } else {
            MinHeap(Vec::with_capacity(cap))
        }
    }
    
    fn new(is_max_heap: bool) -> Self {
        BHeapType::with_capacity(0, is_max_heap)
    }
}

/// 二叉堆  
pub struct BHeap<T> {
    h: BHeapType<T>
}

impl<T> BHeap<T> {
    /// is_max_heap用于指定是否是最大堆, 否则是最小堆  
    pub fn new(is_max_heap: bool) -> Self {
        BHeap {h: BHeapType::new(is_max_heap)}
    }
    
    pub fn with_capacity(cap: usize, is_max_heap: bool) -> Self {
        if is_max_heap {
            BHeap {h: BHeapType::with_capacity(cap, is_max_heap) }
        } else {
            BHeap {h: BHeapType::with_capacity(cap, is_max_heap)}
        }
    }
    
    pub fn capacity(&self) -> usize {
        self.h.capacity()
    }
    
    pub fn len(&self) -> usize {
        self.h.len()
    }
    
    pub fn is_max_heap(&self) -> bool {
        self.h.is_max_heap()
    }
    
    pub fn is_min_heap(&self) -> bool {
        !self.h.is_max_heap()
    }

    #[inline]
    fn parent(idx: usize) -> usize {
        idx >> 1
    }

    #[inline]
    fn left(idx: usize) -> usize {
        idx << 1
    }

    #[inline]
    fn right(idx: usize) -> usize {
        (idx << 1) + 1
    }

    fn max_heapify(v: &mut [T], idx: usize)
        where T: PartialOrd
    {
        let (h, l, r) = (&*v, BHeap::<T>::left(idx), BHeap::<T>::right(idx));
        let mut largest = if l < h.len() && h[l] > h[idx] {
            l
        } else {
            idx
        };

        if r < h.len() && h[r] > h[largest] {
            largest = r;
        }

        if largest != idx {
            v.swap(idx, largest);
            BHeap::max_heapify(v, largest);
        }
    }

    fn min_heapify(v: &mut [T], idx: usize)
        where T: PartialOrd
    {
        let (h, l, r) = (&*v, BHeap::<T>::left(idx), BHeap::<T>::right(idx));
        let mut largest = if l < h.len() && h[l] < h[idx] {
            l
        } else {
            idx
        };

        if r < h.len() && h[r] < h[largest] {
            largest = r;
        }

        if largest != idx {
            v.swap(idx, largest);
            BHeap::min_heapify(v, largest);
        }
    }

    pub fn push(&mut self, item: T)
        where T: PartialOrd
    {
        self.h.as_vec_mut().push(item);
        let mut idx = self.len() - 1;
        
        if self.is_max_heap() {
            while idx > 0 && self.h.as_vec_ref()[BHeap::<T>::parent(idx)] < self.h.as_vec_ref()[idx] {
                self.h.as_vec_mut().swap(BHeap::<T>::parent(idx), idx);
                idx = BHeap::<T>::parent(idx);
            }
        } else {
            while idx > 0 && self.h.as_vec_ref()[BHeap::<T>::parent(idx)] > self.h.as_vec_ref()[idx] {
                self.h.as_vec_mut().swap(BHeap::<T>::parent(idx), idx);
                idx = BHeap::<T>::parent(idx);
            }
        }
    }
    
    /// 最大堆返回的是堆中最大值, 最小堆返回的是堆中最大值  
    pub fn pop(&mut self) -> Option<T>
        where T: PartialOrd
    {
        if self.len() > 0 {
            let r = Some(self.h.as_vec_mut().remove(0));
            let mut idx = self.len() >> 1;
            let len = idx;
            if self.is_max_heap() {
                for _ in 0..=len {
                    BHeap::max_heapify(self.h.as_slice_mut(), idx);
                    idx = idx.saturating_sub(1);
                }
            } else {
                for _ in 0..=len {
                    BHeap::min_heapify(self.h.as_slice_mut(), idx);
                    idx = idx.saturating_sub(1);
                }
            }
            r
        } else {
            None
        }
    }
    
    pub fn into_vec(self) -> Vec<T> {
        self.h.into_vec()
    }
    
    /// 无论最大堆还是最小堆返回的都是升序  
    pub fn into_sorted_vec(mut self) -> Vec<T>
        where T: PartialOrd
    {
        if self.is_min_heap() {
            for i in 1..self.len() {
                let x = &mut self.h.as_vec_mut()[i..];
                BHeap::min_heapify(x, 0);
            }
        } else {
            let len = self.len();
            for i in 0..(len - 1) {
                self.h.as_vec_mut().swap(0, len - 1 - i);
                let x = &mut self.h.as_vec_mut()[(i+1)..];
                BHeap::max_heapify(x, 0);
            }
        }
        self.h.into_vec()
    }
    
    pub fn max(&self) -> Option<&T>
        where T: PartialOrd
    {
        if self.is_max_heap() {
            self.h.as_vec_ref().first()
        } else {
            self.h.as_vec_ref().iter().max_by(|x, y| x.partial_cmp(y).unwrap())
        }
    }
    
    pub fn min(&self) -> Option<&T>
        where T: PartialOrd
    {
        if self.is_min_heap() {
            self.h.as_vec_ref().first()
        } else {
            self.h.as_vec_ref().iter().min_by(|x, y| x.partial_cmp(y).unwrap())
        }
    }
}


