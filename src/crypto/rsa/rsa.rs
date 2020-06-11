//! RSA  
//! 
//! RFC2313 RSA v1.5
//! RFC3447 RSA v2.1
//! RFC8017 RSA v2.2
//! 
//! https://www.cnblogs.com/mengsuenyan/p/12706003.html



use crate::math::big::{Nat, BigInt};
use crate::crypto::rand::{CryptoRng, prime};
use std::io::Read;

/// RSA公钥  
#[derive(Clone)]
pub struct PublicKey {
    n: Nat,                      // 模数
    e: Nat,                       // 公钥指数
    size_: usize,
}

/// RSA私钥  
#[derive(Clone)]
pub struct PrivateKey {
    p_key: PublicKey,
    d: Nat,                   // 私钥指数
    p: Nat,
    q: Nat,
}

impl PublicKey {
    /// 模数的字节长度  
    pub fn size(&self) -> usize {
        self.size_
    }
    
    fn check(&self) -> Result<(), &str> {
        if self.n.is_nan() || self.n == 0 {
            Err("crypto/rsa: missing public modulus")
        } else if self.e < 2 {
            Err("crypto/rsa: public exponent too small")
        } else if self.e >  ((1 << 31) - 1) {
            Err("crypto/rsa: public exponent too large")
        } else {
            Ok(())
        }
    }
    
    /// data^e mod n
    pub fn encrypt(&self, data: &Nat) -> Nat {
        self.check().unwrap();
        data.pow_mod(&self.e, &self.n)
    }
}

impl PrivateKey {
    pub fn public_key(&self) -> PublicKey {
        self.p_key.clone()
    }
    
    #[inline]
    fn modulus(&self) -> &Nat {
        &self.p_key.n
    }
    
    pub fn size(&self) -> usize {
        self.p_key.size_
    }
    
    /// ciphter_text^d mod n
    pub fn decrypt(&self, cipher_text: &Nat) -> Nat {
        self.p_key.check().unwrap();
        
        if cipher_text > self.modulus() {
            panic!("crypto/rsa: descryption error");
        }

        cipher_text.pow_mod(&self.d, self.modulus())
    }
    
    /// 密钥生成  
    /// 记phi(n)为模n乘法群Z的规模;  
    /// 欧拉定理: 对于任意整数n>1, a^phi(n)=1(mod n)对所有a属于Z成立;  
    /// 费马定理: 如果p是质数, 则a^(p-1)=1(mod p)对于所有a属于Z成立;  
    pub fn generate_key<'a, Rd>(bits: usize, test_nums: usize) -> Result<PrivateKey, &'a str>
        where Rd: CryptoRng + Read + Default
    {
        // n = p * q
        // gcd(e,p-1) = 1, gcd(e,q-1)=1
        // (d*e-1) % (p-1) = 0, (d*e-1)%(q-1)=0
        let (p_bits, q_bits) = (bits >> 1, bits - (bits >> 1));
        let e = BigInt::from(65537u32);
        let mut prikey = PrivateKey {
            p_key: PublicKey {
                n: Nat::nan(),
                e: Nat::nan(),
                size_: 0,
            },
            d: Nat::nan(),
            p: Nat::nan(),
            q: Nat::nan(),
        };

        loop {
            let p = prime::<Rd>(p_bits, test_nums);
            if p.is_err() {
                return Err(p.err().unwrap());
            }
            let q = prime::<Rd>(q_bits, test_nums);
            if q.is_err() {
                return Err(q.err().unwrap());
            }
            let (p, q) = (p.unwrap(), q.unwrap());
            
            if &p == &q {
                continue;
            }
            
            let n = &p * &q;
            if n.bits_len() != bits {
                continue;
            }
            
            let totient = &(&p - 1) * &(&q - 1);
            let tmp = BigInt::from(totient);
            let ok = e.mod_inverse(&tmp);
            if ok.is_some() {
                prikey.p_key.n = n;
                prikey.p_key.e = e.to_nat();
                prikey.d = ok.unwrap().to_nat();
                prikey.p = p;
                prikey.q = q;
                break;
            }
        }
        
        Ok(prikey)
    }
}
