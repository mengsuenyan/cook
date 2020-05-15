//! 二叉搜索树  


use std::fmt;
use std::ptr::NonNull;
use std::marker::PhantomData;
use crate::gds::bnode::BNode;

struct BSTreeNodeProperty;

type NodePtr<T> = NonNull<BNode<T, BSTreeNodeProperty>>;
type NodeType<T> = Option<NodePtr<T>>;

pub struct BSTree<T> {
    root: NodeType<T>,
    len: usize,
}

/// 中序遍历二叉树  
#[derive(Clone)]
pub struct Iter<'a, T: 'a> {
    stack: Vec<NodePtr<T>>,
    len: usize,
    phantom: PhantomData<&'a BNode<T, BSTreeNodeProperty>>,
}

/// 中序遍历二叉树  
pub struct IterMut<'a, T: 'a> {
    tree: &'a mut BSTree<T>,
    stack: Vec<NodePtr<T>>,
    len: usize,
    phantom: PhantomData<&'a BNode<T, BSTreeNodeProperty>>,
}

pub struct IntoIter<T> {
    tree: BSTree<T>,
    stack: Vec<NodePtr<T>>,
    phantom: PhantomData<T>
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            match self.stack.pop() {
                Some(x) => unsafe {
                    let now_node = x.as_ref().right();
                    BSTree::dfs(&mut self.stack, now_node);
                    self.len -= 1;
                    Some((*x.as_ptr()).element())
                },
                _ => None,
            }
        }
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
    
    fn last(self) -> Option<Self::Item> {
        match self.stack.first() {
            Some(x) => unsafe {
                match x.as_ref().right_most() {
                    Some(y) => Some((*y.as_ptr()).element()),
                    _ => Some((*x.as_ptr()).element()),
                }
            },
            _ => None,
        }
    }
}


impl<T: fmt::Debug> fmt::Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Iter").field(&self.len).finish()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            match self.stack.pop() {
                Some(x) => unsafe {
                    let now_node = x.as_ref().right();
                    BSTree::dfs(&mut self.stack, now_node);
                    self.len -= 1;
                    Some((*x.as_ptr()).element_mut())
                },
                _ => None,
            }
        }
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
    
    fn last(mut self) -> Option<Self::Item> {
        match self.stack.first_mut() {
            Some(x) => unsafe {
                match x.as_ref().right_most() {
                    Some(y) => Some((*y.as_ptr()).element_mut()),
                    _ => Some((*x.as_ptr()).element_mut()),
                }
            },
            _ => None,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for IterMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IterMut").field(&self.tree).finish()
    }
}

impl<'a, T> Iterator for IntoIter<T> {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.tree.len() == 0 {
            None
        } else {
            match self.stack.pop() {
                Some(x) => unsafe {
                    let now_node = x.as_ref().right();
                    BSTree::dfs(&mut self.stack, now_node);
                    self.tree.len -= 1;
                    let x_box = Box::from_raw(x.as_ptr());
                    Some(x_box.into_element())
                },
                _ => None,
            }
        }
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.stack.len(), Some(self.stack.len()))
    }
}

impl<T: fmt::Debug> fmt::Debug for IntoIter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IntoIter").field(&self.tree).finish()
    }
}

impl<T> BSTree<T> {
    pub fn new() -> Self {
        BSTree {
            root: None,
            len: 0,
        }
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    pub fn iter(&self) -> Iter<'_, T> {
        let mut v = Vec::with_capacity(self.len);
        
        BSTree::dfs(&mut v, &self.root);
        
        Iter {
            stack: v,
            len: self.len,
            phantom: PhantomData,
        }
    }
    
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        let mut v = Vec::with_capacity(self.len);

        BSTree::dfs(&mut v, &self.root);
        
        IterMut {
            stack: v,
            len: self.len,
            tree: self,
            phantom: PhantomData,
        }
    }
    
    fn inner_find(node: &NodeType<T>, val: &T) -> NodeType<T>
        where T: PartialOrd
    {
        let mut node = node;
        while node.is_some() {
            let x = BSTree::inner_to_ref(node);
            if val < x.element() {
                node = x.left();
            } else if val > x.element() {
                node = x.right();
            } else {
                break;
            }
        }
        
        node.clone()
    }
    
    /// 查找二叉树是否存在指定元素, 并返回该元素  
    pub fn find(&self, val: &T) -> Option<&T> 
        where T: PartialOrd
    {
        // 未用迭代器, 速度快些
        match BSTree::inner_find(&self.root, val) {
            Some(x) => unsafe {
                Some((*x.as_ptr()).element())
            },
            _ => None,
        }
    }
    
    /// note: 不要修改T的关键字, 否则树的性质改变;
    pub fn find_mut(&mut self, val: &T) -> Option<&mut T> 
        where T: PartialOrd
    {
        match BSTree::inner_find(&self.root, val) {
            Some(x) => unsafe {
                Some((*x.as_ptr()).element_mut())
            },
            _ => None,
        }
    }
    
    fn inner_minimum(node: &NodeType<T>) -> NodeType<T> {
        let mut node = node;
        if node.is_some() {
            let mut x = BSTree::inner_to_ref(node);
            while x.left().is_some() {
                x = BSTree::inner_to_ref(x.left());
                node = x.left();
            }
        }
        node.clone()
    }
    
    fn inner_maximum(node: &NodeType<T>) -> NodeType<T> {
        let mut node = node;
        if node.is_some() {
            let mut x = BSTree::inner_to_ref(node);
            while x.right().is_some() {
                x = BSTree::inner_to_ref(x.right());
                node = x.right();
            }
        }
        node.clone()
    }
    
    /// 二叉搜索树最小值  
    pub fn minimum(&self) -> Option<&T> {
        match BSTree::inner_minimum(&self.root) {
            Some(x) => unsafe {
                Some((*x.as_ptr()).element())
            },
            _ => None,
        }
    }
    
    pub fn minimum_mut(&mut self) -> Option<&mut T> {
        match BSTree::inner_minimum(&self.root) {
            Some(x) => unsafe {
                Some((*x.as_ptr()).element_mut())
            },
            _ => None,
        }
    }

    pub fn maximum(&self) -> Option<&T> {
        match BSTree::inner_maximum(&self.root) {
            Some(x) => unsafe {
                Some((*x.as_ptr()).element())
            },
            _ => None,
        }
    }

    pub fn maximum_mut(&mut self) -> Option<&mut T> {
        match BSTree::inner_maximum(&self.root) {
            Some(x) => unsafe {
                Some((*x.as_ptr()).element_mut())
            },
            _ => None,
        }
    }
    
    /// 插入关键字为val的节点  
    pub fn insert(&mut self, val: T) 
        where T: PartialOrd
    {
        let mut y = None;
        let mut x = self.root;
        
        while x.is_some() {
            y = x;
            let node = BSTree::inner_to_ref(&y);
            if &val < node.element() {
                x = node.left().clone();
            } else {
                x = node.right().clone();
            }
        }
        
        if y.is_none() {
            self.root = NonNull::new(Box::into_raw(Box::new(BNode::new(val, BSTreeNodeProperty))));
        } else {
            let mut z = BNode::new(val, BSTreeNodeProperty);
            std::mem::replace(z.parent_mut(), y);
            
            let node = BSTree::inner_to_mut(&mut y);
            if z.element() < node.element() {
                std::mem::replace(node.left_mut(), NonNull::new(Box::into_raw(Box::new(z))));
            } else {
                std::mem::replace(node.right_mut(), NonNull::new(Box::into_raw(Box::new(z))));
            }
        }
    }
    
    /// 子树v替换u  
    fn transplant(&mut self, u: NodeType<T>, v: NodeType<T>) {
        let mut v= v;
        if u.is_none() || BSTree::inner_to_ref(&u).parent().is_none() {
            self.root = v;
        } else {
            let cur = BSTree::inner_to_ref(&u);
            let (mut p, p_left) = cur.parent_left();
            let pn = BSTree::inner_to_mut(&mut p);
            if u == p_left {
                std::mem::replace(pn.left_mut(), v);
            } else {
                std::mem::replace(pn.right_mut(), v);
            }
            
            if v.is_some() {
                let v = BSTree::inner_to_mut(&mut v);
                std::mem::replace(v.parent_mut(), p);
            }
        }
    }
    
    /// 删除关键字为val的节点  
    pub fn delete(&mut self, val: &T) -> bool
        where T: PartialOrd
    {
        let z = BSTree::inner_find(&self.root, val);
        if z.is_none() {
            return false;
        }
        
        let zn = BSTree::inner_to_ref(&z);
        if zn.left().is_none() {
            self.transplant(z, zn.right().clone());
        } else if zn.right().is_none() {
            self.transplant(z, zn.left().clone());
        } else {
            let mut y = BSTree::inner_minimum(zn.right());
            let yc = y;
            let yn = BSTree::inner_to_mut(&mut y);
            if yn.parent() != &z {
                self.transplant(yc, yn.right().clone());
                std::mem::replace(yn.right_mut(), zn.right().clone());
                let yr_p = BSTree::inner_to_mut(yn.right_mut());
                std::mem::replace(yr_p.parent_mut(), yc);
            }
            
            self.transplant(z, yc);
            std::mem::replace(yn.left_mut(), zn.left().clone());
            let yl_p = BSTree::inner_to_mut(yn.left_mut());
            std::mem::replace(yl_p.parent_mut(), yc);
        }
        
        let z_ptr = unsafe {
            Box::from_raw(z.unwrap().as_ptr())
        };
        
        z_ptr.into_element();
        
        self.len -= 1;
        
        true
    }
    
    /// node必须是Some
    fn inner_to_ref(node: &NodeType<T>) -> &BNode<T, BSTreeNodeProperty> {
        unsafe {
            node.as_ref().unwrap().as_ref()
        }
    }

    fn inner_to_mut(node: &mut NodeType<T>) -> &mut BNode<T, BSTreeNodeProperty> {
        unsafe {
            node.as_mut().unwrap().as_mut()
        }
    }
    
    fn dfs(stack: &mut Vec<NodePtr<T>>, node: &NodeType<T>) {
        let mut node = node.clone();

        while node.is_some() {
            let left = BSTree::inner_to_ref(&node).left().clone();
            match node {
                Some(x) => stack.push(x),
                _ => {},
            };
            node = left;
        }
    }


    fn to_vec_recur(node: &NodeType<T>, v: &mut Vec<T>) 
        where T: Clone
    {
        if node.is_some() {
            let node = BSTree::inner_to_ref(&node);
            BSTree::to_vec_recur(node.left(), v);
            v.push(node.element().clone());
            BSTree::to_vec_recur(node.right(), v);
        }
    }

    pub fn to_vec(&self) -> Vec<T> 
        where T:Clone
    {
        let mut v = Vec::new();

        BSTree::to_vec_recur(&self.root, &mut v);

        v
    }
}

impl<T> Default for BSTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IntoIterator for BSTree<T> {
    type Item = T;
    type IntoIter = IntoIter<T>; 

    fn into_iter(self) -> Self::IntoIter {
        let mut v = Vec::with_capacity(self.len());
        
        BSTree::dfs(&mut v, &self.root);
        
        IntoIter {
            tree: self,
            stack: v,
            phantom: PhantomData
        }
    }
}

impl<'a, T> IntoIterator for &'a BSTree<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut BSTree<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Drop for BSTree<T> {
    fn drop(&mut self) {
        let mut itr = self.into_iter();
        while let Some(_) = itr.next() {
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for BSTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}



