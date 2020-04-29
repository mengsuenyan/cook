//! 哈希trait

use std::hash::Hasher;
use std::vec::Vec;

/// 继承标准库中Hasher  
/// 标准库中的Hasher的finish方法返回u64长度的hash值, 当实现GenericHash的Hash
/// 算法计算出来的hash值长度不足64位长度时, 高位补0对齐到64位长度; 当hash长度超过
/// 64位时截断高位, 仅保留低64位  
pub trait GenericHasher: Hasher {
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
    fn append_to_slice(&self, data: &[u8]) -> Vec<u8>;
}

pub trait GenericHasher32: GenericHasher {
    /// 返回32位hash值
    fn finish32() -> u32;
}

pub trait GenericHasher64: GenericHasher {
    /// 返回64位Hash值
    fn finish64() -> u32;
}

pub trait GenericHasher128: GenericHasher {
    /// 返回128位Hash值
    fn finish128() -> u128;
}
