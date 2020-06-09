use std::fmt::{Debug, Formatter, Display};
use std::str::FromStr;
use crate::encoding::json::{JsonError, JsonErrorKind};

const JSONNULL_STR: &str = "null";

#[derive(Clone, Copy)]
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
