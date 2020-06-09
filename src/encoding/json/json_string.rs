use std::fmt::{Display, Debug, Formatter};
use std::str::FromStr;
use crate::encoding::json::{JsonError, JsonErrorKind};

#[derive(Clone)]
pub struct JsonString {
    string_: String
}

impl JsonString {
    pub fn new(s: &str) -> JsonString {
        JsonString {
            string_: String::from(s)
        }
    }
    
    pub fn len(&self) -> usize {
        self.string_.len()
    }
}

impl Default for JsonString {
    fn default() -> Self {
        Self::new("")
    }
}

macro_rules! impl_jsonstring_fmt {
    ($Type: ty) => {
        impl $Type for JsonString {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "\"{}\"", self.string_)
            }
        }
    };
    
    ($Type1:ty, $($Type2: ty), *) => {
        impl_jsonstring_fmt!($Type1);
        impl_jsonstring_fmt!($($Type2), *);
    };
}


impl_jsonstring_fmt!(Debug, Display);

impl From<&str> for JsonString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<&String> for JsonString {
    fn from(s: &String) -> Self {
        JsonString {
            string_: s.clone()
        }
    }
}

impl From<String> for JsonString {
    fn from(s: String) -> Self {
        JsonString {
            string_: s
        }
    }
}

impl FromStr for JsonString {
    type Err = JsonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<String>() {
            Ok(x) => Ok(JsonString::from(x)),
            Err(y) => Err(JsonError {
                kind: JsonErrorKind::ParseJsonStringError {
                    des: format!("{}", y)
                }
            })
        }
    }
}