//! SHA-1
//! https://www.cnblogs.com/mengsuenyan/p/12697811.html#toc

use crate::crypto::sha::const_tables as mct;
use std::hash::Hasher;
use crate::hash::{GenericHasher, GenericHasherSum};

pub struct Sha1Digest {
    digest: [u32; mct::SHA1_DIGEST_WSIZE],
    buf: [u8; mct::SHA1_BLOCK_SIZE],
    idx: usize,
    len: usize,
}

macro_rules! sha1_upd_digest {
    ($a: ident, $b: ident, $c: ident, $d: ident, $e: ident, $A: ident, $B: ident, $C: ident, $D: ident, $E: ident) => {
        {
            let (aa, bb, cc, dd, ee) = ($A, $B, $C, $D, $E);
            $a = aa;
            $b = bb;
            $c = cc;
            $d = dd;
            $e = ee;
        };
    };
}

impl Sha1Digest {
    pub fn new() -> Sha1Digest {
        Sha1Digest {
            digest: mct::SHA1_INIT,
            buf: [0; mct::SHA1_BLOCK_SIZE],
            idx: 0,
            len: 0,
        }
    }
    
    #[inline]
    fn f_word_extract(w: &mut [u32; mct::SHA1_BLOCK_SIZE/mct::SHA1_WORD_LEN], s: usize) -> u32 {
        w[s&0xf] = (w[(s+13)&0xf] ^ w[(s+8)&0xf] ^ w[(s+2)&0xf] ^ w[s&0xf]).rotate_left(1);
        // w[s&0xf] = (w[(s-3)&0xf] ^ w[(s-8)&0xf] ^ w[(s-14)&0xf] ^ w[(s-16)&0xf]).rotate_left(1);
        w[s&0xf]
    }
    
    fn update(&self, data_block: &[u8]) -> (u32, u32, u32, u32, u32) {
        let mut chunk = 0;

        let (mut h0, mut h1, mut h2, mut h3, mut h4) = (self.digest[0], self.digest[1], self.digest[2], self.digest[3], self.digest[4]);
        while chunk < data_block.len() {
            let bytes = &data_block[chunk..(chunk+mct::SHA1_BLOCK_SIZE)];
            
            const LEN: usize = mct::SHA1_BLOCK_SIZE / mct::SHA1_WORD_LEN;
            let mut word = [0u32; LEN];
            let mut bytes_itr = bytes.iter();
            for i in 0..LEN {
                let v = [*bytes_itr.next().unwrap(), *bytes_itr.next().unwrap(), *bytes_itr.next().unwrap(), *bytes_itr.next().unwrap()];
                word[i] = u32::from_be_bytes(v);
            }
            
            let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
            
            let mut j = 0;
            while j < 16 {
                let t = a.rotate_left(5).wrapping_add(mct::f_ch(b, c, d)).wrapping_add(e).wrapping_add(mct::SHA1_K[0]).wrapping_add(word[j]);
                let b_p = b.rotate_left(30);
                sha1_upd_digest!(a, b, c, d, e, t, a, b_p, c, d);
                j += 1;
            }
            
            while j < 20 {
                let t = a.rotate_left(5).wrapping_add(mct::f_ch(b, c, d)).wrapping_add(e).wrapping_add(mct::SHA1_K[0]).wrapping_add(Sha1Digest::f_word_extract(&mut word, j));
                let b_p = b.rotate_left(30);
                sha1_upd_digest!(a, b, c, d, e, t, a, b_p, c, d);
                j += 1;
            }
            
            while j < 40 {
                let t = a.rotate_left(5).wrapping_add(mct::f_parity(b, c, d)).wrapping_add(e).wrapping_add(mct::SHA1_K[1]).wrapping_add(Sha1Digest::f_word_extract(&mut word, j));
                let b_p = b.rotate_left(30);
                sha1_upd_digest!(a, b, c, d, e, t, a, b_p, c, d);
                j += 1;
            }
            
            while j < 60 {
                let t = a.rotate_left(5).wrapping_add(mct::f_maj(b, c, d)).wrapping_add(e).wrapping_add(mct::SHA1_K[2]).wrapping_add(Sha1Digest::f_word_extract(&mut word, j));
                let b_p = b.rotate_left(30);
                sha1_upd_digest!(a, b, c, d, e, t, a, b_p, c, d);
                j += 1;
            }
            
            while j < 80 {
                let t = a.rotate_left(5).wrapping_add(mct::f_parity(b, c, d)).wrapping_add(e).wrapping_add(mct::SHA1_K[3]).wrapping_add(Sha1Digest::f_word_extract(&mut word, j));
                let b_p = b.rotate_left(30);
                sha1_upd_digest!(a, b, c, d, e, t, a, b_p, c, d);
                j += 1;
            }
            
            h0 = h0.wrapping_add(a);
            h1 = h1.wrapping_add(b);
            h2 = h2.wrapping_add(c);
            h3 = h3.wrapping_add(d);
            h4 = h4.wrapping_add(e);
            chunk += mct::SHA1_BLOCK_SIZE;
        }
        
        (h0, h1, h2, h3, h4)
    }
    
    fn upd_digest(&mut self, h: &(u32, u32, u32, u32, u32)) {
        self.digest[0] = h.0;
        self.digest[1] = h.1;
        self.digest[2] = h.2;
        self.digest[3] = h.3;
        self.digest[4] = h.4;
    }
}

impl Hasher for Sha1Digest {
    fn finish(&self) -> u64 {
        let l = self.digest[0].to_be_bytes();
        let u = self.digest[1].to_be_bytes();
        let v = [l[0], l[1], l[2], l[3], u[0], u[1], u[2], u[3]];
        u64::from_le_bytes(v)
    }

    fn write(&mut self, mut bytes: &[u8]) {
        self.len += bytes.len();
        
        if self.idx > 0 {
            let min = std::cmp::min(mct::SHA1_BLOCK_SIZE - self.idx, bytes.len());
            let dst = &mut self.buf[self.idx..(self.idx+min)];
            let src = &bytes[0..min];
            dst.copy_from_slice(src);
            self.idx += min;
            
            if self.idx == mct::SHA1_BLOCK_SIZE {
                let data_block = &self.buf[..];
                let h = self.update(data_block);
                self.upd_digest(&h);
                self.idx = 0;
            }
            
            bytes = &bytes[min..];
        }
        
        if bytes.len() > mct::SHA1_BLOCK_SIZE {
            let n = bytes.len() & (!(mct::SHA1_BLOCK_SIZE - 1));
            let data_block = &bytes[0..n];
            let h = self.update(data_block);
            self.upd_digest(&h);
            bytes = &bytes[n..];
        }
        
        if bytes.len() > 0 {
            let dst = &mut self.buf[..bytes.len()];
            dst.copy_from_slice(bytes);
            self.idx += bytes.len();
        }
    }
}

impl GenericHasher for Sha1Digest {
    fn block_size(&self) -> usize {
        mct::SHA1_BLOCK_SIZE
    }

    fn reset(&mut self) {
        self.digest = mct::SHA1_INIT;
        self.idx = 0;
        self.len = 0;
    }

    fn size(&self) -> usize {
        mct::SHA1_DIGEST_SIZE
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
        let mut tmp = [0u8; mct::SHA1_BLOCK_SIZE];
        tmp[0] = 0x80;
        let len = self.len;
        if len % mct::SHA1_BLOCK_SIZE < 56 {
            self.write(&tmp[0..(56-(len%mct::SHA1_BLOCK_SIZE))]);
        } else {
            self.write(&tmp[0..(64+56-(len%mct::SHA1_BLOCK_SIZE))]);
        }
        
        let len = (len as u64) << 3;
        let len_bytes = len.to_be_bytes();
        self.write(&len_bytes[..]);
        
        if self.idx != 0 {
            Err("not padded")
        } else {
            Ok(&*self)
        }
    }
}

impl GenericHasherSum<[u8; 20]> for Sha1Digest {
    fn sum(&self) -> [u8; 20] {
        let h0 = self.digest[0].to_be_bytes();
        let h1 = self.digest[1].to_be_bytes();
        let h2 = self.digest[2].to_be_bytes();
        let h3 = self.digest[3].to_be_bytes();
        let h4 = self.digest[4].to_be_bytes();
        [
            h0[0], h0[1], h0[2], h0[3],
            h1[0], h1[1], h1[2], h1[3],
            h2[0], h2[1], h2[2], h2[3],
            h3[0], h3[1], h3[2], h3[3],
            h4[0], h4[1], h4[2], h4[3],
        ]
    }
}

impl Default for Sha1Digest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hasher;
    use crate::encoding::Bytes;
    use crate::hash::{GenericHasher, GenericHasherSum};
    
    #[test]
    fn sha1() {
        let cases = [
            ("da39a3ee5e6b4b0d3255bfef95601890afd80709", ""),
            ("86f7e437faa5a7fce15d1ddcb9eaeaea377667b8", "a"),
            ("da23614e02469a0d7c7bd1bdab5c9c474b1904dc", "ab"),
            ("a9993e364706816aba3e25717850c26c9cd0d89d", "abc"),
            ("81fe8bfe87576c3ecb22426f8e57847382917acf", "abcd"),
            ("03de6c570bfe24bfc328ccd7ca46b76eadaf4334", "abcde"),
            ("1f8ac10f23c5b5bc1167bda84b833e5c057a77d2", "abcdef"),
            ("2fb5e13419fc89246865e7a324f476ec624e8740", "abcdefg"),
            ("425af12a0743502b322e93a015bcf868e324d56a", "abcdefgh"),
            ("c63b19f1e4c8b5f76b25c49b8b87f57d8e4872a1", "abcdefghi"),
            ("d68c19a0a345b7eab78d5e11e991c026ec60db63", "abcdefghij"),
            ("ebf81ddcbe5bf13aaabdc4d65354fdf2044f38a7", "Discard medicine more than two years old."),
            ("e5dea09392dd886ca63531aaa00571dc07554bb6", "He who has a shady past knows that nice guys finish last."),
            ("45988f7234467b94e3e9494434c96ee3609d8f8f", "I wouldn't marry him with a ten foot pole."),
            ("55dee037eb7460d5a692d1ce11330b260e40c988", "Free! Free!/A trip/to Mars/for 900/empty jars/Burma Shave"),
            ("b7bc5fb91080c7de6b582ea281f8a396d7c0aee8", "The days of the digital watch are numbered.  -Tom Stoppard"),
            ("c3aed9358f7c77f523afe86135f06b95b3999797", "Nepal premier won't resign."),
            ("6e29d302bf6e3a5e4305ff318d983197d6906bb9", "For every action there is an equal and opposite government program."),
            ("597f6a540010f94c15d71806a99a2c8710e747bd", "His money is twice tainted: 'taint yours and 'taint mine."),
            ("6859733b2590a8a091cecf50086febc5ceef1e80", "There is no reason for any individual to have a computer in their home. -Ken Olsen, 1977"),
            ("514b2630ec089b8aee18795fc0cf1f4860cdacad", "It's a tiny change to the code and not completely disgusting. - Bob Manchek"),
            ("c5ca0d4a7b6676fc7aa72caa41cc3d5df567ed69", "size:  a.out:  bad magic"),
            ("74c51fa9a04eadc8c1bbeaa7fc442f834b90a00a", "The major problem is with sendmail.  -Mark Horton"),
            ("0b4c4ce5f52c3ad2821852a8dc00217fa18b8b66", "Give me a rock, paper and scissors and I will move the world.  CCFestoon"),
            ("3ae7937dd790315beb0f48330e8642237c61550a", "If the enemy is within range, then so are you."),
            ("410a2b296df92b9a47412b13281df8f830a9f44b", "It's well we cannot hear the screams/That we create in others' dreams."),
            ("841e7c85ca1adcddbdd0187f1289acb5c642f7f5", "You remind me of a TV show, but that's all right: I watch it anyway."),
            ("163173b825d03b952601376b25212df66763e1db", "C is as portable as Stonehedge!!"),
            ("32b0377f2687eb88e22106f133c586ab314d5279", "Even if I could be Shakespeare, I think I should still choose to be Faraday. - A. Huxley"),
            ("0885aaf99b569542fd165fa44e322718f4a984e0", "The fugacity of a constituent in a mixture of gases at a given temperature is proportional to its mole fraction.  Lewis-Randall Rule"),
            ("6627d6904d71420b0bf3886ab629623538689f45", "How can you write a big system without C++?  -Paul Glick"),
            ("76245dbf96f661bd221046197ab8b9f063f11bad", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"),
        ];
        
        let mut sha1 = super::Sha1Digest::new();
        for ele in cases.iter() {
            sha1.write(ele.1.as_bytes());
            let s = sha1.check_sum().unwrap().sum();
            // println!("{}", cvt_bytes_to_str(&s[..]));
            assert_eq!(Bytes::cvt_bytes_to_str(&s[..]).as_str(), ele.0, "case=>{}", ele.1);
            sha1.reset()
        }
    }
}

