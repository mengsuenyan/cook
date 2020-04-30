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
const FNV_PRIME128: u128 = 0x13bu128 + (1u128 << 88);

macro_rules! fnv_update_macro {
    ($FnvName: ident, $DigestType: ty, $Prime: ident, $PlaceHolder1: literal, $PlaceHolder2: literal) => {
        fn update(&self, bytes: &[u8]) -> $DigestType {
            let mut d = self.digest;
            for &b in bytes.iter() {
                d ^= b as $DigestType;
                d = d.wrapping_mul($Prime);
            }
            d
        }
    };
    ($FnvName: ident, $DigestType: ty, $Prime: ident, $Placeholder: literal) => {
        fn update(&self, bytes: &[u8]) -> $DigestType {
            let mut d = self.digest;
            for &b in bytes.iter() {
                d = d.wrapping_mul($Prime);
                d ^= b as $DigestType;
            }
            d
        }
    };
}

macro_rules! fnv_generate_code_macro {
    ($FnvName: ident, $DigestType: ty, $Offset: ident, $Prime: ident, $($IsFnva: literal),+) => {
        pub struct $FnvName {
            digest: $DigestType,
        }
        
        impl $FnvName {
            pub fn new() -> $FnvName {
                $FnvName {
                    digest: $Offset
                }
            }
            
            fnv_update_macro!($FnvName, $DigestType, $Prime, $($IsFnva),*);
        }
        
        impl Hasher for $FnvName {
            fn finish(&self) -> u64 {
                if std::mem::size_of::<$DigestType>() > std::mem::size_of::<u128>() {
                    ((self.digest as u128) & (std::u64::MAX as u128)) as u64
                } else {
                    self.digest as u64
                }
            }
        
            fn write(&mut self, bytes: &[u8]) {
                self.digest = self.update(bytes);
            }
        }
        
        impl GenericHasher for $FnvName {
            fn block_size(&self) -> usize {
                std::mem::size_of::<$DigestType>()
            }
            
            fn reset(&mut self) {
                self.digest = $Offset;
            }
            
            fn size(&self) -> usize {
                std::mem::size_of::<$DigestType>()
            }
            
            fn append_to_vec(&self, data: &mut Vec<u8>) -> usize {
                let len = data.len();
                let d = self.digest.to_be_bytes();
                for &ele in d.iter() {
                    data.push(ele);
                }
                data.len() - len
            }
            
            fn append_to_slice(&self, data: &[u8]) -> Vec<u8> {
                let mut v = data.to_vec();
                self.append_to_vec(&mut v);
                v
            }
        }
        
        impl GenericHasherSum<$DigestType> for $FnvName {
            fn sum(&self) -> $DigestType {
                self.digest
            }
        }
    };
}

fnv_generate_code_macro!(Fnv32, u32, FNV_OFFSET32, FNV_PRIME32, false);
fnv_generate_code_macro!(Fnva32, u32, FNV_OFFSET32, FNV_PRIME32, true, true);
fnv_generate_code_macro!(Fnv64, u64, FNV_OFFSET64, FNV_PRIME64, false);
fnv_generate_code_macro!(Fnva64, u64, FNV_OFFSET64, FNV_PRIME64, true, true);
fnv_generate_code_macro!(Fnv128, u128, FNV_OFFSET128, FNV_PRIME128, false);
fnv_generate_code_macro!(Fnva128, u128, FNV_OFFSET128, FNV_PRIME128, true, true);

#[cfg(test)]
mod tests {
    use super::*;
    
    //These test cases come from golang source code

    #[test]
    fn fnv32() {
        let v = [
            (0x811c9dc5u32, ""),
            (0x050c5d7eu32, "a"),
            (0x70772d38u32, "ab"),
            (0x439c2f4bu32, "abc"),
        ];
        
        let mut h = Fnv32::new();
        for ele in v.iter() {
            h.write(ele.1.as_bytes());
            assert_eq!(ele.0, h.sum());
            h.reset();
        }
    }
    
    #[test]
    fn fnva32() {
        let v = [
            (0x811c9dc5u32, ""),
            (0xe40c292cu32, "a"),
            (0x4d2505cau32, "ab"), 
            (0x1a47e90bu32, "abc"), 
        ];

        let mut h = Fnva32::new();
        for ele in v.iter() {
            h.write(ele.1.as_bytes());
            assert_eq!(ele.0, h.sum());
            h.reset();
        }
    }

    #[test]
    fn fnv64() {
        let v = [
            (0xcbf29ce484222325u64, ""),
            (0xaf63bd4c8601b7beu64, "a"), 
            (0x08326707b4eb37b8u64, "ab"), 
            (0xd8dcca186bafadcbu64, "abc"), 
        ];

        let mut h = Fnv64::new();
        for ele in v.iter() {
            h.write(ele.1.as_bytes());
            assert_eq!(ele.0, h.sum());
            h.reset();
        }
    }
    
    #[test]
    fn fnva64() {
        let v = [
            (0xcbf29ce484222325u64, ""),
            (0xaf63dc4c8601ec8cu64, "a"),
            (0x089c4407b545986au64, "ab"),
            (0xe71fa2190541574bu64, "abc"),
        ];

        let mut h = Fnva64::new();
        for ele in v.iter() {
            h.write(ele.1.as_bytes());
            assert_eq!(ele.0, h.sum());
            h.reset();
        }
    }
    
    #[test]
    fn fnv128() {
        let v = [
            (0x6c62272e07bb014262b821756295c58du128, ""),
            (0xd228cb69101a8caf78912b704e4a141eu128, "a"),
            (0x880945aeeab1be95aa073305526c088u128, "ab"),
            (0xa68bb2a4348b5822836dbc78c6aee73bu128, "abc"),
        ];

        let mut h = Fnv128::new();
        for ele in v.iter() {
            h.write(ele.1.as_bytes());
            assert_eq!(ele.0, h.sum());
            h.reset();
        }
    }
    
    #[test]
    fn fnva128() {
        let v = [
            (0x6c62272e07bb014262b821756295c58du128, ""),
            (0xd228cb696f1a8caf78912b704e4a8964u128, "a"),
            (0x08809544bbab1be95aa0733055b69a62u128, "ab"),
            (0xa68d622cec8b5822836dbc7977af7f3bu128, "abc"),
        ];

        let mut h = Fnva128::new();
        for ele in v.iter() {
            h.write(ele.1.as_bytes());
            assert_eq!(ele.0, h.sum());
            h.reset();
        }
    }
}
