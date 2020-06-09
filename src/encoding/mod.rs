//! 编码相关

pub trait Encoder {
    type Item;
    type Output;
    
    /// 将数据编码为另一种形式存储在dst中, dst原来数据会清零;  
    fn encode(&self, dst: &mut Vec<Self::Item>, src: &[Self::Item]) -> Result<Self::Output, &'static str>;

    /// 将数据编码为另一种形式存储在dst中, 数据会附加到dst尾部;  
    fn encode_append(&self, dst: &mut Vec<Self::Item>, src: &[Self::Item]) -> Result<Self::Output, &'static str> {
        let mut buf = Vec::new();
        
        let r = self.encode(&mut buf, src);
        
        dst.append(&mut buf);
        r
    }
}

pub trait Decoder {
    type Item;
    type Output;
    
    /// 将数据编码为另一种形式存储在dst中, dst原来数据会清零;  
    fn decode(&self, dst: &mut Vec<Self::Item>, src: &[Self::Item]) -> Result<Self::Output, &'static str>;

    fn decode_append(&self, dst: &mut Vec<Self::Item>, src: &[Self::Item]) -> Result<Self::Output, &'static str> {
        let mut buf = Vec::new();
        
        let r = self.decode(&mut buf, src);
        
        dst.append(&mut buf);
        r
    }
}

pub trait Transformer: Encoder + Decoder {}

mod bytes;
mod cvt;

pub use bytes::Bytes;
pub use cvt::Cvt;

pub mod base_enc;

pub mod json;