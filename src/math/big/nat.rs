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
        let mut cnt = 0;
        for ele in nat.iter().rev() {
            if *ele == zero {
                cnt += 1;
            } else {
                break;
            }
        }
        if cnt == nat.len() {
            nat.truncate(1);
        } else {
            nat.truncate(nat.len() - cnt);
        }
    }

    #[inline]
    fn nan() -> Nat {
        Nat { nat: Vec::new() }
    }

    /// 右移超过val的位数时, 返回值为0
    #[inline]
    fn shr_as_c<T>(val: T, bits: T) -> T
    where
        T: Shr<Output = T> + From<u32> + PartialOrd + Copy,
    {
        let len: T = T::from((std::mem::size_of::<T>() << 3) as u32);

        if bits < len {
            val >> bits
        } else {
            T::from(0u32)
        }
    }

    /// 左移超过val的位数时, 返回值为0
    #[inline]
    fn shl_as_c<T>(val: T, bits: T) -> T
    where
        T: Shl<Output = T> + From<u32> + PartialOrd + Copy,
    {
        let len = T::from((std::mem::size_of::<T>() << 3) as u32);
        if bits < len {
            val << bits
        } else {
            T::from(0u32)
        }
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
            let mut round_nat = Nat {
                nat: Vec::with_capacity(min.len() + max.len()),
            };
            let round = round_nat.as_vec_mut();
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
            nat += &round_nat;
        }

        let mut cnt = 0usize;
        for ele in nat.as_vec().iter().rev() {
            if *ele == 0 {
                cnt += 1;
            } else {
                break;
            }
        }

        let len = nat.as_vec_mut().len();
        if cnt == len {
            nat.as_vec_mut().truncate(1);
        } else {
            nat.as_vec_mut().truncate(len - cnt);
        }

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
        if self == &Nat::from_u8(0) {
            return 1;
        }

        let num = self.num();
        let mut cnt = 0usize;
        let &last = self.as_vec().last().unwrap();
        for i in 0..32u32 {
            if (last & (1 << (31 - i))) == 0 {
                cnt += 1;
            } else {
                break;
            }
        }

        ((num - 1) << 5) + (32 - cnt)
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
            true => {
                let num = v.len() >> 2;
                let num_arr = &v[0..(num << 2)];
                let rem_arr = &v[(num << 2)..];
                let mut num_arr_itr = num_arr.iter();

                let mut nat = Vec::with_capacity(num + 1);
                for _ in 0..num {
                    let b0 = *num_arr_itr.next().unwrap() as u32;
                    let b1 = *num_arr_itr.next().unwrap() as u32;
                    let b2 = *num_arr_itr.next().unwrap() as u32;
                    let b3 = *num_arr_itr.next().unwrap() as u32;
                    let ele = (b3 << 24) | (b2 << 16) | (b1 << 8) | b0;
                    nat.push(ele);
                }

                let mut val = 0u32;
                for ele in rem_arr {
                    val <<= 8;
                    val += *ele as u32;
                }
                if val > 0 {
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
        let mut result = &*self + rhs;

        let nat = self.as_vec_mut();
        nat.clear();
        nat.append(result.as_vec_mut());
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

impl<'b> SubAssign<&'b Nat> for Nat {
    fn sub_assign(&mut self, rhs: &'b Nat) {
        let mut result = &*self - rhs;

        let nat = self.as_vec_mut();
        nat.clear();
        nat.append(result.as_vec_mut());
    }
}

impl<'a, 'b> Mul<&'b Nat> for &'a Nat {
    type Output = Nat;

    fn mul(self, rhs: &'b Nat) -> Self::Output {
        let zero = Nat::from_u8(0);
        if self.is_nan() || rhs.is_nan() {
            return Nat::nan();
        } else if self == &zero || rhs == &zero {
            return zero;
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

        let num_len = self.bits_len();
        let den_len = rhs.bits_len();
        if num_len == den_len {
            return self - rhs;
        }

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

impl<'b> RemAssign<&'b Nat> for Nat {
    fn rem_assign(&mut self, rhs: &'b Nat) {
        let mut result = &*self % rhs;
        let nat = self.as_vec_mut();
        nat.clear();
        nat.append(result.as_vec_mut());
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

        let bits_len = self.bits_len();
        if rhs >= bits_len {
            Nat { nat: vec![0] }
        } else {
            let num = rhs / 32usize;
            let rom = rhs % 32usize;
            let mut nat = Vec::with_capacity(self.num() - num);
            let arr = self.as_vec().as_slice();
            let arr = &arr[num..];
            if rom == 0 {
                nat.extend_from_slice(arr);
            } else {
                let rom = rom as u32;
                let mut arr_itr = arr.iter();
                let mut pre = *arr_itr.next().unwrap();
                for &ele in arr_itr {
                    let val = (pre >> rom) | Nat::shl_as_c(ele, 32 - rom);
                    pre = ele;
                    nat.push(val);
                }
                let pre = pre >> rom;
                if pre > 0 {
                    nat.push(pre);
                }
            }
            Nat { nat }
        }
    }
}

impl ShrAssign<usize> for Nat {
    fn shr_assign(&mut self, rhs: usize) {
        let mut result = &*self >> rhs;
        let nat = self.as_vec_mut();
        nat.clear();
        nat.append(result.as_vec_mut());
    }
}

impl Shl<usize> for &Nat {
    type Output = Nat;

    fn shl(self, rhs: usize) -> Self::Output {
        if self.is_nan() {
            return Nat::nan();
        }

        let num = rhs / 32usize;
        let rom = rhs % 32usize;
        let mut nat: Vec<u32> = Vec::with_capacity(self.num() + num + 1);
        nat.resize(num, 0u32);
        let itr = self.as_vec().iter();
        let mut pre = 0u32;
        let rom = rom as u32;
        for &ele in itr {
            let val = (ele << rom) | pre;
            pre = Nat::shr_as_c(ele, 32 - rom);
            nat.push(val);
        }

        if pre > 0 {
            nat.push(pre);
        }

        Nat { nat }
    }
}

impl ShlAssign<usize> for Nat {
    fn shl_assign(&mut self, rhs: usize) {
        let mut result = &*self << rhs;
        let nat = self.as_vec_mut();
        nat.clear();
        nat.append(result.as_vec_mut());
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

impl PartialOrd for Nat {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        if self.is_nan() || rhs.is_nan() {
            None
        } else if self.num() > rhs.num() {
            Some(Ordering::Greater)
        } else if self.num() < rhs.num() {
            Some(Ordering::Less)
        } else {
            let mut relation = Some(Ordering::Equal);

            let mut itr = rhs.as_vec().iter().rev();
            'g: for min in self.as_vec().iter().rev() {
                match itr.next() {
                    Some(x) => {
                        if min > x {
                            relation = Some(Ordering::Greater);
                            break 'g;
                        } else if min < x {
                            relation = Some(Ordering::Less);
                            break 'g;
                        }
                    }
                    _ => {}
                };
            }

            relation
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
    }
}
