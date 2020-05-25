#![allow(unused)]
//! RSA  
//! 
//! RFC2313 RSA v1.5
//! RFC3447 RSA v2.1
//! RFC8017 RSA v2.2
//! 
//! https://www.cnblogs.com/mengsuenyan/p/12706003.html



use crate::math::big::BigInt;

/// RSA公钥  
pub struct PublicKey {
    n: BigInt,                      // 模数
    e: usize,                       // 公钥指数
    size_: usize,
}

pub struct CRTValue {
    exp: BigInt,                        // d mod (prime - 1)
    coeff: BigInt,                      // r*coeff = 1 mod prime
    r: BigInt,                          // product of prime prior to this (include p and q)
}

pub struct PrecomputedValues {
    dp: BigInt,                     // d mod (p - 1)
    dq: BigInt,                     // d mod (q - 1)

    // CRTValues is used for the 3rd and subsequent primes. Due to a
    // historical accident, the CRT for the first two primes is handled
    // differently in PKCS#1 and interoperability is sufficiently
    // important that we mirror this.
    crt_values: CRTValue,
}

/// RSA私钥  
pub struct PrivateKey {
    p_key: PublicKey,
    d: usize,                   // 私钥指数
    p: Vec<BigInt>,                // 质数p,q
}

impl PublicKey {
    /// 模数的字节长度  
    pub fn size(&self) -> usize {
        self.size_
    }
    
    pub fn check(&self) -> Result<(), &str> {
        if self.n.is_nan() {
            Err("crypto/rsa: missing public modulus")
        } else if self.e < 2 {
            Err("crypto/rsa: public exponent too small")
        } else if self.e >  ((1 << 31) - 1) {
            Err("crypto/rsa: public exponent too large")
        } else {
            Ok(())
        }
    }
}
