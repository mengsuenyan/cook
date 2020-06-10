use std::fmt::{Display, Formatter, Debug};
use std::str::FromStr;
use crate::encoding::json::{JsonError, JsonErrorKind, Json};
use std::convert::TryFrom;

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

impl PartialEq for JsonNumber {
    fn eq(&self, other: &Self) -> bool {
        let x = format!("{}", self);
        let y = format!("{}", other);
        x == y
    }
}

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
        let s = s.trim();
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

impl From<i8> for JsonNumber {
    fn from(num: i8) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumISize(num as isize)
        }
    }
}

impl From<u8> for JsonNumber {
    fn from(num: u8) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumISize(num as isize)
        }
    }
}

impl From<i16> for JsonNumber {
    fn from(num: i16) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumISize(num as isize)
        }
    }
}

impl From<u16> for JsonNumber {
    fn from(num: u16) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumISize(num as isize)
        }
    }
}

impl From<i32> for JsonNumber {
    fn from(num: i32) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumISize(num as isize)
        }
    }
}

impl From<u32> for JsonNumber {
    #[cfg(target_pointer_width = "32")]
    fn from(num: u32) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumI128(num as i128)
        }
    }

    #[cfg(target_pointer_width = "64")]
    fn from(num: u32) -> Self { 
        JsonNumber {
            num: JsonSubNum::JsonNumISize(num as isize)
        }
    }
}

impl From<i64> for JsonNumber {
    #[cfg(target_pointer_width = "64")]
    fn from(num: i64) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumISize(num as isize)
        }
    }
    
    #[cfg(target_pointer_width = "32")]
    fn from(num: i64) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumI128(num as i128)
        }
    }
}

impl From<u64> for JsonNumber {
    fn from(num: u64) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumI128(num as i128)
        }
    }
}

impl From<isize> for JsonNumber {
    fn from(num: isize) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumISize(num)
        }
    }
}

impl From<usize> for JsonNumber {
    fn from(num: usize) -> Self {
        JsonNumber {
            num: JsonSubNum::JSonNumUSize(num)
        }
    }
}

impl From<u128> for JsonNumber {
    fn from(num: u128) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumU128(num)
        }
    }
}

impl From<i128> for JsonNumber {
    fn from(num: i128) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumI128(num)
        }
    }
}

impl From<f32> for JsonNumber {
    fn from(num: f32) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumF32(num)
        }
    }
}

impl From<f64> for JsonNumber {
    fn from(num: f64) -> Self {
        JsonNumber {
            num: JsonSubNum::JsonNumF64(num)
        }
    }
}

impl TryFrom<Json> for JsonNumber {
    type Error = JsonError;

    fn try_from(value: Json) -> Result<Self, Self::Error> {
        value.to_json_number()
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding::json::JsonNumber;

    #[test]
    fn json_number() {
        let cases = [
            ("1.234568", JsonNumber::from(1.234568)),
            ("80258750757", JsonNumber::from(80258750757u64)),
            ("-80258750757", JsonNumber::from(-80258750757i64)),
        ];

        for ele in cases.iter() {
            let json = ele.0.parse::<JsonNumber>();
            assert_eq!(json.unwrap(), ele.1);
        }
    }
}
