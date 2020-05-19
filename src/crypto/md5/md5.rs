//! MD5消息摘要算法  
//! RFC-1321  
//! https://www.cnblogs.com/mengsuenyan/p/12697709.html


use crate::crypto::md5::const_tables as mct;
use std::hash::Hasher;
use crate::hash::{GenericHasher, GenericHasherSum};

pub struct Md5Digest {
    digest: [u32; 4],
    buf: [u8; mct::MD5_BLOCK_SIZE],
    idx: usize,
    len: usize,
}

macro_rules! md5_upd_digest {
    ($Obj: ident, $A: ident, $B: ident, $C: ident, $D: ident) => {
        $Obj.digest[0] = $A;
        $Obj.digest[1] = $B;
        $Obj.digest[2] = $C;
        $Obj.digest[3] = $D;
    };
}

impl Md5Digest {
    pub fn new() -> Md5Digest {
        Md5Digest {
            digest: mct::MD5_INIT,
            buf: [0; mct::MD5_BLOCK_SIZE],
            idx: 0,
            len: 0,
        }
    }
    
    fn update(&self, data_block: &[u8]) -> (u32, u32, u32, u32) {
        let (mut a, mut b, mut c, mut d) = (self.digest[0], self.digest[1], self.digest[2], self.digest[3]);
        
        let mut i = 0;
        while i < data_block.len() {
            let (aa, bb, cc, dd) = (a, b, c, d);
            let mut x = [0u32; 16];
            let msg = &data_block[i..(i+mct::MD5_BLOCK_SIZE)];
            let mut msg_itr = msg.iter();
            for j in 0..16 {
                let v = [*msg_itr.next().unwrap(), *msg_itr.next().unwrap(), *msg_itr.next().unwrap(), *msg_itr.next().unwrap()];
                x[j] = u32::from_le_bytes(v);
            }

            // round 1
            a = b.wrapping_add((((c^d)&b)^d).wrapping_add(a).wrapping_add(x[0]).wrapping_add(0xd76aa478).rotate_left(7));
            d = a.wrapping_add((((b^c)&a)^c).wrapping_add(d).wrapping_add(x[1]).wrapping_add(0xe8c7b756).rotate_left(12));
            c = d.wrapping_add((((a^b)&d)^b).wrapping_add(c).wrapping_add(x[2]).wrapping_add(0x242070db).rotate_left(17));
            b = c.wrapping_add((((d^a)&c)^a).wrapping_add(b).wrapping_add(x[3]).wrapping_add(0xc1bdceee).rotate_left(22));
            a = b.wrapping_add((((c^d)&b)^d).wrapping_add(a).wrapping_add(x[4]).wrapping_add(0xf57c0faf).rotate_left(7));
            d = a.wrapping_add((((b^c)&a)^c).wrapping_add(d).wrapping_add(x[5]).wrapping_add(0x4787c62a).rotate_left(12));
            c = d.wrapping_add((((a^b)&d)^b).wrapping_add(c).wrapping_add(x[6]).wrapping_add(0xa8304613).rotate_left(17));
            b = c.wrapping_add((((d^a)&c)^a).wrapping_add(b).wrapping_add(x[7]).wrapping_add(0xfd469501).rotate_left(22));
            a = b.wrapping_add((((c^d)&b)^d).wrapping_add(a).wrapping_add(x[8]).wrapping_add(0x698098d8).rotate_left(7));
            d = a.wrapping_add((((b^c)&a)^c).wrapping_add(d).wrapping_add(x[9]).wrapping_add(0x8b44f7af).rotate_left(12));
            c = d.wrapping_add((((a^b)&d)^b).wrapping_add(c).wrapping_add(x[10]).wrapping_add(0xffff5bb1).rotate_left(17));
            b = c.wrapping_add((((d^a)&c)^a).wrapping_add(b).wrapping_add(x[11]).wrapping_add(0x895cd7be).rotate_left(22));
            a = b.wrapping_add((((c^d)&b)^d).wrapping_add(a).wrapping_add(x[12]).wrapping_add(0x6b901122).rotate_left(7));
            d = a.wrapping_add((((b^c)&a)^c).wrapping_add(d).wrapping_add(x[13]).wrapping_add(0xfd987193).rotate_left(12));
            c = d.wrapping_add((((a^b)&d)^b).wrapping_add(c).wrapping_add(x[14]).wrapping_add(0xa679438e).rotate_left(17));
            b = c.wrapping_add((((d^a)&c)^a).wrapping_add(b).wrapping_add(x[15]).wrapping_add(0x49b40821).rotate_left(22));

            // round 2
            a = b.wrapping_add((((b^c)&d)^c).wrapping_add(a).wrapping_add(x[1]).wrapping_add(0xf61e2562).rotate_left(5));
            d = a.wrapping_add((((a^b)&c)^b).wrapping_add(d).wrapping_add(x[6]).wrapping_add(0xc040b340).rotate_left(9));
            c = d.wrapping_add((((d^a)&b)^a).wrapping_add(c).wrapping_add(x[11]).wrapping_add(0x265e5a51).rotate_left(14));
            b = c.wrapping_add((((c^d)&a)^d).wrapping_add(b).wrapping_add(x[0]).wrapping_add(0xe9b6c7aa).rotate_left(20));
            a = b.wrapping_add((((b^c)&d)^c).wrapping_add(a).wrapping_add(x[5]).wrapping_add(0xd62f105d).rotate_left(5));
            d = a.wrapping_add((((a^b)&c)^b).wrapping_add(d).wrapping_add(x[10]).wrapping_add(0x02441453).rotate_left(9));
            c = d.wrapping_add((((d^a)&b)^a).wrapping_add(c).wrapping_add(x[15]).wrapping_add(0xd8a1e681).rotate_left(14));
            b = c.wrapping_add((((c^d)&a)^d).wrapping_add(b).wrapping_add(x[4]).wrapping_add(0xe7d3fbc8).rotate_left(20));
            a = b.wrapping_add((((b^c)&d)^c).wrapping_add(a).wrapping_add(x[9]).wrapping_add(0x21e1cde6).rotate_left(5));
            d = a.wrapping_add((((a^b)&c)^b).wrapping_add(d).wrapping_add(x[14]).wrapping_add(0xc33707d6).rotate_left(9));
            c = d.wrapping_add((((d^a)&b)^a).wrapping_add(c).wrapping_add(x[3]).wrapping_add(0xf4d50d87).rotate_left(14));
            b = c.wrapping_add((((c^d)&a)^d).wrapping_add(b).wrapping_add(x[8]).wrapping_add(0x455a14ed).rotate_left(20));
            a = b.wrapping_add((((b^c)&d)^c).wrapping_add(a).wrapping_add(x[13]).wrapping_add(0xa9e3e905).rotate_left(5));
            d = a.wrapping_add((((a^b)&c)^b).wrapping_add(d).wrapping_add(x[2]).wrapping_add(0xfcefa3f8).rotate_left(9));
            c = d.wrapping_add((((d^a)&b)^a).wrapping_add(c).wrapping_add(x[7]).wrapping_add(0x676f02d9).rotate_left(14));
            b = c.wrapping_add((((c^d)&a)^d).wrapping_add(b).wrapping_add(x[12]).wrapping_add(0x8d2a4c8a).rotate_left(20));

            // round 3
            a = b.wrapping_add((b^c^d).wrapping_add(a).wrapping_add(x[5]).wrapping_add(0xfffa3942).rotate_left(4));
            d = a.wrapping_add((a^b^c).wrapping_add(d).wrapping_add(x[8]).wrapping_add(0x8771f681).rotate_left(11));
            c = d.wrapping_add((d^a^b).wrapping_add(c).wrapping_add(x[11]).wrapping_add(0x6d9d6122).rotate_left(16));
            b = c.wrapping_add((c^d^a).wrapping_add(b).wrapping_add(x[14]).wrapping_add(0xfde5380c).rotate_left(23));
            a = b.wrapping_add((b^c^d).wrapping_add(a).wrapping_add(x[1]).wrapping_add(0xa4beea44).rotate_left(4));
            d = a.wrapping_add((a^b^c).wrapping_add(d).wrapping_add(x[4]).wrapping_add(0x4bdecfa9).rotate_left(11));
            c = d.wrapping_add((d^a^b).wrapping_add(c).wrapping_add(x[7]).wrapping_add(0xf6bb4b60).rotate_left(16));
            b = c.wrapping_add((c^d^a).wrapping_add(b).wrapping_add(x[10]).wrapping_add(0xbebfbc70).rotate_left(23));
            a = b.wrapping_add((b^c^d).wrapping_add(a).wrapping_add(x[13]).wrapping_add(0x289b7ec6).rotate_left(4));
            d = a.wrapping_add((a^b^c).wrapping_add(d).wrapping_add(x[0]).wrapping_add(0xeaa127fa).rotate_left(11));
            c = d.wrapping_add((d^a^b).wrapping_add(c).wrapping_add(x[3]).wrapping_add(0xd4ef3085).rotate_left(16));
            b = c.wrapping_add((c^d^a).wrapping_add(b).wrapping_add(x[6]).wrapping_add(0x04881d05).rotate_left(23));
            a = b.wrapping_add((b^c^d).wrapping_add(a).wrapping_add(x[9]).wrapping_add(0xd9d4d039).rotate_left(4));
            d = a.wrapping_add((a^b^c).wrapping_add(d).wrapping_add(x[12]).wrapping_add(0xe6db99e5).rotate_left(11));
            c = d.wrapping_add((d^a^b).wrapping_add(c).wrapping_add(x[15]).wrapping_add(0x1fa27cf8).rotate_left(16));
            b = c.wrapping_add((c^d^a).wrapping_add(b).wrapping_add(x[2]).wrapping_add(0xc4ac5665).rotate_left(23));
            
            // round 4
            a = b.wrapping_add((c^(b|(!d))).wrapping_add(a).wrapping_add(x[0]).wrapping_add(0xf4292244).rotate_left(6));
            d = a.wrapping_add((b^(a|(!c))).wrapping_add(d).wrapping_add(x[7]).wrapping_add(0x432aff97).rotate_left(10));
            c = d.wrapping_add((a^(d|(!b))).wrapping_add(c).wrapping_add(x[14]).wrapping_add(0xab9423a7).rotate_left(15));
            b = c.wrapping_add((d^(c|(!a))).wrapping_add(b).wrapping_add(x[5]).wrapping_add(0xfc93a039).rotate_left(21));
            a = b.wrapping_add((c^(b|(!d))).wrapping_add(a).wrapping_add(x[12]).wrapping_add(0x655b59c3).rotate_left(6));
            d = a.wrapping_add((b^(a|(!c))).wrapping_add(d).wrapping_add(x[3]).wrapping_add(0x8f0ccc92).rotate_left(10));
            c = d.wrapping_add((a^(d|(!b))).wrapping_add(c).wrapping_add(x[10]).wrapping_add(0xffeff47d).rotate_left(15));
            b = c.wrapping_add((d^(c|(!a))).wrapping_add(b).wrapping_add(x[1]).wrapping_add(0x85845dd1).rotate_left(21));
            a = b.wrapping_add((c^(b|(!d))).wrapping_add(a).wrapping_add(x[8]).wrapping_add(0x6fa87e4f).rotate_left(6));
            d = a.wrapping_add((b^(a|(!c))).wrapping_add(d).wrapping_add(x[15]).wrapping_add(0xfe2ce6e0).rotate_left(10));
            c = d.wrapping_add((a^(d|(!b))).wrapping_add(c).wrapping_add(x[6]).wrapping_add(0xa3014314).rotate_left(15));
            b = c.wrapping_add((d^(c|(!a))).wrapping_add(b).wrapping_add(x[13]).wrapping_add(0x4e0811a1).rotate_left(21));
            a = b.wrapping_add((c^(b|(!d))).wrapping_add(a).wrapping_add(x[4]).wrapping_add(0xf7537e82).rotate_left(6));
            d = a.wrapping_add((b^(a|(!c))).wrapping_add(d).wrapping_add(x[11]).wrapping_add(0xbd3af235).rotate_left(10));
            c = d.wrapping_add((a^(d|(!b))).wrapping_add(c).wrapping_add(x[2]).wrapping_add(0x2ad7d2bb).rotate_left(15));
            b = c.wrapping_add((d^(c|(!a))).wrapping_add(b).wrapping_add(x[9]).wrapping_add(0xeb86d391).rotate_left(21));
            
            // add saved state
            a = a.wrapping_add(aa);
            b = b.wrapping_add(bb);
            c = c.wrapping_add(cc);
            d = d.wrapping_add(dd);
            
            i += mct::MD5_BLOCK_SIZE;
        }
        (a, b, c, d)
    }
    
}

impl Hasher for Md5Digest {
    fn finish(&self) -> u64 {
        let l = self.digest[0].to_le_bytes();
        let u = self.digest[1].to_le_bytes();
        let v = [l[0], l[1], l[2], l[3], u[0], u[1], u[2], u[3]];
        u64::from_be_bytes(v)
    }
    
    fn write(&mut self, mut bytes: &[u8]) {
        let blen = bytes.len();
        self.len += blen;
        
        if self.idx > 0 {
            let min = std::cmp::min(mct::MD5_BLOCK_SIZE - self.idx, bytes.len());
            let dst = &mut self.buf[self.idx..(self.idx+min)];
            let src = &bytes[0..min];
            dst.copy_from_slice(src);
            self.idx += min;

            if self.idx == mct::MD5_BLOCK_SIZE {
                let data_block = &self.buf[..];
                let (a, b, c, d ) = self.update(data_block);
                md5_upd_digest!(self, a, b, c, d);
                self.idx = 0;
            }
            
            bytes = &bytes[min..];
        }
        
        if bytes.len() >= mct::MD5_BLOCK_SIZE {
            let n = bytes.len() & (!(mct::MD5_BLOCK_SIZE - 1));
            let data_block = &bytes[0..n];
            let (a, b, c, d ) = self.update(data_block);
            md5_upd_digest!(self, a, b, c, d);
            bytes = &bytes[n..];
        }
        
        if bytes.len() > 0 {
            let dst = &mut self.buf[..bytes.len()];
            dst.copy_from_slice(bytes);
            self.idx += bytes.len();
        }
    }
}

impl GenericHasher for Md5Digest {
    fn block_size(&self) -> usize {
        mct::MD5_BLOCK_SIZE
    }
    
    fn reset(&mut self) {
        self.digest = mct::MD5_INIT;
        self.idx = 0;
        self.len = 0;
    }
    
    fn size(&self) -> usize {
        mct::MD5_DIGEST_SIZE
    }
    
    fn append_to_vec(&self, data: &mut Vec<u8>) -> usize {
        let len = data.len();
        
        let h: [u8;16] = self.sum();
        for &ele in h.iter() {
            data.push(ele);
        }
        
        data.len() - len
    }
    
    fn check_sum(&mut self) -> Result<&Self, &str> {
        // 补0x80, 然后填充0对齐到56字节, 然后按从低字节到高字节填充位长度
        let mut tmp = [0u8;1+63+8];
        tmp[0] = 0x80;
        let pad_len = 55usize.wrapping_sub(self.len) % 64;
        let len = (self.len << 3) as u64;
        let src = len.to_le_bytes();
        let dst = &mut tmp[(1+pad_len)..(1+pad_len+8)];
        dst.copy_from_slice(&src[..]);
        self.write(&tmp[0..(1+pad_len+8)]);
        
        if self.idx != 0 {
            Err("not padded")
        } else {
            Ok(&*self)
        }
    }
}

impl GenericHasherSum<u128> for Md5Digest {
    fn sum(&self) -> u128 {
        u128::from_be_bytes(self.sum_as(&[0u8;16]))
    }
}

impl GenericHasherSum<[u32; 4]> for Md5Digest {
    fn sum(&self) -> [u32; 4] {
        self.digest
    }
}

impl GenericHasherSum<[u8; 16]> for Md5Digest {
    fn sum(&self) -> [u8; 16] {
        let v0 = self.digest[0].to_le_bytes();
        let v1 = self.digest[1].to_le_bytes();
        let v2 = self.digest[2].to_le_bytes();
        let v3 = self.digest[3].to_le_bytes();
        [
            v0[0], v0[1], v0[2], v0[3],
            v1[0], v1[1], v1[2], v1[3],
            v2[0], v2[1], v2[2], v2[3],
            v3[0], v3[1], v3[2], v3[3],
        ]
    }
}

impl Default for Md5Digest {
    fn default() -> Self {
        Md5Digest::new()
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hasher;
    use crate::hash::{GenericHasher, GenericHasherSum};

    #[test]
    fn md5() {
        let cases = [
            (0xd41d8cd98f00b204e9800998ecf8427eu128, ""),
            (0x0cc175b9c0f1b6a831c399e269772661u128, "a"),
            (0x187ef4436122d1cc2f40dc2b92f0eba0u128, "ab"),
            (0x900150983cd24fb0d6963f7d28e17f72u128, "abc"),
            (0xe2fc714c4727ee9395f324cd2e7f331fu128, "abcd"),
            (0xab56b4d92b40713acc5af89985d4b786u128, "abcde"),
            (0xe80b5017098950fc58aad83c8c14978eu128, "abcdef"),
            (0x7ac66c0f148de9519b8bd264312c4d64u128, "abcdefg"),
            (0xe8dc4081b13434b45189a720b77b6818u128, "abcdefgh"),
            (0x8aa99b1f439ff71293e95357bac6fd94u128, "abcdefghi"),
            (0xa925576942e94b2ef57a066101b48876u128, "abcdefghij"),
            (0xd747fc1719c7eacb84058196cfe56d57u128, "Discard medicine more than two years old."),
            (0xbff2dcb37ef3a44ba43ab144768ca837u128, "He who has a shady past knows that nice guys finish last."),
            (0x0441015ecb54a7342d017ed1bcfdbea5u128, "I wouldn't marry him with a ten foot pole."),
            (0x9e3cac8e9e9757a60c3ea391130d3689u128, "Free! Free!/A trip/to Mars/for 900/empty jars/Burma Shave"),
            (0xa0f04459b031f916a59a35cc482dc039u128, "The days of the digital watch are numbered.  -Tom Stoppard"),
            (0xe7a48e0fe884faf31475d2a04b1362ccu128, "Nepal premier won't resign."),
            (0x637d2fe925c07c113800509964fb0e06u128, "For every action there is an equal and opposite government program."),
            (0x834a8d18d5c6562119cf4c7f5086cb71u128, "His money is twice tainted: 'taint yours and 'taint mine."),
            (0xde3a4d2fd6c73ec2db2abad23b444281u128, "There is no reason for any individual to have a computer in their home. -Ken Olsen, 1977"),
            (0xacf203f997e2cf74ea3aff86985aefafu128, "It's a tiny change to the code and not completely disgusting. - Bob Manchek"),
            (0xe1c1384cb4d2221dfdd7c795a4222c9au128, "size:  a.out:  bad magic"),
            (0xc90f3ddecc54f34228c063d7525bf644u128, "The major problem is with sendmail.  -Mark Horton"),
            (0xcdf7ab6c1fd49bd9933c43f3ea5af185u128, "Give me a rock, paper and scissors and I will move the world.  CCFestoon"),
            (0x83bc85234942fc883c063cbd7f0ad5d0u128, "If the enemy is within range, then so are you."),
            (0x277cbe255686b48dd7e8f389394d9299u128, "It's well we cannot hear the screams/That we create in others' dreams."),
            (0xfd3fb0a7ffb8af16603f3d3af98f8e1fu128, "You remind me of a TV show, but that's all right: I watch it anyway."),
            (0x469b13a78ebf297ecda64d4723655154u128, "C is as portable as Stonehedge!!"),
            (0x63eb3a2f466410104731c4b037600110u128, "Even if I could be Shakespeare, I think I should still choose to be Faraday. - A. Huxley"),
            (0x72c2ed7592debca1c90fc0100f931a2fu128, "The fugacity of a constituent in a mixture of gases at a given temperature is proportional to its mole fraction.  Lewis-Randall Rule"),
            (0x132f7619d33b523b1d9e5bd8e0928355u128, "How can you write a big system without C++?  -Paul Glick"),
        ];
        
        let mut md5 = super::Md5Digest::new();
        for ele in cases.iter() {
            md5.write(ele.1.as_bytes());
            // println!("{:x}", md5.check_sum().unwrap().sum_as(&0u128));
            // println!("{:?}", md5.sum_as(&[0u8;16]));
            assert_eq!(md5.check_sum().unwrap().sum_as(&0u128), ele.0, "cases=>{}", ele.1);
            md5.reset();
        }
    }
}
