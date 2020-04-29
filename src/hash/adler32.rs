//! Adler-32校验算法, RFC1950
//! C = s1 + s2 * 2^16;
//! s1 = 1; s2 = 0;
//! s1 = (s1+byte) % Mod;
//! s2 = (s1 + s2) % Mod;
//! https://www.cnblogs.com/mengsuenyan/p/12802387.html
//! https://mengsuenyan.gitee.io/docs/CS/%E5%B8%B8%E7%94%A8%E6%A0%A1%E9%AA%8C%E5%92%8C(Hash)%E7%AE%97%E6%B3%95.html

use crate::hash::{GenericHasher, GenericHasherSum};
use std::hash::Hasher;

/// 小于2^16的最大质数
const ADLER32_MOD: u32 = 65521;

/// n是满足下式的最大值:  
/// 255 * n * (n+1) / 2 + (n+1) * (mod-1) <= 2^32 - 1  
const ADLER32_NMAX: usize = 5552;

/// Adler32算法hash值的字节长度
const ADLER32_SIZE: usize = 4;

pub struct Adler32 {
    digest: u32,
}

impl Adler32 {
    pub fn new() -> Adler32 {
        Adler32 { digest: 1 }
    }

    fn update(&self, bytes: &[u8]) -> u32 {
        let (mut s1, mut s2) = (self.digest & 0xffff, self.digest >> 16);

        let num = bytes.len() / ADLER32_NMAX;
        let rem = bytes.len() % ADLER32_NMAX;
        let mut idx = 0;
        for _ in 0..num {
            let end = idx + ADLER32_NMAX;
            let b = &bytes[idx..end];
            for &ele in b.iter() {
                s1 += ele as u32;
                s2 += s1;
            }

            s1 %= ADLER32_MOD;
            s2 %= ADLER32_MOD;
            idx = end;
        }

        let end = idx + rem;
        let b = &bytes[idx..end];
        for &ele in b.iter() {
            s1 += ele as u32;
            s2 += s1;
        }
        s1 %= ADLER32_MOD;
        s2 %= ADLER32_MOD;

        s2 << 16 | s1
    }
}

impl Hasher for Adler32 {
    fn finish(&self) -> u64 {
        u64::from(self.sum())
    }

    fn write(&mut self, bytes: &[u8]) {
        self.digest = self.update(bytes);
    }
}

impl GenericHasher for Adler32 {
    fn block_size(&self) -> usize {
        ADLER32_SIZE
    }

    fn reset(&mut self) {
        self.digest = 1u32;
    }

    fn size(&self) -> usize {
        ADLER32_SIZE
    }

    fn append_to_vec(&self, data: &mut Vec<u8>) -> usize {
        let len = data.len();

        let h = self.sum();
        for &ele in h.to_be_bytes().iter() {
            data.push(ele)
        }

        data.len() - len
    }

    fn append_to_slice(&self, data: &[u8]) -> Vec<u8> {
        let mut v = data.to_vec();
        self.append_to_vec(&mut v);
        v
    }
}

impl GenericHasherSum<u32> for Adler32 {
    fn sum(&self) -> u32 {
        self.digest
    }
}

#[cfg(test)]
mod tests {
    //! this come from golang source code

    use crate::hash::{Adler32, GenericHasher, GenericHasherSum};
    use std::hash::Hasher;

    const TEST_DATA: [(u32, &str);32] = [
        (0x00000001, ""),
        (0x00620062, "a"),
        (0x012600c4, "ab"),
        (0x024d0127, "abc"),
        (0x03d8018b, "abcd"),
        (0x05c801f0, "abcde"),
        (0x081e0256, "abcdef"),
        (0x0adb02bd, "abcdefg"),
        (0x0e000325, "abcdefgh"),
        (0x118e038e, "abcdefghi"),
        (0x158603f8, "abcdefghij"),
        (0x3f090f02, "Discard medicine more than two years old."),
        (0x46d81477, "He who has a shady past knows that nice guys finish last."),
        (0x40ee0ee1, "I wouldn't marry him with a ten foot pole."),
        (0x16661315, "Free! Free!/A trip/to Mars/for 900/empty jars/Burma Shave"),
        (0x5b2e1480, "The days of the digital watch are numbered.  -Tom Stoppard"),
        (0x8c3c09ea, "Nepal premier won't resign."),
        (0x45ac18fd, "For every action there is an equal and opposite government program."),
        (0x53c61462, "His money is twice tainted: 'taint yours and 'taint mine."),
        (0x7e511e63, "There is no reason for any individual to have a computer in their home. -Ken Olsen, 1977"),
        (0xe4801a6a, "It's a tiny change to the code and not completely disgusting. - Bob Manchek"),
        (0x61b507df, "size:  a.out:  bad magic"),
        (0xb8631171, "The major problem is with sendmail.  -Mark Horton"),
        (0x8b5e1904, "Give me a rock, paper and scissors and I will move the world.  CCFestoon"),
        (0x7cc6102b, "If the enemy is within range, then so are you."),
        (0x700318e7, "It's well we cannot hear the screams/That we create in others' dreams."),
        (0x1e601747, "You remind me of a TV show, but that's all right: I watch it anyway."),
        (0xb55b0b09, "C is as portable as Stonehedge!!"),
        (0x39111dd0, "Even if I could be Shakespeare, I think I should still choose to be Faraday. - A. Huxley"),
        (0x91dd304f, "The fugacity of a constituent in a mixture of gases at a given temperature is proportional to its mole fraction.  Lewis-Randall Rule"),
        (0x2e5d1316, "How can you write a big system without C++?  -Paul Glick"),
        (0xd0201df6, "'Invariant assertions' is the most elegant programming technique!  -Tom Szymanski"),
    ];

    #[test]
    fn test_adler32_small() {
        let mut h = Adler32::new();
        for ele in TEST_DATA.iter() {
            h.write(ele.1.as_bytes());
            assert_eq!(ele.0, h.sum());
            h.reset();
        }
    }

    #[test]
    fn test_adler32_big() {
        let mo = [
            (0x211297c8, 5548, b'8'),
            (0xbaa198c8, 5549, b'9'),
            (0x553499be, 5550, b'0'),
            (0xf0c19abe, 5551, b'1'),
            (0x8d5c9bbe, 5552, b'2'),
            (0x2af69cbe, 5553, b'3'),
            (0xc9809dbe, 5554, b'4'),
            (0x69189ebe, 5555, b'5'),
        ];
        let mut h = Adler32::new();
        let mut v = vec![0xffu8; 5548];
        for ele in mo.iter() {
            h.write(v.as_slice());
            h.write_u8(ele.2);
            assert_eq!(ele.0, h.sum());
            h.reset();
            v.push(0xff);
        }

        let v = vec![0x00u8; 100000];
        h.write(v.as_slice());
        assert_eq!(0x86af0001, h.sum());
        h.reset();

        let v = vec![b'a'; 100000];
        h.write(v.as_slice());
        assert_eq!(0x79660b4d, h.sum());
        h.reset();

        let s = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".repeat(10000);
        h.write(s.as_bytes());
        assert_eq!(0x110588ee, h.sum());
        h.reset();
    }
}
