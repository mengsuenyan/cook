//! CRC循环冗余校验  
//! 多项式二进制顺序为: 低位在前, 高位在后  
//! 

use std::hash::Hasher;
use crate::hash::{GenericHasher, GenericHasherSum};

/// IEEE is by far and away the most common CRC-32 polynomial.
/// Used by ethernet (IEEE 802.3), v.42, fddi, gzip, zip, png, ...
const CRC32_IEEE: u32 = 0xedb88320u32;

/// Castagnoli's polynomial, used in iSCSI.
/// Has better error detection characteristics than IEEE.
/// https://dx.doi.org/10.1109/26.231911
const CRC32_CASTAGNOLI: u32 = 0x82f63b78u32;

/// Koopman's polynomial.
/// Also has better error detection characteristics than IEEE.
/// https://dx.doi.org/10.1109/DSN.2002.1028931
const CRC32_KOOPMAN: u32 = 0xeb31d82eu32;

/// 生成多项式poly的二进制顺序为: 低位在前, 高位在后, 低位对应低次幂, 高位对应高次幂  
pub struct Crc32 {
    digest: u32,
    table: [u32; 256]
}


impl Crc32 {
    /// 生成多项式poly的二进制顺序为: 低位在前, 高位在后, 低位对应低次幂, 高位对应高次幂  
    pub fn new(poly: u32) -> Crc32 {
        Crc32 {
            digest: 0,
            table: Crc32::make_table(poly)
        }
    }

    /// IEEE is by far and away the most common CRC-32 polynomial.
    /// Used by ethernet (IEEE 802.3), v.42, fddi, gzip, zip, png, ...
    pub fn from_ieee_poly() -> Crc32 {
        Crc32::new(CRC32_IEEE)
    }

    /// Castagnoli's polynomial, used in iSCSI.
    /// Has better error detection characteristics than IEEE.
    /// https://dx.doi.org/10.1109/26.231911
    pub fn from_castagnoli_poly() -> Crc32 {
        Crc32::new(CRC32_CASTAGNOLI)
    }

    /// Koopman's polynomial.
    /// Also has better error detection characteristics than IEEE.
    /// https://dx.doi.org/10.1109/DSN.2002.1028931
    pub fn from_koopman_poly() -> Crc32 {
        Crc32::new(CRC32_KOOPMAN)
    }
    
    fn update(&mut self, bytes: &[u8]) -> u32 {
        let mut crc = !self.digest;
        for &ele in bytes {
            let idx = ((crc & 0xff) as u8) ^ ele;
            crc = self.table[idx as usize] ^ (crc >> 8)
        }
        !crc
    }

    fn make_table(poly: u32) -> [u32; 256] {
        let mut v = [0u32; 256];
        for (i, ele) in v.iter_mut().enumerate() {
            let mut crc = i as u32;
            for _ in 0..8 {
                if crc & 1 == 1 {
                    crc = (crc >> 1) ^ poly;
                } else {
                    crc >>= 1;
                }
            }
            *ele = crc;
        }
        v
    }
}

impl Hasher for Crc32 {
    fn finish(&self) -> u64 {
        self.digest as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        self.digest = self.update(bytes);
    }
}

impl GenericHasher for Crc32 {
    fn block_size(&self) -> usize {
        std::mem::size_of_val(&self.digest)
    }

    fn reset(&mut self) {
        self.digest = 0;
    }

    fn size(&self) -> usize {
        std::mem::size_of_val(&self.digest)
    }

    fn append_to_vec(&self, data: &mut Vec<u8>) -> usize {
        let len = data.len();
        let v = self.sum().to_be_bytes();
        for &ele in v.iter() {
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

impl GenericHasherSum<u32> for Crc32 {
    fn sum(&self) -> u32 {
        self.digest
    }
}


/// The ISO polynomial, defined in ISO 3309 and used in HDLC.
const CRC64_ISO:u64 = 0xD800000000000000u64;

/// The ECMA polynomial, defined in ECMA 182.
const CRC64_ECMA: u64 = 0xC96C5795D7870F42u64;

pub struct Crc64 {
    digest: u64,
    table: [u64;256]
}

impl Crc64 {
    /// 生成多项式poly的二进制顺序为: 低位在前, 高位在后, 低位对应低次幂, 高位对应高次幂  
    pub fn new(poly: u64) -> Crc64 {
        Crc64 {
            digest: 0,
            table: Crc64::make_table(poly)
        }
    }

    /// The ISO polynomial, defined in ISO 3309 and used in HDLC.
    pub fn from_iso_poly() -> Crc64 {
        Crc64::new(CRC64_ISO)
    }

    /// The ECMA polynomial, defined in ECMA 182.
    pub fn from_ecma_poly() -> Crc64 {
        Crc64::new(CRC64_ECMA)
    }

    fn update(&mut self, bytes: &[u8]) -> u64 {
        let mut crc = !self.digest;
        for &ele in bytes {
            let idx = ((crc & 0xff) as u8) ^ ele;
            crc = self.table[idx as usize] ^ (crc >> 8)
        }
        !crc
    }

    fn make_table(poly: u64) -> [u64; 256] {
        let mut v = [0u64; 256];
        for (i, ele) in v.iter_mut().enumerate() {
            let mut crc = i as u64;
            for _ in 0..8 {
                if crc & 1 == 1 {
                    crc = (crc >> 1) ^ poly;
                } else {
                    crc >>= 1;
                }
            }
            *ele = crc;
        }
        v
    }
}

impl Hasher for Crc64 {
    fn finish(&self) -> u64 {
        self.digest
    }

    fn write(&mut self, bytes: &[u8]) {
        self.digest = self.update(bytes);
    }
}

impl GenericHasher for Crc64 {
    fn block_size(&self) -> usize {
        std::mem::size_of_val(&self.digest)
    }

    fn reset(&mut self) {
        self.digest = 0;
    }

    fn size(&self) -> usize {
        std::mem::size_of_val(&self.digest)
    }

    fn append_to_vec(&self, data: &mut Vec<u8>) -> usize {
        let len = data.len();
        let v = self.sum().to_be_bytes();
        for &ele in v.iter() {
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

impl GenericHasherSum<u64> for Crc64 {
    fn sum(&self) -> u64 {
        self.digest
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn crc64() {
        let cases = [
        (0x0u64, 0x0u64, ""),
        (0x3420000000000000u64, 0x330284772e652b05u64, "a"),
        (0x36c4200000000000u64, 0xbc6573200e84b046u64, "ab"),
        (0x3776c42000000000u64, 0x2cd8094a1a277627u64, "abc"),
        (0x336776c420000000u64, 0x3c9d28596e5960bau64, "abcd"),
        (0x32d36776c4200000u64, 0x40bdf58fb0895f2u64, "abcde"),
        (0x3002d36776c42000u64, 0xd08e9f8545a700f4u64, "abcdef"),
        (0x31b002d36776c420u64, 0xec20a3a8cc710e66u64, "abcdefg"),
        (0xe21b002d36776c4u64, 0x67b4f30a647a0c59u64, "abcdefgh"),
        (0x8b6e21b002d36776u64, 0x9966f6c89d56ef8eu64, "abcdefghi"),
        (0x7f5b6e21b002d367u64, 0x32093a2ecd5773f4u64, "abcdefghij"),
        (0x8ec0e7c835bf9cdfu64, 0x8a0825223ea6d221u64, "Discard medicine more than two years old."),
        (0xc7db1759e2be5ab4u64, 0x8562c0ac2ab9a00du64, "He who has a shady past knows that nice guys finish last."),
        (0xfbf9d9603a6fa020u64, 0x3ee2a39c083f38b4u64, "I wouldn't marry him with a ten foot pole."),
        (0xeafc4211a6daa0efu64, 0x1f603830353e518au64, "Free! Free!/A trip/to Mars/for 900/empty jars/Burma Shave"),
        (0x3e05b21c7a4dc4dau64, 0x2fd681d7b2421fdu64, "The days of the digital watch are numbered.  -Tom Stoppard"),
        (0x5255866ad6ef28a6u64, 0x790ef2b16a745a41u64, "Nepal premier won't resign."),
        (0x8a79895be1e9c361u64, 0x3ef8f06daccdcddfu64, "For every action there is an equal and opposite government program."),
        (0x8878963a649d4916u64, 0x49e41b2660b106du64, "His money is twice tainted: 'taint yours and 'taint mine."),
        (0xa7b9d53ea87eb82fu64, 0x561cc0cfa235ac68u64, "There is no reason for any individual to have a computer in their home. -Ken Olsen, 1977"),
        (0xdb6805c0966a2f9cu64, 0xd4fe9ef082e69f59u64, "It's a tiny change to the code and not completely disgusting. - Bob Manchek"),
        (0xf3553c65dacdadd2u64, 0xe3b5e46cd8d63a4du64, "size:  a.out:  bad magic"),
        (0x9d5e034087a676b9u64, 0x865aaf6b94f2a051u64, "The major problem is with sendmail.  -Mark Horton"),
        (0xa6db2d7f8da96417u64, 0x7eca10d2f8136eb4u64, "Give me a rock, paper and scissors and I will move the world.  CCFestoon"),
        (0x325e00cd2fe819f9u64, 0xd7dd118c98e98727u64, "If the enemy is within range, then so are you."),
        (0x88c6600ce58ae4c6u64, 0x70fb33c119c29318u64, "It's well we cannot hear the screams/That we create in others' dreams."),
        (0x28c4a3f3b769e078u64, 0x57c891e39a97d9b7u64, "You remind me of a TV show, but that's all right: I watch it anyway."),
        (0xa698a34c9d9f1dcau64, 0xa1f46ba20ad06eb7u64, "C is as portable as Stonehedge!!"),
        (0xf6c1e2a8c26c5cfcu64, 0x7ad25fafa1710407u64, "Even if I could be Shakespeare, I think I should still choose to be Faraday. - A. Huxley"),
        (0xd402559dfe9b70cu64, 0x73cef1666185c13fu64, "The fugacity of a constituent in a mixture of gases at a given temperature is proportional to its mole fraction.  Lewis-Randall Rule"),
        (0xdb6efff26aa94946u64, 0xb41858f73c389602u64, "How can you write a big system without C++?  -Paul Glick"),
        (0xe7fcf1006b503b61u64, 0x27db187fc15bbc72u64, "This is a test of the emergency broadcast system."),
        ];
        
        let (mut iso_crc, mut ecma_crc) = (Crc64::from_iso_poly(), Crc64::from_ecma_poly()); 
        for ele in cases.iter() {
            iso_crc.write(ele.2.as_bytes());
            ecma_crc.write(ele.2.as_bytes());
            assert_eq!(ele.0, iso_crc.sum());
            assert_eq!(ele.1, ecma_crc.sum());
            iso_crc.reset();
            ecma_crc.reset();
        }
    }
    
    #[test]
    fn crc32() {
        let cases = [
            (0x0u32, 0x0u32, ""),
            (0xe8b7be43u32, 0xc1d04330u32, "a"),
            (0x9e83486du32, 0xe2a22936u32, "ab"),
            (0x352441c2u32, 0x364b3fb7u32, "abc"),
            (0xed82cd11u32, 0x92c80a31u32, "abcd"),
            (0x8587d865u32, 0xc450d697u32, "abcde"),
            (0x4b8e39efu32, 0x53bceff1u32, "abcdef"),
            (0x312a6aa6u32, 0xe627f441u32, "abcdefg"),
            (0xaeef2a50u32, 0xa9421b7u32, "abcdefgh"),
            (0x8da988afu32, 0x2ddc99fcu32, "abcdefghi"),
            (0x3981703au32, 0xe6599437u32, "abcdefghij"),
            (0x6b9cdfe7u32, 0xb2cc01feu32, "Discard medicine more than two years old."),
            (0xc90ef73fu32, 0xe28207fu32, "He who has a shady past knows that nice guys finish last."),
            (0xb902341fu32, 0xbe93f964u32, "I wouldn't marry him with a ten foot pole."),
            (0x42080e8u32, 0x9e3be0c3u32, "Free! Free!/A trip/to Mars/for 900/empty jars/Burma Shave"),
            (0x154c6d11u32, 0xf505ef04u32, "The days of the digital watch are numbered.  -Tom Stoppard"),
            (0x4c418325u32, 0x85d3dc82u32, "Nepal premier won't resign."),
            (0x33955150u32, 0xc5142380u32, "For every action there is an equal and opposite government program."),
            (0x26216a4bu32, 0x75eb77ddu32, "His money is twice tainted: 'taint yours and 'taint mine."),
            (0x1abbe45eu32, 0x91ebe9f7u32, "There is no reason for any individual to have a computer in their home. -Ken Olsen, 1977"),
            (0xc89a94f7u32, 0xf0b1168eu32, "It's a tiny change to the code and not completely disgusting. - Bob Manchek"),
            (0xab3abe14u32, 0x572b74e2u32, "size:  a.out:  bad magic"),
            (0xbab102b6u32, 0x8a58a6d5u32, "The major problem is with sendmail.  -Mark Horton"),
            (0x999149d7u32, 0x9c426c50u32, "Give me a rock, paper and scissors and I will move the world.  CCFestoon"),
            (0x6d52a33cu32, 0x735400a4u32, "If the enemy is within range, then so are you."),
            (0x90631e8du32, 0xbec49c95u32, "It's well we cannot hear the screams/That we create in others' dreams."),
            (0x78309130u32, 0xa95a2079u32, "You remind me of a TV show, but that's all right: I watch it anyway."),
            (0x7d0a377fu32, 0xde2e65c5u32, "C is as portable as Stonehedge!!"),
            (0x8c79fd79u32, 0x297a88edu32, "Even if I could be Shakespeare, I think I should still choose to be Faraday. - A. Huxley"),
            (0xa20b7167u32, 0x66ed1d8bu32, "The fugacity of a constituent in a mixture of gases at a given temperature is proportional to its mole fraction.  Lewis-Randall Rule"),
            (0x8e0bb443u32, 0xdcded527u32, "How can you write a big system without C++?  -Paul Glick"),
        ];
        
        let (mut ieee, mut castagnoli) = (Crc32::from_ieee_poly(), Crc32::from_castagnoli_poly());
        for ele in cases.iter() {
            ieee.write(ele.2.as_bytes());
            castagnoli.write(ele.2.as_bytes());
            assert_eq!(ele.0, ieee.sum());
            assert_eq!(ele.1, castagnoli.sum());
            ieee.reset();
            castagnoli.reset();
        }
    }
}
