//! PKCS1 v1.5  
//! https://www.cnblogs.com/mengsuenyan/p/12706003.html

use crate::crypto::rsa::{PublicKey, PrivateKey};
use crate::crypto::Cipher;
use crate::crypto::rand::CryptoRng;
use std::io::Read;
use std::marker::PhantomData;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::math::big::Nat;

#[derive(Clone, Copy)]
pub enum PKCSType {
    Pkcs1V1_5
}

pub struct PKCS<Rd> {
    pk_type: PKCSType,
    pub_key: Option<PublicKey>,
    pri_key: Option<PrivateKey>,
    phantom: PhantomData<Rd>,
}

impl<Rd> PKCS<Rd>
    where Rd: CryptoRng + Default + Read
{
    /// 如果pub_key.is_none(), 而pri_key.is_some(), 那么pub_key会从pri_key生成
    pub fn new(pub_key: Option<PublicKey>, pri_key: Option<PrivateKey>, pkcs_type: PKCSType) -> Self {
        let pub_key = if pub_key.is_none() && pri_key.is_some() {
            Some(pri_key.as_ref().unwrap().public_key())
        } else {
            pub_key
        };
        
        PKCS {
            pk_type: pkcs_type,
            pub_key,
            pri_key,
            phantom: PhantomData,
        }
    }
    
    fn encrypt_pkcs1_v1_5(&self, dst: &mut Vec<u8>, data_block: &[u8]) {
        if self.pub_key.is_none() {
            panic!("no public key");
        } if self.block_size() < 11 {
            panic!("pubkey size must be greater than {} bytes", 11);
        } else if data_block.len() > (self.block_size() - 11) {
            panic!("data_block length must be less than {} bytes!", self.block_size() - 11);
        }
        
        let pubkey = self.pub_key.as_ref().unwrap();
        // EB = 0x00 | BT | PS | 0x00 | data_block
        let mut eb = Vec::new();
        let len = pubkey.size();
        eb.resize(len, 0u8);
        let mut eb_itr = eb.iter_mut();
        eb_itr.next();
        *eb_itr.next().unwrap() = 0x02;
        
        let mut rand = Rd::default();
        // ps填充非0随机数
        let ps = &mut eb[2..(len - data_block.len() - 1)];
        rand.read_exact(ps).unwrap();
        for ele in ps.iter_mut() {
            while *ele == 0 {
                let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
                *ele = (t & 0xff) as u8;
            }
        }
        
        let msg = &mut eb[(len - data_block.len())..];
        msg.clone_from_slice(data_block);
        eb.reverse();
        let m = Nat::from_vec(&eb);
        
        let ery = pubkey.encrypt(&m);
        let ery = ery.as_slice();
        let (dlen, difflen) = if pubkey.size() > (ery.len() << 2) {
            (pubkey.size(), pubkey.size() - (ery.len() << 2))
        } else {
            (ery.len() << 2, 0)
        };
        dst.resize(dlen, 0);
        
        unsafe {
            dst.as_mut_ptr().add(difflen).copy_from_nonoverlapping(std::mem::transmute::<*const u32, *const u8>(ery.as_ptr()), ery.len() << 2);
        }
    }
    
    fn decrypt_pkcs1_v1_5(&self, dst: &mut Vec<u8>, cipher_text: &[u8]) {
        if self.pri_key.is_none() {
            panic!("no private key");
        }
         
        let prikey = self.pri_key.as_ref().unwrap();
        if prikey.size() < 11 {
            panic!("prikey size must be greater than 11");
        }
        
        let c = Nat::from_slice(cipher_text);
        let m = prikey.decrypt(&c);
        let m = m.as_slice();
        
        let (dlen, difflen) = if prikey.size() > (m.len() << 2) {
            (prikey.size(), prikey.size() - (m.len() << 2))
        } else {
            (m.len() << 2, 0)
        };
        
        dst.resize(dlen, 0);
        
        unsafe {
            dst.as_mut_ptr().add(difflen).copy_from_nonoverlapping(std::mem::transmute::<*const u32, *const u8>(m.as_ptr()), m.len() << 2);
        }
    }
    
    pub fn pkcs_type(&self) -> PKCSType {
        self.pk_type
    }
    
    fn extract_pkcs1_v1_5(&self, dst: &[u8]) -> Option<usize> {
        if dst.len() < 11 {
            None
        } else {
            if dst[0] != 0 {
                None
            } else if dst[1] != 0x02 {
                None
            } else {
                let ps = &dst[2..];
                let mut idx = 2;
                for &ele in ps {
                    if ele != 0 {
                        idx += 1;
                    } else {
                        break;
                    }
                }
                
                if idx >= 10 && idx < dst.len() {
                    Some(idx + 1)
                } else {
                    None
                }
            }
        }
    }
    
    /// 定位原始明文数据在解密后dst中所处的位置, None表示不是该pkcs加密后解密的数据  
    pub fn extract_ciphertext_position(&self, dst: &[u8]) -> Option<usize> {
        match self.pk_type {
            PKCSType::Pkcs1V1_5 => self.extract_pkcs1_v1_5(dst),
        }
    }
}

impl<Rd> Cipher for PKCS<Rd>
    where Rd: CryptoRng + Default + Read
{
    fn block_size(&self) -> usize {
        if self.pub_key.is_none() {
            return 0;
        }
        
        match self.pk_type {
            PKCSType::Pkcs1V1_5 => self.pub_key.as_ref().unwrap().size(),
        }
    }

    fn encrypt(&self, dst: &mut Vec<u8>, data_block: &[u8]) {
        self.encrypt_pkcs1_v1_5(dst, data_block)
    }

    fn decrypt(&self, dst: &mut Vec<u8>, cipher_text: &[u8]) {
        self.decrypt_pkcs1_v1_5(dst, cipher_text);
    }
}
