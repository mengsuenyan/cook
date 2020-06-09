use crate::encoding::json::json_array::JsonArray;
use crate::encoding::json::json_object::JsonObject;
use crate::encoding::json::json_null::JsonNull;
use crate::encoding::json::json_bool::JsonBool;
use crate::encoding::json::json_string::JsonString;
use crate::encoding::json::json_number::JsonNumber;

#[derive(Clone)]
enum JsonEntity {
    JsonEntityNull(JsonNull),
    JsonEntityBool(JsonBool),
    JsonEntityNumber(JsonNumber),
    JsonEntityString(JsonString),
    JsonEntityArray(JsonArray),
    JsonEntityObject(JsonObject)
}

use JsonEntity::{JsonEntityNull, JsonEntityBool, JsonEntityString, JsonEntityNumber, JsonEntityArray, JsonEntityObject};
use std::fmt::{Debug, Formatter, Display};

#[derive(Clone)]
pub struct Json {
    entity: JsonEntity,
}

impl Json {
    // 默认是null
    pub fn new() -> Json {
        Json {
            entity: JsonEntityNull(JsonNull::new())
        }
    }
    
    pub fn is_null(&self) -> bool {
        match self.entity {
            JsonEntity::JsonEntityNull(..) => true,
            _ => false,
        }
    }
    
    pub fn is_bool(&self) -> bool {
        match self.entity { 
            JsonEntity::JsonEntityBool(..) => true,
            _ => false,
        }
    }
    
    pub fn is_string(&self) -> bool {
        match &self.entity {
            JsonEntity::JsonEntityString(..) => true,
            _ => false,
        }
    }
    
    pub fn is_array(&self) -> bool {
        match &self.entity {
            JsonEntity::JsonEntityArray(..) => true,
            _ => false,
        }
    }
    
    pub fn is_object(&self) -> bool {
        match &self.entity {
            JsonEntity::JsonEntityObject(..) => true,
            _ => false,
        }
    }
    
}

impl From<JsonNull> for Json {
    fn from(val: JsonNull) -> Self {
        Json {
            entity: JsonEntityNull(val),
        }
    }
}

impl From<JsonBool> for Json {
    fn from(val: JsonBool) -> Self {
        Json {
            entity: JsonEntityBool(val),
        }
    }
}

impl From<JsonNumber> for Json {
    fn from(val: JsonNumber) -> Self {
        Json {
            entity: JsonEntityNumber(val),
        }
    }
}

impl From<JsonString> for Json {
    fn from(val: JsonString) -> Self {
        Json {
            entity: JsonEntityString(val),
        }
    }
}

impl From<JsonArray> for Json {
    fn from(val: JsonArray) -> Self {
        Json {
            entity: JsonEntityArray(val),
        }
    }
}

impl From<JsonObject> for Json {
    fn from(val: JsonObject) -> Self {
        Json {
            entity: JsonEntityObject(val),
        }
    }
}

impl Default for Json {
    fn default() -> Self {
        Json::new()
    }
}

macro_rules! impl_json_fmt {
    ($Self: ident, $FmtStr: literal, $Format: ident) => {
        match &$Self.entity {
            JsonEntityNull(x) => {
                write!($Format, $FmtStr, x)
            },
            JsonEntityBool(x) => {
                write!($Format, $FmtStr, x)
            },
            JsonEntityNumber(x) => {
                write!($Format, $FmtStr, x)
            },
            JsonEntityString(x) => {
                write!($Format, $FmtStr, x)
            },
            JsonEntityArray(x) => {
                write!($Format, $FmtStr, x)
            },
            JsonEntityObject(x) => {
                write!($Format, $FmtStr, x)
            }
        }
    };
}

impl Debug for Json {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        impl_json_fmt!(self, "{:?}", f)
    }
}

impl Display for Json {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        impl_json_fmt!(self, "{:}", f)
    }
}

