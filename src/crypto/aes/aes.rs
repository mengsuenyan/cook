//! AES加密
//! FIPS 197  
//! https://www.cnblogs.com/mengsuenyan/p/12697694.html

use crate::crypto::aes::const_tables as mct;
use crate::crypto::Cipher;

/// N_k: 密钥的字长;  
/// N_b: 明文块的字长;  
/// N_r: 加密轮数;  
/// 字长: 32位bits;  
/// 
/// ||$N_k$|$N_b$|$N_r$|
/// |:---:|:---:|:---:|:---:|
/// |AES-128|4|4|10|
/// |AES-192|6|4|12|
/// |AES-256|8|4|14|
/// 
/// AES-256加密14轮, Key_Schedule后有$N_b$ * ($N_r$ + 1)组密钥  
trait AesCipherBase {
    // 内部调用, 不进行参数检查  
    #[inline]
    fn nk_nb_nr(key: &[u8]) -> (usize, usize, usize) {
        match key.len() {
            16 => (4, 4, 10),
            24 => (6, 4, 12),
            _ => (8, 4, 14),
        }
    }
    
    #[inline]
    fn sub_word(w: u32) -> u32 {
        let idx = w.to_be_bytes();
        let v = [
            mct::AES_SBOX0[idx[0] as usize],
            mct::AES_SBOX0[idx[1] as usize],
            mct::AES_SBOX0[idx[2] as usize],
            mct::AES_SBOX0[idx[3] as usize],
            ];
        u32::from_be_bytes(v)
    }
    
    /// RoundKey, 假设key是合法的  
    fn key_schedule(key: &[u8], enc: &mut [u32; 60], dec: &mut [u32; 60]) {
        let (nk, _, nr) = Self::nk_nb_nr(key);
        
        let mut itr = key.iter();
        for i in 0..nk {
            let v = [*itr.next().unwrap(), *itr.next().unwrap(), *itr.next().unwrap(), *itr.next().unwrap()];
            enc[i] = u32::from_be_bytes(v);
        }
        
        let n = (nr + 1) << 2;
        for i in nk..n {
            let mut t = enc[i - 1];
            if (i % nk) == 0 {
                t = Self::sub_word(t.rotate_left(8)) ^ mct::AES_POWX[(i / nk) - 1];
            } else if (nk > 6) && ((i % nk) == 4) {
                t = Self::sub_word(t);
            }
            enc[i] = enc[i - nk] ^ t;
        }
        
        let mut i = 0;
        while i < n {
            let ei = n - i - 4;
            for j in 0..4 {
                let mut x = enc[ei + j];
                if i > 0 && (i + 4) < n {
                    let v = x.to_be_bytes();
                    let (v0, v1, v2, v3) = (v[0] as usize, v[1] as usize, v[2] as usize, v[3] as usize);
                    x = mct::AES_TD0[mct::AES_SBOX0[v0] as usize] ^ mct::AES_TD1[mct::AES_SBOX0[v1] as usize] ^
                        mct::AES_TD2[mct::AES_SBOX0[v2] as usize] ^ mct::AES_TD3[mct::AES_SBOX0[v3] as usize];
                }
                dec[i+j] = x;
            }
            
            i += 4;
        }
    }
    
    fn crypt_block(key: &[u32], nr: usize, dst: &mut [u8], src: &[u8]) {
        let mut src_itr = src.iter();
        let mut s = [0u32; 4];
        for i in 0..4 {
            let v = [*src_itr.next().unwrap(), *src_itr.next().unwrap(), *src_itr.next().unwrap(), *src_itr.next().unwrap()];
            s[i] = u32::from_be_bytes(v);
        }

        // AddRoundKey
        let (mut s0, mut s1, mut s2, mut s3) = (s[0] ^ key[0], s[1] ^ key[1], s[2] ^ key[2], s[3] ^ key[3]);

        // SubBytes -> ShiftRows -> MixColumns -> AddRoundKey
        let mut k = 4;
        for _ in 0..(nr - 1) {
            let (v0, v1, v2, v3) = (s0.to_be_bytes(), s1.to_be_bytes(), s2.to_be_bytes(), s3.to_be_bytes());
            let t0  = key[k+0] ^ mct::AES_TE0[v0[0] as usize] ^ mct::AES_TE1[v1[1] as usize] ^ mct::AES_TE2[v2[2] as usize] ^ mct::AES_TE3[v3[3] as usize];
            let t1  = key[k+1] ^ mct::AES_TE0[v1[0] as usize] ^ mct::AES_TE1[v2[1] as usize] ^ mct::AES_TE2[v3[2] as usize] ^ mct::AES_TE3[v0[3] as usize];
            let t2  = key[k+2] ^ mct::AES_TE0[v2[0] as usize] ^ mct::AES_TE1[v3[1] as usize] ^ mct::AES_TE2[v0[2] as usize] ^ mct::AES_TE3[v1[3] as usize];
            let t3  = key[k+3] ^ mct::AES_TE0[v3[0] as usize] ^ mct::AES_TE1[v0[1] as usize] ^ mct::AES_TE2[v1[2] as usize] ^ mct::AES_TE3[v2[3] as usize];
            s0 = t0;
            s1 = t1;
            s2 = t2;
            s3 = t3;
            k += 4;
        }
        
        // SubBytes -> ShiftRows -> AddRoundKey
        let (v0, v1, v2, v3) = (s0.to_be_bytes(), s1.to_be_bytes(), s2.to_be_bytes(), s3.to_be_bytes());
        let tmp0 = [mct::AES_SBOX0[v0[0] as usize], mct::AES_SBOX0[v1[1] as usize], mct::AES_SBOX0[v2[2] as usize], mct::AES_SBOX0[v3[3] as usize]];
        let tmp1 = [mct::AES_SBOX0[v1[0] as usize], mct::AES_SBOX0[v2[1] as usize], mct::AES_SBOX0[v3[2] as usize], mct::AES_SBOX0[v0[3] as usize]];
        let tmp2 = [mct::AES_SBOX0[v2[0] as usize], mct::AES_SBOX0[v3[1] as usize], mct::AES_SBOX0[v0[2] as usize], mct::AES_SBOX0[v1[3] as usize]];
        let tmp3 = [mct::AES_SBOX0[v3[0] as usize], mct::AES_SBOX0[v0[1] as usize], mct::AES_SBOX0[v1[2] as usize], mct::AES_SBOX0[v2[3] as usize]];
        s0 = u32::from_be_bytes(tmp0);
        s1 = u32::from_be_bytes(tmp1);
        s2 = u32::from_be_bytes(tmp2);
        s3 = u32::from_be_bytes(tmp3);
        s0 ^= key[k+0];
        s1 ^= key[k+1];
        s2 ^= key[k+2];
        s3 ^= key[k+3];
        
        let s = [s0, s1, s2, s3];
        let mut dst_itr = dst.iter_mut();
        for &ele in s.iter() {
            let v = ele.to_be_bytes();
            for &x in v.iter() {
                *dst_itr.next().unwrap() = x;
            }
        }
    }
    
    fn decrypt_block(key: &[u32], nr: usize, dst: &mut [u8], src: &[u8]) {
        let mut src_itr = src.iter();
        let mut s = [0u32; 4];
        for i in 0..4 {
            let v = [*src_itr.next().unwrap(), *src_itr.next().unwrap(), *src_itr.next().unwrap(), *src_itr.next().unwrap()];
            s[i] = u32::from_be_bytes(v);
        }

        // AddRoundKey
        let (mut s0, mut s1, mut s2, mut s3) = (s[0] ^ key[0], s[1] ^ key[1], s[2] ^ key[2], s[3] ^ key[3]);

        // SubBytes -> ShiftRows -> MixColumns -> AddRoundKey
        let mut k = 4;
        for _ in 0..(nr - 1) {
            let (v0, v1, v2, v3) = (s0.to_be_bytes(), s1.to_be_bytes(), s2.to_be_bytes(), s3.to_be_bytes());
            let t0  = key[k+0] ^ mct::AES_TD0[v0[0] as usize] ^ mct::AES_TD1[v3[1] as usize] ^ mct::AES_TD2[v2[2] as usize] ^ mct::AES_TD3[v1[3] as usize];
            let t1  = key[k+1] ^ mct::AES_TD0[v1[0] as usize] ^ mct::AES_TD1[v0[1] as usize] ^ mct::AES_TD2[v3[2] as usize] ^ mct::AES_TD3[v2[3] as usize];
            let t2  = key[k+2] ^ mct::AES_TD0[v2[0] as usize] ^ mct::AES_TD1[v1[1] as usize] ^ mct::AES_TD2[v0[2] as usize] ^ mct::AES_TD3[v3[3] as usize];
            let t3  = key[k+3] ^ mct::AES_TD0[v3[0] as usize] ^ mct::AES_TD1[v2[1] as usize] ^ mct::AES_TD2[v1[2] as usize] ^ mct::AES_TD3[v0[3] as usize];
            s0 = t0;
            s1 = t1;
            s2 = t2;
            s3 = t3;
            k += 4;
        }

        // SubBytes -> ShiftRows -> AddRoundKey
        let (v0, v1, v2, v3) = (s0.to_be_bytes(), s1.to_be_bytes(), s2.to_be_bytes(), s3.to_be_bytes());
        let tmp0 = [mct::AES_SBOX1[v0[0] as usize], mct::AES_SBOX1[v3[1] as usize], mct::AES_SBOX1[v2[2] as usize], mct::AES_SBOX1[v1[3] as usize]];
        let tmp1 = [mct::AES_SBOX1[v1[0] as usize], mct::AES_SBOX1[v0[1] as usize], mct::AES_SBOX1[v3[2] as usize], mct::AES_SBOX1[v2[3] as usize]];
        let tmp2 = [mct::AES_SBOX1[v2[0] as usize], mct::AES_SBOX1[v1[1] as usize], mct::AES_SBOX1[v0[2] as usize], mct::AES_SBOX1[v3[3] as usize]];
        let tmp3 = [mct::AES_SBOX1[v3[0] as usize], mct::AES_SBOX1[v2[1] as usize], mct::AES_SBOX1[v1[2] as usize], mct::AES_SBOX1[v0[3] as usize]];
        s0 = u32::from_be_bytes(tmp0);
        s1 = u32::from_be_bytes(tmp1);
        s2 = u32::from_be_bytes(tmp2);
        s3 = u32::from_be_bytes(tmp3);
        s0 ^= key[k+0];
        s1 ^= key[k+1];
        s2 ^= key[k+2];
        s3 ^= key[k+3];

        let s = [s0, s1, s2, s3];
        let mut dst_itr = dst.iter_mut();
        for &ele in s.iter() {
            let v = ele.to_be_bytes();
            for &x in v.iter() {
                *dst_itr.next().unwrap() = x;
            }
        }
    }
}

pub struct Aes128Cipher {
    enc_ks: [u32; 44],
    dec_ks: [u32; 44],
}

impl AesCipherBase for Aes128Cipher {}

impl Aes128Cipher {
    pub fn new(key: [u8; 16]) -> Aes128Cipher {
        let (mut enc_kc, mut dec_kc) = ([0u32; 60], [0u32; 60]);
        Self::key_schedule(key.as_ref(), &mut enc_kc, &mut dec_kc);
        let (mut enc_ks, mut dec_ks) = ([0u32; 44], [0u32; 44]);
        let (dst, src) = (&mut enc_ks[..], &enc_kc[0..44]);
        dst.copy_from_slice(src);
        let (dst, src) = (&mut dec_ks[..], &dec_kc[0..44]);
        dst.copy_from_slice(src);
        
        Aes128Cipher {
            enc_ks,
            dec_ks,
        }
    }
}

impl Cipher for Aes128Cipher {
    fn block_size(&self) -> usize {
        mct::AES_BLOCK_SIZE
    }

    fn encrypt(&self, dst: &mut Vec<u8>, data_block: &[u8]) {
        if data_block.len() == mct::AES_BLOCK_SIZE {
            dst.clear();
            dst.resize(mct::AES_BLOCK_SIZE, 0);
            Self::crypt_block(self.enc_ks.as_ref(), 10, dst.as_mut_slice(), data_block);
        } else {
            panic!("data_block size is not {}.", mct::AES_BLOCK_SIZE);
        }
    }

    fn decrypt(&self, dst: &mut Vec<u8>, cipher_text: &[u8]) {
        if cipher_text.len() == mct::AES_BLOCK_SIZE {
            dst.clear();
            dst.resize(mct::AES_BLOCK_SIZE, 0);
            Self::decrypt_block(self.dec_ks.as_ref(), 10, dst.as_mut_slice(), cipher_text);
        } else {
            panic!("data_block size is not {}.", mct::AES_BLOCK_SIZE);
        }
    }
}


pub struct Aes192Cipher {
    enc_ks: [u32; 52],
    dec_ks: [u32; 52],
}

impl AesCipherBase for Aes192Cipher {}

impl Aes192Cipher {
    pub fn new(key: [u8; 24]) -> Aes192Cipher {
        let (mut enc_kc, mut dec_kc) = ([0u32; 60], [0u32; 60]);
        Self::key_schedule(key.as_ref(), &mut enc_kc, &mut dec_kc);
        let (mut enc_ks, mut dec_ks) = ([0u32; 52], [0u32; 52]);
        let (dst, src) = (&mut enc_ks[..], &enc_kc[0..52]);
        dst.copy_from_slice(src);
        let (dst, src) = (&mut dec_ks[..], &dec_kc[0..52]);
        dst.copy_from_slice(src);

        Aes192Cipher {
            enc_ks,
            dec_ks,
        }
    }
}

impl Cipher for Aes192Cipher {
    fn block_size(&self) -> usize {
        mct::AES_BLOCK_SIZE
    }

    fn encrypt(&self, dst: &mut Vec<u8>, data_block: &[u8]) {
        if data_block.len() == mct::AES_BLOCK_SIZE {
            dst.clear();
            dst.resize(mct::AES_BLOCK_SIZE, 0);
            Self::crypt_block(self.enc_ks.as_ref(), 12, dst.as_mut_slice(), data_block);
        } else {
            panic!("data_block size is not {}.", mct::AES_BLOCK_SIZE);
        }
    }

    fn decrypt(&self, dst: &mut Vec<u8>, cipher_text: &[u8]) {
        if cipher_text.len() == mct::AES_BLOCK_SIZE {
            dst.clear();
            dst.resize(mct::AES_BLOCK_SIZE, 0);
            Self::decrypt_block(self.dec_ks.as_ref(), 12, dst.as_mut_slice(), cipher_text);
        } else {
            panic!("data_block size is not {}.", mct::AES_BLOCK_SIZE);
        }
    }
}



pub struct Aes256Cipher {
    enc_ks: [u32; 60],
    dec_ks: [u32; 60],
}

impl AesCipherBase for Aes256Cipher {}

impl Aes256Cipher {
    pub fn new(key: [u8; 32]) -> Aes256Cipher {
        let (mut enc_ks, mut dec_ks) = ([0u32; 60], [0u32; 60]);
        Self::key_schedule(key.as_ref(), &mut enc_ks, &mut dec_ks);
        
        Aes256Cipher {
            enc_ks,
            dec_ks,
        }
    }
}

impl Cipher for Aes256Cipher {
    fn block_size(&self) -> usize {
        mct::AES_BLOCK_SIZE
    }

    fn encrypt(&self, dst: &mut Vec<u8>, data_block: &[u8]) {
        if data_block.len() == mct::AES_BLOCK_SIZE {
            dst.clear();
            dst.resize(mct::AES_BLOCK_SIZE, 0);
            Self::crypt_block(self.enc_ks.as_ref(), 14, dst.as_mut_slice(), data_block);
        } else {
            panic!("data_block size is not {}.", mct::AES_BLOCK_SIZE);
        }
    }

    fn decrypt(&self, dst: &mut Vec<u8>, cipher_text: &[u8]) {
        if cipher_text.len() == mct::AES_BLOCK_SIZE {
            dst.clear();
            dst.resize(mct::AES_BLOCK_SIZE, 0);
            Self::decrypt_block(self.dec_ks.as_ref(), 14, dst.as_mut_slice(), cipher_text);
        } else {
            panic!("data_block size is not {}.", mct::AES_BLOCK_SIZE);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::{Aes128Cipher, Cipher, Aes192Cipher, Aes256Cipher};

    #[test]
    fn aes128() {
        let cases = [
            (
                // Appendix B.
                [0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c],
                [0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d, 0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37, 0x07, 0x34],
                [0x39, 0x25, 0x84, 0x1d, 0x02, 0xdc, 0x09, 0xfb, 0xdc, 0x11, 0x85, 0x97, 0x19, 0x6a, 0x0b, 0x32],
            ),
            (
                // Appendix C.1.  AES-128
                [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f],
                [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff],
                [0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, 0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4, 0xc5, 0x5a],
            ),
        ];

        for ele in cases.iter() {
            let cipher = Aes128Cipher::new(ele.0);
            let mut dst0 = Vec::new();
            cipher.encrypt(&mut dst0, ele.1.as_ref());
            assert_eq!(dst0.as_slice(), ele.2.as_ref());
            // println!("{:?}->{:?}", dst0, ele.2);
            let mut dst1 = Vec::new();
            cipher.decrypt(&mut dst1, ele.2.as_ref());
            // println!("{:?}->{:?}", dst1, ele.1);
            assert_eq!(dst1.as_slice(), ele.1.as_ref());
        }
    }
    
    #[test]
    fn aes192() {
        let cases = [
            (
                // Appendix C.2.  AES-192
                [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
                    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,],
                [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff],
                [0xdd, 0xa9, 0x7c, 0xa4, 0x86, 0x4c, 0xdf, 0xe0, 0x6e, 0xaf, 0x70, 0xa0, 0xec, 0x0d, 0x71, 0x91],
            ),
        ];
        
        for ele in cases.iter() {
            let cipher = Aes192Cipher::new(ele.0);
            let mut dst0 = Vec::new();
            cipher.encrypt(&mut dst0, ele.1.as_ref());
            assert_eq!(dst0.as_slice(), ele.2.as_ref(), "cases=>{:?}", ele.0);
            // println!("{:?}->{:?}", dst0, ele.2);
            let mut dst1 = Vec::new();
            cipher.decrypt(&mut dst1, ele.2.as_ref());
            // println!("{:?}->{:?}", dst1, ele.1);
            assert_eq!(dst1.as_slice(), ele.1.as_ref());
        }
    }

    #[test]
    fn aes256() {
        let cases = [
            (
                // Appendix C.3.  AES-256
                [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
                    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,],
                [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff],
                [0x8e, 0xa2, 0xb7, 0xca, 0x51, 0x67, 0x45, 0xbf, 0xea, 0xfc, 0x49, 0x90, 0x4b, 0x49, 0x60, 0x89],
            ),
        ];
        
        for ele in cases.iter() {
            let cipher = Aes256Cipher::new(ele.0);
            let mut dst0 = Vec::new();
            cipher.encrypt(&mut dst0, ele.1.as_ref());
            assert_eq!(dst0.as_slice(), ele.2.as_ref(), "cases=>{:?}", ele.0);
            // println!("{:?}->{:?}", dst0, ele.2);
            let mut dst1 = Vec::new();
            cipher.decrypt(&mut dst1, ele.2.as_ref());
            // println!("{:?}->{:?}", dst1, ele.1);
            assert_eq!(dst1.as_slice(), ele.1.as_ref());
        }
    }
}
