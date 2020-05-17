//! 加密算法Trait


pub trait Cipher {
    
    /// 加密算法输入明文数据的块大小(字节)  
    fn block_size(&self) -> usize;
    
    /// 加密明文数据data_block, 输出数据密文  
    fn encrypt(&mut self, data_block: &[u8]) -> Vec<u8>;
    
    /// 解密密文, 输出原始数据  
    fn decrypt(&mut self, cipher_text: &[u8]) -> Vec<u8>;
}
