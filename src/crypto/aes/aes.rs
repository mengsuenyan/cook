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
        let (nk, nb, nr) = Self::nk_nb_nr(key);
        
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
        
        for i in 0..n {
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
        }
    }
    
    fn crypt_block(key: &[u32], nr: usize, dst: &mut [u8], src: &mut [u8]) {
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
        let tmp1 = [mct::AES_SBOX0[v1[0] as usize], mct::AES_SBOX0[v2[1] as usize], mct::AES_SBOX0[v3[2] as usize], mct::AES_SBOX0[v3[0] as usize]];
        let tmp2 = [mct::AES_SBOX0[v2[0] as usize], mct::AES_SBOX0[v3[1] as usize], mct::AES_SBOX0[v0[2] as usize], mct::AES_SBOX0[v3[1] as usize]];
        let tmp3 = [mct::AES_SBOX0[v3[0] as usize], mct::AES_SBOX0[v0[1] as usize], mct::AES_SBOX0[v1[2] as usize], mct::AES_SBOX0[v3[2] as usize]];
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
        unimplemented!()
    }

    fn decrypt(&self, dst: &mut Vec<u8>, cipher_text: &[u8]) {
        unimplemented!()
    }
}
