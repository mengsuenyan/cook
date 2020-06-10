use std::fmt::{Display, Debug, Formatter};
use std::str::FromStr;
use crate::encoding::json::{JsonError, JsonErrorKind, Json};
use std::convert::TryFrom;

#[derive(Clone, PartialEq)]
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

impl TryFrom<Json> for JsonString {
    type Error = JsonError;

    fn try_from(value: Json) -> Result<Self, Self::Error> {
        value.to_json_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding::json::{JsonString};

    #[test]
    fn json_string() {
        let cases = [
            ("庄子", JsonString::from("庄子"), "\"庄子\""),
            ("北冥有鱼, 其名为鲲. 鲲之大, 不知其几千里也.", JsonString::from("北冥有鱼, 其名为鲲. 鲲之大, 不知其几千里也."), "\"北冥有鱼, 其名为鲲. 鲲之大, 不知其几千里也.\""),
        ];
        
        for ele in cases.iter() {
            let json = ele.0.parse::<JsonString>().unwrap();
            assert_eq!(json, ele.1);
            assert_eq!(format!("{}", json), ele.2);
        }
    }
}