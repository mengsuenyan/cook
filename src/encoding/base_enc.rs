//! Base32  
//! RFC4648  
//! https://www.cnblogs.com/mengsuenyan/p/12950518.html

use BaseType::Base32;
use crate::encoding::{Encoder, Decoder};
use crate::encoding::base_enc::BaseType::Base64;

const BASE32_STD: [u8; 32] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'2', b'3', b'4', b'5', b'6', b'7',
];

const BASE32_HEX: [u8; 32] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V',
];

const BASE64_STD: [u8; 64] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'+', b'/',
];

const BASE64_URL: [u8; 64] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'-', b'_',
];

enum BaseType {
    Base32 {tbl: &'static [u8; 32], is_padding: bool},
    Base64 {tbl: &'static [u8; 64], is_padding: bool},
}

pub struct Base {
    base_type: BaseType,
    d_map: [u8; 256],
}

impl Base {
    /// 使用标准编码表
    pub fn base32_std(is_padding: bool) -> Base {
        let mut d_map = [0xffu8; 256];
        for (i, &ele) in BASE32_STD.iter().enumerate() {
            d_map[ele as usize] = i as u8;
        }
        Base {base_type: Base32 {tbl: &BASE32_STD, is_padding}, d_map}
    }
    
    /// 用于NextSECure3(NSEC3)的使用扩展16进制编码表
    pub fn base32_hex(is_padding: bool) -> Base {
        let mut d_map = [0xffu8; 256];
        for (i, &ele) in BASE32_HEX.iter().enumerate() {
            d_map[ele as usize] = i as u8;
        }
        Base {base_type: Base32 {tbl: &BASE32_HEX, is_padding}, d_map}
    }
    
    /// 使用标准编码表  
    pub fn base64_std(is_padding: bool) -> Base {
        let mut d_map = [0xffu8; 256];
        for (i, &ele) in BASE64_STD.iter().enumerate() {
            d_map[ele as usize] = i as u8;
        }
        
        Base {base_type: Base64 {tbl: &BASE64_STD, is_padding}, d_map}
    }
    
    pub fn base64_url(is_padding: bool) -> Base {
        let mut d_map = [0xffu8; 256];
        for (i, &ele) in BASE64_URL.iter().enumerate() {
            d_map[ele as usize] = i as u8;
        }
        
        Base {base_type: Base64 {tbl: &BASE64_URL, is_padding}, d_map}
    }
    
    pub fn is_padding(&self) -> bool {
        match self.base_type {
            Base32{is_padding, ..} => is_padding,
            Base64 {is_padding, ..} => is_padding,
        }
    }
    
    pub fn switch_padding(&mut self, is_padding: bool) {
        match &mut self.base_type {
            Base32 {tbl: _, is_padding: x} => {
                *x = is_padding;
            },
            Base64 {tbl: _, is_padding: x} => {
                *x = is_padding;
            },
        }
    }
    
    #[inline]
    fn base32_enc_fuc(idx: usize, g: &[u8;5]) -> usize {
        match idx {
            0 => (g[0] >> 3) as usize,
            1 => ((g[1] >> 6) | ((g[0] & 0x7) << 2)) as usize,
            2 => ((g[1] >> 1) & 0x1f) as usize,
            3 => ((g[2] >> 4) | ((g[1] & 0x1) << 4)) as usize,
            4 => ((g[3] >> 7) | ((g[2] & 0xf) << 1)) as usize,
            5 => ((g[3] >> 2) & 0x1f) as usize,
            6 => ((g[4] >> 5) | ((g[3] & 0x3) << 3)) as usize,
            _ => (g[4] & 0x1f) as usize,
        }
    }
    
    fn base32_encode(&self, dst: &mut Vec<u8>, src: &[u8], tbl: &[u8; 32], is_padding: bool) -> Result<(), &'static str> {
        let (num, rom) = (src.len() / 5, src.len() % 5);
        let (ori_len, pad_num) = (dst.len(), if rom > 0 { ((5-rom) << 3) / 5 } else {0});
        
        dst.resize((num << 3) + ori_len, 0u8);
        let mut src_itr = src.iter();
        let mut dst_itr = dst.iter_mut().skip(ori_len);
        let idx = Base::base32_enc_fuc;
        
        for _ in 0..num {
            let g = [*src_itr.next().unwrap(), *src_itr.next().unwrap(), *src_itr.next().unwrap(),
            *src_itr.next().unwrap(), *src_itr.next().unwrap()];
            
            // 按照书写顺序5位一组
            for i in 0..8 {
                *dst_itr.next().unwrap() = tbl[idx(i, &g)];
            }
        }

        let mut g = [0u8; 5];
        for (i, &ele) in src_itr.enumerate() {
            g[i] = ele;
        }
        if rom > 0 {
            for j in 0..(8-pad_num) {
                dst.push(tbl[idx(j, &g)])
            }
        }
        
        if is_padding {
            for _ in 0..pad_num {
                dst.push(b'=');
            }
        }
        
        Ok(())
    }

    #[inline]
    fn base32_dec_fuc(dst: &mut [u8; 5], src: &[u8; 8]) {
        dst[0] = (src[0] << 3) | (src[1] >> 2);
        dst[1] = (src[1] << 6) | (src[2] << 1) | (src[3] >> 4);
        dst[2] = (src[3] << 4) | (src[4] >> 1);
        dst[3] = (src[4] << 7) | (src[5] << 2) | (src[6] >> 3);
        dst[4] = (src[6] << 5) | (src[7]);
    }

    fn base32_decode(&self, dst: &mut Vec<u8>, src: &[u8], tbl: &[u8; 256]) -> Result<(), &'static str> {
        if src.is_empty() {
            return Ok(());
        }
        
        let rom = src.len() % 8;
        let mut pad_num = 0;
        
        let tail_len = if rom == 0 {
            for &ele in src.iter().rev() {
                if ele == b'=' {
                    pad_num += 1;
                } else {
                    break;
                }
            }
            if pad_num > 0 { (40 - (pad_num * 5)) >> 3 } else { 0 }
        } else {
            (rom * 5) >> 3
        };
        
        let src = &src[0..(src.len() - pad_num)];
        let num = src.len() >> 3;
        let mut src_itr = src.iter();
        for _ in 0..num {
            let buf = [tbl[*src_itr.next().unwrap() as usize], tbl[*src_itr.next().unwrap() as usize], tbl[*src_itr.next().unwrap() as usize],
            tbl[*src_itr.next().unwrap() as usize], tbl[*src_itr.next().unwrap() as usize], tbl[*src_itr.next().unwrap() as usize],
            tbl[*src_itr.next().unwrap() as usize], tbl[*src_itr.next().unwrap() as usize]];
            let mut tmp = [0xffu8; 5];
            
            Self::base32_dec_fuc(&mut tmp, &buf);
            for &ele in tmp.iter() {
                if ele != 0xff {
                    dst.push(ele);
                } else {
                    return Err("Invalid decoded data");
                }
            }
        }
        
        let mut buf = [0x0u8; 8];
        let mut tmp = [0xffu8; 5];
        for (i, &ele) in src_itr.enumerate() {
            buf[i] = tbl[ele as usize];
        }
        Self::base32_dec_fuc(&mut tmp, &buf);
        for i in 0..tail_len{
            if tmp[i] != 0xff {
                dst.push(tmp[i]);
            } else {
                return Err("Invalid decoded data");
            }
        }
        
        Ok(())
    }
    
    fn base64_encode(&self, dst: &mut Vec<u8>, src: &[u8], tbl: &[u8; 64], is_padding: bool) -> Result<(), &'static str> {
        let num = src.len() / 3;
        
        let mut s = src.iter();
        for _ in 0..num {
            let seg = [0, *s.next().unwrap(), *s.next().unwrap(), *s.next().unwrap()];
            let seg = u32::from_be_bytes(seg);
            dst.push(tbl[((seg >> 18) & 0x3f) as usize]);
            dst.push(tbl[((seg >> 12) & 0x3f) as usize]);
            dst.push(tbl[((seg >> 6) & 0x3f) as usize]);
            dst.push(tbl[(seg & 0x3f) as usize]);
        }
        
        if let Some(&x) = s.next() {
            dst.push(tbl[(x >> 2) as usize]);
            if let Some(&y) = s.next() {
                dst.push(tbl[(((x & 0x3) << 4) | (y >> 4)) as usize]);
                dst.push(tbl[((y & 0xf) << 2) as usize]);
                if is_padding {
                    dst.push(b'=');
                }
            } else {
                dst.push(tbl[((x & 0x3) << 4 ) as usize]);
                if is_padding {
                    dst.push(b'=');
                    dst.push(b'=');
                }
            }
        }

        Ok(())
    }
    
    fn base64_decode(&self, dst: &mut Vec<u8>, src: &[u8], tbl: &[u8; 256]) -> Result<(), &'static str> {
        if src.is_empty() {
            return Ok(());
        }
        
        let mut pad_num = 0;
        if (src.len() & 0x3) == 0 {
            for &ele in src.iter().rev() {
                if ele == b'=' {
                    pad_num += 1;
                } else {
                    break;
                }
            }
        }
        
        let src = &src[0..(src.len() - pad_num)];
        let (num, rom, mut s) = (src.len() >> 2, src.len() & 0x3, src.iter());
        for _ in 0..num {
            let buf = [tbl[*s.next().unwrap() as usize], tbl[*s.next().unwrap() as usize], 
                tbl[*s.next().unwrap() as usize], tbl[*s.next().unwrap() as usize]];
            if buf[0] == 0xff || buf[1] == 0xff || buf[2] == 0xff || buf[3] == 0xff {
                return Err("Invalid decode data");
            }
            dst.push((buf[0] << 2) | (buf[1] >> 4));
            dst.push(((buf[1] & 0xf) << 4) | (buf[2] >> 2));
            dst.push(((buf[2] & 0x3) << 6) | (buf[3]));
        }
        
        let mut buf = [0u8;4];
        for (i, &ele) in s.enumerate() {
            buf[i] = tbl[ele as usize];
        }
        if buf[0] == 0xff || buf[1] == 0xff || buf[2] == 0xff || buf[3] == 0xff {
            return Err("Invalid decode data");
        }
        let (v1, v2) = ((buf[0] << 2) | (buf[1] >> 4), ((buf[1] & 0xf) << 4) | (buf[2] >> 2));

        match rom {
            2 => {
                dst.push(v1);
                Ok(())
            },
            3 => {
                dst.push(v1);
                dst.push(v2);
                Ok(())
            }
            0 => Ok(()),
            _ => Err("Invalid decode data")
        }
    }
}

impl Encoder for Base {
    type Item = u8;
    type Output = ();

    fn encode(&self, dst: &mut Vec<Self::Item>, src: &[Self::Item]) -> Result<Self::Output, &'static str> {
        match self.base_type {
            Base32 {tbl,is_padding} => {
                dst.clear();
                self.base32_encode(dst, src, tbl, is_padding)
            },
            Base64 {tbl, is_padding} => {
                dst.clear();
                self.base64_encode(dst, src, tbl, is_padding)
            }
        }
    }

    fn encode_append(&self, dst: &mut Vec<Self::Item>, src: &[Self::Item]) -> Result<Self::Output, &'static str> {
        match self.base_type {
            Base32 {tbl,is_padding} => {
                self.base32_encode(dst, src, tbl, is_padding)
            }, 
            Base64 {tbl, is_padding} => {
                self.base64_encode(dst, src, tbl, is_padding)
            }
        }
    }
}

impl Decoder for Base {
    type Item = u8; 
    type Output = ();

    fn decode(&self, dst: &mut Vec<Self::Item>, src: &[Self::Item]) -> Result<Self::Output, &'static str> {
        match self.base_type {
            Base32 {..} => {
                dst.clear();
                self.base32_decode(dst, src, &self.d_map)
            },
            Base64 {..} => {
                dst.clear();
                self.base64_decode(dst, src, &self.d_map)
            }
        }
    }

    fn decode_append(&self, dst: &mut Vec<Self::Item>, src: &[Self::Item]) -> Result<Self::Output, &'static str> {
        match self.base_type {
            Base32 {..} => {
                self.base32_decode(dst, src, &self.d_map)
            },
            Base64 {..} => {
                self.base64_decode(dst, src, &self.d_map)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding::base_enc::Base;
    use crate::encoding::{Encoder, Decoder};

    #[test]
    fn base32() {
        let cases = [
            // RFC 4648 examples
            ("", ""),
            ("f", "MY======"),
            ("fo", "MZXQ===="),
            ("foo", "MZXW6==="),
            ("foob", "MZXW6YQ="),
            ("fooba", "MZXW6YTB"),
            ("foobar", "MZXW6YTBOI======"),

            // Wikipedia examples, converted to base32
            ("sure.", "ON2XEZJO"),
            ("sure", "ON2XEZI="),
            ("sur", "ON2XE==="),
            ("su", "ON2Q===="),
            ("leasure.", "NRSWC43VOJSS4==="),
            ("easure.", "MVQXG5LSMUXA===="),
            ("asure.", "MFZXK4TFFY======"),
            ("sure.", "ON2XEZJO"),
            ("Twas brillig, and the slithy toves", "KR3WC4ZAMJZGS3DMNFTSYIDBNZSCA5DIMUQHG3DJORUHSIDUN53GK4Y="),
            ("320934fsafkf f90349q018-n`mcmfsaf98rn-23-nb7-bnnwewrwf2930i00802`sfafe902390u9u32fjsalkdsafjs932905u20852080fjjjjjjjj3290owhqmnbnannnqf902349021831-042890482-14-75-753flafdjsakoweioqri32u1oupf30458058-23skdfsalieup39821074klasfj;sa  vasf328-la; fapiow[rq[[[[[[[[[[[[[[[[3209784732-5-1",
            "GMZDAOJTGRTHGYLGNNTCAZRZGAZTIOLRGAYTQLLOMBWWG3LGONQWMOJYOJXC2MRTFVXGENZNMJXG453FO5ZHOZRSHEZTA2JQGA4DAMTAONTGCZTFHEYDEMZZGB2TS5JTGJTGU43BNRVWI43BMZVHGOJTGI4TANLVGIYDQNJSGA4DAZTKNJVGU2TKNJVDGMRZGBXXO2DRNVXGE3TBNZXG44LGHEYDEMZUHEYDEMJYGMYS2MBUGI4DSMBUHAZC2MJUFU3TKLJXGUZWM3DBMZSGU43BNNXXOZLJN5YXE2JTGJ2TC33VOBTDGMBUGU4DANJYFUZDG43LMRTHGYLMNFSXK4BTHE4DEMJQG42GW3DBONTGUO3TMEQCA5TBONTDGMRYFVWGCOZAMZQXA2LPO5NXE4K3LNNVWW23LNNVWW23LNNVWW23GMZDAOJXHA2DOMZSFU2S2MI="),
        ];
        
        let mut base = Base::base32_std(true);
        let mut v = Vec::new();
        for ele in cases.iter() {
            base.encode(&mut v, ele.0.as_bytes()).unwrap();
            assert_eq!(String::from_utf8_lossy(v.as_slice()), ele.1);
            base.decode(&mut v, ele.1.as_bytes()).unwrap();
            assert_eq!(String::from_utf8_lossy(v.as_slice()), ele.0);
        }
        
        base.switch_padding(false);
        for ele in cases.iter() {
            base.encode(&mut v, ele.0.as_bytes()).unwrap();
            assert_eq!(String::from_utf8_lossy(v.as_slice()), String::from(ele.1).replace('=', ""));
            base.decode(&mut v, ele.1.as_bytes()).unwrap();
            assert_eq!(String::from_utf8_lossy(v.as_slice()), ele.0);
        }
    }
    
    #[test]
    fn base64() {
        let cases = [
            // RFC 4648 examples
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),

            // Wikipedia examples
            ("sure.", "c3VyZS4="),
            ("sure", "c3VyZQ=="),
            ("sur", "c3Vy"),
            ("su", "c3U="),
            ("leasure.", "bGVhc3VyZS4="),
            ("easure.", "ZWFzdXJlLg=="),
            ("asure.", "YXN1cmUu"),
            ("sure.", "c3VyZS4="),
            ("Twas brillig, and the slithy toves",
            "VHdhcyBicmlsbGlnLCBhbmQgdGhlIHNsaXRoeSB0b3Zlcw=="),
            ("320934fsafkf f90349q018-n`mcmfsaf98rn-23-nb7-bnnwewrwf2930i00802`sfafe902390u9u32fjsalkdsafjs932905u20852080fjjjjjjjj3290owhqmnbnannnqf902349021831-042890482-14-75-753flafdjsakoweioqri32u1oupf30458058-23skdfsalieup39821074klasfj;sa  vasf328-la; fapiow[rq[[[[[[[[[[[[[[[[3209784732-5-1",
            "MzIwOTM0ZnNhZmtmIGY5MDM0OXEwMTgtbmBtY21mc2FmOThybi0yMy1uYjctYm5ud2V3cndmMjkzMGkwMDgwMmBzZmFmZTkwMjM5MHU5dTMyZmpzYWxrZHNhZmpzOTMyOTA1dTIwODUyMDgwZmpqampqampqMzI5MG93aHFtbmJuYW5ubnFmOTAyMzQ5MDIxODMxLTA0Mjg5MDQ4Mi0xNC03NS03NTNmbGFmZGpzYWtvd2Vpb3FyaTMydTFvdXBmMzA0NTgwNTgtMjNza2Rmc2FsaWV1cDM5ODIxMDc0a2xhc2ZqO3NhICB2YXNmMzI4LWxhOyBmYXBpb3dbcnFbW1tbW1tbW1tbW1tbW1tbMzIwOTc4NDczMi01LTE=")
        ];
        
        let mut base = Base::base64_std(true);
        let mut v = Vec::new();
        for ele in cases.iter() {
            base.encode(&mut v, ele.0.as_bytes()).unwrap();
            assert_eq!(String::from_utf8_lossy(v.as_slice()), ele.1);
            base.decode(&mut v, ele.1.as_bytes()).unwrap();
            assert_eq!(String::from_utf8_lossy(v.as_slice()), ele.0);
        }
        
        base.switch_padding(false);
        for ele in cases.iter() {
            base.encode(&mut v, ele.0.as_bytes()).unwrap();
            assert_eq!(String::from_utf8_lossy(v.as_slice()), String::from(ele.1).replace('=', ""));
            base.decode(&mut v, ele.1.as_bytes()).unwrap();
            assert_eq!(String::from_utf8_lossy(v.as_slice()), ele.0);
        }

        // RFC 3548 examples
        base.switch_padding(true);
        let cases = [([0x14u8, 0xfb, 0x9c, 0x03, 0xd9, 0x7e], "FPucA9l+"),];
        let mut v = Vec::new();
        for ele in cases.iter() {
            base.encode(&mut v, ele.0.as_ref()).unwrap();
            assert_eq!(String::from_utf8_lossy(v.as_slice()), ele.1);
            base.decode(&mut v, ele.1.as_bytes()).unwrap();
            assert_eq!(v.as_slice(), ele.0.as_ref());
        }
        let cases = [([0x14,0xfb,0x9c,0x03,0xd9], "FPucA9k="),];
        let mut v = Vec::new();
        for ele in cases.iter() {
            base.encode(&mut v, ele.0.as_ref()).unwrap();
            assert_eq!(String::from_utf8_lossy(v.as_slice()), ele.1);
            base.decode(&mut v, ele.1.as_bytes()).unwrap();
            assert_eq!(v.as_slice(), ele.0.as_ref());
        }
        let cases = [([0x14,0xfb,0x9c,0x03], "FPucAw=="),];
        let mut v = Vec::new();
        for ele in cases.iter() {
            base.encode(&mut v, ele.0.as_ref()).unwrap();
            assert_eq!(String::from_utf8_lossy(v.as_slice()), ele.1);
            base.decode(&mut v, ele.1.as_bytes()).unwrap();
            assert_eq!(v.as_slice(), ele.0.as_ref());
        }
    }
}
