//! 红黑树  
//! - 每个节点或红或黑;  
//! - 根节点是黑色的;  
//! - 如果一个非叶子节点是红色的, 那么其两个子节点是黑色的;  
//! - 对每个节点, 从该节点到其所有后代节点的简单路径上, 均包含相同数据的黑色节点;  

// TODO: 从BSTree抽象过来

use std::fmt;
use std::marker::PhantomData;
use std::ptr::NonNull;
use crate::gds::BNode;

#[derive(Copy, Clone)]
enum NodeColor {
    Red,
    Black,
}

struct RBTreeNodeProperty<T> {
    color: NodeColor,
    phantom: PhantomData<T>,
}

impl<T> Clone for RBTreeNodeProperty<T> {
    fn clone(&self) -> Self {
        RBTreeNodeProperty {
            color: self.color,
            phantom: PhantomData,
        }
    }
}

impl<T> RBTreeNodeProperty<T> {
    /// 新节点默认是红色的  
    fn new() -> Self {
        RBTreeNodeProperty {
            color: NodeColor::Red,
            phantom: PhantomData,
        }
    }
    
    fn is_red(&self) -> bool {
        match self.color {
            NodeColor::Red => true,
            _ => false,
        }
    }
    
    fn is_black(&self) -> bool {
        !self.is_red()
    }
    
    fn color(&self) -> &NodeColor {
        &self.color
    }
    
    fn color_mut(&mut self) -> &mut NodeColor {
        &mut self.color
    }
}

type Node<T> = BNode<T, RBTreeNodeProperty<T>>;
type NodePtr<T> = NonNull<Node<T>>;
type NodeType<T> = Option<NodePtr<T>>;

pub struct RBTree<T> {
    root: NodeType<T>,
    len: usize,
}

impl<T> RBTree<T> {
    pub fn new() -> Self {
        RBTree {
            root: None,
            len: 0,
        }
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    /// v must be some value  
    fn cvt_to_node(v: &NodeType<T>) -> &Node<T> {
        unsafe {
            v.as_ref().unwrap().as_ref()
        }
    }
    
    fn cvt_to_node_mut(v: &mut NodeType<T>) -> &mut Node<T> {
        unsafe {
            v.as_mut().unwrap().as_mut()
        }
    }
    
    fn node_property(x: &NodeType<T>) -> Option<RBTreeNodeProperty<T>> {
        if x.is_some() {
            let node = RBTree::cvt_to_node(x);
            Some(node.property().clone())
        } else {
            None
        }
    }
    
    /// 在以根节点node的二叉树中查找是否存在val节点
    fn inner_find(mut node: &NodeType<T>, val: T) -> NodeType<T>
        where T: PartialOrd
    {
        while node.is_some() {
            let x = RBTree::cvt_to_node(&node);
            if x.element() < &val {
                node = x.left();
            } else if x.element() > &val {
                node = x.right();
            } else {
                break;
            }
        }
        
        node.clone()
    }
    
    pub fn find(&self, val: T) -> Option<&T>
        where T: PartialOrd
    {
        let node = RBTree::inner_find(&self.root, val);
        match node {
            Some(x) => unsafe {
                Some((*x.as_ptr()).element())
            },
            _ => None,
        }
    }
    
    /// note: 不要修改T的关键字, 否则树的性质改变;  
    pub fn find_mut(&mut self, val: T) -> Option<&mut T>
        where T: PartialOrd
    {
        let node = RBTree::inner_find(&self.root, val);
        match node {
            Some(x) => unsafe {
                Some((*x.as_ptr()).element_mut())
            },
            _ => None,
        }
    }
    
    fn inner_min(mut node: &NodeType<T>) -> NodeType<T> {
        
        while node.is_some() {
            let x = RBTree::cvt_to_node(node);
            if x.left().is_some() {
                node = x.left();
            } else {
                break;
            }
        }
        
        node.clone()
    }
    
    pub fn min(&self) -> Option<&T> {
        let node = RBTree::inner_min(&self.root);
        match node {
            Some(x) => unsafe {
                Some((*x.as_ptr()).element())
            },
            _ => None,
        }
    }
    
    /// note: 不要修改T的关键字
    pub fn min_mut(&mut self) -> Option<&mut T> {
        let node = RBTree::inner_min(&self.root);
        match node {
            Some(x) => unsafe {
                Some((*x.as_ptr()).element_mut())
            },
            _ => None,
        }
    }
    
    fn inner_max(mut node: &NodeType<T>) -> NodeType<T> {
        while node.is_some() {
            let x = RBTree::cvt_to_node(&node);
            if x.right().is_some() {
                node = x.right();
            } else {
                break;
            }
        }
        
        node.clone()
    }
    
    pub fn max(&self) -> Option<&T> {
        let node = RBTree::inner_max(&self.root);
        match node {
            Some(x) => unsafe {
                Some((*x.as_ptr()).element())
            },
            _ => None,
        }
    }
    
    /// note: 不要修改T的关键字  
    pub fn max_mut(&mut self) -> Option<&mut T> {
        let node = RBTree::inner_max(&self.root);
        match node {
            Some(x) => unsafe {
                Some((*x.as_ptr()).element_mut())
            },
            _ => None,
        }
    }
    
    /// ```text
    ///      x          <----right_rotate----                y
    ///  xl     y       ---left_rotate-->                x     yr
    ///       yl yr                                   xl   yl
    /// ```
    fn left_rotate(&mut self, mut x: NodeType<T>) {
        if x.is_none() {
            return;
        }
        
        let x_copy = x;
        let x_node= RBTree::cvt_to_node_mut(&mut x);
        let mut y = x_node.right().clone();
        let y_copy = y;
        if y.is_none() {
            return;
        }
        
        let y_node = RBTree::cvt_to_node_mut(&mut y);
        std::mem::replace(x_node.right_mut(), y_node.left().clone());
        
        if y_node.left().is_some() {
            let y_left_node = RBTree::cvt_to_node_mut(y_node.left_mut());
            std::mem::replace(y_left_node.parent_mut(), x_copy);
        }
        
        std::mem::replace(y_node.parent_mut(), x_node.parent().clone());
        
        let (x_parent, mut x_parent_left) = x_node.parent_left();
        if x_parent.is_none() {
            self.root = y_copy;
        } else if x_copy == x_parent_left {
            let x_parent_left_node = RBTree::cvt_to_node_mut(&mut x_parent_left);
            std::mem::replace(x_parent_left_node.left_mut(), y_copy);
        } else {
            let x_parent_left_node = RBTree::cvt_to_node_mut(&mut x_parent_left);
            std::mem::replace(x_parent_left_node.right_mut(), y_copy);
        }
        
        std::mem::replace(y_node.left_mut(), x_copy);
        std::mem::replace(x_node.parent_mut(), y_copy);
    }
    
    fn right_rotate(&mut self, mut y: NodeType<T>) {
        if y.is_none() {
            return;
        }
        
        let y_copy = y;
        let y_node = RBTree::cvt_to_node_mut(&mut y);
        let mut x = y_node.left().clone();
        let x_copy = x;
        
        if x.is_none() {
            return;
        }
        
        let x_node = RBTree::cvt_to_node_mut(&mut x);
        std::mem::replace(y_node.left_mut(), x_node.right().clone());
        
        if x_node.right().is_some() {
            let x_right_node = RBTree::cvt_to_node_mut(x_node.right_mut());
            std::mem::replace(x_right_node.parent_mut(), y_copy);
        }
        
        std::mem::replace(x_node.right_mut(), y_copy);
        let (mut y_parent, y_parent_right) = y_node.parent_right();
        if y_parent.is_none() {
            self.root = x_copy;
        } else if y_copy == y_parent_right {
            let y_parent_node = RBTree::cvt_to_node_mut(&mut y_parent);
            std::mem::replace(y_parent_node.right_mut(), x_copy);
        } else {
            let y_parent_node = RBTree::cvt_to_node_mut(&mut y_parent);
            std::mem::replace(y_parent_node.left_mut(), x_copy);
        }

        std::mem::replace(x_node.parent_mut(), y_node.parent().clone());
        std::mem::replace(y_node.parent_mut(), x_copy);
    }
    
    /// 红黑树性质维持  
    fn insert_fixup(&mut self, z: NodeType<T>) {
        let (mut z, mut z_copy) = (z, z);
        let mut z_node = RBTree::cvt_to_node_mut(&mut z);
        let (mut z_parent, mut z_parent_right) = z_node.parent_right();
        let mut z_parent_copy = z_parent;
        let z_parent_pro = RBTree::node_property(&z_parent);
        
        if z_parent.is_none() {
            *z_node.property_mut().color_mut() = NodeColor::Black;
            return;
        }
        
        let mut z_parent_node = RBTree::cvt_to_node_mut(&mut z_parent);
        let mut z_p_p = z_parent_node.parent().clone();
        
        // 破坏了性质: 如果一个非叶子节点是红色的, 那么其两个子节点是黑色的;
        while z_parent_pro.is_some() && z_p_p.is_some() && z_parent_pro.as_ref().unwrap().is_red() {
            let z_p_p_node = RBTree::cvt_to_node_mut(&mut z_p_p);
            
            // 重复代码, 便于调试, ok后宏替换
            let z_next = if &z_parent_copy == z_p_p_node.left() {
                let mut y = z_p_p_node.right_mut();
                let y_node_pro = RBTree::node_property(&y);
                if y_node_pro.is_some() && y_node_pro.unwrap().is_red() {
                    // z的叔节点y是红色的, 则父叔节点黑, 爷节点红
                    let y_node = RBTree::cvt_to_node_mut(&mut y);
                    *z_parent_node.property_mut().color_mut() = NodeColor::Black;
                    *y_node.property_mut().color_mut() = NodeColor::Black;
                    *z_p_p_node.property_mut().color_mut() = NodeColor::Red;
                    z_p_p
                } else {
                    let mut z_next = if z_copy == z_parent_right {
                        // z的叔节点是黑色的, 且z是一个右孩子, 则左旋处理, 父节点编程z的左子节点
                        self.left_rotate(z_parent_copy);
                        z_parent_copy
                    } else {
                        z_copy
                    };
                    
                    // 父节点红, 爷节点黑, 再右旋
                    let z_next_p_node = RBTree::cvt_to_node_mut(&mut z_next);
                    let mut z_next_p_p = z_next_p_node.parent().clone();
                    let z_next_p_p_node = RBTree::cvt_to_node_mut(&mut z_next_p_p);
                    
                    *z_next_p_node.property_mut().color_mut() = NodeColor::Red;
                    *z_next_p_p_node.property_mut().color_mut() = NodeColor::Black;
                    self.right_rotate(z_next_p_p);
                    
                    z_next
                }
            } else {
                // z的父节点是爷节点的右节点, 镜像操作
                let mut y = z_p_p_node.left_mut();
                let y_node_pro = RBTree::node_property(&y);
                if y_node_pro.is_some() && y_node_pro.unwrap().is_red() {
                    let y_node = RBTree::cvt_to_node_mut(&mut y);
                    *z_parent_node.property_mut().color_mut() = NodeColor::Black;
                    *y_node.property_mut().color_mut() = NodeColor::Black;
                    *z_p_p_node.property_mut().color_mut() = NodeColor::Red;
                    z_p_p
                } else {
                    let mut z_next = if z_copy != z_parent_right {
                        self.right_rotate(z_parent_copy);
                        z_parent_copy
                    } else {
                        z_copy
                    };

                    let z_next_p_node = RBTree::cvt_to_node_mut(&mut z_next);
                    let mut z_next_p_p = z_next_p_node.parent().clone();
                    let z_next_p_p_node = RBTree::cvt_to_node_mut(&mut z_next_p_p);

                    *z_next_p_node.property_mut().color_mut() = NodeColor::Red;
                    *z_next_p_p_node.property_mut().color_mut() = NodeColor::Black;
                    self.left_rotate(z_next_p_p);

                    z_next
                }
            };
            
            // 更新变量
            z = z_next;
            z_copy = z_next;
            z_node = RBTree::cvt_to_node_mut(&mut z);
            let (x, y) = z_node.parent_right();
            z_parent = x; z_parent_right = y;
            z_parent_copy = z_parent;
            if z_parent.is_none() {
                return;
            }
            z_parent_node = RBTree::cvt_to_node_mut(&mut z_parent);
            z_p_p = z_parent_node.parent().clone()
        }
    }

    pub fn insert(&mut self, val: T)
        where T: PartialOrd 
    {
        let mut z_pro = RBTreeNodeProperty::new();
        *z_pro.color_mut() = NodeColor::Red;
        let mut z = NodePtr::new(Box::into_raw(Box::new(BNode::new(val, z_pro))));
        let (z_copy, z_node) = (z, RBTree::cvt_to_node_mut(&mut z));
        
        let (mut x, mut y) = (self.root, None);
        
        while x.is_some() {
            y = x;
            let x_node = RBTree::cvt_to_node_mut(&mut x);
            if z_node.element() < x_node.element() {
                x = x_node.left().clone();
            } else {
                x = x_node.right().clone();
            }
        }
        
        std::mem::replace(z_node.parent_mut(), y);
        
        if y.is_none() {
            self.root = z;
        } else {
            let y_node = RBTree::cvt_to_node_mut(&mut y);
            if z_node.element() < y_node.element() {
                std::mem::replace(y_node.left_mut(), z_copy);
            } else {
                std::mem::replace(y_node.right_mut(), z_copy);
            }
        }

        self.len += 1;
        self.insert_fixup(z_copy);
    }
    
    /// v节点替换u节点  
    fn transplant(&mut self, mut u: NodeType<T>, mut v: NodeType<T>) {
        if u.is_none() {
            self.root = v;
            return;
        }
        
        let u_copy= u;
        let u_node = RBTree::cvt_to_node_mut(&mut u);
        let (mut u_parent, u_parent_left) = u_node.parent_left();
        
        if u_parent.is_none() {
            self.root = v;
        } else if u_copy == u_parent_left {
            let u_parent_node = RBTree::cvt_to_node_mut(&mut u_parent);
            std::mem::replace(u_parent_node.left_mut(), v);
        } else {
            let u_parent_node = RBTree::cvt_to_node_mut(&mut u_parent);
            std::mem::replace(u_parent_node.right_mut(), v);
        }
        
        if v.is_some() {
            let v_node = RBTree::cvt_to_node_mut(&mut v);
            std::mem::replace(v_node.parent_mut(), u_parent);
        }
    }

    /// 删除的是根节点, 那么新根节点需是黑色  
    /// 删除的是黑色节点, 那么该节点后续的简单路径上少了一个黑色节点, 需增加一个黑色节点  
    fn delete_fixup(&mut self, mut x: NodeType<T>) {
        if x.is_none() || x == self.root {
            return;
        }
        
        let (mut x_copy, mut x_node) = (x, RBTree::cvt_to_node_mut(&mut x));
        let mut x_pro = x_node.property().clone();
        
        while x_copy != self.root && x_pro.is_black() {
            let (mut x_p, x_p_l, x_p_r) = x_node.parent_left_right();
            let x_p_copy = x_p;
            
            if x_p.is_none() || x_p_l.is_none() || x_p_r.is_none() {
                break;
            }
            
            // 保证x的父节点的左右子树的黑色节点一致  
            if x_copy == x_p_l {
                let mut w = x_p_r;
                let w_node = RBTree::cvt_to_node_mut(&mut w);
                // x的兄弟节点是红色, 准备从兄弟节点等效拿一个黑节点过来
                if w_node.property().is_red() {
                    *w_node.property_mut().color_mut() = NodeColor::Black;
                    let x_p_node = RBTree::cvt_to_node_mut(&mut x_p);
                    *x_p_node.property_mut().color_mut() = NodeColor::Red;
                    self.left_rotate(x_p_copy);
                    w = RBTree::cvt_to_node_mut(x_node.parent_mut()).right().clone();
                }
                
                if w.is_some() {
                    let w_copy = w;
                    let w_node = RBTree::cvt_to_node_mut(&mut w);
                    let (mut w_l, w_r) = w_node.left_right();
                    let (w_l_pro, w_r_pro) = (RBTree::node_property(&w_l), RBTree::node_property(&w_r));
                    if w_l_pro.is_some() && w_r_pro.is_some() && w_l_pro.unwrap().is_black() && w_r_pro.as_ref().unwrap().is_black() {
                        *w_node.property_mut().color_mut() = NodeColor::Red;

                        x = x_node.parent().clone();
                        x_copy = x;
                        x_node = RBTree::cvt_to_node_mut(&mut x);
                        x_pro = x_node.property().clone();
                    } else {
                        if w_r_pro.is_some() && w_r_pro.unwrap().is_black() {
                            if w_l.is_some() {
                                let w_l_node = RBTree::cvt_to_node_mut(&mut w_l);
                                *w_l_node.property_mut().color_mut() = NodeColor::Black;
                            }
                            *w_node.property_mut().color_mut() = NodeColor::Red;
                            self.right_rotate(w_copy);
                            w = RBTree::cvt_to_node_mut(x_node.parent_mut()).right().clone();
                        }
                        let w_node = RBTree::cvt_to_node_mut(&mut w);
                        let x_p_node = RBTree::cvt_to_node_mut(x_node.parent_mut());
                        *w_node.property_mut().color_mut() = x_p_node.property_mut().color().clone();
                        *x_p_node.property_mut().color_mut() = NodeColor::Black;
                        *RBTree::cvt_to_node_mut(w_node.right_mut()).property_mut().color_mut() = NodeColor::Black;
                        self.left_rotate(x_p_copy);
                        x = self.root;
                        x_copy = x;
                        x_node = RBTree::cvt_to_node_mut(&mut x);
                        x_pro = x_node.property().clone();
                    }
                }
            } else {
                let mut w = x_p_l;
                let w_node = RBTree::cvt_to_node_mut(&mut w);
                // x的兄弟节点是红色, 准备从兄弟节点等效拿一个黑节点过来
                if w_node.property().is_red() {
                    *w_node.property_mut().color_mut() = NodeColor::Black;
                    let x_p_node = RBTree::cvt_to_node_mut(&mut x_p);
                    *x_p_node.property_mut().color_mut() = NodeColor::Red;
                    self.right_rotate(x_p_copy);
                    w = RBTree::cvt_to_node_mut(x_node.parent_mut()).left().clone();
                }

                if w.is_some() {
                    let w_copy = w;
                    let w_node = RBTree::cvt_to_node_mut(&mut w);
                    let (w_l, mut w_r) = w_node.left_right();
                    let (w_l_pro, w_r_pro) = (RBTree::node_property(&w_l), RBTree::node_property(&w_r));
                    if w_l_pro.is_some() && w_r_pro.is_some() && w_l_pro.as_ref().unwrap().is_black() && w_r_pro.unwrap().is_black() {
                        *w_node.property_mut().color_mut() = NodeColor::Red;

                        x = x_node.parent().clone();
                        x_copy = x;
                        x_node = RBTree::cvt_to_node_mut(&mut x);
                        x_pro = x_node.property().clone();
                    } else {
                        if w_l_pro.is_some() && w_l_pro.unwrap().is_black() {
                            if w_r.is_some() {
                                let w_r_node = RBTree::cvt_to_node_mut(&mut w_r);
                                *w_r_node.property_mut().color_mut() = NodeColor::Black;
                            }
                            *w_node.property_mut().color_mut() = NodeColor::Red;
                            self.right_rotate(w_copy);
                            w = RBTree::cvt_to_node_mut(x_node.parent_mut()).left().clone();
                        }
                        let w_node = RBTree::cvt_to_node_mut(&mut w);
                        let x_p_node = RBTree::cvt_to_node_mut(x_node.parent_mut());
                        *w_node.property_mut().color_mut() = x_p_node.property_mut().color().clone();
                        *x_p_node.property_mut().color_mut() = NodeColor::Black;
                        *RBTree::cvt_to_node_mut(w_node.left_mut()).property_mut().color_mut() = NodeColor::Black;
                        self.right_rotate(x_p_copy);
                        x = self.root;
                        x_copy = x;
                        x_node = RBTree::cvt_to_node_mut(&mut x);
                        x_pro = x_node.property().clone();
                    }
                }
            }
        }
        
        *x_node.property_mut().color_mut() = NodeColor::Black;
    }
    
    pub fn delete(&mut self, val: T) -> bool
        where T: PartialOrd
    {
        let mut z = RBTree::inner_find(&self.root, val);
        if z.is_none() {
            return false;
        }
        let (mut y, z_copy, z_node) = (z, z, RBTree::cvt_to_node_mut(&mut z));
        let z_pro = z_node.property().clone();
        
        let x = if z_node.left().is_none() {
            let x = z_node.right().clone();
            self.transplant(z_copy, z_node.right().clone());
            x
        } else if z_node.right().is_none() {
            let x = z_node.left().clone();
            self.transplant(z_copy, z_node.left().clone());
            x
        } else {
            // 找z的后驱y
            y = RBTree::inner_min(z_node.right());
            let y_copy = y;
            let y_node = RBTree::cvt_to_node_mut(&mut y);
            let x = y_node.right().clone();
            
            // y非z的子节点, y的右子树替换y的位置, y替换z的位置
            if y_node.parent() != &z_copy {
                self.transplant(y_copy, x);
                let mut z_right = z_node.right().clone();
                std::mem::replace(y_node.right_mut(), z_right);
                let z_right_node = RBTree::cvt_to_node_mut(&mut z_right);
                std::mem::replace(z_right_node.parent_mut(), y_copy);
            }

            self.transplant(z_copy, y_copy);
            *y_node.property_mut().color_mut() = z_pro.color().clone();
            let mut z_left = z_node.left().clone();
            std::mem::replace(y_node.left_mut(), z_left);
            let z_left_node = RBTree::cvt_to_node_mut(&mut z_left);
            std::mem::replace(z_left_node.parent_mut(), y_copy);

            x
        };

        unsafe {
            Box::from_raw(z_copy.unwrap().as_ptr()).into_element();
        }
        self.len -= 1;
        
        let y_node = RBTree::cvt_to_node_mut(&mut y);
        if y_node.property().is_black() {
            self.delete_fixup(x);
        }
        
        true
    }

    fn dfs(stack: &mut Vec<NodePtr<T>>, node: &NodeType<T>) {
        let mut node = node.clone();

        while node.is_some() {
            let left = RBTree::cvt_to_node(&node).left().clone();
            match node {
                Some(x) => stack.push(x),
                _ => {},
            };
            node = left;
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            stack: Vec::with_capacity(self.len()),
            len: self.len,
            phantom: PhantomData,
        }
    }
    
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            stack: Vec::with_capacity(self.len()),
            len: self.len(),
            tree: self,
            phantom: PhantomData,
        }
    }
}

impl<T> Default for RBTree<T> {
    fn default() -> Self {
        RBTree::new()
    }
}


impl<T> IntoIterator for RBTree<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let mut v = Vec::with_capacity(self.len());

        RBTree::dfs(&mut v, &self.root);

        IntoIter {
            tree: self,
            stack: v,
            phantom: PhantomData
        }
    }
}

impl<'a, T> IntoIterator for &'a RBTree<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut RBTree<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Drop for RBTree<T> {
    fn drop(&mut self) {
        let mut v = Vec::with_capacity(self.len());
        RBTree::dfs(&mut v, &self.root);

        match v.pop() {
            Some(x) => unsafe {
                let now_node = x.as_ref().right();
                RBTree::dfs(&mut v, now_node);
                Box::from_raw(x.as_ptr()).into_element();
            },
            _ => {}
        };
    }
}


impl<T: fmt::Debug> fmt::Debug for RBTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

#[derive(Clone)]
pub struct Iter<'a, T: 'a> {
    stack: Vec<NodePtr<T>>,
    len: usize,
    phantom: PhantomData<&'a BNode<T, RBTreeNodeProperty<T>>>,
}

/// 中序遍历二叉树  
pub struct IterMut<'a, T: 'a> {
    tree: &'a mut RBTree<T>,
    stack: Vec<NodePtr<T>>,
    len: usize,
    phantom: PhantomData<&'a BNode<T, RBTreeNodeProperty<T>>>,
}

pub struct IntoIter<T> {
    tree: RBTree<T>,
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
                    RBTree::dfs(&mut self.stack, now_node);
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

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            match self.stack.pop() {
                Some(x) => unsafe {
                    let now_node = x.as_ref().right();
                    RBTree::dfs(&mut self.stack, now_node);
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

impl<'a, T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tree.len() == 0 {
            None
        } else {
            match self.stack.pop() {
                Some(x) => unsafe {
                    let now_node = x.as_ref().right();
                    RBTree::dfs(&mut self.stack, now_node);
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

impl<T: fmt::Debug> fmt::Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Iter").field(&self.len).finish()
    }
}

impl<T: fmt::Debug> fmt::Debug for IterMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IterMut").field(&self.tree).finish()
    }
}

impl<T: fmt::Debug> fmt::Debug for IntoIter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IntoIter").field(&self.tree).finish()
    }
}
