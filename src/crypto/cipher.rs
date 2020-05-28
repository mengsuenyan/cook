//! 加密算法Trait


pub trait Cipher {
    
    /// 加密算法输入明文数据的块大小(字节)  
    fn block_size(&self) -> usize;
    
    /// 加密明文数据data_block, 输出数据密文  
    /// 
    /// # panics
    /// 
    /// data_block字节大小不等于block_size()时会panic  
    fn encrypt(&self, dst: &mut Vec<u8>, data_block: &[u8]);
    
    /// 解密密文, 输出原始数据  
    /// 
    /// # pnaics
    /// 
    /// data_block字节大小不等于block_size()时会panic  
    fn decrypt(&self, dst: &mut Vec<u8>, cipher_text: &[u8]);
}
