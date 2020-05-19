//! SHA-256
//! https://www.cnblogs.com/mengsuenyan/p/12697811.html#toc

use crate::crypto::sha::const_tables as mct;
use std::hash::Hasher;
use crate::hash::{GenericHasher, GenericHasherSum};

trait Sha256SeriesDigest {
    
    #[inline]
    fn rotate_s0(x: u32) -> u32 {
        x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
    }
    
    #[inline]
    fn rotate_s1(x: u32) -> u32 {
        x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
    }
    
    #[inline]
    fn rotate_d0(x: u32) -> u32 {
        x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
    }

    #[inline]
    fn rotate_d1(x: u32) -> u32 {
        x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
    }
    
    fn sha256_update(data_block: &[u8], digest: &mut [u32; mct::SHA256_DIGEST_WSIZE]) {
        let mut chunk = 0;
        
        while chunk < data_block.len() {
            let block = &data_block[chunk..(chunk+mct::SHA256_BLOCK_SIZE)];
            const LEN: usize = mct::SHA256_BLOCK_SIZE / mct::SHA256_WORD_LEN;
            let mut word = [0u32; 64];
            let mut itr = block.iter();
            for i in 0..LEN {
                let v = [*itr.next().unwrap(), *itr.next().unwrap(), *itr.next().unwrap(), *itr.next().unwrap()];
                word[i] = u32::from_be_bytes(v);
            }
            
            for j in LEN..64 {
                word[j] = Self::rotate_d1(word[j-2]).wrapping_add(word[j-7]).wrapping_add(Self::rotate_d0(word[j-15])).wrapping_add(word[j-16]);
            }

            let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h) = (digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7]);
            for j in 0..64 {
                // if j > 15 {
                //     word[j] = Self::rotate_d1(word[j-2]).wrapping_add(word[j-7]).wrapping_add(Self::rotate_d0(word[j-15])).wrapping_add(word[j-16]);
                // }
                let t1 = h.wrapping_add(Self::rotate_s1(e)).wrapping_add(mct::f_ch(e,f,g)).wrapping_add(mct::SHA256_K[j]).wrapping_add(word[j]);
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
            chunk += mct::SHA256_BLOCK_SIZE;
        }
    }
    
    fn copy_digest_to(&self, h: &mut [u32; mct::SHA256_DIGEST_WSIZE]);
    fn update_digest_from(&mut self, h: &[u32; mct::SHA256_DIGEST_WSIZE]);

    fn buf_idx(&mut self) -> &mut usize;
    
    fn cur_msg_len(&mut self) -> &mut usize;
    
    fn buf(&mut self) -> &mut [u8;mct::SHA256_BLOCK_SIZE];
    
    fn sha256_write(&mut self, mut bytes: &[u8]) {
        let mut h = [0u32; mct::SHA256_DIGEST_WSIZE];
        self.copy_digest_to(&mut h);
        
        *self.cur_msg_len() += bytes.len();
        let idx = *self.buf_idx();
        if idx > 0 {
            let min = std::cmp::min(mct::SHA256_BLOCK_SIZE - idx, bytes.len());
            let dst = &mut self.buf()[idx..(idx+min)];
            let src = &bytes[0..min];
            dst.copy_from_slice(src);
            *self.buf_idx() += min;
            if *self.buf_idx() == mct::SHA256_BLOCK_SIZE {
                let data_block = &self.buf()[..];
                Self::sha256_update(data_block, &mut h);
                self.update_digest_from(&h);
                *self.buf_idx() = 0;
            }
            
            bytes = &bytes[min..];
        }

        if bytes.len() > mct::SHA256_BLOCK_SIZE {
            let n = bytes.len() & (!(mct::SHA256_BLOCK_SIZE - 1));
            let data_block = &bytes[0..n];
            Self::sha256_update(data_block, &mut h);
            self.update_digest_from(&h);
            bytes = &bytes[n..];
        }

        if bytes.len() > 0 {
            let dst = &mut self.buf()[..bytes.len()];
            dst.copy_from_slice(bytes);
            *self.buf_idx() += bytes.len();
        }
    }
    
    fn sha256_check_sum(&mut self) -> Result<&Self, &str> {
        let mut tmp = [0u8; mct::SHA256_BLOCK_SIZE];
        tmp[0] = 0x80;
        let len = *self.cur_msg_len();
        if len % mct::SHA256_BLOCK_SIZE < 56 {
            self.sha256_write(&tmp[0..(56-(len%mct::SHA256_BLOCK_SIZE))]);
        } else {
            self.sha256_write(&tmp[0..(64+56-(len%mct::SHA256_BLOCK_SIZE))]);
        }

        let len = (len as u64) << 3;
        let len_bytes = len.to_be_bytes();
        self.sha256_write(&len_bytes[..]);

        if *self.buf_idx() != 0 {
            Err("not padded")
        } else {
            Ok(&*self)
        }
    }
}

pub struct Sha256Digest {
    digest: [u32; mct::SHA256_DIGEST_WSIZE],
    buf: [u8; mct::SHA256_BLOCK_SIZE],
    idx: usize,
    len: usize,
}

impl Sha256Digest {
    pub fn new() -> Self {
        Sha256Digest {
            digest: mct::SHA256_INIT,
            buf: [0u8; mct::SHA256_BLOCK_SIZE],
            idx: 0,
            len: 0,
        }
    }
}

impl Default for Sha256Digest {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256SeriesDigest for Sha256Digest {
    fn copy_digest_to(&self, h: &mut [u32; 8]) {
        *h = self.digest;
    }

    fn update_digest_from(&mut self, h: &[u32; 8]) {
        self.digest = *h;
    }

    fn buf_idx(&mut self) -> &mut usize {
        &mut self.idx
    }

    fn cur_msg_len(&mut self) -> &mut usize {
        &mut self.len
    }

    fn buf(&mut self) -> &mut [u8; 64] {
        &mut self.buf
    }
}

impl Hasher for Sha256Digest {
    fn finish(&self) -> u64 {
        let l = self.digest[0].to_be_bytes();
        let u = self.digest[1].to_be_bytes();
        let v = [l[0], l[1], l[2], l[3], u[0], u[1], u[2], u[3]];
        u64::from_le_bytes(v)
    }

    fn write(&mut self, bytes: &[u8]) {
        self.sha256_write(bytes);
    }
}

impl GenericHasher for Sha256Digest {
    fn block_size(&self) -> usize {
        mct::SHA256_BLOCK_SIZE
    }

    fn reset(&mut self) {
        self.digest = mct::SHA256_INIT;
        self.idx = 0;
        self.len = 0;
    }

    fn size(&self) -> usize {
        mct::SHA256_DIGEST_SIZE
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
        self.sha256_check_sum()
    }
}

impl GenericHasherSum<[u8; mct::SHA256_DIGEST_SIZE]> for Sha256Digest {
    fn sum(&self) -> [u8; mct::SHA256_DIGEST_SIZE] {
        let h0 = self.digest[0].to_be_bytes();
        let h1 = self.digest[1].to_be_bytes();
        let h2 = self.digest[2].to_be_bytes();
        let h3 = self.digest[3].to_be_bytes();
        let h4 = self.digest[4].to_be_bytes();
        let h5 = self.digest[5].to_be_bytes();
        let h6 = self.digest[6].to_be_bytes();
        let h7 = self.digest[7].to_be_bytes();
        [
            h0[0], h0[1], h0[2], h0[3],
            h1[0], h1[1], h1[2], h1[3],
            h2[0], h2[1], h2[2], h2[3],
            h3[0], h3[1], h3[2], h3[3],
            h4[0], h4[1], h4[2], h4[3],
            h5[0], h5[1], h5[2], h5[3],
            h6[0], h6[1], h6[2], h6[3],
            h7[0], h7[1], h7[2], h7[3],
        ]
    }
    
    fn sum_copy_to(&self, v: &mut [u8; mct::SHA256_DIGEST_SIZE])  {
        let mut itr = v.iter_mut();
        for &ele in self.digest.iter() {
            let v = ele.to_be_bytes();
            *itr.next().unwrap() = v[0];
            *itr.next().unwrap() = v[1];
            *itr.next().unwrap() = v[2];
            *itr.next().unwrap() = v[3];
        }
    }
}

pub struct Sha224Digest {
    digest: Sha256Digest
}

impl Sha224Digest {
    pub fn new() -> Self {
        Sha224Digest {
            digest: Sha256Digest {
                digest: mct::SHA224_INIT,
                buf: [0u8; mct::SHA256_BLOCK_SIZE],
                idx: 0,
                len: 0,
            }
        }
    }
}

impl Default for Sha224Digest {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256SeriesDigest for Sha224Digest {
    fn copy_digest_to(&self, h: &mut [u32; 8]) {
        self.digest.copy_digest_to(h)
    }

    fn update_digest_from(&mut self, h: &[u32; 8]) {
        self.digest.update_digest_from(h)
    }

    fn buf_idx(&mut self) -> &mut usize {
        self.digest.buf_idx()
    }

    fn cur_msg_len(&mut self) -> &mut usize {
        self.digest.cur_msg_len()
    }

    fn buf(&mut self) -> &mut [u8; 64] {
        self.digest.buf()
    }
}

impl Hasher for Sha224Digest {
    fn finish(&self) -> u64 {
        self.digest.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.digest.write(bytes);
    }
}

impl GenericHasher for Sha224Digest {
    fn block_size(&self) -> usize {
        mct::SHA224_BLOCK_SIZE
    }

    fn reset(&mut self) {
        self.digest.digest = mct::SHA224_INIT;
        self.digest.idx = 0;
        self.digest.len = 0;
    }

    fn size(&self) -> usize {
        mct::SHA224_DIGEST_SIZE
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

impl GenericHasherSum<[u8; mct::SHA224_DIGEST_SIZE]> for Sha224Digest {
    fn sum(&self) -> [u8; 28] {
        let h0 = self.digest.digest[0].to_be_bytes();
        let h1 = self.digest.digest[1].to_be_bytes();
        let h2 = self.digest.digest[2].to_be_bytes();
        let h3 = self.digest.digest[3].to_be_bytes();
        let h4 = self.digest.digest[4].to_be_bytes();
        let h5 = self.digest.digest[5].to_be_bytes();
        let h6 = self.digest.digest[6].to_be_bytes();
        [
            h0[0], h0[1], h0[2], h0[3],
            h1[0], h1[1], h1[2], h1[3],
            h2[0], h2[1], h2[2], h2[3],
            h3[0], h3[1], h3[2], h3[3],
            h4[0], h4[1], h4[2], h4[3],
            h5[0], h5[1], h5[2], h5[3],
            h6[0], h6[1], h6[2], h6[3],
        ]
    }

    fn sum_copy_to(&self, v: &mut [u8; 28]) {
        let mut itr = v.iter_mut();
        let mut ele_itr = self.digest.digest.iter();
        for _ in 0..mct::SHA224_DIGEST_WSIZE {
            let &ele = ele_itr.next().unwrap();
            let v = ele.to_be_bytes();
            *itr.next().unwrap() = v[0];
            *itr.next().unwrap() = v[1];
            *itr.next().unwrap() = v[2];
            *itr.next().unwrap() = v[3];
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::sha::Sha256Digest;
    use std::hash::Hasher;
    use crate::hash::{GenericHasher, GenericHasherSum};
    use crate::encoding::Bytes;
    use crate::crypto::sha::sha256::Sha224Digest;

    #[test]
    fn sha256() {
        let cases = [
            ("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", ""),
            ("ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb", "a"),
            ("fb8e20fc2e4c3f248c60c39bd652f3c1347298bb977b8b4d5903b85055620603", "ab"),
            ("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", "abc"),
            ("88d4266fd4e6338d13b845fcf289579d209c897823b9217da3e161936f031589", "abcd"),
            ("36bbe50ed96841d10443bcb670d6554f0a34b761be67ec9c4a8ad2c0c44ca42c", "abcde"),
            ("bef57ec7f53a6d40beb640a780a639c83bc29ac8a9816f1fc6c5c6dcd93c4721", "abcdef"),
            ("7d1a54127b222502f5b79b5fb0803061152a44f92b37e23c6527baf665d4da9a", "abcdefg"),
            ("9c56cc51b374c3ba189210d5b6d4bf57790d351c96c47c02190ecf1e430635ab", "abcdefgh"),
            ("19cc02f26df43cc571bc9ed7b0c4d29224a3ec229529221725ef76d021c8326f", "abcdefghi"),
            ("72399361da6a7754fec986dca5b7cbaf1c810a28ded4abaf56b2106d06cb78b0", "abcdefghij"),
            ("a144061c271f152da4d151034508fed1c138b8c976339de229c3bb6d4bbb4fce", "Discard medicine more than two years old."),
            ("6dae5caa713a10ad04b46028bf6dad68837c581616a1589a265a11288d4bb5c4", "He who has a shady past knows that nice guys finish last."),
            ("ae7a702a9509039ddbf29f0765e70d0001177914b86459284dab8b348c2dce3f", "I wouldn't marry him with a ten foot pole."),
            ("6748450b01c568586715291dfa3ee018da07d36bb7ea6f180c1af6270215c64f", "Free! Free!/A trip/to Mars/for 900/empty jars/Burma Shave"),
            ("14b82014ad2b11f661b5ae6a99b75105c2ffac278cd071cd6c05832793635774", "The days of the digital watch are numbered.  -Tom Stoppard"),
            ("7102cfd76e2e324889eece5d6c41921b1e142a4ac5a2692be78803097f6a48d8", "Nepal premier won't resign."),
            ("23b1018cd81db1d67983c5f7417c44da9deb582459e378d7a068552ea649dc9f", "For every action there is an equal and opposite government program."),
            ("8001f190dfb527261c4cfcab70c98e8097a7a1922129bc4096950e57c7999a5a", "His money is twice tainted: 'taint yours and 'taint mine."),
            ("8c87deb65505c3993eb24b7a150c4155e82eee6960cf0c3a8114ff736d69cad5", "There is no reason for any individual to have a computer in their home. -Ken Olsen, 1977"),
            ("bfb0a67a19cdec3646498b2e0f751bddc41bba4b7f30081b0b932aad214d16d7", "It's a tiny change to the code and not completely disgusting. - Bob Manchek"),
            ("7f9a0b9bf56332e19f5a0ec1ad9c1425a153da1c624868fda44561d6b74daf36", "size:  a.out:  bad magic"),
            ("b13f81b8aad9e3666879af19886140904f7f429ef083286195982a7588858cfc", "The major problem is with sendmail.  -Mark Horton"),
            ("b26c38d61519e894480c70c8374ea35aa0ad05b2ae3d6674eec5f52a69305ed4", "Give me a rock, paper and scissors and I will move the world.  CCFestoon"),
            ("049d5e26d4f10222cd841a119e38bd8d2e0d1129728688449575d4ff42b842c1", "If the enemy is within range, then so are you."),
            ("0e116838e3cc1c1a14cd045397e29b4d087aa11b0853fc69ec82e90330d60949", "It's well we cannot hear the screams/That we create in others' dreams."),
            ("4f7d8eb5bcf11de2a56b971021a444aa4eafd6ecd0f307b5109e4e776cd0fe46", "You remind me of a TV show, but that's all right: I watch it anyway."),
            ("61c0cc4c4bd8406d5120b3fb4ebc31ce87667c162f29468b3c779675a85aebce", "C is as portable as Stonehedge!!"),
            ("1fb2eb3688093c4a3f80cd87a5547e2ce940a4f923243a79a2a1e242220693ac", "Even if I could be Shakespeare, I think I should still choose to be Faraday. - A. Huxley"),
            ("395585ce30617b62c80b93e8208ce866d4edc811a177fdb4b82d3911d8696423", "The fugacity of a constituent in a mixture of gases at a given temperature is proportional to its mole fraction.  Lewis-Randall Rule"),
            ("4f9b189a13d030838269dce846b16a1ce9ce81fe63e65de2f636863336a98fe6", "How can you write a big system without C++?  -Paul Glick"),
        ];
        
        let mut sha256 = Sha256Digest::new();
        for ele in cases.iter() {
            sha256.write(ele.1.as_bytes());
            assert_eq!(ele.0, Bytes::cvt_bytes_to_str(sha256.check_sum().unwrap().sum().as_ref()), "cases=>{}", ele.1);
            sha256.reset();
        }
    }
    
    #[test]
    fn sha224() {
        let cases = [
            ("d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f", ""),
            ("abd37534c7d9a2efb9465de931cd7055ffdb8879563ae98078d6d6d5", "a"),
            ("db3cda86d4429a1d39c148989566b38f7bda0156296bd364ba2f878b", "ab"),
            ("23097d223405d8228642a477bda255b32aadbce4bda0b3f7e36c9da7", "abc"),
            ("a76654d8e3550e9a2d67a0eeb6c67b220e5885eddd3fde135806e601", "abcd"),
            ("bdd03d560993e675516ba5a50638b6531ac2ac3d5847c61916cfced6", "abcde"),
            ("7043631cb415556a275a4ebecb802c74ee9f6153908e1792a90b6a98", "abcdef"),
            ("d1884e711701ad81abe0c77a3b0ea12e19ba9af64077286c72fc602d", "abcdefg"),
            ("17eb7d40f0356f8598e89eafad5f6c759b1f822975d9c9b737c8a517", "abcdefgh"),
            ("aeb35915346c584db820d2de7af3929ffafef9222a9bcb26516c7334", "abcdefghi"),
            ("d35e1e5af29ddb0d7e154357df4ad9842afee527c689ee547f753188", "abcdefghij"),
            ("19297f1cef7ddc8a7e947f5c5a341e10f7245045e425db67043988d7", "Discard medicine more than two years old."),
            ("0f10c2eb436251f777fbbd125e260d36aecf180411726c7c885f599a", "He who has a shady past knows that nice guys finish last."),
            ("4d1842104919f314cad8a3cd20b3cba7e8ed3e7abed62b57441358f6", "I wouldn't marry him with a ten foot pole."),
            ("a8ba85c6fe0c48fbffc72bbb2f03fcdbc87ae2dc7a56804d1590fb3b", "Free! Free!/A trip/to Mars/for 900/empty jars/Burma Shave"),
            ("5543fbab26e67e8885b1a852d567d1cb8b9bfe42e0899584c50449a9", "The days of the digital watch are numbered.  -Tom Stoppard"),
            ("65ca107390f5da9efa05d28e57b221657edc7e43a9a18fb15b053ddb", "Nepal premier won't resign."),
            ("84953962be366305a9cc9b5cd16ed019edc37ac96c0deb3e12cca116", "For every action there is an equal and opposite government program."),
            ("35a189ce987151dfd00b3577583cc6a74b9869eecf894459cb52038d", "His money is twice tainted: 'taint yours and 'taint mine."),
            ("2fc333713983edfd4ef2c0da6fb6d6415afb94987c91e4069eb063e6", "There is no reason for any individual to have a computer in their home. -Ken Olsen, 1977"),
            ("cbe32d38d577a1b355960a4bc3c659c2dc4670859a19777a875842c4", "It's a tiny change to the code and not completely disgusting. - Bob Manchek"),
            ("a2dc118ce959e027576413a7b440c875cdc8d40df9141d6ef78a57e1", "size:  a.out:  bad magic"),
            ("d10787e24052bcff26dc484787a54ed819e4e4511c54890ee977bf81", "The major problem is with sendmail.  -Mark Horton"),
            ("62efcf16ab8a893acdf2f348aaf06b63039ff1bf55508c830532c9fb", "Give me a rock, paper and scissors and I will move the world.  CCFestoon"),
            ("3e9b7e4613c59f58665104c5fa86c272db5d3a2ff30df5bb194a5c99", "If the enemy is within range, then so are you."),
            ("5999c208b8bdf6d471bb7c359ac5b829e73a8211dff686143a4e7f18", "It's well we cannot hear the screams/That we create in others' dreams."),
            ("3b2d67ff54eabc4ef737b14edf87c64280ef582bcdf2a6d56908b405", "You remind me of a TV show, but that's all right: I watch it anyway."),
            ("d0733595d20e4d3d6b5c565a445814d1bbb2fd08b9a3b8ffb97930c6", "C is as portable as Stonehedge!!"),
            ("43fb8aeed8a833175c9295c1165415f98c866ef08a4922959d673507", "Even if I could be Shakespeare, I think I should still choose to be Faraday. - A. Huxley"),
            ("ec18e66e93afc4fb1604bc2baedbfd20b44c43d76e65c0996d7851c6", "The fugacity of a constituent in a mixture of gases at a given temperature is proportional to its mole fraction.  Lewis-Randall Rule"),
            ("86ed2eaa9c75ba98396e5c9fb2f679ecf0ea2ed1e0ee9ceecb4a9332", "How can you write a big system without C++?  -Paul Glick"),
        ];
        
        let mut sha224 = Sha224Digest::new();
        for ele in cases.iter() {
            sha224.write(ele.1.as_bytes());
            assert_eq!(ele.0, Bytes::cvt_bytes_to_str(sha224.check_sum().unwrap().sum().as_ref()), "case=>{}", ele.1);
            sha224.reset();
        }
    }
}

