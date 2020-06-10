use std::fmt::{Debug, Formatter, Display};
use std::str::FromStr;
use crate::encoding::json::{JsonError, JsonErrorKind, Json};
use std::convert::TryFrom;

const JSONNULL_STR: &str = "null";

#[derive(Clone, Copy, PartialEq)]
pub struct JsonNull;

impl JsonNull {
    pub fn new() -> JsonNull {
        JsonNull{}
    }
}

impl Default for JsonNull {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for JsonNull {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(JSONNULL_STR)
    }
}

impl Display for JsonNull {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(JSONNULL_STR)
    }
}

impl FromStr for JsonNull {
    type Err = JsonError; 
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == JSONNULL_STR {
            Ok(JsonNull::new())
        } else {
            Err(JsonError {
                kind: JsonErrorKind::ParseJsonNullError {
                    des: format!("cannot transform '{}' to JsonNull", s),
                }
            })
        }
    }
}

impl TryFrom<Json> for JsonNull {
    type Error = JsonError;

    fn try_from(value: Json) -> Result<Self, Self::Error> {
        value.to_json_null()
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding::json::JsonNull;

    #[test]
    fn json_null() {
        let cases = [
            ("null", true),
        ];
        for ele in cases.iter() {
            let json = ele.0.parse::<JsonNull>();
            if ele.1 {
                assert_eq!(json.unwrap(), JsonNull::new());
            } else {
                assert!(json.is_err());
            }
        }
    }
}
