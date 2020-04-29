//! Fnv-1, Fnv-1a算法
//! C = INV
//! Fnv-1
//! C = C * Prime;
//! C = C ^ byte;
//! Fnv-1a
//! C = C ^ byte;
//! C = C * Prime;
//! https://www.cnblogs.com/mengsuenyan/p/12802387.html
//! https://mengsuenyan.gitee.io/docs/CS/%E5%B8%B8%E7%94%A8%E6%A0%A1%E9%AA%8C%E5%92%8C(Hash)%E7%AE%97%E6%B3%95.html

use crate::hash::{GenericHasher, GenericHasherSum};
use std::hash::Hasher;

const FNV_OFFSET32: u32 = 2166136261u32;
const FNV_OFFSET64: u64 = 14695981039346656037u64;
const FNV_OFFSET128: u128 = 0x6c62272e07bb014262b821756295c58du128;
const FNV_PRIME32: u32 = 16777619u32;
const FNV_PRIME64: u64 = 1099511628211u64;
const FNV_PRIME128: u128 = 0x13bu128 + (2u128 << 88);

const FNV32_SIZE: usize = 4;
const FNV64_SIZE: usize = 8;
const FNV128_SIZE: usize = 16;

/// Fnv算法有Fnv-1和Fnv-1a两种, 每种对应如下:  
/// Fnv-1: Fnv32, Fnv64, Fnv128;  
/// Fnv-1a: Fnva32, Fnva64, Fnva128;  
/// 后缀32/64/128表示计算出来的hash值位长度;  
/// 注意: hash值的截断和对齐. 例如Fnv64哈希器, 调用sum()->u32则会发生截断(仅保留低32位);  
enum FnvMode {
    Fnv32 { digest: u32 },
    Fnv64 { digest: u64 },
    Fnv128 { digest: u128 },
    Fnva32 { digest: u32 },
    Fnva64 { digest: u64 },
    Fnva128 { digest: u128 },
}

/// Fnv哈希生成器
pub struct Fnv {
    mode: FnvMode,
}

macro_rules! fnv_update_macro {
    ($FucName: ident, $FucNamea: ident, $Prime: ident, $Digest: ty) => {
        #[inline]
        fn $FucName(digest: $Digest, bytes: &[u8]) -> $Digest {
            let mut h = digest;
            for &b in bytes.iter() {
                h = h.wrapping_mul($Prime);
                h ^= b as $Digest;
            }
            h
        }

        #[inline]
        fn $FucNamea(digest: $Digest, bytes: &[u8]) -> $Digest {
            let mut h = digest;
            for &b in bytes.iter() {
                h ^= b as $Digest;
                h = h.wrapping_mul($Prime);
            }
            h
        }
    };
}

impl Fnv {
    /// 返回hash值长度最大程度满足字节长度want_hash_size的Fnv Hasher  
    pub fn new(want_hash_size: usize, is_fnva: bool) -> Fnv {
        Fnv {
            mode: match is_fnva {
                true => {
                    if want_hash_size > FNV64_SIZE {
                        FnvMode::Fnva128 {
                            digest: FNV_OFFSET128,
                        }
                    } else if want_hash_size > FNV32_SIZE {
                        FnvMode::Fnva64 {
                            digest: FNV_OFFSET64,
                        }
                    } else {
                        FnvMode::Fnva32 {
                            digest: FNV_OFFSET32,
                        }
                    }
                }
                _ => {
                    if want_hash_size > FNV64_SIZE {
                        FnvMode::Fnv128 {
                            digest: FNV_OFFSET128,
                        }
                    } else if want_hash_size > FNV32_SIZE {
                        FnvMode::Fnv64 {
                            digest: FNV_OFFSET64,
                        }
                    } else {
                        FnvMode::Fnv32 {
                            digest: FNV_OFFSET32,
                        }
                    }
                }
            },
        }
    }
    
    /// 转换Fnv hash值的大小及算法类型, 该函数会重置Fnv的状态  
    pub fn switch_size_type(&mut self, want_hash_size: usize, is_fnva: bool) {
        
        let fnv = Fnv::new(want_hash_size, is_fnva);
        
        std::mem::replace(self, fnv);
    }

    /// 转换Fnv hash值的大小, 该函数会重置Fnv的状态  
    pub fn switch_size(&mut self, want_hash_size: usize) {
        self.switch_size_type(want_hash_size, self.is_fnva());
    }
    
    /// 转换Fnv hash值的类型, 该函数会重置Fnv的状态  
    pub fn switch_type (&mut self, is_fnva: bool) {
        self.switch_size_type(self.size(), is_fnva);
    }
    
    /// 是否是fnv-1a
    fn is_fnva(&self) -> bool {
        match self.mode {
            FnvMode::Fnv128 {..} | FnvMode::Fnv64 {..} | FnvMode::Fnv32 {..} => false,
            _ => true
        }
    }

    fnv_update_macro!(update_fnv32, update_fnva32, FNV_PRIME32, u32);
    fnv_update_macro!(update_fnv64, update_fnva64, FNV_PRIME64, u64);
    fnv_update_macro!(update_fnv128, update_fnva128, FNV_PRIME128, u128);
}

impl Hasher for Fnv {
    fn finish(&self) -> u64 {
        match &self.mode {
            FnvMode::Fnv128 { digest } | FnvMode::Fnva128 { digest } => {
                (digest & (std::u64::MAX as u128)) as u64
            },
            FnvMode::Fnv64 { digest } | FnvMode::Fnva64 { digest } => *digest,
            FnvMode::Fnv32 { digest } | FnvMode::Fnva32 { digest } => u64::from(*digest),
        }
    }

    fn write(&mut self, bytes: &[u8]) {
        match &mut self.mode {
            FnvMode::Fnv32 { digest } => {
                *digest = Fnv::update_fnv32(*digest, bytes);
            },
            FnvMode::Fnv64 { digest } => {
                *digest = Fnv::update_fnv64(*digest, bytes);
            },
            FnvMode::Fnv128 { digest } => {
                *digest = Fnv::update_fnv128(*digest, bytes);
            },
            FnvMode::Fnva32 { digest} => {
                *digest = Fnv::update_fnva32(*digest, bytes);
            },
            FnvMode::Fnva64 {digest} => {
                *digest = Fnv::update_fnva64(*digest, bytes);
            },
            FnvMode::Fnva128 {digest} => {
                *digest = Fnv::update_fnva128(*digest, bytes)
            }
        }
    }
}

impl GenericHasher for Fnv {
    fn block_size(&self) -> usize {
        match self.mode {
            FnvMode::Fnva128 {..} | FnvMode::Fnv128 {..} => FNV128_SIZE,
            FnvMode::Fnva64 {..} | FnvMode::Fnv64 {..} => FNV64_SIZE,
            FnvMode::Fnva32 {..} | FnvMode::Fnv32 {..} => FNV32_SIZE,
        }
    }

    fn reset(&mut self) {
        match &mut self.mode {
            FnvMode::Fnva128 {digest}| FnvMode::Fnv128 {digest} => *digest = FNV_OFFSET128,
            FnvMode::Fnva64 {digest} | FnvMode::Fnv64 {digest} => *digest = FNV_OFFSET64,
            FnvMode::Fnva32 {digest} | FnvMode::Fnv32 {digest} => *digest = FNV_OFFSET32,
        }
    }

    fn size(&self) -> usize {
        match self.mode {
            FnvMode::Fnva128 {..} | FnvMode::Fnv128 {..} => FNV128_SIZE,
            FnvMode::Fnva64 {..} | FnvMode::Fnv64 {..} => FNV64_SIZE,
            FnvMode::Fnva32 {..} | FnvMode::Fnv32 {..} => FNV32_SIZE,
        }
    }

    fn append_to_vec(&self, data: &mut Vec<u8>) -> usize {
        let len = data.len();
        match self.mode {
            FnvMode::Fnva128 {digest}| FnvMode::Fnv128 {digest} => {
                for &ele in digest.to_be_bytes().iter() {
                    data.push(ele);
                }
            },
            FnvMode::Fnva64 {digest} | FnvMode::Fnv64 {digest} => {
                for &ele in digest.to_be_bytes().iter() {
                    data.push(ele);
                }
            },
            FnvMode::Fnva32 {digest} | FnvMode::Fnv32 {digest} => {
                for &ele in digest.to_be_bytes().iter() {
                    data.push(ele);
                }
            },
        };
        
        data.len() - len
    }

    fn append_to_slice(&self, data: &[u8]) -> Vec<u8> {
        let mut v = data.to_vec();
        self.append_to_vec(&mut v);
        v
    }
}

macro_rules! fnv_impl_generic_hasher_sum_macro {
    ($Type: ty, $Expr1: expr, $Expr2: expr, $Expr3: expr) => {
        impl GenericHasherSum<$Type> for Fnv {
            fn sum(&self) -> $Type {
                match self.mode {
                    FnvMode::Fnva128 {digest}| FnvMode::Fnv128 {digest} => $Expr1(digest),
                    FnvMode::Fnva64 {digest} | FnvMode::Fnv64 {digest} => $Expr2(digest),
                    FnvMode::Fnva32 {digest} | FnvMode::Fnv32 {digest} => $Expr3(digest),
                }
            }
        }
    };
}

fnv_impl_generic_hasher_sum_macro!(u32, |d| { (d & (std::u32::MAX as u128)) as u32}, |d|{ (d & (std::u32::MAX as u64)) as u32}, |d|{d});
fnv_impl_generic_hasher_sum_macro!(u64, |d| { (d & (std::u64::MAX as u128)) as u64}, |d|{ d }, |d|{d as u64});
fnv_impl_generic_hasher_sum_macro!(u128, |d| {d}, |d|{d as u128}, |d|{d as u128});

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
