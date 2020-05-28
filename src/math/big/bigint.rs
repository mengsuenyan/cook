//! This file implements signed multi-precision integers.

use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    convert::From,
    fmt::{Binary, Debug, Display, Error, Formatter, LowerHex, Octal, UpperHex},
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div,
        DivAssign, Mul, MulAssign, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub,
        SubAssign,
    },
};

use crate::math::big::Nat;

#[derive(Clone, Copy, PartialEq)]
enum BigIntType {
    Pos = 0,
    Neg = 1,
}
use BigIntType::{Neg, Pos};

/// 任意长度大整数BigInt
///
/// 算术运算: +, -, *, /, %. 注意: /0, %0结果为NaN.
///
/// 按位逻辑运算: &, |, ^, !.
///
/// 关系运算: <, >, <=, >=, ==.
///
/// 支持从u8/u16/u32/usize/u64/u128/i8/i16/i32/isize/i64/i128转换为BigInt.
///
/// 支持从Vec<u8>, &[u8]转换为BigInt.
///
/// 支持&str, String二进制/八进制/十进制/十六进制转换为BigInt.
///
/// 支持二进制/八进制/十进制/十六进制格式化输出(不支持输出对齐), 输出时符号位以'+','-'表示.
///
/// 注: 不支持Inf无穷大的自然数, 诸如Nat::from_u8(1)/Nat::from_u8(0)之类的操作会得到NaN;
///
#[derive(Clone)]
pub struct BigInt {
    nat: Nat,
    bi_type: BigIntType,
}

impl BigInt {
    fn parse(s: &str) -> (BigIntType, &str, u8) {
        let mut s_itr = s.bytes();
        let p1 = s_itr.next();
        let p2 = s_itr.next();
        let p3 = s_itr.next();

        let (bi_type, s1, s2, idx) = match p1 {
            Some(b'+') => (Pos, p2, p3, 1),
            Some(b'-') => (Neg, p2, p3, 1),
            Some(_) => (Pos, p1, p2, 0),
            _ => (Pos, p1, p2, 0),
        };

        match s1 {
            Some(b'0') => {
                match s2 {
                    Some(b'x') => (bi_type, &s[(idx + 2)..], 16),
                    Some(b'X') => (bi_type, &s[(idx + 2)..], 16),
                    Some(b'b') => (bi_type, &s[(idx + 2)..], 2),
                    Some(_) => (bi_type, &s[(idx + 1)..], 8),
                    None => (Pos, s, 10), // "0"
                }
            }
            Some(_) => (bi_type, &s[idx..], 10),
            None => (Pos, s, 10),
        }
    }
    
    pub fn bits_len(&self) -> usize {
        self.get_nat().bits_len()
    }

    #[inline]
    fn get_nat(&self) -> &Nat {
        &self.nat
    }

    #[inline]
    fn get_nat_mut(&mut self) -> &mut Nat {
        &mut self.nat
    }

    #[inline]
    fn is_same_sign(&self, rhs: &BigInt) -> bool {
        self.bi_type == rhs.bi_type
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.get_nat() == &Nat::from_u8(0)
    }

    #[allow(dead_code)]
    #[inline]
    fn nan() -> BigInt {
        BigInt {
            bi_type: Pos,
            nat: Nat::from_str("ff", 10),
        }
    }

    #[inline]
    pub fn is_nan(&self) -> bool {
        self.nat.is_nan()
    }
    
    pub fn abs(&self) -> Self {
        BigInt {
            nat: self.nat.clone(),
            bi_type: BigIntType::Pos,
        }
    }

    /// 求公约数, 返回(d, x, y), 其中:
    /// d = gcd(self, other); d = self * x + other * y;  
    /// 如果self和other为非自然数, 那么返回None, 否则返回Some((d, x, y));  
    /// 
    /// 特殊情况:  
    /// gcd(a, 0) = a;  
    /// gcd(0, 0) = 0;  
    pub fn gcd(&self, other: &Self) -> Option<(BigInt, BigInt, BigInt)> {
        let zero = BigInt::from(0);

        if self.is_nan() || other.is_nan() {
            None
        } else if (self == &zero) && (other == &zero) {
            Some((zero.clone(), zero.clone(), zero))
        } else if (self == &zero) && (other != &zero) {
            Some((other.abs(), zero, BigInt::from(1)))
        } else if (self != &zero) && (other == &zero) {
            Some((self.abs(), BigInt::from(1), zero))
        } else {
            Some(BigInt::gcd_extend(self, other))
        }
    }

    fn gcd_extend(a: &Self, b: &Self) -> (BigInt, BigInt, BigInt) {
        let zero = BigInt::from(0);
        
        if b == &zero {
            (a.clone(), BigInt::from(1), BigInt::from(0))
        } else {
            let rem = a % b;
            let div = a / b;
            let (d_p, x_p, y_p) = BigInt::gcd_extend(b, &rem);
            let yy = &x_p - &(&div * &y_p);
            let (d, x, y) = (d_p, y_p, yy);
            (d, x, y)
        }
    }
    
    /// <<算法导论>>  
    /// 定理31.23: 若有d=gcd(a, n), 假设对于某些整数x'和y', 有d=ax'+ny'. 如果d|b, 则方程
    /// ax=b(mod n)有一个解的值位x0, 则x0=x'(b/d) mod n;  
    /// 假设方程ax=b(mod n)有解(即d|b, d=gcd(a,n)), 且x0是该方程的任意一个解. 因此, 该方程对模
    /// n恰有d个不同的解, 分别为xi=x0+i(n/d), 这里i=0,1,...,d-1;  
    pub fn mod_inverse(&self, other: &Self) -> Option<BigInt> {
        let zero = BigInt::from(0);
        
        if self.is_nan() || other.is_nan() {
            None
        } else {
            match self.gcd(other) {
                Some(g) => if other < &zero {
                    Some(other + &g.1)
                } else {
                    Some(g.1)
                },
                None => None
            }
        }
    }
    
    pub fn to_nat(&self) -> Nat {
        self.nat.clone()
    }
}

macro_rules! bi_impl_from_macro {
    ($Tgt: ty, Sign) => {
        impl From<$Tgt> for BigInt {
            fn from(val: $Tgt) -> BigInt {
                BigInt {
                    nat: if val == <$Tgt>::min_value() {
                        Nat::from_u128((<$Tgt>::max_value() as u128) + 1)
                    } else {
                        Nat::from_u128(val.abs() as u128)
                    },
                    bi_type: if val < 0 { Neg } else { Pos },
                }
            }
        }
    };
    ($Tgt: ty, USign) => {
        impl From<$Tgt> for BigInt {
            fn from(val: $Tgt) -> BigInt {
                BigInt {
                    nat: Nat::from_u128(val as u128),
                    bi_type: Pos,
                }
            }
        }
    };
}

bi_impl_from_macro!(i8, Sign);
bi_impl_from_macro!(i16, Sign);
bi_impl_from_macro!(i32, Sign);
bi_impl_from_macro!(isize, Sign);
bi_impl_from_macro!(i64, Sign);
bi_impl_from_macro!(i128, Sign);
bi_impl_from_macro!(u8, USign);
bi_impl_from_macro!(u16, USign);
bi_impl_from_macro!(u32, USign);
bi_impl_from_macro!(usize, USign);
bi_impl_from_macro!(u64, USign);
bi_impl_from_macro!(u128, USign);

impl From<Vec<u8>> for BigInt {
    fn from(v: Vec<u8>) -> Self {
        BigInt {
            nat: Nat::from_vec(&v),
            bi_type: Pos,
        }
    }
}

impl From<Nat> for BigInt {
    fn from(nat: Nat) -> Self {
        BigInt {
            nat,
            bi_type: Pos,
        }
    }
}

impl Into<u64> for BigInt {
    fn into(self) -> u64 {
        match self.nat.to_u64() {
            Some(x) => x,
            _ => 0,
        }
    }
}

impl From<&str> for BigInt {
    fn from(s: &str) -> BigInt {
        let (bi_type, ss, base) = BigInt::parse(s);
        BigInt {
            nat: Nat::new(ss, base),
            bi_type,
        }
    }
}

impl From<&String> for BigInt {
    fn from(s: &String) -> BigInt {
        BigInt::from(s.as_str())
    }
}

macro_rules! bi_impl_from_vec_macro {
    ($Type: ty, $FromName: ident) => {
        impl From<&$Type> for BigInt {
            fn from(v: &$Type) -> BigInt {
                BigInt {
                    nat: Nat::$FromName(v),
                    bi_type: Pos,
                }
            }
        }
    };
}

bi_impl_from_vec_macro!(Vec<u8>, from_vec);
bi_impl_from_vec_macro!([u8], from_slice);

impl PartialEq for BigInt {
    fn eq(&self, rhs: &BigInt) -> bool {
        if self.get_nat().eq(rhs.get_nat()) {
            if self.is_zero() {
                true
            } else {
                self.is_same_sign(rhs)
            }
        } else {
            false
        }
    }
}

macro_rules! bi_pos_neg_mat_macro {
    ($Self:ident, $Rhs:ident, $PosPos:expr, $PosNeg:expr, $NegPos: expr, $NegNeg: expr) => {
        match $Self.bi_type {
            Pos => match $Rhs.bi_type {
                Pos => $PosPos,
                Neg => $PosNeg,
            },
            Neg => match $Rhs.bi_type {
                Pos => $NegPos,
                Neg => $NegNeg,
            },
        }
    };
}

impl PartialOrd for BigInt {
    fn partial_cmp(&self, rhs: &BigInt) -> Option<Ordering> {
        match self.get_nat().partial_cmp(rhs.get_nat()) {
            None => None,
            Some(Ordering::Equal) => {
                if self.is_zero() {
                    Some(Ordering::Equal)
                } else {
                    bi_pos_neg_mat_macro!(
                        self,
                        rhs,
                        { Some(Ordering::Equal) },
                        { Some(Ordering::Greater) },
                        { Some(Ordering::Less) },
                        { Some(Ordering::Equal) }
                    )
                }
            }
            Some(Ordering::Less) => bi_pos_neg_mat_macro!(
                self,
                rhs,
                { Some(Ordering::Less) },
                { Some(Ordering::Greater) },
                { Some(Ordering::Less) },
                { Some(Ordering::Greater) }
            ),
            Some(Ordering::Greater) => bi_pos_neg_mat_macro!(
                self,
                rhs,
                { Some(Ordering::Greater) },
                { Some(Ordering::Greater) },
                { Some(Ordering::Less) },
                { Some(Ordering::Less) }
            ),
        }
    }
}

impl<'a, 'b> BitAnd<&'b BigInt> for &'a BigInt {
    type Output = BigInt;
    fn bitand(self, rhs: &'b BigInt) -> Self::Output {
        BigInt {
            nat: self.get_nat() & rhs.get_nat(),
            bi_type: bi_pos_neg_mat_macro!(self, rhs, { Pos }, { Pos }, { Pos }, { Neg }),
        }
    }
}

impl<'b> BitAndAssign<&'b BigInt> for BigInt {
    fn bitand_assign(&mut self, rhs: &'b BigInt) {
        self.bi_type = bi_pos_neg_mat_macro!(self, rhs, { Pos }, { Pos }, { Pos }, { Neg });
        *self.get_nat_mut() &= rhs.get_nat();
    }
}

impl<'a, 'b> BitOr<&'b BigInt> for &'a BigInt {
    type Output = BigInt;

    fn bitor(self, rhs: &'b BigInt) -> Self::Output {
        BigInt {
            bi_type: bi_pos_neg_mat_macro!(self, rhs, { Pos }, { Neg }, { Neg }, { Neg }),
            nat: self.get_nat() | rhs.get_nat(),
        }
    }
}

impl<'b> BitOrAssign<&'b BigInt> for BigInt {
    fn bitor_assign(&mut self, rhs: &'b BigInt) {
        self.bi_type = bi_pos_neg_mat_macro!(self, rhs, { Pos }, { Neg }, { Neg }, { Neg });
        *self.get_nat_mut() |= rhs.get_nat();
    }
}

impl<'a, 'b> BitXor<&'b BigInt> for &'a BigInt {
    type Output = BigInt;
    fn bitxor(self, rhs: &'b BigInt) -> Self::Output {
        BigInt {
            bi_type: bi_pos_neg_mat_macro!(self, rhs, { Pos }, { Neg }, { Neg }, { Pos }),
            nat: self.get_nat() ^ rhs.get_nat(),
        }
    }
}

impl<'b> BitXorAssign<&'b BigInt> for BigInt {
    fn bitxor_assign(&mut self, rhs: &'b BigInt) {
        self.bi_type = bi_pos_neg_mat_macro!(self, rhs, { Pos }, { Neg }, { Neg }, { Pos });
        *self.get_nat_mut() ^= rhs.get_nat();
    }
}

impl Not for &BigInt {
    type Output = BigInt;
    fn not(self) -> Self::Output {
        BigInt {
            bi_type: match self.bi_type {
                Pos => Neg,
                Neg => Pos,
            },
            nat: self.get_nat().not(),
        }
    }
}

impl<'a, 'b> Add<&'b BigInt> for &'a BigInt {
    type Output = BigInt;
    fn add(self, rhs: &'b BigInt) -> Self::Output {
        let is_great = self.get_nat() >= rhs.get_nat();
        bi_pos_neg_mat_macro!(
            self,
            rhs,
            {
                BigInt {
                    nat: self.get_nat() + rhs.get_nat(),
                    bi_type: Pos,
                }
            },
            {
                if is_great {
                    BigInt {
                        nat: self.get_nat() - rhs.get_nat(),
                        bi_type: Pos,
                    }
                } else {
                    BigInt {
                        nat: rhs.get_nat() - self.get_nat(),
                        bi_type: Neg,
                    }
                }
            },
            {
                if is_great {
                    BigInt {
                        nat: self.get_nat() - rhs.get_nat(),
                        bi_type: Neg,
                    }
                } else {
                    BigInt {
                        nat: rhs.get_nat() - self.get_nat(),
                        bi_type: Pos,
                    }
                }
            },
            {
                BigInt {
                    nat: self.get_nat() + rhs.get_nat(),
                    bi_type: Neg,
                }
            }
        )
    }
}

impl<'b> AddAssign<&'b BigInt> for BigInt {
    fn add_assign(&mut self, rhs: &'b BigInt) {
        let is_great = self.get_nat() >= rhs.get_nat();
        bi_pos_neg_mat_macro!(
            self,
            rhs,
            {
                self.bi_type = Pos;
                *self.get_nat_mut() += rhs.get_nat();
            },
            {
                if is_great {
                    self.bi_type = Pos;
                    *self.get_nat_mut() -= rhs.get_nat();
                } else {
                    self.bi_type = Neg;
                    *self.get_nat_mut() -= rhs.get_nat();
                }
            },
            {
                if is_great {
                    self.bi_type = Neg;
                    *self.get_nat_mut() -= rhs.get_nat();
                } else {
                    self.bi_type = Pos;
                    *self.get_nat_mut() -= rhs.get_nat();
                }
            },
            {
                self.bi_type = Neg;
                *self.get_nat_mut() += rhs.get_nat();
            }
        );
    }
}

impl<'a, 'b> Sub<&'b BigInt> for &'a BigInt {
    type Output = BigInt;
    fn sub(self, rhs: &'b BigInt) -> Self::Output {
        let is_great = self.get_nat() >= rhs.get_nat();
        bi_pos_neg_mat_macro!(
            self,
            rhs,
            {
                BigInt {
                    nat: self.get_nat() - rhs.get_nat(),
                    bi_type: if is_great { Pos } else { Neg },
                }
            },
            {
                BigInt {
                    nat: self.get_nat() + rhs.get_nat(),
                    bi_type: Pos,
                }
            },
            {
                BigInt {
                    nat: self.get_nat() + rhs.get_nat(),
                    bi_type: Neg,
                }
            },
            {
                BigInt {
                    nat: self.get_nat() - rhs.get_nat(),
                    bi_type: if is_great { Neg } else { Pos },
                }
            }
        )
    }
}

impl<'b> SubAssign<&'b BigInt> for BigInt {
    fn sub_assign(&mut self, rhs: &'b BigInt) {
        let is_great = self.get_nat() >= rhs.get_nat();
        bi_pos_neg_mat_macro!(
            self,
            rhs,
            {
                self.bi_type = if is_great { Pos } else { Neg };
                *self.get_nat_mut() -= rhs.get_nat();
            },
            {
                self.bi_type = Pos;
                *self.get_nat_mut() += rhs.get_nat();
            },
            {
                self.bi_type = Neg;
                *self.get_nat_mut() += rhs.get_nat();
            },
            {
                self.bi_type = if is_great { Neg } else { Pos };
                *self.get_nat_mut() -= rhs.get_nat();
            }
        );
    }
}

impl std::ops::Neg for &BigInt {
    type Output = BigInt;
    fn neg(self) -> Self::Output {
        BigInt {
            nat: self.get_nat().clone(),
            bi_type: if self.is_zero() {
                Pos
            } else {
                match self.bi_type {
                    Pos => Neg,
                    Neg => Pos,
                }
            },
        }
    }
}

impl<'a, 'b> Mul<&'b BigInt> for &'a BigInt {
    type Output = BigInt;
    fn mul(self, rhs: &'b BigInt) -> Self::Output {
        BigInt {
            nat: self.get_nat() * rhs.get_nat(),
            bi_type: if self.is_same_sign(rhs) { Pos } else { Neg },
        }
    }
}

impl<'b> MulAssign<&'b BigInt> for BigInt {
    fn mul_assign(&mut self, rhs: &'b BigInt) {
        *self.get_nat_mut() *= rhs.get_nat();
        self.bi_type = if self.is_same_sign(rhs) { Pos } else { Neg }
    }
}

impl<'a, 'b> Div<&'b BigInt> for &'a BigInt {
    type Output = BigInt;
    fn div(self, rhs: &'b BigInt) -> Self::Output {
        BigInt {
            nat: self.get_nat() / rhs.get_nat(),
            bi_type: if self.is_same_sign(rhs) { Pos } else { Neg },
        }
    }
}

impl<'b> DivAssign<&'b BigInt> for BigInt {
    fn div_assign(&mut self, rhs: &'b BigInt) {
        *self.get_nat_mut() /= rhs.get_nat();
        self.bi_type = if self.is_same_sign(rhs) { Pos } else { Neg };
    }
}

impl<'a, 'b> Rem<&'b BigInt> for &'a BigInt {
    type Output = BigInt;
    fn rem(self, rhs: &'b BigInt) -> Self::Output {
        BigInt {
            nat: self.get_nat() % rhs.get_nat(),
            bi_type: if self.is_same_sign(rhs) { Pos } else { Neg },
        }
    }
}

impl<'b> RemAssign<&'b BigInt> for BigInt {
    fn rem_assign(&mut self, rhs: &'b BigInt) {
        *self.get_nat_mut() %= rhs.get_nat();
        self.bi_type = if self.is_same_sign(rhs) { Pos } else { Neg };
    }
}

impl Shl<usize> for &BigInt {
    type Output = BigInt;
    fn shl(self, rhs: usize) -> Self::Output {
        BigInt {
            bi_type: self.bi_type,
            nat: self.get_nat() << rhs,
        }
    }
}

impl ShlAssign<usize> for BigInt {
    fn shl_assign(&mut self, rhs: usize) {
        *self.get_nat_mut() <<= rhs;
    }
}

impl Shr<usize> for &BigInt {
    type Output = BigInt;
    fn shr(self, rhs: usize) -> Self::Output {
        let nat = self.get_nat() >> rhs;
        BigInt {
            bi_type: if &nat == &Nat::from_u8(0) {
                Pos
            } else {
                self.bi_type
            },
            nat,
        }
    }
}

impl ShrAssign<usize> for BigInt {
    fn shr_assign(&mut self, rhs: usize) {
        *self.get_nat_mut() >>= rhs;
        if self.get_nat() == &Nat::from_u8(0) {
            self.bi_type = Pos;
        }
    }
}

macro_rules! bi_impl_fmt_macro {
    ($Trait: ty, $Fmt: literal) => {
        impl $Trait for BigInt {
            fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                let nat = format!($Fmt, self.get_nat());
                if !self.is_nan() {
                    let si = if self.bi_type == Pos { '+' } else { '-' };
                    write!(f, "{}{}", si, nat)
                } else {
                    write!(f, "{}", nat)
                }
            }
        }
    };
}

bi_impl_fmt_macro!(Binary, "{:b}");
bi_impl_fmt_macro!(Octal, "{:o}");
bi_impl_fmt_macro!(Display, "{}");
bi_impl_fmt_macro!(LowerHex, "{:x}");
bi_impl_fmt_macro!(UpperHex, "{:X}");
bi_impl_fmt_macro!(Debug, "{:x}");

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_bi_from_and_fmt_help {
        ($Type: ty, $Fmt1: literal, $Fmt2: literal, $Si: ident, $Min: ident, $Max: ident) => {{
            let min_str = format!($Fmt1, $Si, $Min);
            let max_str = format!($Fmt1, '+', $Max);
            let bi_max = BigInt::from(<$Type>::max_value());
            let bi_min = BigInt::from(<$Type>::min_value());
            assert_eq!(
                format!($Fmt2, bi_max),
                max_str,
                "{}->{}",
                stringify!($Type),
                $Fmt2
            );
            assert_eq!(
                format!($Fmt2, bi_min),
                min_str,
                "{}->{}",
                stringify!($Type),
                $Fmt2
            );
        }};
    }

    macro_rules! test_bi_from_and_fmt {
        ($Type: ty) => {
            let (max, min, si) = if <$Type>::min_value() == 0 {
                (<$Type>::max_value(), <$Type>::min_value() as u128, '+')
            } else {
                (<$Type>::max_value(), (<$Type>::max_value() as u128 ) + 1, '-')
            };

            test_bi_from_and_fmt_help!($Type, "{}{}", "{}", si, min, max);
            test_bi_from_and_fmt_help!($Type, "{}{:b}", "{:b}", si, min, max);
            test_bi_from_and_fmt_help!($Type, "{}{:o}", "{:o}", si, min, max);
            test_bi_from_and_fmt_help!($Type, "{}{:x}", "{:x}", si, min, max);
            test_bi_from_and_fmt_help!($Type, "{}{:X}", "{:X}", si, min, max);
            test_bi_from_and_fmt_help!($Type, "{}{:x}", "{:?}", si, min, max);
        };
        ($Type: ty, $($Type1: ty), +) => {
            test_bi_from_and_fmt!($Type);
            test_bi_from_and_fmt!($($Type1), +);
        };
    }

    #[test]
    fn test_from_and_fmt() {
        test_bi_from_and_fmt!(i8, i16, i32, isize, i64, i128, u8, u16, u32, usize, u64, u128);
    }

    #[test]
    fn test_relation_arith() {
        let l1 = BigInt::from(std::u128::MAX);
        let l2 = BigInt::from(std::u128::MAX);
        let l_sum = BigInt::from("0x1fffffffffffffffffffffffffffffffe");
        let s1 = BigInt::from(std::u8::MAX);
        let s2 = BigInt::from(std::u8::MAX);
        let s_sum = BigInt::from("0x1fe");
        let nan = BigInt::nan();
        assert!(l1 == l2);
        assert!(l1 <= l2);
        assert!(l1 <= l_sum);
        assert!(l2 < l_sum);
        assert!(s_sum > s1);
        assert!(s_sum >= s2);
        assert!(nan != nan);
        assert!(l1 != nan);
        assert!(nan != l1);
        assert_eq!(BigInt::from(0u8), BigInt::from(0i128));
    }

    #[test]
    fn test_logical_arith() {
        let l1 = BigInt::from(std::u128::MAX);
        let l2 = BigInt::from(std::u128::MAX);

        assert_eq!(&l1 & &l2, l1);
        assert_eq!(&l1 | &l2, l2);
        assert_eq!(&l1 ^ &l2, BigInt::from(0));
        assert_eq!(!&l1, BigInt::from(0));
        assert_eq!(
            format!("{}", &l1 & &BigInt::nan()),
            format!("{}", BigInt::nan())
        );

        let l1 = BigInt::from("0xfffffffffffffffffffffffffffffffffff3222222222222222222234900000000000000ffffffffffffffffffffff");
        let l2 = BigInt::from("0xff9000000000000000000000322222222222223209053065839583093285340530493058304593058390584");
        assert_eq!(&l1 ^ &l2, BigInt::from("0xfffffff006fffffffffffffffffffffcddd1000000000102b271247b7058309328534053fb6cfa7cfba6cfa7c6fa7b"));
        assert_eq!(&l1 | &l2, BigInt::from("0xfffffffffffffffffffffffffffffffffff3222222222322b273267b7958309328534053ffffffffffffffffffffff"));
        assert_eq!(&l1 & &l2, BigInt::from("0xff9000000000000000000000322222222222222200002020009000000000000000493058304593058390584"));
        assert_eq!(!&l2, BigInt::from("-0x6fffffffffffffffffffffcdddddddddddddcdf6facf9a7c6a7cf6cd7acbfacfb6cfa7cfba6cfa7c6fa7b"));
        assert_eq!(!&BigInt::from("0b11000011"), BigInt::from("-0b111100"));
    }

    #[test]
    fn test_shift_arith() {
        let l2 = BigInt::from("0xff9000000000000000000000322222222222223209053065839583093285340530493058304593058390584");
        let l3 = BigInt::from("0x1ff20000000000000000000006444444444444464120a60cb072b0612650a680a609260b0608b260b0720b08");
        assert_eq!(&l2 << 1, l3);
        assert_eq!(&l2 << 0, l2);
        assert_eq!(&l2 << 30, BigInt::from("0x3fe4000000000000000000000c8888888888888c82414c1960e560c24ca14d014c124c160c1164c160e416100000000"));
        assert_eq!(&l2 << 10000, BigInt::from("0xff90000000000000000000003222222222222232090530658395830932853405304930583045930583905840000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"));
        assert_eq!(&l2 >> 4, BigInt::from("0xff900000000000000000000032222222222222320905306583958309328534053049305830459305839058"));
        assert_eq!(&l2 >> 1, BigInt::from("0x7fc800000000000000000000191111111111111904829832c1cac18499429a029824982c1822c982c1c82c2"));
        assert_eq!(&l2 >> 0, l2);
        assert_eq!(&l2 >> 1001, BigInt::from(0));
        assert_eq!(&BigInt::from(0) << 0, BigInt::from(0));
        assert_eq!(&BigInt::from(0) << 3, BigInt::from(0));
    }

    #[test]
    fn test_add() {
        let mut l1 = BigInt::from(std::u128::MAX);
        let l2 = BigInt::from(std::u128::MAX);
        let sum = BigInt::from("0x1fffffffffffffffffffffffffffffffe");
        assert_eq!(&l1 + &l2, sum);
        l1 += &l2;
        assert_eq!(l1, sum);
        assert_eq!(
            &l1 + &BigInt::from(1),
            BigInt::from("0x1ffffffffffffffffffffffffffffffff")
        );
        let l1 = BigInt::from("0xfffffffffffffffffffffffffffffffffff3222222222222222222234900000000000000ffffffffffffffffffffff");
        let l2 = BigInt::from("0xff9000000000000000000000322222222222223209053065839583093285340530493058304593058390584");
        let sum = BigInt::from("0x10000000ff900000000000000000000032215444444444542b275287b82583093285340540493058304593058390583");
        assert_eq!(&l1 + &l2, sum, "{}=====>{}======{}", l1, l2, sum);

        let s1 = BigInt::from(std::u8::MAX);
        let s2 = BigInt::from(std::u8::MAX);
        let sum = BigInt::from("0x1fe");
        assert_eq!(&s1 + &s2, sum);

        let nan = BigInt::nan();
        assert_eq!(format!("{:x}", &nan + &l1), format!("{:x}", nan));
    }

    #[test]
    fn test_sub() {
        let l1 = BigInt::from(std::u128::MAX);
        let l2 = BigInt::from(std::u8::MAX);
        assert_eq!(&l1 - &l1, BigInt::from(0));
        assert_eq!(
            &l1 - &l2,
            BigInt::from(std::u128::MAX - (std::u8::MAX as u128))
        );
        assert_eq!(&l2 - &l1, -&(&l1 - &l2));
        let l1 = BigInt::from("0xfffffffffffffffffffffffffffffffffff3222222222222222222234900000000000000ffffffffffffffffffffff");
        let l2 = BigInt::from("0x32888f300000000000000322222229750348593045830670204");
        let sub = BigInt::from("0xfffffffffffffffffffffffffffffffffff32222221ef9992f22222348ffffffcdddddde68afcb7a6cfba7cf98fdfb");
        assert_eq!(&l1 - &l2, sub);
        assert_eq!(&l2 - &l1, -&sub);
    }

    #[test]
    fn test_mul() {
        let l1 = BigInt::from(10);
        assert_eq!(&l1 * &l1, BigInt::from(100));
        assert_eq!(&l1 * &BigInt::from(0), BigInt::from(0));
        assert_eq!(&l1 * &BigInt::from(1), l1);
        let l1 = BigInt::from("0xf9058301048250fabddeabf9320480284932084552541");
        let l2 = BigInt::from("0xf329053910428502fabcd9230494035242429890eacb");
        let m = BigInt::from("0xec882250900ba90c2088a4a5ee549ecc5152d7a50683a82daa24e03f6d6409468abf1ce1f01d9be845021f48b");
        assert_eq!(&l1 * &l2, m);
    }

    #[test]
    fn test_div() {
        let l1 = BigInt::from(100);
        let l2 = BigInt::from(10);
        assert_eq!(&l1 / &l2, BigInt::from(10));
        let l1 = BigInt::from("0xfffffffffff32908329058205820");
        let l2 = BigInt::from("0xff");
        let quo = BigInt::from("0x10101010100f41d2557e84060b8");
        assert_eq!(&l1 / &l2, quo);
        assert_eq!(&l2 / &l1, BigInt::from(0));
        let l1 = BigInt::from("0x39025820857032850384502853503850325fa3242de121");
        let l2 = BigInt::from("0x2048537058358afedead392582075275");
        let quo = BigInt::from("0x1c414f70ec1f027");
        assert_eq!(&l1 / &l2, quo);
        let l1 = BigInt::from(0x1ad7f29abcau128);
        assert_eq!(&l1 / &BigInt::from(10), BigInt::from(184467440737u128));
    }

    #[test]
    fn test_rem() {
        let l1 = BigInt::from("0xffffffffffffff000000000000");
        let l2 = BigInt::from(255u8);
        assert_eq!(&l1 % &l2, BigInt::from(0));
        let l1 = BigInt::from("0x39025820857032850384502853503850325fa3242de121");
        let l2 = BigInt::from("0x2048537058358afedead392582075275");
        let rem = BigInt::from("0xab9de6183b632a33dc2601ae78da14e");
        assert_eq!(&l1 % &l2, rem);
        let l1 = BigInt::from("0xfffffffffff32908329058205820");
        let l2 = BigInt::from("0xff");
        let quo = BigInt::from("0xd8");
        assert_eq!(&l1 % &l2, quo);
    }
}
