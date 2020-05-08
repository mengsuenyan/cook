//! 双向链表  
//! 

use std::{fmt, cmp};
use std::ptr::NonNull;
use std::marker::PhantomData;

#[cfg(test)]
mod tests;

struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
    ele: T,
}

pub struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
    phantom: PhantomData<T>
}

pub struct Iter<'a, T: 'a> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
    phantom: PhantomData<&'a Node<T>>,
}

pub struct IterMut<'a, T: 'a> {
    list: &'a mut LinkedList<T>,
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
}

pub struct IntoIter<T> {
    list: LinkedList<T>,
}

impl<T: fmt::Debug> fmt::Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Iter").field(&self.len).finish()
    }
}

impl<T> Clone for Iter<'_, T> {
    fn clone(&self) -> Self {
        Iter {
            head: self.head,
            tail: self.tail,
            len: self.len,
            phantom: PhantomData
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for IterMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IterMut").field(&self.list).field(&self.len).finish()
    }
}

impl<T: fmt::Debug> fmt::Debug for IntoIter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IntoIter").field(&self.list).finish()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.head.map(|node| {
                let node = unsafe {
                    &*node.as_ptr()
                };
                self.head = node.next;
                self.len -= 1;
                &node.ele
            })
        }
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
    
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    
    fn next_back(&mut self) -> Option<&'a T> {
        if self.len == 0 {
            None
        } else {
            self.tail.map(|node| unsafe {
                // Need an unbound lifetime to get 'a
                let node = &*node.as_ptr();
                self.len -= 1;
                self.tail = node.prev;
                &node.ele
            })
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        if self.len == 0 {
            None
        } else {
            self.head.map(|node| unsafe {
                let node = &mut *node.as_ptr();
                self.len -= 1;
                self.head = node.next;
                &mut node.ele
            })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    fn last(mut self) -> Option<&'a mut T> {
        self.next_back()
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<&'a mut T> {
        if self.len == 0 {
            None
        } else {
            self.tail.map(|node| unsafe {
                let node = &mut *node.as_ptr();
                self.len -= 1;
                self.tail = node.prev;
                &mut node.ele
            })
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    
    fn next(&mut self) -> Option<T> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

impl <T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T> Node<T> {
    fn new(ele: T) -> Self {
        Node {next: None, prev: None, ele}
    }
    
    /// Consume Box  
    fn into_element(self: Box<Self>) -> T {
        self.ele
    }
    
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {head: None, tail: None, len: 0, phantom: PhantomData}
    }
    
    /// 在链表末尾插入节点  
    pub fn push_back(&mut self, ele: T) {
        let mut node = Box::new(Node::new(ele));
        node.next = None;
        node.prev = self.tail;
        
        let node = NonNull::new(Box::into_raw(node));
        match self.tail {
            None => self.head = node,
            Some(tail) => unsafe {
                (*tail.as_ptr()).next = node;
            },
        }
        
        self.tail = node;
        self.len += 1;
    }
    
    /// 在链表头插入节点  
    pub fn push_front(&mut self, ele: T) {
        let mut node = Box::new(Node::new(ele));
        node.next = self.head;
        node.prev = None;
        
        let node = NonNull::new(Box::into_raw(node));
        match self.head {
            None => self.tail = node,
            Some(head) => unsafe {
                (*head.as_ptr()).prev = node;
            }
        }
        
        self.head = node;
        self.len += 1;
    }
    
    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.map(|node| {
            let node = unsafe {
                Box::from_raw(node.as_ptr())
            };
            self.tail = node.prev;
            
            match self.tail {
                None => self.head = None,
                Some(tail) => unsafe {
                    (*tail.as_ptr()).next = None;
                },
            };
            
            self.len -= 1;
            node.into_element()
        })
    }
    
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.map(|node| {
            let node = unsafe {
                Box::from_raw(node.as_ptr())
            };
            self.head = node.next;
            
            match self.head {
                None => self.tail = None,
                Some(head) => unsafe {
                    (*head.as_ptr()).prev = None;
                },
            };
            
            self.len -= 1;
            node.into_element()
        })
    }
    
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            head: self.head,
            tail: self.tail,
            len: self.len,
            phantom: PhantomData,
        }
    }
    
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            head: self.head,
            tail: self.tail,
            len: self.len,
            list: self,
        }
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    /// 查找第idx个节点;  
    pub fn find_by_idx(&self, idx: usize) -> Option<&T> {
        let mut itr = self.iter().skip(idx);
        itr.next()
    }
    
    pub fn find_mut_by_idx(&mut self, idx: usize) -> Option<&mut T> {
        let mut itr = self.iter_mut().skip(idx);
        itr.next()
    }
    
    pub fn find_by_val(&self, val: &T) -> Option<usize> 
        where T: PartialEq {
        self.iter().position(|x| {x == val})
    }
    
    /// 删除第idx个节点, 返回该位置的数据  
    pub fn delete_by_idx(&mut self, idx: usize) -> Option<T> {
        if idx >= self.len {
            None
        } else {
            let mut cnt = 0;
            let mut node = self.head;
            while cnt != idx {
                node = unsafe {
                    (*node.unwrap().as_ptr()).next
                };
                cnt += 1;
            }
            
            unsafe {
                let nd = node.unwrap().as_ptr();
                if node == self.head && node == self.tail {
                    self.head = None;
                    self.tail = None;
                } else if node == self.head {
                    self.head = (*node.unwrap().as_ptr()).next;
                    (*nd).prev = None;
                } else if node == self.tail {
                    self.tail = (*node.unwrap().as_ptr()).prev;
                    (*nd).next = None;
                } else {
                    (*(*nd).next.unwrap().as_ptr()).prev = (*nd).prev;
                    (*(*nd).prev.unwrap().as_ptr()).next = (*nd).next;
                }
                
                let node = Box::from_raw(nd);
                self.len -= 1;
                Some(node.into_element())
            }
        }
    }
    
    /// 删除指定的数据(首次遇到的)  
    pub fn delete_by_val(&mut self, val: &T) 
        where T: PartialEq
    {
        let mut node = self.head;
        unsafe {
            while let Some(x) = node {
                if val == &(*x.as_ptr()).ele {
                    break;
                }
                node = (*x.as_ptr()).next;
            }
            
            let nd = node.unwrap().as_ptr();
            if node == self.head && node == self.tail {
                self.head = None;
                self.tail = None;
            } else if node == self.head {
                self.head = (*node.unwrap().as_ptr()).next;
                (*nd).prev = None;
            } else if node == self.tail {
                self.tail = (*node.unwrap().as_ptr()).prev;
                (*nd).next = None;
            } else {
                (*(*nd).next.unwrap().as_ptr()).prev = (*nd).prev;
                (*(*nd).prev.unwrap().as_ptr()).next = (*nd).next;
            }

            let node = Box::from_raw(nd);
            self.len -= 1;
            node.into_element();
        }
    }

    pub fn contains(&self, val: &T) -> bool
        where T: cmp::PartialEq {
        self.iter().any(|e| e == val)
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_back() {
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {list: self}
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        LinkedList::new()
    }
}

impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }

    fn ne(&self, other: &Self) -> bool {
        self.len() != other.len() || self.iter().ne(other)
    }
}
