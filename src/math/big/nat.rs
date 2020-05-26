use std::{
    cmp::Ordering,
    cmp::{PartialEq, PartialOrd},
    fmt::{Binary, Debug, Display, Error, Formatter, LowerHex, Octal, UpperHex},
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div,
        DivAssign, Mul, MulAssign, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub,
        SubAssign,
    },
    vec::Vec,
};
use crate::math::rand::{Seed, Source, RngSource};

const HEX_BASIC_LEN: usize = 8;
const DEC_BASIC_LEN: usize = 10;
const OCT_BASIC_LEN: usize = 10; // 10 * 3 + 2
const BIN_BASIC_LEN: usize = 32;

/// 自然数Nat:
///
/// 算术操作支持: +, -, *, /, %, |, &, ^, !, <, >, ==, >=, <=;
///
/// 格式化输出支持: 二进制/八进制/十进制/十六进制格式化输出(不支持输出对齐);
///
/// NaN支持: NaN参加任何非逻辑运算, 结果也是NaN. NaN和任何自然数逻辑运算的结果都为false;
///
/// 支持从u8/u16/u32/usize/u64/u128转换为Nat;
///
/// 支持从二进制/八进制/十进制/十六进制字符串转换为Nat. 其中, 字符串不能不包含0b/0x格式头;
///
/// 注: 不支持Inf无穷大的自然数, 诸如Nat::from_u8(1)/Nat::from_u8(0)之类的操作会得到NaN;
#[derive(Clone)]
pub struct Nat {
    nat: Vec<u32>,
}

macro_rules! nat_from_bytes_macro {
    ($nat: ident, $bytes: ident, $step: ident, $arith: expr) => {
        let num = $bytes.len() / $step;
        for idx in 0..num {
            let end = $bytes.len() - $step * idx;
            let start = end - $step;
            let seg = &$bytes[start..end];
            let mut val = 0;
            for (i, ele) in seg.iter().rev().enumerate() {
                val += $arith(ele, i);
            }
            $nat.push(val);
        }

        let end = $bytes.len() - $step * num;
        let seg = &$bytes[0..end];
        let mut val = 0;
        for (i, ele) in seg.iter().rev().enumerate() {
            val += $arith(ele, i);
        }

        if val > 0 {
            $nat.push(val);
        }
    };
}

macro_rules! nat_from_basic_type {
    ($fuc_name: ident, $type: ty, 1) => {
        pub fn $fuc_name(val: $type) -> Nat {
            let mut nat: Vec<u32> = Vec::new();
            let step = std::mem::size_of::<$type>() / std::mem::size_of::<u32>();
            for i in 0..step {
                let ele = ((val >> (i << 5)) & (0xffffffff as $type)) as u32;
                nat.push(ele);
            }
            Nat::trim_last_zeros(&mut nat, 0);
            Nat { nat }
        }
    };
    ($fuc_name: ident, $type: ty, 0) => {
        pub fn $fuc_name(val: $type) -> Nat {
            Nat {
                nat: vec![val as u32],
            }
        }
    };
}

impl Nat {
    fn from_hex_bytes(bytes: &[u8]) -> Nat {
        let mut nat = Vec::with_capacity(bytes.len() >> 2 + 1);

        nat_from_bytes_macro!(nat, bytes, HEX_BASIC_LEN, |&ele, i| -> u32 {
            if ele <= b'9' {
                ((ele - b'0') as u32) << (i << 2)
            } else if ele <= b'F' {
                ((ele - b'A' + 10) as u32) << (i << 2)
            } else {
                ((ele - b'a' + 10) as u32) << (i << 2)
            }
        });

        Nat { nat }
    }

    fn from_oct_bytes(bytes: &[u8]) -> Nat {
        let mut nat = Nat::from_u8(0);
        let num = bytes.len() / OCT_BASIC_LEN;
        nat.as_vec_mut().reserve(num + 1);
        let len = num * OCT_BASIC_LEN;
        let num_arr = &bytes[0..len];
        let rem_arr = &bytes[len..bytes.len()];

        let mut num_arr_itr = num_arr.iter();
        for _ in 0..num {
            nat <<= OCT_BASIC_LEN * 3;
            let mut val = 0u32;
            for _ in 0..OCT_BASIC_LEN {
                val <<= 3;
                let ele = num_arr_itr.next().unwrap() - b'0';
                val += ele as u32;
            }
            nat += &Nat::from_u32(val);
        }

        if rem_arr.len() > 0 {
            nat <<= rem_arr.len() * 3;
            let mut val = 0u32;
            for ele in rem_arr {
                val <<= 3;
                val += (ele - b'0') as u32;
            }
            nat += &Nat::from_u32(val);
        }

        nat
    }

    fn from_bin_bytes(bytes: &[u8]) -> Nat {
        let mut nat = Vec::with_capacity(bytes.len() >> 5 + 1);

        nat_from_bytes_macro!(nat, bytes, BIN_BASIC_LEN, |&ele, i| -> u32 {
            ((ele - b'0') as u32) << i
        });

        Nat { nat }
    }

    fn from_dec_bytes(bytes: &[u8]) -> Nat {
        let mut nat = Nat::from_u8(0);
        nat.as_vec_mut().reserve(bytes.len() / DEC_BASIC_LEN);

        for ele in bytes {
            let n2 = &nat << 1;
            nat <<= 3;
            nat += &n2;
            let val = (ele - b'0') as u32;
            nat += &Nat::from_u32(val);
        }

        nat
    }

    /// 截掉高位多余的0
    fn trim_last_zeros<T: PartialOrd>(nat: &mut Vec<T>, zero: T)
    where
        T: PartialEq,
    {
        while nat.len() > 1 && *nat.last().unwrap() == zero {
            nat.pop();
        }
    }
    
    /// 返回以二进制表示时, 末尾的连续是0的个数  
    fn trailling_zeros(&self) -> usize {
        if self.is_nan() {
            0
        } else {
            let mut cnt = 0usize;
            for &ele in self.as_vec().iter() {
                if ele == 0 {
                    cnt += 32;
                } else {
                    cnt += ele.trailing_zeros() as usize;
                    break;
                }
            }
            
            cnt
        }
    }

    #[inline]
    fn nan() -> Nat {
        Nat { nat: Vec::new() }
    }

    #[inline]
    fn num(&self) -> usize {
        self.nat.len()
    }

    #[inline]
    fn as_vec(&self) -> &Vec<u32> {
        &self.nat
    }

    fn as_vec_mut(&mut self) -> &mut Vec<u32> {
        &mut self.nat
    }

    //TODO: karatsuba
    /// O(n^2)
    fn mul_manual(&self, rhs: &Nat) -> Nat {
        let (min, max) = self.min_max_by_num(rhs);
        const MASK: u64 = 0xffffffff;
        const SHR_BITS: u8 = 32;
        let mut nat = Nat {
            nat: Vec::with_capacity(min.len() + max.len()),
        };

        nat.as_vec_mut().push(0u32);
        // 按32进制计算, 两数相乘最多不超过64位;
        for (i, &min_ele) in min.iter().enumerate() {
            // let mut round_nat = Nat {
            //     nat: Vec::with_capacity(min.len() + max.len()),
            // };
            // let round = round_nat.as_vec_mut();
            let mut round = Vec::with_capacity(min.len() + max.len()) ;
            // 每一轮乘max都左移32位, 额外留出32位作为上一次单步乘的进位
            round.resize(i + 1, 0);
            for &max_ele in max {
                let carry = round.pop().unwrap() as u64;
                let x = (min_ele as u64) * (max_ele as u64);
                let (y, c) = x.overflowing_add(carry);
                round.push((y & MASK) as u32);
                round.push((y >> SHR_BITS) as u32);
                if c {
                    round.push(1);
                }
            }
            nat += &Nat{ nat: round};
        }
        
        Self::trim_last_zeros(nat.as_vec_mut(), 0);

        nat
    }
    
    #[inline]
    fn min_max_by_num<'a>(&'a self, rhs: &'a Nat) -> (&'a Vec<u32>, &'a Vec<u32>) {
        if self.num() < rhs.num() {
            (&self.nat, &rhs.nat)
        } else {
            (&rhs.nat, &self.nat)
        }
    }
    
    fn parse_base(s: &str) -> Option<(u8, &str)> {
        let s = s.trim();
        if !s.is_empty() && s.is_ascii() {
            let mut bytes = s.as_bytes().iter();
            match bytes.next() {
                Some(b'0') => {
                    match bytes.next() { 
                        Some(b'x') => Some((16, &s[2..])),
                        Some(b'b') => Some((2, &s[2..])),
                        Some(..) => Some((8, &s[1..])),
                        _ => Some((10, s)),
                    }
                },
                Some(..) => Some((10, s)),
                None => None,
            }
        } else {
            None
        }
    }

    fn check_str(s: &str, base: u8) -> Option<&[u8]> {
        let s = s.trim();
        if s.is_ascii() && !s.is_empty() {
            let bytes = s.as_bytes();
            let check_ok = match base {
                2 => bytes.iter().all(|&x| -> bool { x == b'0' || x == b'1' }),
                8 => bytes.iter().all(|&x| -> bool { x >= b'0' && x <= b'7' }),
                10 => bytes.iter().all(|&x| -> bool { x >= b'0' && x <= b'9' }),
                16 => bytes.iter().all(|&x| -> bool {
                    (x >= b'0' && x <= b'9') || (x >= b'a' && x <= b'f') || (x >= b'A' && x <= b'F')
                }),
                _ => false,
            };

            if check_ok {
                Some(bytes)
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline]
    fn trim_as_bytes(s: &[u8]) -> &[u8] {
        let mut cnt = 0;
        for &ele in s {
            if ele == b'0' {
                cnt += 1;
            } else {
                break;
            }
        }

        &s[cnt..]
    }

    #[inline]
    pub fn is_nan(&self) -> bool {
        self.nat.is_empty()
    }

    /// 二进制位长度
    #[inline]
    pub fn bits_len(&self) -> usize {
        if self.is_nan() {
            0
        } else if self.num() == 1 {
            let first = *self.as_vec().first().unwrap();
            if first == 0 {
                1
            } else {
                (32 - first.leading_zeros()) as usize
            }
        } else {
            let (num, last) = (self.num()-1, *self.as_vec().last().unwrap());
            (num << 5) + (32 - last.leading_zeros()) as usize
        }
    }
    
    pub fn new(s: &str, base: u8) -> Nat {
        Nat::from_str(s, base)
    }

    /// s必须是二进制/八进制/十进制/十六进制的数字字符串
    /// base: 2/8/10/16
    pub fn from_str(s: &str, base: u8) -> Nat {
        match Nat::check_str(s, base) {
            Some(x) => {
                let x = Nat::trim_as_bytes(x);
                match base {
                    2 => Nat::from_bin_bytes(x),
                    8 => Nat::from_oct_bytes(x),
                    10 => Nat::from_dec_bytes(x),
                    16 => Nat::from_hex_bytes(x),
                    _ => Nat::nan(),
                }
            }
            _ => Nat::nan(),
        }
    }

    /// 小端模式, 低字节是低位
    pub fn from_slice(v: &[u8]) -> Nat {
        match v.is_empty() {
            false => {
                let num = v.len() >> 2;
                let num_arr = &v[0..(num << 2)];
                let rem_arr = &v[(num << 2)..];
                let mut num_arr_itr = num_arr.iter();

                let mut nat = Vec::with_capacity(num + 1);
                for _ in 0..num {
                    let barr = [*num_arr_itr.next().unwrap(),*num_arr_itr.next().unwrap(),*num_arr_itr.next().unwrap(),*num_arr_itr.next().unwrap()];
                    // let ele = (b3 << 24) | (b2 << 16) | (b1 << 8) | b0;
                    let ele = u32::from_le_bytes(barr);
                    nat.push(ele);
                }

                if !rem_arr.is_empty() {
                    let mut val = 0u32;
                    for  &ele in rem_arr.iter().rev() {
                        val <<= 8;
                        val += ele as u32;
                    }
                    nat.push(val);
                }

                Nat { nat }
            }
            _ => Nat::nan(),
        }
    }

    /// 小端模式, 低字节是低位
    pub fn from_vec(v: &Vec<u8>) -> Nat {
        Nat::from_slice(v.as_slice())
    }

    nat_from_basic_type!(from_u8, u8, 0);
    nat_from_basic_type!(from_u16, u16, 0);
    nat_from_basic_type!(from_u32, u32, 1);
    nat_from_basic_type!(from_u64, u64, 1);
    nat_from_basic_type!(from_u128, u128, 1);
    nat_from_basic_type!(from_usize, usize, 1);
    
    /// 如果self>u64::max_value, 那么会截断前64位返回;  
    pub fn to_u64(&self) -> Option<u64> {
        if self.is_nan() {
            None
        } else {
            if self.nat.len() < 2 {
                Some((*self.nat.first().unwrap()) as u64)
            } else {
                let (f, s) = (self.nat[0] as u64, self.nat[1] as u64);
                Some((s << 32) | f)
            }
        }
    }

    /// 
    pub fn probably_prime(&self, n: usize) -> bool {
        let zero = Nat::from_u8(0);
        if self.is_nan() || (self == &zero) {
            return false;
        }

        const PRIME_BIT_MASK: u128 = 1<<2 | 1<<3 | 1<<5 | 1<<7 |
            1<<11 | 1<<13 | 1<<17 | 1<<19 | 1<<23 | 1<<29 | 1<<31 |
            1<<37 | 1<<41 | 1<<43 | 1<<47 | 1<<53 | 1<<59 | 1<<61 | 1<<67 |
            1<<71 | 1<<73 | 1<<79 | 1<<83 | 1<<89 | 1<<97 | 1<<101 |
            1<<103 | 1<<107 | 1<<109 | 1<< 113 | 1<<127;

        let x = self.nat[0] as u128;
        // 小素数直接判断
        if (self.nat.len() == 1) && (x < 128) {
            return ((1<<x) & PRIME_BIT_MASK) != 0;
        }
        
        // 偶数
        if x & 0x1 == 0 {
            return false;
        }

        const PRIMES_A: u32 = 3 * 5 * 7 * 11 * 13 * 17 * 19 * 23 * 37;
        const PRIMES_B: u32 = 29 * 31 * 41 * 43 * 47 * 53;
        let (ra, rb) = ((self % PRIMES_A).unwrap(), (self % PRIMES_B).unwrap());
        if ra%3 == 0 || ra%5 == 0 || ra%7 == 0 || ra%11 == 0 || ra%13 == 0 || ra%17 == 0 || ra%19 == 0 || ra%23 == 0 || ra%37 == 0 ||
            rb%29 == 0 || rb%31 == 0 || rb%41 == 0 || rb%43 == 0 || rb%47 == 0 || rb%53 == 0 {
            return false
        }

        self.prime_validate_by_miller_rabin(n+1) && self.prime_validate_by_lucas()
    }

    /// probablyPrimeLucas reports whether n passes the "almost extra strong" Lucas probable prime test,
    /// using Baillie-OEIS parameter selection. This corresponds to "AESLPSP" on Jacobsen's tables (link below).
    /// The combination of this test and a Miller-Rabin/Fermat test with base 2 gives a Baillie-PSW test.
    ///
    /// References:
    ///
    /// Baillie and Wagstaff, "Lucas Pseudoprimes", Mathematics of Computation 35(152),
    /// October 1980, pp. 1391-1417, especially page 1401.
    /// https://www.ams.org/journals/mcom/1980-35-152/S0025-5718-1980-0583518-6/S0025-5718-1980-0583518-6.pdf
    ///
    /// Grantham, "Frobenius Pseudoprimes", Mathematics of Computation 70(234),
    /// March 2000, pp. 873-891.
    /// https://www.ams.org/journals/mcom/2001-70-234/S0025-5718-00-01197-2/S0025-5718-00-01197-2.pdf
    ///
    /// Baillie, "Extra strong Lucas pseudoprimes", OEIS A217719, https://oeis.org/A217719.
    ///
    /// Jacobsen, "Pseudoprime Statistics, Tables, and Data", http://ntheory.org/pseudoprimes.html.
    ///
    /// Nicely, "The Baillie-PSW Primality Test", http://www.trnicely.net/misc/bpsw.html.
    /// (Note that Nicely's definition of the "extra strong" test gives the wrong Jacobi condition,
    /// as pointed out by Jacobsen.)
    ///
    /// Crandall and Pomerance, Prime Numbers: A Computational Perspective, 2nd ed.
    /// Springer, 2005.
    /// note: Miller-Rabin算法目前可以通过所有测试示例, 故lucas算法暂不实现
    fn prime_validate_by_lucas(&self) -> bool {
        // Baillie-OEIS "method C" for choosing D, P, Q,
        // as in https://oeis.org/A217719/a217719.txt:
        // try increasing P ≥ 3 such that D = P² - 4 (so Q = 1)
        // until Jacobi(D, n) = -1.
        // The search is expected to succeed for non-square n after just a few trials.
        // After more than expected failures, check whether n is square
        // (which would cause Jacobi(D, n) = 1 for all D not dividing n).
        true
    }
    
    // fn jacobi(&self, rhs: &Nat) -> isize {
    // }
    
    /// miller-rabin素数测试   
    /// 对于任意奇数n>2和正整数s, miller-rabin素数测试出错的概率至多为2^(-s)  
    /// 
    /// note: 内部调用函数, self是大于2的奇数, s>0  
    fn prime_validate_by_miller_rabin(&self, s: usize) -> bool {
        let mut rng = RngSource::new(*self.as_vec().first().unwrap() as i64);
        for _ in 0..s {
            let a = Nat::random(&mut rng, self);
            if a.miller_rabin_witness(self) {
                return false;
            }
        }
        
        true
    }
    
    /// 判断n是否是合数  
    fn miller_rabin_witness(&self, n: &Nat) -> bool {
        let n_m1 = n - 1u32;
        let t = n_m1.trailling_zeros();
        let u = &n_m1 >> t;
        
        let mut xi_m1 = self.pow_mod(&u, n);
        for _ in 1..=t {
            let xi = &(&xi_m1 * &xi_m1) % n;
            if xi == 1 && xi_m1 != 1 && xi_m1 != n_m1 {
                return true;
            }
            xi_m1 = xi;
        }

        xi_m1 != 1
    }
    
    /// 产生一个[0, limit)之间的随机数  
    fn random<Rng: Seed<i64> + Source<u64, i64>>(rand: &mut Rng, limit: &Nat) -> Nat {
        if limit.is_nan() || limit == &0u32 {
            return Nat::nan()
        }

        let bits_len = limit.bits_len();
        let (num, rem) = ((bits_len + 31) / 32, (bits_len % 32) as u32);
        let mut nat = Nat::from("");
        nat.as_vec_mut().resize(num, 0u32);
        let mask = if rem != 0 {
            (1u32 << rem) - 1
        } else {
            u32::max_value()
        };
        
        // let mut cnt = 0;
        loop {
        // while cnt < 10 {
            let mut itr = nat.as_vec_mut().iter_mut();
            while let Some(x) = itr.next() {
                let r: u64 = rand.rng();
                let (low, high) = ((r & (u32::max_value() as u64)) as u32, (r >> 32) as u32);
                *x = low;
                match itr.next() {
                    Some(y) => *y = high,
                    _ => {},
                };
            }
            
            *nat.as_vec_mut().last_mut().unwrap() &= mask;
            if &nat < limit {
                break;
            }
            // cnt += 1;
        }
        
        // if cnt == 10 {
        //     *nat.as_vec_mut().last_mut().unwrap() &= mask >> 1;
        // }
        
        while nat.as_vec().len() > 1 && *nat.as_vec().last().unwrap() == 0 {
            nat.as_vec_mut().pop();
        }
        
        nat
    }
    
    /// self^b mod n;  
    /// 如果n==0, 那份结果是self^b;  
    pub fn pow_mod(&self, b: &Nat, n: &Nat) -> Nat {
        if self.is_nan() || b.is_nan() || n.is_nan() {
            return Nat::nan();
        }
        
        let bits_len = b.bits_len();
        if n == &0u32 {
            self.pow(b)
        } else if n == &1u32 {
            Nat::from_u8(0)
        } else {
            // 反复平方法 
            let mut d = Nat::from_u8(1);
            for i in 0..bits_len {
                d = &(&d * &d) % n;
                
                if b.check_bit_is_one(bits_len - i - 1, bits_len) {
                    d = &(&d * self) % n;
                }
            }
            
            d
        }
    }
    
    pub fn pow(&self, b: &Nat) -> Nat {
        if self.is_nan() || b.is_nan() {
            return Nat::nan();
        }
        
        let bits_len = b.bits_len();
        if bits_len == 1 {
            if (self == &0u32) && (b == &0u32) {
                Nat::from_u8(1)
            } else {
                self.clone()
            }
        } else {
            let mut pre = self.clone();
            let mut cur = if b.check_bit_is_one(0, bits_len) {
                self.clone()
            } else { 
                Nat::from_u8(1)
            };
            
            for i in 1..bits_len {
                pre = &pre * &pre;
                if b.check_bit_is_one(i, bits_len) {
                    cur = &cur * &pre;
                }
            }
            
            cur
        }
    }
    
    /// 调用者bits_len是当前自然数的位长度  
    fn check_bit_is_one(&self, idx: usize, bits_len: usize) -> bool {
        if idx >= bits_len {
            false
        } else {
            let (num, rem) = (idx >> 5, (idx % 32) as u32);
            let ele = self.as_vec()[num];
            (ele & (1u32 << rem)) != 0
        }
    }
}

impl From<u8> for Nat {
    fn from(v: u8) -> Self {
        Nat {
            nat: vec![v as u32]
        }
    }
}

impl From<u16> for Nat {
    fn from(v: u16) -> Self {
        Nat {
            nat: vec![v as u32]
        }
    }
}

impl From<u32> for Nat {
    fn from(v: u32) -> Self {
        Nat {
            nat: vec![v]
        }
    }
}

impl From<u64> for Nat {
    fn from(v: u64) -> Self {
        let (fir, sec) = ((v & (u32::max_value() as u64)) as u32, (v >> 32) as u32);
        Nat {
            nat: vec![fir, sec],
        }
    }
}

impl From<usize> for Nat {
    fn from(v: usize) -> Self {
        let v = v.to_le_bytes();
        let mut itr = v.iter();
        let mut nat = Vec::new();
        
        let len = v.len() / 4;
        for _ in 0..len {
            let arr = [*itr.next().unwrap(), *itr.next().unwrap(), *itr.next().unwrap(), *itr.next().unwrap()];
            nat.push(u32::from_le_bytes(arr));
        }
        
        Nat {nat}
    }
}

impl From<u128> for Nat {
    fn from(v: u128) -> Self {
        let (v0, v1, v2, v3) = ((v & (u32::max_value() as u128)) as u32,
                                ((v>>32) & (u32::max_value() as u128)) as u32,
                                ((v>>64) & (u32::max_value() as u128)) as u32,
                                (v >> 96) as u32);
        Nat {
            nat: vec![v0, v1, v2, v3],
        }
    }
}

impl Default for Nat {
    fn default() -> Self {
        Self::nan()
    }
}

impl From<&str> for Nat {
    fn from(s: &str) -> Self {
        match Self::parse_base(s) {
            Some((base, x)) => Self::from_str(x, base),
            _ => Nat::nan()
        }
    }
}

impl<'a, 'b> Add<&'b Nat> for &'a Nat {
    type Output = Nat;
    fn add(self, rhs: &'b Nat) -> Self::Output {
        if self.is_nan() || rhs.is_nan() {
            return Nat::nan();
        }

        let (min, max) = self.min_max_by_num(rhs);
        let mut carry = 0u32;
        let mut min_itr = min.iter();
        let mut nat = Vec::with_capacity(max.len() + 1);
        for max_ele in max {
            match min_itr.next() {
                Some(&s) => {
                    let (x, cx) = max_ele.overflowing_add(carry);
                    let (y, cy) = x.overflowing_add(s);
                    nat.push(y);
                    carry = (cx as u32) + (cy as u32);
                }
                None => {
                    let (x, c) = max_ele.overflowing_add(carry);
                    nat.push(x);
                    carry = c as u32;
                }
            }
        }

        if carry > 0 {
            nat.push(carry);
        }

        Nat { nat }
    }
}

impl<'b> AddAssign<&'b Nat> for Nat {
    fn add_assign(&mut self, rhs: &'b Nat) {
        let result = &*self + rhs;

        let nat = self.as_vec_mut();
        nat.resize(result.num(), 0);
        nat.copy_from_slice(result.as_vec())
    }
}

/// note: x - y == |x-y| if x less than y
impl<'a, 'b> Sub<&'b Nat> for &'a Nat {
    type Output = Nat;

    fn sub(self, rhs: &'b Nat) -> Self::Output {
        if self.is_nan() || rhs.is_nan() {
            return Nat::nan();
        }
        let (min, max) = if self < rhs {
            (self.as_vec(), rhs.as_vec())
        } else {
            (rhs.as_vec(), self.as_vec())
        };

        let mut nat: Vec<u32> = Vec::new();
        let mut min_itr = min.iter();
        let mut carry = 0u32;
        for ele in max {
            match min_itr.next() {
                Some(&s) => {
                    let (x, c0) = ele.overflowing_sub(carry);
                    let (y, c1) = x.overflowing_sub(s);
                    nat.push(y);
                    carry = (c0 as u32) + (c1 as u32);
                }
                None => {
                    let (x, c) = ele.overflowing_sub(carry);
                    nat.push(x);
                    carry = c as u32;
                }
            };
        }

        Nat::trim_last_zeros(&mut nat, 0);
        Nat { nat }
    }
}

impl Sub<u32> for &Nat {
    type Output = Nat;

    fn sub(self, rhs: u32) -> Nat {
        if self.is_nan() {
            Nat::nan()
        } else if self.as_vec().len() == 1 {
            let x = *self.as_vec().first().unwrap();
            let y = if x > rhs {
                x - rhs
            } else {
                rhs - x
            };
            Nat::from_u32(y)
        } else {
            let mut nat = Nat::nan();
            
            let mut itr = self.as_vec().iter();
            let x = (*itr.next().unwrap()).overflowing_sub(rhs);
            nat.as_vec_mut().push(x.0);
            let mut cnt = x.1 as u32;
            
            for &ele in itr {
                let y = ele.overflowing_sub(cnt);
                cnt = y.1 as u32;
                nat.as_vec_mut().push(y.0);
            }
            
            while nat.as_vec().len() > 1 && *nat.as_vec().last().unwrap() == 0 {
                nat.as_vec_mut().pop();
            }
            
            nat
        }
    }
}

impl<'b> SubAssign<&'b Nat> for Nat {
    fn sub_assign(&mut self, rhs: &'b Nat) {
        let result = &*self - rhs;

        let nat = self.as_vec_mut();
        nat.resize(result.num(), 0);
        nat.copy_from_slice(result.as_vec());
    }
}

impl SubAssign<u32> for Nat {
    fn sub_assign(&mut self, rhs: u32) {
        let nat = &*self - rhs;
        drop(std::mem::replace(self, nat));
    }
}

impl<'a, 'b> Mul<&'b Nat> for &'a Nat {
    type Output = Nat;

    fn mul(self, rhs: &'b Nat) -> Self::Output {
        if self.is_nan() || rhs.is_nan() {
            return Nat::nan();
        } else if self == &0 || rhs == &0 {
            return Nat::from_u8(0);
        }

        self.mul_manual(rhs)
    }
}

impl<'b> MulAssign<&'b Nat> for Nat {
    fn mul_assign(&mut self, rhs: &'b Nat) {
        let mut result = &*self * rhs;
        let nat = self.as_vec_mut();
        nat.clear();
        nat.append(result.as_vec_mut());
    }
}

impl<'a, 'b> Div<&'b Nat> for &'a Nat {
    type Output = Nat;

    fn div(self, rhs: &'b Nat) -> Self::Output {
        if self.is_nan() || rhs.is_nan() || (rhs == &Nat::from_u8(0)) {
            return Nat::nan();
        } else if self < rhs {
            return Nat::from_u8(0u8);
        }

        let num_len = self.bits_len();
        let den_len = rhs.bits_len();

        if num_len == den_len {
            return Nat { nat: vec![1] };
        }

        // 手工除, 最多需要轮询self.bits_len() - rhs.bits_len()次,
        // 每次需要2次加法运算, 及最少2次左移(左移多一次则轮询就少一次)
        let mut nat = Nat { nat: vec![0] };
        let one = Nat { nat: vec![1] };
        let mut self_copy = self.clone();
        loop {
            if self_copy >= *rhs {
                let mut shift = self_copy.bits_len() - den_len;
                let mut den = rhs << shift;

                while den > self_copy {
                    den >>= 1;
                    shift -= 1;
                }

                self_copy -= &den;
                nat += &(&one << shift);
            } else {
                break;
            }
        }

        nat
    }
}

impl<'b> DivAssign<&'b Nat> for Nat {
    fn div_assign(&mut self, rhs: &'b Nat) {
        let mut result = &*self / rhs;
        let nat = self.as_vec_mut();
        nat.clear();
        nat.append(result.as_vec_mut());
    }
}

impl<'a, 'b> Rem<&'b Nat> for &'a Nat {
    type Output = Nat;
    fn rem(self, rhs: &'b Nat) -> Self::Output {
        if self.is_nan() || rhs.is_nan() || (rhs == &Nat::from_u8(0)) {
            return Nat::nan();
        } else if self < rhs {
            return self.clone();
        }

        // let num_len = self.bits_len();
        let den_len = rhs.bits_len();
        // if num_len == den_len {
        //     return self - rhs;
        // }
        //
        let mut self_copy = self.clone();
        loop {
            if self_copy < *rhs {
                break;
            } else {
                let shift = self_copy.bits_len() - den_len;
                let mut den = rhs << shift;
                if den > self_copy {
                    den >>= 1;
                }
                self_copy -= &den;
            }
        }

        self_copy
    }
}

impl Rem<u32> for &Nat {
    type Output = Option<u32>;
    fn rem(self, rhs: u32) -> Self::Output {
        if self.is_nan() || rhs == 0 {
            return None;
        }
        
        let (mut r, rhs) = (0, rhs as u64);
        for &ele in self.as_vec().iter().rev() {
            let m = ((r as u64) << 32) + (ele as u64);
            if m < rhs {
                r = m;
            } else {
                r = m % rhs;
            }
        }
        
        Some(r as u32)
    }
}

impl<'b> RemAssign<&'b Nat> for Nat {
    fn rem_assign(&mut self, rhs: &'b Nat) {
        let mut result = &*self % rhs;
        let nat = self.as_vec_mut();
        nat.clear();
        nat.append(result.as_vec_mut());
    }
}

impl RemAssign<u32> for Nat {
    fn rem_assign(&mut self, rhs: u32) {
        let res = &*self % rhs;
        match res {
            Some(x) => {
                self.as_vec_mut().clear();
                self.as_vec_mut().push(x);
            },
            None => {
                self.as_vec_mut().clear();
            },
        }
    }
}

/// x & y, 当x和y位长度不一致时, 位长度较小的高位补0以使长度对齐参加与运算
impl<'a, 'b> BitAnd<&'b Nat> for &'a Nat {
    type Output = Nat;

    fn bitand(self, rhs: &'b Nat) -> Self::Output {
        if self.is_nan() || rhs.is_nan() {
            return Nat::nan();
        }

        let (min, max) = self.min_max_by_num(rhs);
        let mut nat: Vec<u32> = Vec::with_capacity(min.len());
        let mut max_itr = max.iter();

        for x in min {
            match max_itr.next() {
                Some(&y) => {
                    nat.push(x & y);
                }
                _ => {}
            }
        }
        Nat::trim_last_zeros(&mut nat, 0);

        Nat { nat }
    }
}

impl BitAnd<u32> for &Nat {
    type Output = Option<u32>;
    fn bitand(self, rhs: u32) -> Self::Output {
        if self.is_nan() {
            None
        } else {
            let first = self.as_vec()[0];
            Some(first & rhs)
        }
    }
}

impl BitAndAssign<u32> for Nat {
    fn bitand_assign(&mut self, rhs: u32) {
        match &*self & rhs {
            Some(x) => {
                self.as_vec_mut().clear();
                self.as_vec_mut().push(x);
            },
            None => {
                self.as_vec_mut().clear();
            }
        }
    }
}

impl BitAnd for Nat {
    type Output = Nat;

    fn bitand(self, rhs: Self) -> Self::Output {
        &self & &rhs
    }
}

impl<'b> BitAndAssign<&'b Nat> for Nat {
    fn bitand_assign(&mut self, rhs: &'b Nat) {
        let mut result = &*self & rhs;
        let nat = self.as_vec_mut();
        nat.clear();
        nat.append(result.as_vec_mut());
    }
}

/// x | y, 当x和y位长度不一致时, 位长度较小的高位补0以使长度对齐参加或运算
impl<'a, 'b> BitOr<&'b Nat> for &'a Nat {
    type Output = Nat;
    fn bitor(self, rhs: &'b Nat) -> Self::Output {
        if self.is_nan() || rhs.is_nan() {
            return Nat::nan();
        }

        let (min, max) = self.min_max_by_num(rhs);
        let mut min_itr = min.iter();
        let mut nat: Vec<u32> = Vec::with_capacity(max.len());

        for &x in max {
            match min_itr.next() {
                Some(&y) => {
                    nat.push(x | y);
                }
                None => {
                    nat.push(x);
                }
            }
        }
        Nat::trim_last_zeros(&mut nat, 0);

        Nat { nat }
    }
}

impl<'b> BitOrAssign<&'b Nat> for Nat {
    fn bitor_assign(&mut self, rhs: &'b Nat) {
        let mut result = &*self | rhs;
        let nat = self.as_vec_mut();
        nat.clear();
        nat.append(result.as_vec_mut());
    }
}

/// note: x ^ y, 位长度较小的高位补0对齐参加异或运算
impl<'a, 'b> BitXor<&'b Nat> for &'a Nat {
    type Output = Nat;

    fn bitxor(self, rhs: &'b Nat) -> Self::Output {
        if self.is_nan() || rhs.is_nan() {
            return Nat::nan();
        }

        let (min, max) = self.min_max_by_num(rhs);
        let mut nat = Vec::with_capacity(max.len());
        let mut min_itr = min.iter();

        for &x in max {
            match min_itr.next() {
                Some(&y) => {
                    nat.push(x ^ y);
                }
                None => {
                    nat.push(x);
                }
            }
        }

        Nat::trim_last_zeros(&mut nat, 0);

        Nat { nat }
    }
}

impl<'b> BitXorAssign<&'b Nat> for Nat {
    fn bitxor_assign(&mut self, rhs: &'b Nat) {
        let mut result = &*self ^ rhs;
        let nat = self.as_vec_mut();
        nat.clear();
        nat.append(result.as_vec_mut());
    }
}

impl Not for &Nat {
    type Output = Nat;

    fn not(self) -> Self::Output {
        if self.is_nan() {
            return Nat::nan();
        }

        let mut nat: Vec<u32> = Vec::with_capacity(self.num());
        let arr = self.as_vec();
        let len = arr.len() - 1;
        let arr_1 = &arr[0..len];
        for &ele in arr_1 {
            let val = !ele;
            nat.push(val);
        }

        let &last = arr.last().unwrap();
        let rem = (self.bits_len() - (len << 5)) as u32;
        let val = (!last) & (0xffffffff >> (32 - rem));
        nat.push(val);

        Nat::trim_last_zeros(&mut nat, 0);

        Nat { nat }
    }
}

impl Shr<usize> for &Nat {
    type Output = Nat;

    fn shr(self, rhs: usize) -> Self::Output {
        if self.is_nan() {
            return Nat::nan();
        }

        let (num, rom) = (rhs >> 5, rhs % 32);
        if self.num() <= num {
            Nat::from_u8(0)
        } else {
            let mut nat = Vec::new();

            let tmp = &self.as_vec()[num..];
            if rom != 0 {
                nat.reserve(self.num() - num);
                let (rom_comp, rom) = ((32 - rom) as u32, rom as u32);
                let mut itr = tmp.iter();
                let mut pre = *itr.next().unwrap();
                for &ele in itr {
                    let val = (pre >> rom) | (ele << rom_comp);
                    pre = ele;
                    nat.push(val);
                }
                
                let pre = pre >> rom;
                if pre > 0 {
                    nat.push(pre);
                }
            } else {
                nat.resize(self.num() - num, 0);
                nat.copy_from_slice(tmp);
            }
            
            Nat {nat}
        }
    }
}

impl ShrAssign<usize> for Nat {
    fn shr_assign(&mut self, rhs: usize) {
        let result = &*self >> rhs;
        let nat = self.as_vec_mut();
        nat.resize(result.num(), 0);
        nat.copy_from_slice(result.as_vec());
    }
}

impl Shl<usize> for &Nat {
    type Output = Nat;

    fn shl(self, rhs: usize) -> Self::Output {
        if self.is_nan() {
            return Nat::nan();
        }

        let num = rhs >> 5;
        let rom = rhs % 32usize;
        let mut nat: Vec<u32> = Vec::with_capacity(self.num() + num + 1);

        if rom != 0 {
            nat.resize(num, 0u32);
            let itr = self.as_vec().iter();
            let mut pre = 0u32;
            let rom = rom as u32;
            let rom_comp = 32 - rom;
            for &ele in itr {
                let val = (ele << rom) | pre;
                pre = ele >> rom_comp;
                nat.push(val);
            }

            if pre > 0 {
                nat.push(pre);
            }
        } else {
            nat.resize(num + self.num(), 0);
            let tmp = &mut nat.as_mut_slice()[num..];
            tmp.copy_from_slice(self.as_vec().as_slice());
        }

        Nat { nat }
    }
}

impl ShlAssign<usize> for Nat {
    fn shl_assign(&mut self, rhs: usize) {
        let result = &*self << rhs;
        let nat = self.as_vec_mut();
        nat.resize(result.num(), 0);
        nat.copy_from_slice(result.as_vec());
    }
}

impl PartialEq for Nat {
    fn eq(&self, rhs: &Self) -> bool {
        if self.is_nan() || rhs.is_nan() {
            false
        } else {
            self.as_vec() == rhs.as_vec()
        }
    }
}

impl PartialEq<u32> for Nat {
    fn eq(&self, rhs: &u32) -> bool {
        if self.is_nan() {
            false
        } else {
            if self.as_vec().len() == 1 {
                self.as_vec().first().unwrap() == rhs
            } else {
                false
            }
        }
    }
}

impl PartialOrd for Nat {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        if self.is_nan() || rhs.is_nan() {
            None
        } else {
            let (lhs_lens, rhs_lens) = (self.num(), rhs.num());
            
            if lhs_lens > rhs_lens {
                Some(Ordering::Greater)
            } else if lhs_lens < rhs_lens {
                Some(Ordering::Less)
            } else {
                let mut itr = rhs.as_vec().iter().rev();
                for &min in self.as_vec().iter().rev() {
                    match itr.next() {
                        Some(&x) => {
                            if min > x {
                                return Some(Ordering::Greater);
                            } else if min < x {
                                return Some(Ordering::Less);
                            }
                        },
                        _ => {
                            return None;
                        },
                    };
                }

                Some(Ordering::Equal)
            }
        }
    }
}

impl PartialOrd<u32> for Nat {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        if self.is_nan() {
            None
        } else if self.as_vec().len() == 1 {
            let first = self.as_vec().first().unwrap();
            if first < other {
                Some(Ordering::Less)
            } else if first > other {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Equal)
            }
        } else {
            Some(Ordering::Greater)
        }
    }
}

macro_rules! nat_fmt_impl_macro {
    ($trait_name: ident, $fmt_str: literal) => {
        impl $trait_name for Nat {
            fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                if self.is_nan() {
                    return write!(f, "{}", "NaN");
                }

                let nat = self.as_vec();
                let mut nat_str = Vec::new();

                for ele in nat {
                    let s = format!($fmt_str, ele);
                    nat_str.push(s);
                }

                let mut last = nat_str.pop().unwrap().as_bytes().to_vec();
                last.reverse();
                Nat::trim_last_zeros(&mut last, b'0');
                last.reverse();
                let s = String::from_utf8(last).unwrap();
                nat_str.push(s);

                nat_str.reverse();
                let s = nat_str.as_slice().join("");
                write!(f, "{}", s)
            }
        }
    };
}

nat_fmt_impl_macro!(Binary, "{:032b}");
nat_fmt_impl_macro!(LowerHex, "{:08x}");
nat_fmt_impl_macro!(Debug, "{:08x}");
nat_fmt_impl_macro!(UpperHex, "{:08X}");

impl Octal for Nat {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if self.is_nan() {
            return write!(f, "{}", "NaN");
        }

        let nat = self.as_vec().as_slice();
        let mut nat_str = Vec::with_capacity(self.num() * 11);

        let mut pre = 0u32;
        for (i, ele) in nat.iter().enumerate() {
            pre = match i % 3 {
                0 => {
                    for idx in 0..10u32 {
                        let val = (ele >> (idx * 3)) & 0x7u32;
                        let s = format!("{:o}", val);
                        nat_str.push(s);
                    }
                    ele >> 30
                }
                1 => {
                    let val = ((ele & 0x1) << 2) | pre;
                    let s = format!("{:o}", val);
                    nat_str.push(s);
                    let ele = ele >> 1;
                    for idx in 0..10u32 {
                        let val = (ele >> (idx * 3)) & 0x7u32;
                        let s = format!("{:o}", val);
                        nat_str.push(s);
                    }
                    ele >> 30
                }
                _ => {
                    let val = ((ele & 0x3) << 1) | pre;
                    let s = format!("{:o}", val);
                    nat_str.push(s);
                    let ele = ele >> 2;
                    for idx in 0..10u32 {
                        let val = (ele >> (idx * 3)) & 0x7u32;
                        let s = format!("{:o}", val);
                        nat_str.push(s);
                    }
                    0
                }
            };
        }

        if pre > 0 {
            let s = format!("{:o}", pre);
            nat_str.push(s);
        }

        Nat::trim_last_zeros(&mut nat_str, String::from("0"));

        nat_str.reverse();
        let s = nat_str.as_slice().join("");
        write!(f, "{}", s)
    }
}

impl Display for Nat {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if self.is_nan() {
            return write!(f, "{}", "NaN");
        } else if *self == Nat::from_u8(0) {
            return write!(f, "{}", 0);
        }

        let mut nat = self.clone();
        let mut nat_str = Vec::with_capacity(nat.bits_len() / DEC_BASIC_LEN);
        let ten = Nat::from_u8(10);
        while nat != Nat::from_u8(0) {
            let rem = &nat % &ten;
            nat /= &ten;
            let val = rem.as_vec().last().unwrap();
            nat_str.push(format!("{}", val));
        }
        nat_str.reverse();

        write!(f, "{}", nat_str.as_slice().join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    macro_rules! test_nat_equal_tgt {
        ($tgt: ident, ($fuc_name:ident, $basic_type: ty)) => {
            let id = format!("({},{})", stringify!($fuc_name), stringify!($basic_type));
            let src = Nat::$fuc_name(std::u8::MAX as $basic_type);
            assert_eq!($tgt, src, "{}: {}({:x})", id, stringify!($fuc_name), std::u8::MAX);
            assert_ne!(src, Nat::nan(), "{}: {:x}!={:x}", id, src, Nat::nan());

            let n1 = Nat::$fuc_name(<$basic_type>::max_value());

            let s = format!("{:x}", <$basic_type>::max_value());
            assert_eq!(format!("{:x}", n1), s, "{}: format{{:x}}", id);
            assert_eq!(n1, Nat::from_str(s.as_str(), 16), "{}: from_str(\"{}\", {})", id, s, 16);

            let s = format!("{}", <$basic_type>::max_value());
            assert_eq!(format!("{}", n1), s, "{}: format{{}}", id);
            assert_eq!(n1, Nat::from_str(s.as_str(), 10), "{}: from_str(\"{}\", {})", id, s, 10);

            let s = format!("{:o}", <$basic_type>::max_value());
            assert_eq!(format!("{:o}", n1), s, "{}: format{{:o}}", id);
            assert_eq!(n1, Nat::from_str(s.as_str(), 8), "{}: from_str(\"{}\", {})", id, s, 8);

            let s = format!("{:b}", <$basic_type>::max_value());
            assert_eq!(format!("{:b}", n1), s, "{}: format{{:b}}", id);
            assert_eq!(n1, Nat::from_str(s.as_str(), 2), "{}: from_str(\"{}\", {})", id, s, 2);
        };
        ($tgt: ident, ($fuc_name: ident, $basic_type: ty)$(, ($fuc_name_l: ident, $basic_type_l: ty))+) => {
            test_nat_equal_tgt!($tgt, ($fuc_name, $basic_type));
            test_nat_equal_tgt!($tgt$(, ($fuc_name_l, $basic_type_l))+);
        };
    }

    #[test]
    fn test_nat_from_and_fmt() {
        let n1 = Nat::from_u8(std::u8::MAX);
        test_nat_equal_tgt!(
            n1,
            (from_u8, u8),
            (from_u16, u16),
            (from_u32, u32),
            (from_u64, u64),
            (from_u128, u128)
        );
    }

    #[test]
    fn test_nat_relation_arith() {
        let l1 = Nat::from_u128(std::u128::MAX);
        let l2 = Nat::from_u128(std::u128::MAX);
        let l_sum = Nat::from_str("1fffffffffffffffffffffffffffffffe", 16);
        let s1 = Nat::from_u8(std::u8::MAX);
        let s2 = Nat::from_u8(std::u8::MAX);
        let s_sum = Nat::from_str("1fe", 16);
        let nan = Nat::nan();
        assert!(l1 == l2);
        assert!(l1 <= l2);
        assert!(l1 <= l_sum);
        assert!(l2 < l_sum);
        assert!(s_sum > s1);
        assert!(s_sum >= s2);
        assert!(nan != nan);
        assert!(l1 != nan);
        assert!(nan != l1);
        assert_eq!(Nat::from_u8(0), Nat::from_u128(0));
    }

    #[test]
    fn test_nat_logical_arith() {
        let l1 = Nat::from_u128(std::u128::MAX);
        let l2 = Nat::from_u128(std::u128::MAX);

        assert_eq!(&l1 & &l2, l1);
        assert_eq!(&l1 | &l2, l2);
        assert_eq!(&l1 ^ &l2, Nat::from_u8(0));
        assert_eq!(!&l1, Nat::from_u128(0));
        assert_eq!(format!("{}", &l1 & &Nat::nan()), format!("{}", Nat::nan()));

        let l1 = Nat::from_str("fffffffffffffffffffffffffffffffffff3222222222222222222234900000000000000ffffffffffffffffffffff", 16);
        let l2 = Nat::from_str("ff9000000000000000000000322222222222223209053065839583093285340530493058304593058390584", 16);
        assert_eq!(&l1 ^ &l2, Nat::from_str("fffffff006fffffffffffffffffffffcddd1000000000102b271247b7058309328534053fb6cfa7cfba6cfa7c6fa7b", 16));
        assert_eq!(&l1 | &l2, Nat::from_str("fffffffffffffffffffffffffffffffffff3222222222322b273267b7958309328534053ffffffffffffffffffffff", 16));
        assert_eq!(&l1 & &l2, Nat::from_str("ff9000000000000000000000322222222222222200002020009000000000000000493058304593058390584", 16));
        assert_eq!(!&l2, Nat::from_str("6fffffffffffffffffffffcdddddddddddddcdf6facf9a7c6a7cf6cd7acbfacfb6cfa7cfba6cfa7c6fa7b", 16));
        assert_eq!(!&Nat::from_str("11000011", 2), Nat::from_str("111100", 2));
    }

    #[test]
    fn test_nat_shift_arith() {
        let l2 = Nat::from_str("ff9000000000000000000000322222222222223209053065839583093285340530493058304593058390584", 16);
        let l3 = Nat::from_str("1ff20000000000000000000006444444444444464120a60cb072b0612650a680a609260b0608b260b0720b08", 16);
        assert_eq!(&l2 << 1, l3);
        assert_eq!(&l2 << 0, l2);
        assert_eq!(&l2 << 30, Nat::from_str("3fe4000000000000000000000c8888888888888c82414c1960e560c24ca14d014c124c160c1164c160e416100000000", 16));
        assert_eq!(&l2 << 10000, Nat::from_str("ff90000000000000000000003222222222222232090530658395830932853405304930583045930583905840000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", 16));
        assert_eq!(&l2 >> 4, Nat::from_str("ff900000000000000000000032222222222222320905306583958309328534053049305830459305839058", 16));
        assert_eq!(&l2 >> 1, Nat::from_str("7fc800000000000000000000191111111111111904829832c1cac18499429a029824982c1822c982c1c82c2", 16));
        assert_eq!(&l2 >> 0, l2);
        assert_eq!(&l2 >> 1001, Nat::from_u8(0));
        assert_eq!(&Nat::from_u8(0) << 0, Nat::from_u8(0));
        assert_eq!(&Nat::from_u8(0) << 3, Nat::from_u8(0));
    }

    #[test]
    fn test_nat_add() {
        let mut l1 = Nat::from_u128(std::u128::MAX);
        let l2 = Nat::from_u128(std::u128::MAX);
        let sum = Nat::from_str("1fffffffffffffffffffffffffffffffe", 16);
        assert_eq!(&l1 + &l2, sum);
        l1 += &l2;
        assert_eq!(l1, sum);
        assert_eq!(
            &l1 + &Nat::from_u8(1),
            Nat::from_str("1ffffffffffffffffffffffffffffffff", 16)
        );
        let l1 = Nat::from_str("fffffffffffffffffffffffffffffffffff3222222222222222222234900000000000000ffffffffffffffffffffff", 16);
        let l2 = Nat::from_str("ff9000000000000000000000322222222222223209053065839583093285340530493058304593058390584", 16);
        let sum = Nat::from_str("10000000ff900000000000000000000032215444444444542b275287b82583093285340540493058304593058390583", 16);
        assert_eq!(&l1 + &l2, sum, "{}=====>{}======{}", l1, l2, sum);

        let s1 = Nat::from_u8(std::u8::MAX);
        let s2 = Nat::from_u8(std::u8::MAX);
        let sum = Nat::from_str("1fe", 16);
        assert_eq!(&s1 + &s2, sum);

        let nan = Nat::nan();
        assert_eq!(format!("{:x}", &nan + &l1), format!("{:x}", nan));
    }

    #[test]
    fn test_nat_sub() {
        let l1 = Nat::from_u128(std::u128::MAX);
        let l2 = Nat::from_u8(std::u8::MAX);
        assert_eq!(&l1 - &l1, Nat::from_u8(0));
        assert_eq!(&l1 - 255u32, &l1 - &l2);
        assert_eq!(
            &l1 - &l2,
            Nat::from_u128(std::u128::MAX - (std::u8::MAX as u128))
        );
        assert_eq!(&l2 - &l1, &l1 - &l2);
        let l1 = Nat::from_str("fffffffffffffffffffffffffffffffffff3222222222222222222234900000000000000ffffffffffffffffffffff", 16);
        let l2 = Nat::from_str("32888f300000000000000322222229750348593045830670204", 16);
        let sub = Nat::from_str("fffffffffffffffffffffffffffffffffff32222221ef9992f22222348ffffffcdddddde68afcb7a6cfba7cf98fdfb", 16);
        assert_eq!(&l1 - &l2, sub);
        assert_eq!(&l2 - &l1, sub);
        let l1 = Nat::from_str("32f3289577420805237534573", 16);
        let l2 = Nat::from_u32(u32::max_value());
        assert_eq!(&l1 - &l2, &l1 - u32::max_value());
    }

    #[test]
    fn test_nat_mul() {
        let l1 = Nat::from_u8(10);
        assert_eq!(&l1 * &l1, Nat::from_u8(100));
        assert_eq!(&l1 * &Nat::from_u8(0), Nat::from_u8(0));
        assert_eq!(&l1 * &Nat::from_u8(1), l1);
        let l1 = Nat::from_str("f9058301048250fabddeabf9320480284932084552541", 16);
        let l2 = Nat::from_str("f329053910428502fabcd9230494035242429890eacb", 16);
        let m = Nat::from_str("ec882250900ba90c2088a4a5ee549ecc5152d7a50683a82daa24e03f6d6409468abf1ce1f01d9be845021f48b", 16);
        assert_eq!(&l1 * &l2, m);
        let left = Nat::from_u8(2);
        let right = Nat::from_u8(125);
        assert_eq!(left.pow(&right), Nat::from_u128(1<<125));
    }

    #[test]
    fn test_nat_div() {
        let l1 = Nat::from_u8(100);
        let l2 = Nat::from_u8(10);
        assert_eq!(&l1 / &l2, Nat::from_u8(10));
        let l1 = Nat::from_str("fffffffffff32908329058205820", 16);
        let l2 = Nat::from_str("ff", 16);
        let quo = Nat::from_str("10101010100f41d2557e84060b8", 16);
        assert_eq!(&l1 / &l2, quo);
        assert_eq!(&l2 / &l1, Nat::from_u8(0));
        let l1 = Nat::from_str("39025820857032850384502853503850325fa3242de121", 16);
        let l2 = Nat::from_str("2048537058358afedead392582075275", 16);
        let quo = Nat::from_str("1c414f70ec1f027", 16);
        assert_eq!(&l1 / &l2, quo);
        let l1 = Nat::from_u128(0x1ad7f29abca);
        assert_eq!(&l1 / &Nat::from_u8(10), Nat::from_u128(184467440737));
    }

    #[test]
    fn test_nat_rem() {
        let l1 = Nat::from_str("ffffffffffffff000000000000", 16);
        let l2 = Nat::from_u8(255);
        assert_eq!(&l1 % &l2, Nat::from_u8(0));
        let l1 = Nat::from_str("39025820857032850384502853503850325fa3242de121", 16);
        let l2 = Nat::from_str("2048537058358afedead392582075275", 16);
        let rem = Nat::from_str("ab9de6183b632a33dc2601ae78da14e", 16);
        assert_eq!(&l1 % &l2, rem);
        let l1 = Nat::from_str("fffffffffff32908329058205820", 16);
        let l2 = Nat::from_str("ff", 16);
        let quo = Nat::from_str("d8", 16);
        assert_eq!(&l1 % &l2, quo);
        assert_eq!(&l1 % 255u32, Some(0xd8u32));
    }
    
    #[test]
    fn pow_mod() {
        let cases = [
            ("0", "0", "0", "1"),
            ("0", "0", "1", "0"),
            ("1", "1", "1", "0"),
            ("2", "1", "1", "0"),
            ("2", "2", "1", "0"),
            ("10", "100000000000", "1", "0"),
            ("0x8000000000000000", "2", "0", "0x40000000000000000000000000000000"),
            ("0x8000000000000000", "2", "6719", "4944"),
            ("0x8000000000000000", "3", "6719", "5447"),
            ("0x8000000000000000", "1000", "6719", "1603"),
            ("0x8000000000000000", "1000000", "6719", "3199"),
            (
                "2938462938472983472983659726349017249287491026512746239764525612965293865296239471239874193284792387498274256129746192347",
                "298472983472983471903246121093472394872319615612417471234712061",
                "29834729834729834729347290846729561262544958723956495615629569234729836259263598127342374289365912465901365498236492183464",
                "23537740700184054162508175125554701713153216681790245129157191391322321508055833908509185839069455749219131480588829346291",
            ),
            (
                "11521922904531591643048817447554701904414021819823889996244743037378330903763518501116638828335352811871131385129455853417360623007349090150042001944696604737499160174391019030572483602867266711107136838523916077674888297896995042968746762200926853379",
                "426343618817810911523",
                "444747819283133684179",
                "42",
            ),
        ];
        
        for ele in cases.iter() {
            let (a,b,n,res) = (Nat::from(ele.0), Nat::from(ele.1), Nat::from(ele.2), Nat::from(ele.3));
            assert_eq!(a.pow_mod(&b, &n), res, "cases=>{}", ele.0);
        }
    }
    
    #[test]
    fn prime_validate() {
        let cases = [
            "2",
            "3",
            "5",
            "7",
            "11",
            "13756265695458089029",
            "13496181268022124907",
            "10953742525620032441",
            "17908251027575790097",
            
            // https://golang.org/issue/638
            "18699199384836356663",

            "98920366548084643601728869055592650835572950932266967461790948584315647051443",
            "94560208308847015747498523884063394671606671904944666360068158221458669711639",

            // https://primes.utm.edu/lists/small/small3.html
            "449417999055441493994709297093108513015373787049558499205492347871729927573118262811508386655998299074566974373711472560655026288668094291699357843464363003144674940345912431129144354948751003607115263071543163",
            "230975859993204150666423538988557839555560243929065415434980904258310530753006723857139742334640122533598517597674807096648905501653461687601339782814316124971547968912893214002992086353183070342498989426570593",
            "5521712099665906221540423207019333379125265462121169655563495403888449493493629943498064604536961775110765377745550377067893607246020694972959780839151452457728855382113555867743022746090187341871655890805971735385789993",
            "203956878356401977405765866929034577280193993314348263094772646453283062722701277632936616063144088173312372882677123879538709400158306567338328279154499698366071906766440037074217117805690872792848149112022286332144876183376326512083574821647933992961249917319836219304274280243803104015000563790123",

            // ECC primes: https://tools.ietf.org/html/draft-ladd-safecurves-02
            "3618502788666131106986593281521497120414687020801267626233049500247285301239",                                                                                  // Curve1174: 2^251-9
            "57896044618658097711785492504343953926634992332820282019728792003956564819949",                                                                                 // Curve25519: 2^255-19
            "9850501549098619803069760025035903451269934817616361666987073351061430442874302652853566563721228910201656997576599",                                           // E-382: 2^382-105
            "42307582002575910332922579714097346549017899709713998034217522897561970639123926132812109468141778230245837569601494931472367",                                 // Curve41417: 2^414-17
            "6864797660130609714981900799081393217269435300143305409394463459185543183397656052122559640661454554977296311391480858037121987999716643812574028291115057151", // E-521: 2^521-1
        ];
        
        let composites = [
            "0",
            "1",
            "21284175091214687912771199898307297748211672914763848041968395774954376176754",
            "6084766654921918907427900243509372380954290099172559290432744450051395395951",
            "84594350493221918389213352992032324280367711247940675652888030554255915464401",
            "82793403787388584738507275144194252681",

            // Arnault, "Rabin-Miller Primality Test: Composite Numbers Which Pass It",
            // Mathematics of Computation, 64(209) (January 1995), pp. 335-361.
            // strong pseudoprime to prime bases 2 through 29
            "1195068768795265792518361315725116351898245581",
            // strong pseudoprime to all prime bases up to 200
            "8038374574536394912570796143419421081388376882875581458374889175222974273765333652186502336163960045457915042023603208766569966760987284043965408232928738791850869166857328267761771029389697739470167082304286871099974399765441448453411558724506334092790222752962294149842306881685404326457534018329786111298960644845216191652872597534901",
            
            // Extra-strong Lucas pseudoprimes. https://oeis.org/A217719
            "989",
            "3239",
            "5777",
            "10877",
            "27971",
            "29681",
            "30739",
            "31631",
            "39059",
            "72389",
            "73919",
            "75077",
            "100127",
            "113573",
            "125249",
            "137549",
            "137801",
            "153931",
            "155819",
            "161027",
            "162133",
            "189419",
            "218321",
            "231703",
            "249331",
            "370229",
            "429479",
            "430127",
            "459191",
            "473891",
            "480689",
            "600059",
            "621781",
            "632249",
            "635627",

            "3673744903",
            "3281593591",
            "2385076987",
            "2738053141",
            "2009621503",
            "1502682721",
            "255866131",
            "117987841",
            "587861",

            "6368689",
            "8725753",
            "80579735209",
            "105919633",

        ];
        
        let s = 20usize;
        
        let his0 = SystemTime::now();
        for &ele in cases.iter() {
            let nat = Nat::from(ele);
            // println!("{}, {}", nat, nat.probably_prime(s));
            let his = SystemTime::now();
            assert!(nat.probably_prime(s), "case=>{}", ele);
            println!("time: {:?}, case=>{}", SystemTime::now().duration_since(his), ele);
        }
        
        for &ele in composites.iter() {
            let nat = Nat::from(ele);
            let his = SystemTime::now();
            assert!(!nat.probably_prime(s), "case=>{}", ele);
            println!("time: {:?}, case=>{}", SystemTime::now().duration_since(his), ele);
        }
        println!("total time: {:?}", SystemTime::now().duration_since(his0));
    }
}

