//! 通用数据结构
//! 

mod linked_list;
mod linear_buf;
mod stack;

pub use linked_list::LinkedList;
pub use linear_buf::LinearBuf;
pub use stack::Stack;


/// 容量分配策略  
/// OnDemand: 按需分配;  
/// DoubleOnDemand: 需求的二倍扩展;  
/// DoubleCapacity: 容量的二倍扩展;  
#[derive(Copy, Clone)]
pub enum CapacityStrategy {
    OnDemand,
    DoubleOnDemand,
    DoubleCapacity,
}