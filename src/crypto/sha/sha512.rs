//! SHA-512系列  
//! 
//! https://www.cnblogs.com/mengsuenyan/p/12697811.html

use crate::crypto::sha::const_tables as mct;
use std::hash::Hasher;
use crate::hash::{GenericHasher, GenericHasherSum};


trait Sha512SeriesDigest {
    #[inline]
    fn rotate_s0(x: u64) -> u64 {
        x.rotate_right(28) ^ x.rotate_right(34) ^ x.rotate_right(39)
    }

    #[inline]
    fn rotate_s1(x: u64) -> u64 {
        x.rotate_right(14) ^ x.rotate_right(18) ^ x.rotate_right(41)
    }

    #[inline]
    fn rotate_d0(x: u64) -> u64 {
        x.rotate_right(1) ^ x.rotate_right(8) ^ (x >> 7)
    }

    #[inline]
    fn rotate_d1(x: u64) -> u64 {
        x.rotate_right(19) ^ x.rotate_right(61) ^ (x >> 6)
    }
    
    fn sha512_update(data_block: &[u8], digest: &mut [u64; mct::SHA512_DIGEST_WSIZE]) {
        let mut chunk = 0;
        
        while chunk < data_block.len() {
            let bytes = &data_block[chunk..(chunk+mct::SHA512_BLOCK_SIZE)];
            let mut word = [0u64; 80];
            let mut itr = bytes.iter();
            for i in 0..16 {
                let v = [*itr.next().unwrap(), *itr.next().unwrap(), *itr.next().unwrap(), *itr.next().unwrap(),
                                *itr.next().unwrap(), *itr.next().unwrap(), *itr.next().unwrap(), *itr.next().unwrap()];
                word[i] = u64::from_be_bytes(v);
            }
            
            for i in 16..80 {
                word[i] = Self::rotate_d1(word[i-2]).wrapping_add(word[i-7]).wrapping_add(Self::rotate_d0(word[i-15])).wrapping_add(word[i-16]);
            }
            
            let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h) = (digest[0], digest[1], digest[2], digest[3], digest[4],
                digest[5], digest[6], digest[7]);
            
            for j in 0..80 {
                let t1 = h.wrapping_add(Self::rotate_s1(e)).wrapping_add(mct::f_ch(e,f,g)).wrapping_add(mct::SHA512_K[j]).wrapping_add(word[j]);
                let t2 = Self::rotate_s0(a).wrapping_add(mct::f_maj(a,b,c));

                h = g;
                g = f;
                f = e;
                e = d.wrapping_add(t1);
                d = c;
                c = b;
                b = a;
                a = t1.wrapping_add(t2);
            }

            digest[0] = a.wrapping_add(digest[0]);
            digest[1] = b.wrapping_add(digest[1]);
            digest[2] = c.wrapping_add(digest[2]);
            digest[3] = d.wrapping_add(digest[3]);
            digest[4] = e.wrapping_add(digest[4]);
            digest[5] = f.wrapping_add(digest[5]);
            digest[6] = g.wrapping_add(digest[6]);
            digest[7] = h.wrapping_add(digest[7]);
            chunk += mct::SHA512_BLOCK_SIZE;
        }
    }

    fn copy_digest_to(&self, h: &mut [u64; mct::SHA512_DIGEST_WSIZE]);
    fn update_digest_from(&mut self, h: &[u64; mct::SHA512_DIGEST_WSIZE]);

    fn buf_idx(&mut self) -> &mut usize;

    fn cur_msg_len(&mut self) -> &mut usize;

    fn buf(&mut self) -> &mut [u8;mct::SHA512_BLOCK_SIZE];

    fn sha512_write(&mut self, mut bytes: &[u8]) {
        let mut h = [0u64; mct::SHA512_DIGEST_WSIZE];
        self.copy_digest_to(&mut h);

        *self.cur_msg_len() += bytes.len();
        let idx = *self.buf_idx();
        if idx > 0 {
            let min = std::cmp::min(mct::SHA512_BLOCK_SIZE - idx, bytes.len());
            let dst = &mut self.buf()[idx..(idx+min)];
            let src = &bytes[0..min];
            dst.copy_from_slice(src);
            *self.buf_idx() += min;
            if *self.buf_idx() == mct::SHA512_BLOCK_SIZE {
                let data_block = &self.buf()[..];
                Self::sha512_update(data_block, &mut h);
                self.update_digest_from(&h);
                *self.buf_idx() = 0;
            }

            bytes = &bytes[min..];
        }

        if bytes.len() > mct::SHA512_BLOCK_SIZE {
            let n = bytes.len() & (!(mct::SHA512_BLOCK_SIZE - 1));
            let data_block = &bytes[0..n];
            Self::sha512_update(data_block, &mut h);
            self.update_digest_from(&h);
            bytes = &bytes[n..];
        }

        if bytes.len() > 0 {
            let dst = &mut self.buf()[..bytes.len()];
            dst.copy_from_slice(bytes);
            *self.buf_idx() += bytes.len();
        }
    }

    fn sha512_check_sum(&mut self) -> Result<&Self, &str> {
        let mut tmp = [0u8; mct::SHA512_BLOCK_SIZE];
        tmp[0] = 0x80;
        let len = *self.cur_msg_len();
        if len % mct::SHA512_BLOCK_SIZE < 112 {
            self.sha512_write(&tmp[0..(112-(len%mct::SHA512_BLOCK_SIZE))]);
        } else {
            self.sha512_write(&tmp[0..(128+112-(len%mct::SHA512_BLOCK_SIZE))]);
        }

        let len = (len as u128) << 3;
        let len_bytes = len.to_be_bytes();
        self.sha512_write(&len_bytes[..]);

        if *self.buf_idx() != 0 {
            Err("not padded")
        } else {
            Ok(&*self)
        }
    }
}

pub struct Sha512Digest {
    digest: [u64; mct::SHA512_DIGEST_WSIZE],
    buf: [u8; mct::SHA512_BLOCK_SIZE],
    idx: usize,
    len: usize,
}

impl Sha512Digest {
    pub fn new() -> Self {
        Sha512Digest {
            digest: mct::SHA512_INIT,
            buf: [0u8; mct::SHA512_BLOCK_SIZE],
            idx: 0,
            len: 0,
        }
    }

    /// 将其转换为SHA512/t, t是指定的消息摘要位数  
    /// t大于512时返回为空  
    /// note: 产生的消息摘要需要调用者字节截断到t位长度  
    /// todo: Trait支持常量类型参数后修改接口  
    pub fn generate_sha512t(t: u32) -> Option<Self> {
        if t > 512 {
            None
        } else {
            if t == 384 {
                Some(Sha512Digest {
                    digest: mct::SHA512_384INIT,
                    buf: [0u8; mct::SHA512_BLOCK_SIZE],
                    idx: 0,
                    len: 0,
                })
            } else if t == 224 {
                Some(Sha512Digest {
                    digest: mct::SHA512_224INIT,
                    buf: [0u8; mct::SHA512_BLOCK_SIZE],
                    idx: 0,
                    len: 0,
                })
            } else if t == 256 {
                Some(Sha512Digest {
                    digest: mct::SHA512_256INIT,
                    buf: [0u8; mct::SHA512_BLOCK_SIZE],
                    idx: 0,
                    len: 0,
                })
            } else {
                let mut sha512t = Sha512Digest::new();
                for ele in sha512t.digest.iter_mut() {
                    *ele = *ele ^ 0xa5a5a5a5a5a5a5a5u64;
                }
                let s = format!("SHA-512/{}", t);
                sha512t.write(s.as_bytes());
                sha512t.check_sum().unwrap();
                Some(sha512t)
            }
        }
    }
}

impl Sha512SeriesDigest for Sha512Digest {
    fn copy_digest_to(&self, h: &mut [u64; 8]) {
        *h = self.digest;
    }

    fn update_digest_from(&mut self, h: &[u64; 8]) {
        self.digest = *h;
    }

    fn buf_idx(&mut self) -> &mut usize {
        &mut self.idx
    }

    fn cur_msg_len(&mut self) -> &mut usize {
        &mut self.len
    }

    fn buf(&mut self) -> &mut [u8; 128] {
        &mut self.buf
    }
}

impl Hasher for Sha512Digest {
    fn finish(&self) -> u64 {
        let l = self.digest[0].to_be_bytes();
        let u = self.digest[1].to_be_bytes();
        let v = [l[0], l[1], l[2], l[3], u[0], u[1], u[2], u[3]];
        u64::from_le_bytes(v)
    }

    fn write(&mut self, bytes: &[u8]) {
        self.sha512_write(bytes)
    }
}

impl GenericHasher for Sha512Digest {
    fn block_size(&self) -> usize {
        mct::SHA512_BLOCK_SIZE
    }

    fn reset(&mut self) {
        self.digest = mct::SHA512_INIT;
        self.idx = 0;
        self.len = 0;
    }

    fn size(&self) -> usize {
        mct::SHA512_DIGEST_SIZE
    }

    fn append_to_vec(&self, data: &mut Vec<u8>) -> usize {
        let len = data.len();
        let v = self.sum();
        for &ele in v.iter() {
            data.push(ele);
        }
        data.len() - len
    }

    fn check_sum(&mut self) -> Result<&Self, &str> {
        self.sha512_check_sum()
    }
}

impl GenericHasherSum<[u8; mct::SHA512_DIGEST_SIZE]> for Sha512Digest {
    fn sum(&self) -> [u8; 64] {
        let h0 = self.digest[0].to_be_bytes();
        let h1 = self.digest[1].to_be_bytes();
        let h2 = self.digest[2].to_be_bytes();
        let h3 = self.digest[3].to_be_bytes();
        let h4 = self.digest[4].to_be_bytes();
        let h5 = self.digest[5].to_be_bytes();
        let h6 = self.digest[6].to_be_bytes();
        let h7 = self.digest[7].to_be_bytes();
        [
            h0[0], h0[1], h0[2], h0[3], h0[4], h0[5], h0[6], h0[7],
            h1[0], h1[1], h1[2], h1[3], h1[4], h1[5], h1[6], h1[7],
            h2[0], h2[1], h2[2], h2[3], h2[4], h2[5], h2[6], h2[7],
            h3[0], h3[1], h3[2], h3[3], h3[4], h3[5], h3[6], h3[7],
            h4[0], h4[1], h4[2], h4[3], h4[4], h4[5], h4[6], h4[7],
            h5[0], h5[1], h5[2], h5[3], h5[4], h5[5], h5[6], h5[7],
            h6[0], h6[1], h6[2], h6[3], h6[4], h6[5], h6[6], h6[7],
            h7[0], h7[1], h7[2], h7[3], h7[4], h7[5], h7[6], h7[7],
        ]
    }

    fn sum_copy_to(&self, v: &mut [u8; 64]) {
        let mut v_itr = v.iter_mut();
        for ele in self.digest.iter() {
            let x = ele.to_be_bytes();
            *v_itr.next().unwrap() = x[0];
            *v_itr.next().unwrap() = x[1];
            *v_itr.next().unwrap() = x[2];
            *v_itr.next().unwrap() = x[3];
            *v_itr.next().unwrap() = x[4];
            *v_itr.next().unwrap() = x[5];
            *v_itr.next().unwrap() = x[6];
            *v_itr.next().unwrap() = x[7];
        }
    }
}

pub struct Sha512T384Digest {
    digest: Sha512Digest,
}

impl Sha512T384Digest {
    pub fn new() -> Self {
        Sha512T384Digest {
            digest: Sha512Digest {
                digest: mct::SHA512_384INIT,
                buf: [0u8; mct::SHA512_BLOCK_SIZE],
                idx: 0,
                len: 0,
            }
        }
    }
}


pub struct Sha512T224Digest {
    digest: Sha512Digest,
}

impl Sha512T224Digest {
    pub fn new() -> Self {
        Sha512T224Digest {
            digest: Sha512Digest {
                digest: mct::SHA512_224INIT,
                buf: [0u8; mct::SHA512_BLOCK_SIZE],
                idx: 0,
                len: 0,
            }
        }
    }
}

pub struct Sha512T256Digest {
    digest: Sha512Digest,
}

impl Sha512T256Digest {
    pub fn new() -> Self {
        Sha512T256Digest {
            digest: Sha512Digest {
                digest: mct::SHA512_256INIT,
                buf: [0u8; mct::SHA512_BLOCK_SIZE],
                idx: 0,
                len: 0,
            }
        }
    }
}

macro_rules! impl_default_for_sha512 {
    ($SHA: tt) => {
        impl Default for $SHA {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

impl_default_for_sha512!(Sha512Digest);
impl_default_for_sha512!(Sha512T384Digest);
impl_default_for_sha512!(Sha512T224Digest);
impl_default_for_sha512!(Sha512T256Digest);

macro_rules! impl_sha512seriesdigest_for {
    ($Sha: tt, $BlockSize: ident, $DigestSize: ident, $InitTable: ident) => {
        impl Sha512SeriesDigest for $Sha {
            fn copy_digest_to(&self, h: &mut [u64; 8]) {
                self.digest.copy_digest_to(h);
            }

            fn update_digest_from(&mut self, h: &[u64; 8]) {
                self.digest.update_digest_from(h);
            }

            fn buf_idx(&mut self) -> &mut usize {
                self.digest.buf_idx()
            }

            fn cur_msg_len(&mut self) -> &mut usize {
                self.digest.cur_msg_len()
            }

            fn buf(&mut self) -> &mut [u8; 128] {
                self.digest.buf()
            }
        }
        
        impl Hasher for $Sha {
            fn finish(&self) -> u64 {
                let l = self.digest.digest[0].to_be_bytes();
                let u = self.digest.digest[1].to_be_bytes();
                let v = [l[0], l[1], l[2], l[3], u[0], u[1], u[2], u[3]];
                u64::from_le_bytes(v)
            }

            fn write(&mut self, bytes: &[u8]) {
                self.digest.sha512_write(bytes)
            }
        }
        
        impl GenericHasher for $Sha {
            fn block_size(&self) -> usize {
                $BlockSize
            }

            fn reset(&mut self) {
                self.digest.digest = $InitTable;
                self.digest.idx = 0;
                self.digest.len = 0;
            }
    
            fn size(&self) -> usize {
                $DigestSize
            }
    
            fn append_to_vec(&self, data: &mut Vec<u8>) -> usize {
                let len = data.len();
                let v = self.sum();
                for &ele in v.iter() {
                    data.push(ele);
                }
                data.len() - len
            }
    
            fn check_sum(&mut self) -> Result<&Self, &str> {
                let v = self.digest.check_sum().is_ok();
                if v {
                    Ok(&*self)
                } else {
                    Err("Not Padded")
                }
            }
        }
    };
}

use mct::{SHA512T384_BLOCK_SIZE, SHA512T384_DIGEST_SIZE, SHA512_384INIT, 
            SHA512T224_BLOCK_SIZE, SHA512T224_DIGEST_SIZE, SHA512_224INIT,
            SHA512T256_BLOCK_SIZE, SHA512T256_DIGEST_SIZE, SHA512_256INIT};
impl_sha512seriesdigest_for!(Sha512T384Digest, SHA512T384_BLOCK_SIZE, SHA512T384_DIGEST_SIZE, SHA512_384INIT);
impl_sha512seriesdigest_for!(Sha512T224Digest, SHA512T224_BLOCK_SIZE, SHA512T224_DIGEST_SIZE, SHA512_224INIT);
impl_sha512seriesdigest_for!(Sha512T256Digest, SHA512T256_BLOCK_SIZE, SHA512T256_DIGEST_SIZE, SHA512_256INIT);

impl GenericHasherSum<[u8; SHA512T384_DIGEST_SIZE]> for Sha512T384Digest {
    fn sum(&self) -> [u8; 48] {
        let h0 = self.digest.digest[0].to_be_bytes();
        let h1 = self.digest.digest[1].to_be_bytes();
        let h2 = self.digest.digest[2].to_be_bytes();
        let h3 = self.digest.digest[3].to_be_bytes();
        let h4 = self.digest.digest[4].to_be_bytes();
        let h5 = self.digest.digest[5].to_be_bytes();
        [
            h0[0], h0[1], h0[2], h0[3], h0[4], h0[5], h0[6], h0[7],
            h1[0], h1[1], h1[2], h1[3], h1[4], h1[5], h1[6], h1[7],
            h2[0], h2[1], h2[2], h2[3], h2[4], h2[5], h2[6], h2[7],
            h3[0], h3[1], h3[2], h3[3], h3[4], h3[5], h3[6], h3[7],
            h4[0], h4[1], h4[2], h4[3], h4[4], h4[5], h4[6], h4[7],
            h5[0], h5[1], h5[2], h5[3], h5[4], h5[5], h5[6], h5[7],
        ]
    }

    fn sum_copy_to(&self, v: &mut [u8; 48]) {
        let mut itr = v.iter_mut();
        let mut ele_itr = self.digest.digest.iter();
        for _ in 0..mct::SHA512T384_DIGEST_WSIZE {
            let &ele = ele_itr.next().unwrap();
            let v = ele.to_be_bytes();
            *itr.next().unwrap() = v[0];
            *itr.next().unwrap() = v[1];
            *itr.next().unwrap() = v[2];
            *itr.next().unwrap() = v[3];
            *itr.next().unwrap() = v[4];
            *itr.next().unwrap() = v[5];
            *itr.next().unwrap() = v[6];
            *itr.next().unwrap() = v[7];
        }
    }
}

impl GenericHasherSum<[u8; SHA512T224_DIGEST_SIZE]> for Sha512T224Digest {
    fn sum(&self) -> [u8; 28] {
        let h0 = self.digest.digest[0].to_be_bytes();
        let h1 = self.digest.digest[1].to_be_bytes();
        let h2 = self.digest.digest[2].to_be_bytes();
        let h3 = self.digest.digest[3].to_be_bytes();
        [
            h0[0], h0[1], h0[2], h0[3], h0[4], h0[5], h0[6], h0[7],
            h1[0], h1[1], h1[2], h1[3], h1[4], h1[5], h1[6], h1[7],
            h2[0], h2[1], h2[2], h2[3], h2[4], h2[5], h2[6], h2[7],
            h3[0], h3[1], h3[2], h3[3],
        ]
    }

    fn sum_copy_to(&self, v: &mut [u8; 28]) {
        let mut itr = v.iter_mut();
        let mut ele_itr = self.digest.digest.iter();
        for _ in 0..mct::SHA512T224_DIGEST_WSIZE {
            let &ele = ele_itr.next().unwrap();
            let v = ele.to_be_bytes();
            *itr.next().unwrap() = v[0];
            *itr.next().unwrap() = v[1];
            *itr.next().unwrap() = v[2];
            *itr.next().unwrap() = v[3];
            *itr.next().unwrap() = v[4];
            *itr.next().unwrap() = v[5];
            *itr.next().unwrap() = v[6];
            *itr.next().unwrap() = v[7];
        }
        let &ele = ele_itr.next().unwrap();
        let v = ele.to_be_bytes();
        *itr.next().unwrap() = v[0];
        *itr.next().unwrap() = v[1];
        *itr.next().unwrap() = v[2];
        *itr.next().unwrap() = v[3];
    }
}

impl GenericHasherSum<[u8; SHA512T256_DIGEST_SIZE]> for Sha512T256Digest {
    fn sum(&self) -> [u8; 32] {
        let h0 = self.digest.digest[0].to_be_bytes();
        let h1 = self.digest.digest[1].to_be_bytes();
        let h2 = self.digest.digest[2].to_be_bytes();
        let h3 = self.digest.digest[3].to_be_bytes();
        [
            h0[0], h0[1], h0[2], h0[3], h0[4], h0[5], h0[6], h0[7],
            h1[0], h1[1], h1[2], h1[3], h1[4], h1[5], h1[6], h1[7],
            h2[0], h2[1], h2[2], h2[3], h2[4], h2[5], h2[6], h2[7],
            h3[0], h3[1], h3[2], h3[3], h3[4], h3[5], h3[6], h3[7],
        ]
    }

    fn sum_copy_to(&self, v: &mut [u8; 32]) {
        let mut itr = v.iter_mut();
        let mut ele_itr = self.digest.digest.iter();
        for _ in 0..mct::SHA512T256_DIGEST_WSIZE {
            let &ele = ele_itr.next().unwrap();
            let v = ele.to_be_bytes();
            *itr.next().unwrap() = v[0];
            *itr.next().unwrap() = v[1];
            *itr.next().unwrap() = v[2];
            *itr.next().unwrap() = v[3];
            *itr.next().unwrap() = v[4];
            *itr.next().unwrap() = v[5];
            *itr.next().unwrap() = v[6];
            *itr.next().unwrap() = v[7];
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::{Sha512Digest, Sha512T384Digest, Sha512T224Digest, Sha512T256Digest};
    use std::hash::Hasher;
    use crate::encoding::Bytes;
    use crate::hash::{GenericHasher, GenericHasherSum};

    #[test]
    fn sha512() {
        let cases = [
            (
                "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
                "",
            ),
            (
                "1f40fc92da241694750979ee6cf582f2d5d7d28e18335de05abc54d0560e0f5302860c652bf08d560252aa5e74210546f369fbbbce8c12cfc7957b2652fe9a75",
                "a",
            ),
            (
                "2d408a0717ec188158278a796c689044361dc6fdde28d6f04973b80896e1823975cdbf12eb63f9e0591328ee235d80e9b5bf1aa6a44f4617ff3caf6400eb172d",
                "ab",
            ),
            (
                "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
                "abc",
            ),
            (
                "d8022f2060ad6efd297ab73dcc5355c9b214054b0d1776a136a669d26a7d3b14f73aa0d0ebff19ee333368f0164b6419a96da49e3e481753e7e96b716bdccb6f",
                "abcd",
            ),
            (
                "878ae65a92e86cac011a570d4c30a7eaec442b85ce8eca0c2952b5e3cc0628c2e79d889ad4d5c7c626986d452dd86374b6ffaa7cd8b67665bef2289a5c70b0a1",
                "abcde",
            ),
            (
                "e32ef19623e8ed9d267f657a81944b3d07adbb768518068e88435745564e8d4150a0a703be2a7d88b61e3d390c2bb97e2d4c311fdc69d6b1267f05f59aa920e7",
                "abcdef",
            ),
            (
                "d716a4188569b68ab1b6dfac178e570114cdf0ea3a1cc0e31486c3e41241bc6a76424e8c37ab26f096fc85ef9886c8cb634187f4fddff645fb099f1ff54c6b8c",
                "abcdefg",
            ),
            (
                "a3a8c81bc97c2560010d7389bc88aac974a104e0e2381220c6e084c4dccd1d2d17d4f86db31c2a851dc80e6681d74733c55dcd03dd96f6062cdda12a291ae6ce",
                "abcdefgh",
            ),
            (
                "f22d51d25292ca1d0f68f69aedc7897019308cc9db46efb75a03dd494fc7f126c010e8ade6a00a0c1a5f1b75d81e0ed5a93ce98dc9b833db7839247b1d9c24fe",
                "abcdefghi",
            ),
            (
                "ef6b97321f34b1fea2169a7db9e1960b471aa13302a988087357c520be957ca119c3ba68e6b4982c019ec89de3865ccf6a3cda1fe11e59f98d99f1502c8b9745",
                "abcdefghij",
            ),
            (
                "2210d99af9c8bdecda1b4beff822136753d8342505ddce37f1314e2cdbb488c6016bdaa9bd2ffa513dd5de2e4b50f031393d8ab61f773b0e0130d7381e0f8a1d",
                "Discard medicine more than two years old.",
            ),
            (
                "a687a8985b4d8d0a24f115fe272255c6afaf3909225838546159c1ed685c211a203796ae8ecc4c81a5b6315919b3a64f10713da07e341fcdbb08541bf03066ce",
                "He who has a shady past knows that nice guys finish last.",
            ),
            (
                "8ddb0392e818b7d585ab22769a50df660d9f6d559cca3afc5691b8ca91b8451374e42bcdabd64589ed7c91d85f626596228a5c8572677eb98bc6b624befb7af8",
                "I wouldn't marry him with a ten foot pole.",
            ),
            (
                "26ed8f6ca7f8d44b6a8a54ae39640fa8ad5c673f70ee9ce074ba4ef0d483eea00bab2f61d8695d6b34df9c6c48ae36246362200ed820448bdc03a720366a87c6",
                "Free! Free!/A trip/to Mars/for 900/empty jars/Burma Shave",
            ),
            (
                "e5a14bf044be69615aade89afcf1ab0389d5fc302a884d403579d1386a2400c089b0dbb387ed0f463f9ee342f8244d5a38cfbc0e819da9529fbff78368c9a982",
                "The days of the digital watch are numbered.  -Tom Stoppard",
            ),
            (
                "420a1faa48919e14651bed45725abe0f7a58e0f099424c4e5a49194946e38b46c1f8034b18ef169b2e31050d1648e0b982386595f7df47da4b6fd18e55333015",
                "Nepal premier won't resign.",
            ),
            (
                "d926a863beadb20134db07683535c72007b0e695045876254f341ddcccde132a908c5af57baa6a6a9c63e6649bba0c213dc05fadcf9abccea09f23dcfb637fbe",
                "For every action there is an equal and opposite government program.",
            ),
            (
                "9a98dd9bb67d0da7bf83da5313dff4fd60a4bac0094f1b05633690ffa7f6d61de9a1d4f8617937d560833a9aaa9ccafe3fd24db418d0e728833545cadd3ad92d",
                "His money is twice tainted: 'taint yours and 'taint mine.",
            ),
            (
                "d7fde2d2351efade52f4211d3746a0780a26eec3df9b2ed575368a8a1c09ec452402293a8ea4eceb5a4f60064ea29b13cdd86918cd7a4faf366160b009804107",
                "There is no reason for any individual to have a computer in their home. -Ken Olsen, 1977",
            ),
            (
                "b0f35ffa2697359c33a56f5c0cf715c7aeed96da9905ca2698acadb08fbc9e669bf566b6bd5d61a3e86dc22999bcc9f2224e33d1d4f32a228cf9d0349e2db518",
                "It's a tiny change to the code and not completely disgusting. - Bob Manchek",
            ),
            (
                "3d2e5f91778c9e66f7e061293aaa8a8fc742dd3b2e4f483772464b1144189b49273e610e5cccd7a81a19ca1fa70f16b10f1a100a4d8c1372336be8484c64b311",
                "size:  a.out:  bad magic",
            ),
            (
                "b2f68ff58ac015efb1c94c908b0d8c2bf06f491e4de8e6302c49016f7f8a33eac3e959856c7fddbc464de618701338a4b46f76dbfaf9a1e5262b5f40639771c7",
                "The major problem is with sendmail.  -Mark Horton",
            ),
            (
                "d8c92db5fdf52cf8215e4df3b4909d29203ff4d00e9ad0b64a6a4e04dec5e74f62e7c35c7fb881bd5de95442123df8f57a489b0ae616bd326f84d10021121c57",
                "Give me a rock, paper and scissors and I will move the world.  CCFestoon",
            ),
            (
                "19a9f8dc0a233e464e8566ad3ca9b91e459a7b8c4780985b015776e1bf239a19bc233d0556343e2b0a9bc220900b4ebf4f8bdf89ff8efeaf79602d6849e6f72e",
                "If the enemy is within range, then so are you.",
            ),
            (
                "00b4c41f307bde87301cdc5b5ab1ae9a592e8ecbb2021dd7bc4b34e2ace60741cc362560bec566ba35178595a91932b8d5357e2c9cec92d393b0fa7831852476",
                "It's well we cannot hear the screams/That we create in others' dreams.",
            ),
            (
                "91eccc3d5375fd026e4d6787874b1dce201cecd8a27dbded5065728cb2d09c58a3d467bb1faf353bf7ba567e005245d5321b55bc344f7c07b91cb6f26c959be7",
                "You remind me of a TV show, but that's all right: I watch it anyway.",
            ),
            (
                "fabbbe22180f1f137cfdc9556d2570e775d1ae02a597ded43a72a40f9b485d500043b7be128fb9fcd982b83159a0d99aa855a9e7cc4240c00dc01a9bdf8218d7",
                "C is as portable as Stonehedge!!",
            ),
            (
                "2ecdec235c1fa4fc2a154d8fba1dddb8a72a1ad73838b51d792331d143f8b96a9f6fcb0f34d7caa351fe6d88771c4f105040e0392f06e0621689d33b2f3ba92e",
                "Even if I could be Shakespeare, I think I should still choose to be Faraday. - A. Huxley",
            ),
            (
                "7ad681f6f96f82f7abfa7ecc0334e8fa16d3dc1cdc45b60b7af43fe4075d2357c0c1d60e98350f1afb1f2fe7a4d7cd2ad55b88e458e06b73c40b437331f5dab4",
                "The fugacity of a constituent in a mixture of gases at a given temperature is proportional to its mole fraction.  Lewis-Randall Rule",
            ),
            (
                "833f9248ab4a3b9e5131f745fda1ffd2dd435b30e965957e78291c7ab73605fd1912b0794e5c233ab0a12d205a39778d19b83515d6a47003f19cdee51d98c7e0",
                "How can you write a big system without C++?  -Paul Glick",
            ),
        ];
        
        let mut sha512 = Sha512Digest::new();
        for ele in cases.iter() {
            sha512.write(ele.1.as_bytes());
            assert_eq!(ele.0, Bytes::cvt_bytes_to_str(sha512.check_sum().unwrap().sum().as_ref()), "cases=>{}", ele.1);
            sha512.reset();
        }
    }
    
    #[test]
    fn sha512t384() {
        let cases = [
            (
                "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b",
                "",
            ),
            (
                "54a59b9f22b0b80880d8427e548b7c23abd873486e1f035dce9cd697e85175033caa88e6d57bc35efae0b5afd3145f31",
                "a",
            ),
            (
                "c7be03ba5bcaa384727076db0018e99248e1a6e8bd1b9ef58a9ec9dd4eeebb3f48b836201221175befa74ddc3d35afdd",
                "ab",
            ),
            (
                "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7",
                "abc",
            ),
            (
                "1165b3406ff0b52a3d24721f785462ca2276c9f454a116c2b2ba20171a7905ea5a026682eb659c4d5f115c363aa3c79b",
                "abcd",
            ),
            (
                "4c525cbeac729eaf4b4665815bc5db0c84fe6300068a727cf74e2813521565abc0ec57a37ee4d8be89d097c0d2ad52f0",
                "abcde",
            ),
            (
                "c6a4c65b227e7387b9c3e839d44869c4cfca3ef583dea64117859b808c1e3d8ae689e1e314eeef52a6ffe22681aa11f5",
                "abcdef",
            ),
            (
                "9f11fc131123f844c1226f429b6a0a6af0525d9f40f056c7fc16cdf1b06bda08e302554417a59fa7dcf6247421959d22",
                "abcdefg",
            ),
            (
                "9000cd7cada59d1d2eb82912f7f24e5e69cc5517f68283b005fa27c285b61e05edf1ad1a8a9bded6fd29eb87d75ad806",
                "abcdefgh",
            ),
            (
                "ef54915b60cf062b8dd0c29ae3cad69abe6310de63ac081f46ef019c5c90897caefd79b796cfa81139788a260ded52df",
                "abcdefghi",
            ),
            (
                "a12070030a02d86b0ddacd0d3a5b598344513d0a051e7355053e556a0055489c1555399b03342845c4adde2dc44ff66c",
                "abcdefghij",
            ),
            (
                "86f58ec2d74d1b7f8eb0c2ff0967316699639e8d4eb129de54bdf34c96cdbabe200d052149f2dd787f43571ba74670d4",
                "Discard medicine more than two years old.",
            ),
            (
                "ae4a2b639ca9bfa04b1855d5a05fe7f230994f790891c6979103e2605f660c4c1262a48142dcbeb57a1914ba5f7c3fa7",
                "He who has a shady past knows that nice guys finish last.",
            ),
            (
                "40ae213df6436eca952aa6841886fcdb82908ef1576a99c8f49bb9dd5023169f7c53035abdda0b54c302f4974e2105e7",
                "I wouldn't marry him with a ten foot pole.",
            ),
            (
                "e7cf8b873c9bc950f06259aa54309f349cefa72c00d597aebf903e6519a50011dfe355afff064a10701c705693848df9",
                "Free! Free!/A trip/to Mars/for 900/empty jars/Burma Shave",
            ),
            (
                "c3d4f0f4047181c7d39d34703365f7bf70207183caf2c2f6145f04da895ef69124d9cdeb635da636c3a474e61024e29b",
                "The days of the digital watch are numbered.  -Tom Stoppard",
            ),
            (
                "a097aab567e167d5cf93676ed73252a69f9687cb3179bb2d27c9878119e94bf7b7c4b58dc90582edfaf66e11388ed714",
                "Nepal premier won't resign.",
            ),
            (
                "5026ca45c41fc64712eb65065da92f6467541c78f8966d3fe2c8e3fb769a3ec14215f819654b47bd64f7f0eac17184f3",
                "For every action there is an equal and opposite government program.",
            ),
            (
                "ac1cc0f5ac8d5f5514a7b738ac322b7fb52a161b449c3672e9b6a6ad1a5e4b26b001cf3bad24c56598676ca17d4b445a",
                "His money is twice tainted: 'taint yours and 'taint mine.",
            ),
            (
                "722d10c5de371ec0c8c4b5247ac8a5f1d240d68c73f8da13d8b25f0166d6f309bf9561979a111a0049405771d201941a",
                "There is no reason for any individual to have a computer in their home. -Ken Olsen, 1977",
            ),
            (
                "dc2d3ea18bfa10549c63bf2b75b39b5167a80c12aff0e05443168ea87ff149fb0eda5e0bd234eb5d48c7d02ffc5807f1",
                "It's a tiny change to the code and not completely disgusting. - Bob Manchek",
            ),
            (
                "1d67c969e2a945ae5346d2139760261504d4ba164c522443afe19ef3e29b152a4c52445489cfc9d7215e5a450e8e1e4e",
                "size:  a.out:  bad magic",
            ),
            (
                "5ff8e075e465646e7b73ef36d812c6e9f7d60fa6ea0e533e5569b4f73cde53cdd2cc787f33540af57cca3fe467d32fe0",
                "The major problem is with sendmail.  -Mark Horton",
            ),
            (
                "5bd0a997a67c9ae1979a894eb0cde403dde003c9b6f2c03cf21925c42ff4e1176e6df1ca005381612ef18457b9b7ec3b",
                "Give me a rock, paper and scissors and I will move the world.  CCFestoon",
            ),
            (
                "1eee6da33e7e54fc5be52ae23b94b16ba4d2a947ae4505c6a3edfc7401151ea5205ac01b669b56f27d8ef7f175ed7762",
                "If the enemy is within range, then so are you.",
            ),
            (
                "76b06e9dea66bfbb1a96029426dc0dfd7830bd297eb447ff5358d94a87cd00c88b59df2493fef56ecbb5231073892ea9",
                "It's well we cannot hear the screams/That we create in others' dreams.",
            ),
            (
                "12acaf21452cff586143e3f5db0bfdf7802c057e1adf2a619031c4e1b0ccc4208cf6cef8fe722bbaa2fb46a30d9135d8",
                "You remind me of a TV show, but that's all right: I watch it anyway.",
            ),
            (
                "0fc23d7f4183efd186f0bc4fc5db867e026e2146b06cb3d52f4bdbd57d1740122caa853b41868b197b2ac759db39df88",
                "C is as portable as Stonehedge!!",
            ),
            (
                "bc805578a7f85d34a86a32976e1c34fe65cf815186fbef76f46ef99cda10723f971f3f1464d488243f5e29db7488598d",
                "Even if I could be Shakespeare, I think I should still choose to be Faraday. - A. Huxley",
            ),
            (
                "b23918399a12ebf4431559eec3813eaf7412e875fd7464f16d581e473330842d2e96c6be49a7ce3f9bb0b8bc0fcbe0fe",
                "The fugacity of a constituent in a mixture of gases at a given temperature is proportional to its mole fraction.  Lewis-Randall Rule",
            ),
            (
                "1764b700eb1ead52a2fc33cc28975c2180f1b8faa5038d94cffa8d78154aab16e91dd787e7b0303948ebed62561542c8",
                "How can you write a big system without C++?  -Paul Glick",
            ),
        ];
        
        let mut sha512t384 = Sha512T384Digest::new();
        for ele in cases.iter() {
            sha512t384.write(ele.1.as_bytes());
            assert_eq!(ele.0, Bytes::cvt_bytes_to_str(sha512t384.check_sum().unwrap().sum().as_ref()));
            sha512t384.reset();
        }
    }
    
    #[test]
    fn sha512t224() {
        let cases = [
            (
                "6ed0dd02806fa89e25de060c19d3ac86cabb87d6a0ddd05c333b84f4",
                "",
            ),
            (
                "d5cdb9ccc769a5121d4175f2bfdd13d6310e0d3d361ea75d82108327",
                "a",
            ),
            (
                "b35878d07bfedf39fc638af08547eb5d1072d8546319f247b442fbf5",
                "ab",
            ),
            (
                "4634270f707b6a54daae7530460842e20e37ed265ceee9a43e8924aa",
                "abc",
            ),
            (
                "0c9f157ab030fb06e957c14e3938dc5908962e5dd7b66f04a36fc534",
                "abcd",
            ),
            (
                "880e79bb0a1d2c9b7528d851edb6b8342c58c831de98123b432a4515",
                "abcde",
            ),
            (
                "236c829cfea4fd6d4de61ad15fcf34dca62342adaf9f2001c16f29b8",
                "abcdef",
            ),
            (
                "4767af672b3ed107f25018dc22d6fa4b07d156e13b720971e2c4f6bf",
                "abcdefg",
            ),
            (
                "792e25e0ae286d123a38950007e037d3122e76c4ee201668c385edab",
                "abcdefgh",
            ),
            (
                "56b275d36127dc070cda4019baf2ce2579a25d8c67fa2bc9be61b539",
                "abcdefghi",
            ),
            (
                "f809423cbb25e81a2a64aecee2cd5fdc7d91d5db583901fbf1db3116",
                "abcdefghij",
            ),
            (
                "4c46e10b5b72204e509c3c06072cea970bc020cd45a61a0acdfa97ac",
                "Discard medicine more than two years old.",
            ),
            (
                "cb0cef13c1848d91a6d02637c7c520de1914ad4a7aea824671cc328e",
                "He who has a shady past knows that nice guys finish last.",
            ),
            (
                "6c7bd0f3a6544ea698006c2ea583a85f80ea2913590a186db8bb2f1b",
                "I wouldn't marry him with a ten foot pole.",
            ),
            (
                "981323be3eca6ccfa598e58dd74ed8cb05d5f7f6653b7604b684f904",
                "Free! Free!/A trip/to Mars/for 900/empty jars/Burma Shave",
            ),
            (
                "e6fbf82df5138bf361e826903cadf0612cb2986649ba47a57e1bca99",
                "The days of the digital watch are numbered.  -Tom Stoppard",
            ),
            (
                "6ec2cb2ecafc1a9bddaf4caf57344d853e6ded398927d5694fd7714f",
                "Nepal premier won't resign.",
            ),
            (
                "7f62f36e716e0badaf4a4658da9d09bea26357a1bc6aeb8cf7c3ae35",
                "For every action there is an equal and opposite government program.",
            ),
            (
                "45adffcb86a05ee4d91263a6115dda011b805d442c60836963cb8378",
                "His money is twice tainted: 'taint yours and 'taint mine.",
            ),
            (
                "51cb518f1f68daa901a3075a0a5e1acc755b4e5c82cb47687537f880",
                "There is no reason for any individual to have a computer in their home. -Ken Olsen, 1977",
            ),
            (
                "3b59c5e64b0da7bfc18d7017bf458d90f2c83601ff1afc6263ac0993",
                "It's a tiny change to the code and not completely disgusting. - Bob Manchek",
            ),
            (
                "6a9525c0fac0f91b489bc4f0f539b9ec4a156a4e98bc15b655c2c881",
                "size:  a.out:  bad magic",
            ),
            (
                "a1b2b2905b1527d682049c6a76e35c7d8c72551abfe7833ac1be595f",
                "The major problem is with sendmail.  -Mark Horton",
            ),
            (
                "76cf045c76a5f2e3d64d56c3cdba6a25479334611bc375460526f8c1",
                "Give me a rock, paper and scissors and I will move the world.  CCFestoon",
            ),
            (
                "4473671daeecfdb6f6c5bc06b26374aa5e497cc37119fe14144c430c",
                "If the enemy is within range, then so are you.",
            ),
            (
                "6accb6394758523fcd453d47d37ebd10868957a0a9e81c796736abf8",
                "It's well we cannot hear the screams/That we create in others' dreams.",
            ),
            (
                "6f173f4b6eac7f2a73eaa0833c4563752df2c869dc00b7d30219e12e",
                "You remind me of a TV show, but that's all right: I watch it anyway.",
            ),
            (
                "db05bf4d0f73325208755f4af96cfac6cb3db5dbfc323d675d68f938",
                "C is as portable as Stonehedge!!",
            ),
            (
                "05ffa71bb02e855de1aaee1777b3bdbaf7507646f19c4c6aa29933d0",
                "Even if I could be Shakespeare, I think I should still choose to be Faraday. - A. Huxley",
            ),
            (
                "3ad3c89e15b91e6273534c5d18adadbb528e7b840b288f64e81b8c6d",
                "The fugacity of a constituent in a mixture of gases at a given temperature is proportional to its mole fraction.  Lewis-Randall Rule",
            ),
            (
                "e3763669d1b760c1be7bfcb6625f92300a8430419d1dbad57ec9f53c",
                "How can you write a big system without C++?  -Paul Glick",
            ),
        ];
        
        let mut sha512t384 = Sha512T224Digest::new();
        for ele in cases.iter() {
            sha512t384.write(ele.1.as_bytes());
            assert_eq!(ele.0, Bytes::cvt_bytes_to_str(sha512t384.check_sum().unwrap().sum().as_ref()));
            sha512t384.reset();
        }
    }
    
    #[test]
    fn sha512t256() {
        let cases = [
            (
                "c672b8d1ef56ed28ab87c3622c5114069bdd3ad7b8f9737498d0c01ecef0967a",
                "",
            ),
            (
                "455e518824bc0601f9fb858ff5c37d417d67c2f8e0df2babe4808858aea830f8",
                "a",
            ),
            (
                "22d4d37ec6370571af7109fb12eae79673d5f7c83e6e677083faa3cfac3b2c14",
                "ab",
            ),
            (
                "53048e2681941ef99b2e29b76b4c7dabe4c2d0c634fc6d46e0e2f13107e7af23",
                "abc",
            ),
            (
                "d2891c7978be0e24948f37caa415b87cb5cbe2b26b7bad9dc6391b8a6f6ddcc9",
                "abcd",
            ),
            (
                "de8322b46e78b67d4431997070703e9764e03a1237b896fd8b379ed4576e8363",
                "abcde",
            ),
            (
                "e4fdcb11d1ac14e698743acd8805174cea5ddc0d312e3e47f6372032571bad84",
                "abcdef",
            ),
            (
                "a8117f680bdceb5d1443617cbdae9255f6900075422326a972fdd2f65ba9bee3",
                "abcdefg",
            ),
            (
                "a29b9645d2a02a8b582888d044199787220e316bf2e89d1422d3df26bf545bbe",
                "abcdefgh",
            ),
            (
                "b955095330f9c8188d11884ec1679dc44c9c5b25ff9bda700416df9cdd39188f",
                "abcdefghi",
            ),
            (
                "550762913d51eefbcd1a55068fcfc9b154fd11c1078b996df0d926ea59d2a68d",
                "abcdefghij",
            ),
            (
                "690c8ad3916cefd3ad29226d9875965e3ee9ec0d4482eacc248f2ff4aa0d8e5b",
                "Discard medicine more than two years old.",
            ),
            (
                "25938ca49f7ef1178ce81620842b65e576245fcaed86026a36b516b80bb86b3b",
                "He who has a shady past knows that nice guys finish last.",
            ),
            (
                "698e420c3a7038e53d8e73f4be2b02e03b93464ac1a61ebe69f557079921ef65",
                "I wouldn't marry him with a ten foot pole.",
            ),
            (
                "839b414d7e3900ee243aa3d1f9b6955720e64041f5ab9bedd3eb0a08da5a2ca8",
                "Free! Free!/A trip/to Mars/for 900/empty jars/Burma Shave",
            ),
            (
                "5625ecb9d284e54c00b257b67a8cacb25a78db2845c60ef2d29e43c84f236e8e",
                "The days of the digital watch are numbered.  -Tom Stoppard",
            ),
            (
                "9b81d06bca2f985e6ad3249096ff3c0f2a9ec5bb16ef530d738d19d81e7806f2",
                "Nepal premier won't resign.",
            ),
            (
                "08241df8d91edfcd68bb1a1dada6e0ae1475a5c6e7b8f12d8e24ca43a38240a9",
                "For every action there is an equal and opposite government program.",
            ),
            (
                "4ff74d9213a8117745f5d37b5353a774ec81c5dfe65c4c8986a56fc01f2c551e",
                "His money is twice tainted: 'taint yours and 'taint mine.",
            ),
            (
                "b5baf747c307f98849ec881cf0d48605ae4edd386372aea9b26e71db517e650b",
                "There is no reason for any individual to have a computer in their home. -Ken Olsen, 1977",
            ),
            (
                "7eef0538ebd7ecf18611d23b0e1cd26a74d65b929a2e374197dc66e755ca4944",
                "It's a tiny change to the code and not completely disgusting. - Bob Manchek",
            ),
            (
                "d05600964f83f55323104aadab434f32391c029718a7690d08ddb2d7e8708443",
                "size:  a.out:  bad magic",
            ),
            (
                "53ed5f9b5c0b674ac0f3425d9f9a5d462655b07cc90f5d0f692eec093884a607",
                "The major problem is with sendmail.  -Mark Horton",
            ),
            (
                "5a0147685a44eea2435dbd582724efca7637acd9c428e5e1a05115bc3bc2a0e0",
                "Give me a rock, paper and scissors and I will move the world.  CCFestoon",
            ),
            (
                "1152c9b27a99dbf4057d21438f4e63dd0cd0977d5ff12317c64d3b97fcac875a",
                "If the enemy is within range, then so are you.",
            ),
            (
                "105e890f5d5cf1748d9a7b4cdaf58b69855779deebc2097747c2210a17b2cb51",
                "It's well we cannot hear the screams/That we create in others' dreams.",
            ),
            (
                "74644ead770da1434365cd912656fe1aca2056d3039d39f10eb1151bddb32cf3",
                "You remind me of a TV show, but that's all right: I watch it anyway.",
            ),
            (
                "50a234625de5587581883dad9ef399460928032a5ea6bd005d7dc7b68d8cc3d6",
                "C is as portable as Stonehedge!!",
            ),
            (
                "a7a3846005f8a9935a0a2d43e7fd56d95132a9a3609bf3296ef80b8218acffa0",
                "Even if I could be Shakespeare, I think I should still choose to be Faraday. - A. Huxley",
            ),
            (
                "688ff03e367680757aa9906cb1e2ad218c51f4526dc0426ea229a5ba9d002c69",
                "The fugacity of a constituent in a mixture of gases at a given temperature is proportional to its mole fraction.  Lewis-Randall Rule",
            ),
            (
                "3fa46d52094b01021cff5af9a438982b887a5793f624c0a6644149b6b7c3f485",
                "How can you write a big system without C++?  -Paul Glick",
            ),
        ];
        
        let mut sha512t384 = Sha512T256Digest::new();
        // let mut sha512t384 = Sha512Digest::generate_sha512t(256).unwrap();
        for ele in cases.iter() {
            sha512t384.write(ele.1.as_bytes());
            assert_eq!(ele.0, Bytes::cvt_bytes_to_str(sha512t384.check_sum().unwrap().sum().as_ref()));
            sha512t384.reset();
        }
    }
}
