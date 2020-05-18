//! 哈希trait

use std::hash::Hasher;
use std::vec::Vec;

/// 继承标准库中Hasher  
/// 标准库中的Hasher的finish方法返回u64长度的hash值, 当实现GenericHash的Hash
/// 算法计算出来的hash值长度不足64位长度时, 高位补0对齐到64位长度; 当hash长度超过
/// 64位时截断高位, 仅保留低64位  
/// 
/// 实现GenericHasher的标准调用步骤:  
/// ```mermaid
/// graph LR;
///     id0(new) --> id1;
///     id1 -.-> id2(write);
///     id2 --> id3(check_sum);
///     id3 --> id4(sum);
///     id3 --> id5(finish);
///     id4 --> id1;
///     id5 --> id1;
/// ```
pub trait GenericHasher: Hasher
{
    /// 哈希器计算hash值时所使用的潜在块字节长度, 当write时的数据长度
    /// 是块长度的整数倍时, hash值计算的效率可能更高;  
    fn block_size(&self) -> usize;

    /// 重置哈希器
    fn reset(&mut self);

    /// hash值的长度
    fn size(&self) -> usize;

    /// 将Hasher当前计算的hash值添加到data末尾  
    /// 按大端序的方式添加, 即hash值的高字节先添加到data末尾  
    /// 本方法不改变Hasher的内部状态  
    fn append_to_vec(&self, data: &mut Vec<u8>) -> usize;

    /// 将Hasher当前计算的hash值添加到data末尾, 并返回新组装的数据  
    /// 按大端序的方式添加, 即hash值的高字节先添加到data末尾  
    /// 本方法不改变Hasher和data的状态
    fn append_to_slice(&self, data: &[u8]) -> Vec<u8> {
        let mut v = data.to_vec();
        self.append_to_vec(&mut v);
        v
    }
    
    /// 校验和计算  
    fn check_sum(&mut self) -> Result<&Self, &str> {
        Ok(&*self)
    }
}

pub trait GenericHasherSum<T>: GenericHasher {
    /// 返回当前hash值
    fn sum(&self) -> T;
    
    fn sum_as(&self, _v: &T) -> T {
        self.sum()
    }
}
