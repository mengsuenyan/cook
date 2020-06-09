use std::fmt::{Debug, Formatter, Display};
use std::str::FromStr;
use crate::encoding::json::{JsonError, JsonErrorKind};

const JSONBOOL_TRUE_STR: &str = "true";
const JSONBOOL_FALSE_STR: &str = "false";

#[derive(Copy, Clone)]
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
