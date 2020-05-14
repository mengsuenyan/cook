//! 二叉树的节点  

use std::ptr::NonNull;

/// 左/右/父节点, 属性, 及元素  
/// 特别注意: 节点超出生命期被drop时, 其指向的左/右子节点和父节点不会被drop,
/// 需要使用者自己负责drop左/右子节点和父节点;  
pub struct BNode<T, P> {
    left: Option<NonNull<BNode<T, P>>>,
    right: Option<NonNull<BNode<T, P>>>,
    parent: Option<NonNull<BNode<T, P>>>,
    property: P,
    element: T,
}

macro_rules! node_most_macro {
    ($NodeName: ident, $Obj: ident) => {
        let $NodeName = $Obj.$NodeName;
        match $NodeName {
            Some(p) => {
                let mut $NodeName = p.as_ptr();
                unsafe {
                    while let Some(p) = (*$NodeName).$NodeName {
                        $NodeName = p.as_ptr();
                    }
                }
                return NonNull::new($NodeName);
            },
            None => {
                return None;
            }
        }
    };
}

impl<T, P> BNode<T, P> {
    pub fn new(element: T, property: P) -> Self {
        BNode {
            left: None,
            right: None,
            parent: None,
            property,
            element
        }
    }
    
    /// 获得Box分配的BNode节点所有权, 并释放该内存  
    pub fn into_element(self: Box<Self>) -> T {
        self.element
    }
    
    /// 返回最左边的节点(不包括本身)  
    pub fn left_most(&self) -> Option<NonNull<Self>> {
        node_most_macro!(left, self);
    }
    
    /// 返回最右边的节点(不包括本身)  
    pub fn right_most(&self) -> Option<NonNull<Self>> {
        node_most_macro!(right, self);
    }
    
    /// 返回其父节点(不包括本身)  
    pub fn parent_most(&self) -> Option<NonNull<Self>> {
        node_most_macro!(parent, self);
    }
    
    /// 内部使用node必须是Some
    fn inner_to_ref(node: &Option<NonNull<Self>>) -> &Self {
        unsafe {
            node.as_ref().unwrap().as_ref()
        }
    }
    
    /// (父节点, 父节点的右子节点)  
    pub fn parent_right(&self) -> (Option<NonNull<Self>>, Option<NonNull<Self>>) {
        if self.parent.is_some() {
            let node = BNode::inner_to_ref(self.parent());
            if node.right.is_some() {
                (self.parent, node.right)
            } else {
                (self.parent, None)
            }
        } else {
            (None, None)
        }
    }
    
    /// (父节点, 父节点的左子节点)  
    pub fn parent_left(&self) -> (Option<NonNull<Self>>, Option<NonNull<Self>>) {
        if self.parent.is_some() {
            let node = BNode::inner_to_ref(self.parent());
            if node.left.is_some() {
                (self.parent, node.left)
            } else {
                (self.parent, None)
            }
        } else {
            (None, None)
        }
    }
}

//TODO: pro_macro_derive继承实现  
impl <T, P> BNode<T, P> {
    pub fn property(&self) -> &P {
        &self.property
    }
    
    pub fn property_mut(&mut self) -> &mut P {
        &mut self.property
    }
    
    pub fn left(&self) -> &Option<NonNull<Self>> {
        &self.left
    }
    
    pub fn left_mut(&mut self) -> &mut Option<NonNull<Self>> {
        &mut self.left
    }

    pub fn right(&self) -> &Option<NonNull<Self>> {
        &self.right
    }

    pub fn right_mut(&mut self) -> &mut Option<NonNull<Self>> {
        &mut self.right
    }

    pub fn parent(&self) -> &Option<NonNull<Self>> {
        &self.parent
    }

    pub fn parent_mut(&mut self) -> &mut Option<NonNull<Self>> {
        &mut self.parent
    }
    
    pub fn element(&self) -> &T {
        &self.element
    }
    
    pub fn element_mut(&mut self) -> &mut T {
        &mut self.element
    }
}


