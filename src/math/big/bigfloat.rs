//! 任意精度浮点数实现

use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    convert::From,
    fmt::{Binary, Debug, Display, Error, Formatter, LowerHex, Octal, UpperHex},
    num::FpCategory,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use crate::math::big::nat::Nat;

/// 舍入模式  
/// RoundToNearest: 四舍五入;  
/// RoundToNearestEven: 舍入到最近的偶数;  
/// RoundToZero: 向零舍入;  
/// RoundToNegInf: 向负无穷大舍入;  
/// RoundToPosInf: 向正无穷大舍入;  
/// RoundAwayFromZero: 远离零舍入;  
#[derive(Clone, Copy)]
pub enum RoundMode {
    RoundToNearest = 0,
    RoundToNearestEven,
    RoundToZero,
    RoundToNegInf,
    RoundToPosInf,
    RoundAwayFromZero,
}
// use RoundMode::{RoundToNearest, RoundToNearestEven, RoundToZero, RoundToNegInf,
//     RoundToPosInf, RoundAwayFromZero};
use RoundMode::RoundToZero;

const DEFAULT_ROUND_MODE: RoundMode = RoundToZero;

#[derive(Clone, Copy)]
enum Form {
    Finite = 0,
    Inf,
    NaN,
}

// float: 1.ppppp * 2^x
// |bigfloat|: mant * 2^{exp}
// mant: 1ppppp
// exp: 2^{-prec+x}
/// 任意精度浮点数   
/// 默认舍入模式为: RoundToNearest;  
#[derive(Clone)]
pub struct BigFloat {
    sign: bool,
    form: Form,
    mode: RoundMode,
    prec: u32,
    exp: i32,
    mantissa: Nat,
}

impl BigFloat {
    //TODO: 当前只实现了RoundToZero
    fn round(&self) -> BigFloat {
        let len = self.mantissa.bits_len() as u32;
        if len > (self.prec + 1) {
            let shift = (len - self.prec - 1) as usize;
            let mantissa = match self.mode {
                _ => &self.mantissa >> shift,
            };

            BigFloat {
                sign: self.sign,
                form: self.form,
                mode: self.mode,
                prec: self.prec,
                exp: self.exp + (shift as i32),
                mantissa,
            }
        } else {
            self.clone()
        }
    }

    // 有效位数, 精度等于有效位数-1
    fn valid_bits<T>(_val: T) -> u32
    where
        T: PartialOrd,
    {
        if std::mem::size_of::<T>() <= std::mem::size_of::<u32>() {
            (std::mem::size_of::<u32>() as u32) << 3
        } else {
            let m = std::mem::size_of::<T>() / std::mem::size_of::<u32>();
            (m as u32) << 5
        }
    }

    // 原始nat对齐到有效位数
    fn cvt_nat(nat: &Nat, vb: u32) -> (Nat, i32) {
        let len = nat.bits_len() as u32;

        // 超过长度会被截断
        if len < vb {
            let m = vb - len;
            (nat << (m as usize), -(m as i32))
        } else {
            let m = len - vb;
            (nat >> (m as usize), m as i32)
        }
    }

    fn is_same_sign(&self, rhs: &BigFloat) -> bool {
        self.sign == rhs.sign
    }

    fn nan() -> BigFloat {
        BigFloat {
            sign: false,
            form: Form::NaN,
            mode: DEFAULT_ROUND_MODE,
            prec: 0,
            exp: 0,
            mantissa: &Nat::from_u8(1) / &Nat::from_u8(0),
        }
    }

    fn min_max<T>(lhs: T, rhs: T) -> (T, T)
    where
        T: PartialOrd + Copy,
    {
        if lhs < rhs {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        }
    }

    fn min_max_prec(&self, rhs: &BigFloat) -> (u32, u32) {
        BigFloat::min_max(self.prec, rhs.prec)
    }

    fn align_exp(&self, rhs: &BigFloat) -> (Nat, Nat, i32) {
        if self.exp <= rhs.exp {
            let e = rhs.exp - self.exp;
            (
                &self.mantissa << (e as usize),
                rhs.mantissa.clone(),
                rhs.exp,
            )
        } else {
            let e = self.exp - rhs.exp;
            (
                self.mantissa.clone(),
                &rhs.mantissa << (e as usize),
                self.exp,
            )
        }
    }

    #[inline]
    pub fn is_nan(&self) -> bool {
        match self.form {
            Form::NaN => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_finite(&self) -> bool {
        match self.form {
            Form::Finite => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_inf(&self) -> bool {
        match self.form {
            Form::Inf => true,
            _ => false,
        }
    }
}

macro_rules! bf_from_int_macro {
    ($Type: ty, $FucExp: expr) => {
        impl From<$Type> for BigFloat {
            fn from(val: $Type) -> BigFloat {
                let (m, form) = $FucExp(val);
                let p = BigFloat::valid_bits(val);
                let (mantissa, exp) = BigFloat::cvt_nat(&m, p);
                let sign = <$Type>::min_value() != 0;
                BigFloat {
                    sign,
                    form,
                    mode: DEFAULT_ROUND_MODE,
                    prec: p - 1,
                    exp,
                    mantissa,
                }
            }
        }
    };
}

bf_from_int_macro!(u8, |ele| -> (Nat, Form) {
    (Nat::from_u8(ele), Form::Finite)
});
bf_from_int_macro!(u16, |ele| -> (Nat, Form) {
    (Nat::from_u16(ele), Form::Finite)
});
bf_from_int_macro!(u32, |ele| -> (Nat, Form) {
    (Nat::from_u32(ele), Form::Finite)
});
bf_from_int_macro!(usize, |ele| -> (Nat, Form) {
    (Nat::from_usize(ele), Form::Finite)
});
bf_from_int_macro!(u64, |ele| -> (Nat, Form) {
    (Nat::from_u64(ele), Form::Finite)
});
bf_from_int_macro!(u128, |ele| -> (Nat, Form) {
    (Nat::from_u128(ele), Form::Finite)
});
bf_from_int_macro!(i8, |ele: i8| -> (Nat, Form) {
    (Nat::from_u8(ele.abs() as u8), Form::Finite)
});
bf_from_int_macro!(i16, |ele: i16| -> (Nat, Form) {
    (Nat::from_u16(ele.abs() as u16), Form::Finite)
});
bf_from_int_macro!(i32, |ele: i32| -> (Nat, Form) {
    (Nat::from_u32(ele.abs() as u32), Form::Finite)
});
bf_from_int_macro!(isize, |ele: isize| -> (Nat, Form) {
    (Nat::from_usize(ele.abs() as usize), Form::Finite)
});
bf_from_int_macro!(i64, |ele: i64| -> (Nat, Form) {
    (Nat::from_u64(ele.abs() as u64), Form::Finite)
});
bf_from_int_macro!(i128, |ele: i128| -> (Nat, Form) {
    (Nat::from_u128(ele.abs() as u128), Form::Finite)
});

macro_rules! bi_from_float_macro {
    ($Type: ty, $FucName: expr) => {
        impl From<$Type> for BigFloat {
            fn from(val: $Type) -> BigFloat {
                let one = Nat::from_u8(1);
                let zero = Nat::from_u8(0);
                let vb = BigFloat::valid_bits(val);
                let (mode, sign) = (DEFAULT_ROUND_MODE, val < <$Type>::from(0u8));
                let form = match val.classify() {
                    FpCategory::Nan => Form::NaN,
                    FpCategory::Infinite => Form::Inf,
                    _ => Form::Finite,
                };
                let val = if sign { -val } else { val };
                let (mantissa, exp) = match val.classify() {
                    FpCategory::Normal => $FucName(val, true, vb),
                    FpCategory::Zero => (zero, 0),
                    FpCategory::Subnormal => $FucName(val, false, vb),
                    FpCategory::Infinite => (zero, 0),
                    _ => (&one / &zero, 0),
                };

                BigFloat {
                    sign,
                    form,
                    mode,
                    prec: vb - 1,
                    exp,
                    mantissa,
                }
            }
        }
    };
}

// f32: 指数范围[-126,127], 实际指数值: 指数位的值-127
bi_from_float_macro!(f32, |val: f32, is_normal: bool, vb: u32| -> (Nat, i32) {
    let v = val.to_le_bytes();
    let tail = (v[0] as u32) + ((v[1] as u32) << 8) + (((v[2] & 0x7f) as u32) << 16);
    let e = if is_normal {
        (((v[3] << 1) | (v[2] >> 7)) as i32) - 127
    } else {
        -126
    };
    let nat = Nat::from_u32(tail);
    let x = -24 + (nat.bits_len() as i32) + e;
    let (mant, y) = BigFloat::cvt_nat(&nat, vb);
    (mant, x + y)
});

// f64: 指数范围[-1022,1023], 实际指数值: 指数位的值-1023
bi_from_float_macro!(f64, |val: f64, is_normal: bool, vb: u32| -> (Nat, i32) {
    let v = val.to_le_bytes();
    let tail = (v[0] as u64)
        + ((v[1] as u64) << 8)
        + ((v[2] as u64) << 16)
        + ((v[3] as u64) << 24)
        + ((v[4] as u64) << 32)
        + ((v[5] as u64) << 40)
        + (((v[6] & 0xf) as u64) << 48);
    let e = if is_normal {
        (((v[6] >> 4) as i32) | (((v[7] & 0x7f) as i32) << 4)) - 1023
    } else {
        -1022
    };
    let nat = Nat::from_u64(tail);
    let x = -24 + (nat.bits_len() as i32) + e;
    let (mant, y) = BigFloat::cvt_nat(&nat, vb);
    (mant, x + y)
});

impl PartialEq for BigFloat {
    fn eq(&self, rhs: &BigFloat) -> bool {
        if self.is_nan() || rhs.is_nan() {
            false
        } else {
            self.sign == rhs.sign && self.exp == rhs.exp && self.mantissa == rhs.mantissa
        }
    }

    fn ne(&self, rhs: &BigFloat) -> bool {
        if self.is_nan() || rhs.is_nan() {
            false
        } else {
            self.sign != rhs.sign || self.exp != rhs.exp || self.mantissa != rhs.mantissa
        }
    }
}

impl PartialOrd for BigFloat {
    fn partial_cmp(&self, rhs: &BigFloat) -> Option<Ordering> {
        match self.form {
            Form::NaN => None,
            Form::Inf => match rhs.form {
                Form::NaN => None,
                Form::Inf => {
                    if self.is_same_sign(rhs) {
                        Some(Ordering::Equal)
                    } else if rhs.sign {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Less)
                    }
                }
                Form::Finite => {
                    if self.sign {
                        Some(Ordering::Less)
                    } else {
                        Some(Ordering::Greater)
                    }
                }
            },
            Form::Finite => match rhs.form {
                Form::NaN => None,
                Form::Inf => {
                    if rhs.sign {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Less)
                    }
                }
                Form::Finite => {
                    if self.is_same_sign(rhs) {
                        if self.exp == rhs.exp {
                            if self.mantissa > rhs.mantissa {
                                Some(Ordering::Greater)
                            } else if self.mantissa < rhs.mantissa {
                                Some(Ordering::Less)
                            } else {
                                Some(Ordering::Equal)
                            }
                        } else if self.exp < rhs.exp {
                            Some(Ordering::Less)
                        } else {
                            Some(Ordering::Greater)
                        }
                    } else if rhs.sign {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Less)
                    }
                }
            },
        }
    }
}

impl<'a, 'b> Add<&'b BigFloat> for &'a BigFloat {
    type Output = BigFloat;
    fn add(self, rhs: &'b BigFloat) -> Self::Output {
        match self.form {
            Form::NaN => BigFloat::nan(),
            Form::Inf => BigFloat::nan(),
            Form::Finite => match rhs.form {
                Form::Finite => {
                    let (_, p2) = self.min_max_prec(rhs);
                    let (n1, n2, exp) = self.align_exp(rhs);
                    let (mantissa, sign) = if self.is_same_sign(rhs) {
                        (&n1 + &n2, self.sign)
                    } else if rhs.sign {
                        if n1 < n2 {
                            (&n2 - &n1, true)
                        } else {
                            (&n1 - &n2, false)
                        }
                    } else {
                        if n1 < n2 {
                            (&n2 - &n1, false)
                        } else {
                            (&n1 - &n2, true)
                        }
                    };
                    let bf = BigFloat {
                        sign,
                        form: Form::Finite,
                        mode: self.mode,
                        prec: p2,
                        exp,
                        mantissa,
                    };
                    bf.round()
                }
                _ => BigFloat::nan(),
            },
        }
    }
}

impl<'b> AddAssign<&'b BigFloat> for BigFloat {
    fn add_assign(&mut self, rhs: &'b BigFloat) {
        let result = &*self + rhs;
        *self = result;
    }
}

impl<'a, 'b> Sub<&'b BigFloat> for &'a BigFloat {
    type Output = BigFloat;

    fn sub(self, rhs: &'b BigFloat) -> Self::Output {
        let sign = !rhs.sign;
        let mut rhs = rhs.clone();
        rhs.sign = sign;
        self + &rhs
    }
}

impl<'b> SubAssign<&'b BigFloat> for BigFloat {
    fn sub_assign(&mut self, rhs: &'b BigFloat) {
        let sign = !rhs.sign;
        let mut rhs = rhs.clone();
        rhs.sign = sign;
        *self += &rhs;
    }
}

impl<'a, 'b> Mul<&'b BigFloat> for &'a BigFloat {
    type Output = BigFloat;
    fn mul(self, rhs: &'b BigFloat) -> BigFloat {
        match self.form {
            Form::Inf => BigFloat::nan(),
            Form::NaN => BigFloat::nan(),
            Form::Finite => match rhs.form {
                Form::Inf => BigFloat::nan(),
                Form::NaN => BigFloat::nan(),
                Form::Finite => {
                    let (_, p2) = self.min_max_prec(rhs);
                    let exp = self.exp + self.exp;
                    let nat = &self.mantissa * &rhs.mantissa;
                    let bf = BigFloat {
                        sign: !self.is_same_sign(rhs),
                        form: self.form,
                        mode: self.mode,
                        prec: p2,
                        exp,
                        mantissa: nat,
                    };
                    bf.round()
                }
            },
        }
    }
}

impl<'b> MulAssign<&'b BigFloat> for BigFloat {
    fn mul_assign(&mut self, rhs: &'b BigFloat) {
        let result = &*self * rhs;
        *self = result;
    }
}

impl<'a, 'b> Div<&'b BigFloat> for &'a BigFloat {
    type Output = BigFloat;
    fn div(self, rhs: &'b BigFloat) -> BigFloat {
        match self.form {
            Form::Inf => BigFloat::nan(),
            Form::NaN => BigFloat::nan(),
            Form::Finite => match rhs.form {
                Form::Inf => BigFloat::nan(),
                Form::NaN => BigFloat::nan(),
                Form::Finite => {
                    let (_, p2) = self.min_max_prec(rhs);
                    let len = rhs.mantissa.bits_len();
                    let exp = self.exp - (len as i32) - rhs.exp;
                    let n1 = &self.mantissa << len;
                    let nat = &n1 / &rhs.mantissa;
                    let bf = BigFloat {
                        sign: !self.is_same_sign(rhs),
                        form: self.form,
                        mode: self.mode,
                        prec: p2,
                        exp,
                        mantissa: nat,
                    };
                    bf.round()
                }
            },
        }
    }
}

impl<'b> DivAssign<&'b BigFloat> for BigFloat {
    fn div_assign(&mut self, rhs: &'b BigFloat) {
        let result = &*self / rhs;
        *self = result;
    }
}

macro_rules! bf_fmt_macro {
    ($Type: ty, $FmtStr: literal) => {
        impl $Type for BigFloat {
            fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                let sign = if self.sign { '-' } else { '+' };
                let s = format!($FmtStr, sign, self.mantissa, self.exp);
                write!(f, "{}", s)
            }
        }
    };
}

bf_fmt_macro!(Binary, "{}{:b} * 2^({})");
bf_fmt_macro!(Octal, "{}{:o} * 2^({})");
bf_fmt_macro!(LowerHex, "{}{:x} * 2^({})");
bf_fmt_macro!(UpperHex, "{}{:X} * 2^({})");
bf_fmt_macro!(Display, "{}{} * 2^({})");
bf_fmt_macro!(Debug, "{}{} * 2^({})");

#[cfg(test)]
mod tests {
    //TODO: need to test
}
