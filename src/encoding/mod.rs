//! 编码相关

pub trait Encoder<FromT, ToU> {
    type Output;
    type Error: std::fmt::Debug;
    
    /// 将数据编码为另一种形式存储在dst中, dst原来数据会清零;  
    fn encode(&self, dst: ToU, src: FromT) -> Result<Self::Output, Self::Error>;
}

pub trait Decoder<FromT, ToU> {
    type Output;
    type Error: std::fmt::Debug;
    
    /// 将数据编码为另一种形式存储在dst中, dst原来数据会清零;  
    fn decode(&self, dst: ToU, src: FromT) -> Result<Self::Output, Self::Error>;
}

mod bytes;
mod cvt;

pub use bytes::Bytes;
pub use cvt::Cvt;

pub mod base_enc;

pub mod json;