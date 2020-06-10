use std::fmt::{Debug, Formatter, Display};
use std::str::FromStr;
use crate::encoding::json::{JsonError, JsonErrorKind, Json};
use std::convert::TryFrom;

const JSONBOOL_TRUE_STR: &str = "true";
const JSONBOOL_FALSE_STR: &str = "false";

#[derive(Copy, Clone, PartialEq)]
pub struct JsonBool {
    boolean: bool,
}

impl JsonBool {
    pub fn new(boolean: bool) -> JsonBool {
        JsonBool {
            boolean
        }
    }
}

impl Default for JsonBool {
    fn default() -> Self {
        Self::new(false)
    }
}

macro_rules! impl_jsonbool_fmt {
    ($Type: ty) => {
        impl $Type for JsonBool {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self.boolean {
                    true => f.write_str(JSONBOOL_TRUE_STR),
                    false => f.write_str(JSONBOOL_FALSE_STR),
                }
            }
        }
    };
    
    ($Type1: ty, $($Type2: ty), *) => {
        impl_jsonbool_fmt!($Type1);
        impl_jsonbool_fmt!($($Type2), *);
    };
}

impl_jsonbool_fmt!(Debug, Display);

impl From<bool> for JsonBool {
    fn from(boolean: bool) -> Self {
        Self::new(boolean)
    }
}

impl FromStr for JsonBool {
    type Err = JsonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == JSONBOOL_TRUE_STR {
            Ok(JsonBool::new(true))
        } else if s == JSONBOOL_FALSE_STR {
            Ok(JsonBool::new(false))
        } else {
            Err(JsonError {
                kind: JsonErrorKind::ParseJsonBoolError {
                    des: format!("cannot transform `{}` to JsonBool", s),
                }
            })
        }
    }
}

impl TryFrom<Json> for JsonBool {
    type Error = JsonError;

    fn try_from(value: Json) -> Result<Self, Self::Error> {
        value.to_json_bool()
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding::json::JsonBool;

    #[test]
    fn json_bool() {
        let cases = [
            ("true", JsonBool::new(true)),
            ("false", JsonBool::from(false)),
        ];
        
        for ele in cases.iter() {
            let json = ele.0.parse::<JsonBool>();
            assert_eq!(json.unwrap(), ele.1);
        }
    }
}
