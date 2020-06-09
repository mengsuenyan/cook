use std::fmt::{Display, Formatter, Debug};
use std::str::FromStr;
use crate::encoding::json::{JsonError, JsonErrorKind};

// todo: 扩展到任意长度的整数/任意精度的浮点数
#[derive(Clone)]
enum JsonSubNum {
    JsonNumISize(isize),
    JSonNumUSize(usize),
    JsonNumI128(i128),
    JsonNumU128(u128),
    JsonNumF32(f32),
    JsonNumF64(f64),
}

#[derive(Clone)]
pub struct JsonNumber {
    num: JsonSubNum,
}

impl JsonNumber {
    pub fn new(num: isize) -> JsonNumber {
        JsonNumber {
            num: JsonSubNum::JsonNumISize(num)
        }
    }
}

macro_rules! impl_jsonnumber_from {
    ($Type: ty) => {
        impl From<$Type> for JsonNumber {
            fn from(num: $Type) -> Self {
                JsonNumber {
                    num: JsonSubNum::from(num)
                }
            }
        }
    };
    ($Type1: ty, $($Type2: ty), *) => {
        impl_jsonnumber_from!($Type1);
        impl_jsonnumber_from!($($Type2), *);
    };
}

impl_jsonnumber_from!(f32, f64, isize, usize, u128, i128, i8, u8, i16, u16, i32, u32, i64, u64);

impl Display for JsonNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.num.to_string().as_str())
    }
}

impl Debug for JsonNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.num.to_string().as_str())
    }
}

impl Default for JsonNumber {
    fn default() -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumISize(0)
        }
    }
}

impl Display for JsonSubNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self { 
            JsonSubNum::JsonNumISize(num) => write!(f, "{}", num),
            JsonSubNum::JSonNumUSize(num) => write!(f, "{}", num),
            JsonSubNum::JsonNumU128(num) => write!(f, "{}", num),
            JsonSubNum::JsonNumI128(num) => write!(f, "{}", num),
            JsonSubNum::JsonNumF64(num) => write!(f, "{}", num),
            JsonSubNum::JsonNumF32(num) => write!(f, "{}", num),
        }
    }
}

macro_rules! impl_fromstr_block {
    ($Type: ty, $Str_: ident) => {
        match $Str_.parse::<$Type>() {
            Ok(x) => Ok(JsonNumber::from(x)),
            Err(y) => Err(JsonError {
                kind: JsonErrorKind::ParseJsonNumberError {
                    des: format!("{}", y),
                }
            }),
        }
    };
}

impl FromStr for JsonNumber {
    type Err = JsonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.find('.').is_some() {
            impl_fromstr_block!(f64, s)
        } else {
            if s.bytes().next() == Some(b'-') {
                impl_fromstr_block!(i128, s)
            } else {
                impl_fromstr_block!(u128, s)
            }
        }
    }
}

impl From<i8> for JsonSubNum {
    fn from(num: i8) -> Self {
        JsonSubNum::JsonNumISize(num as isize)
    }
}

impl From<u8> for JsonSubNum {
    fn from(num: u8) -> Self {
        JsonSubNum::JsonNumISize(num as isize)
    }
}

impl From<i16> for JsonSubNum {
    fn from(num: i16) -> Self {
        JsonSubNum::JsonNumISize(num as isize)
    }
}

impl From<u16> for JsonSubNum {
    fn from(num: u16) -> Self {
        JsonSubNum::JsonNumISize(num as isize)
    }
}

impl From<i32> for JsonSubNum {
    fn from(num: i32) -> Self {
        JsonSubNum::JsonNumISize(num as isize)
    }
}

impl From<u32> for JsonSubNum {
    #[cfg(target_pointer_width = "32")]
    fn from(num: u32) -> Self {
        JsonSubNum::JsonNumI128(num as i128)
    }

    #[cfg(target_pointer_width = "64")]
    fn from(num: u32) -> Self {
        JsonSubNum::JsonNumISize(num as isize)
    }
}

impl From<i64> for JsonSubNum {
    #[cfg(target_pointer_width = "64")]
    fn from(num: i64) -> Self {
        JsonSubNum::JsonNumISize(num as isize)
    }
    
    #[cfg(target_pointer_width = "32")]
    fn from(num: i64) -> Self {
        JsonSubNum::JsonNumI128(num as i128)
    }
}

impl From<u64> for JsonSubNum {
    fn from(num: u64) -> Self {
        JsonSubNum::JsonNumI128(num as i128)
    }
}

impl From<isize> for JsonSubNum {
    fn from(num: isize) -> Self {
        JsonSubNum::JsonNumISize(num)
    }
}

impl From<usize> for JsonSubNum {
    fn from(num: usize) -> Self {
        JsonSubNum::JSonNumUSize(num)
    }
}

impl From<u128> for JsonSubNum {
    fn from(num: u128) -> Self {
        JsonSubNum::JsonNumU128(num)
    }
}

impl From<i128> for JsonSubNum {
    fn from(num: i128) -> Self {
        JsonSubNum::JsonNumI128(num)
    }
}

impl From<f32> for JsonSubNum {
    fn from(num: f32) -> Self {
        JsonSubNum::JsonNumF32(num)
    }
}

impl From<f64> for JsonSubNum {
    fn from(num: f64) -> Self {
        JsonSubNum::JsonNumF64(num)
    }
}
