use crate::encoding::json::json_array::JsonArray;
use crate::encoding::json::json_object::JsonObject;
use crate::encoding::json::json_null::JsonNull;
use crate::encoding::json::json_bool::JsonBool;
use crate::encoding::json::json_string::JsonString;
use crate::encoding::json::json_number::JsonNumber;

#[derive(Clone)]
enum JsonEntity {
    JsonEntityNone,
    JsonEntityNull(JsonNull),
    JsonEntityBool(JsonBool),
    JsonEntityNumber(JsonNumber),
    JsonEntityString(JsonString),
    JsonEntityArray(JsonArray),
    JsonEntityObject(JsonObject)
}

use JsonEntity::{JsonEntityNull, JsonEntityBool, JsonEntityString, JsonEntityNumber, JsonEntityArray, JsonEntityObject, JsonEntityNone};
use std::fmt::{Debug, Formatter, Display};
use crate::encoding::json::{JsonError, JsonErrorKind, JsonFormatter};
use crate::encoding::{Encoder, Decoder};

#[derive(Clone)]
pub struct Json {
    entity: JsonEntity,
}

impl Json {
    pub fn new() -> Json {
        Json {
            entity: JsonEntityNone
        }
    }
    
    fn entity_name(&self) -> &'static str {
        match &self.entity {
            JsonEntityNull(..) => "JsonNull",
            JsonEntityBool(..) => "JsonBool",
            JsonEntityNumber(..) => "JsonNumber",
            JsonEntityString(..) => "JsonString",
            JsonEntityArray(..) => "JsonArray",
            JsonEntityObject(..) => "JsonObject",
            JsonEntityNone => "",
        }
    }
    
    pub fn to_json_null(&self) -> Result<JsonNull, JsonError> {
        match &self.entity {
            JsonEntityNull(x) => Ok(x.clone()),
            JsonEntityNone => Err(JsonError {kind: JsonErrorKind::TryFromErr(String::from("cannot convert from none to JsonNull"))}),
            _ => Err(JsonError {kind: JsonErrorKind::TryFromErr(format!("cannot convert from {} to JsonNull", self.entity_name()))}),
        }
    }
    
    pub fn to_json_bool(&self) -> Result<JsonBool, JsonError> {
        match &self.entity {
            JsonEntityBool(x) => Ok(x.clone()),
            JsonEntityNone => Err(JsonError {kind: JsonErrorKind::TryFromErr(String::from("cannot convert from none to JsonBool"))}),
            _ => Err(JsonError {kind: JsonErrorKind::TryFromErr(format!("cannot convert from {} to JsonBool", self.entity_name()))}),
        }
    }
    
    pub fn to_json_number(&self) -> Result<JsonNumber, JsonError> {
        match &self.entity {
            JsonEntityNumber(x) => Ok(x.clone()),
            JsonEntityNone => Err(JsonError {kind: JsonErrorKind::TryFromErr(String::from("cannot convert from none to JsonNumber"))}),
            _ => Err(JsonError {kind: JsonErrorKind::TryFromErr(format!("cannot convert from {} to JsonNumber", self.entity_name()))}),
        }
    }
    
    pub fn to_json_string(&self) -> Result<JsonString, JsonError> {
        match &self.entity {
            JsonEntityString(x) => Ok(x.clone()),
            JsonEntityNone => Err(JsonError {kind: JsonErrorKind::TryFromErr(String::from("cannot convert from none to JsonString"))}),
            _ => Err(JsonError {kind: JsonErrorKind::TryFromErr(format!("cannot convert from {} to JsonString", self.entity_name()))}),
        }
    }
    
    /// 如果`self.is_none()`, 则会转为`[]`; 否则,  
    /// `null/bool/number/string/object` 会被转为`[null]/[bool]/[number]/[string]/[object]`;  
    /// `array`->`array`;  
    pub fn to_json_array(&self) -> Result<JsonArray, JsonError> {
        match &self.entity {
            JsonEntityNull(x) => Ok(JsonArray::from(x.clone())),
            JsonEntityBool(x) => Ok(JsonArray::from(x.clone())),
            JsonEntityNumber(x) => Ok(JsonArray::from(x.clone())),
            JsonEntityString(x) => Ok(JsonArray::from(x.clone())),
            JsonEntityArray(x) => Ok(JsonArray::from(x.clone())),
            JsonEntityObject(x) => Ok(JsonArray::from(x.clone())),
            JsonEntityNone => Ok(JsonArray::new()),
        }
    }

    /// 如果`self.is_none()`, 则会转为`[]`; 否则,  
    /// `object` -> `object`
    pub fn to_json_object(&self) -> Result<JsonObject, JsonError> {
        match &self.entity {
            JsonEntityObject(x) => Ok(JsonObject::from(x.clone())),
            JsonEntityNone => Ok(JsonObject::new(true)),
            _ => Err(JsonError {kind: JsonErrorKind::TryFromErr(format!("cannot convert from {} to JsonObject", self.entity_name()))}),
        }
    }
    
    pub fn is_none(&self) -> bool {
        match &self.entity {
            JsonEntity::JsonEntityNone => true,
            _ => false,
        }
    }
    
    pub fn is_null(&self) -> bool {
        match &self.entity {
            JsonEntity::JsonEntityNull(..) => true,
            _ => false,
        }
    }
    
    pub fn is_bool(&self) -> bool {
        match &self.entity { 
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

macro_rules! impl_beauty_ind {
    ($Dst:ident, $CurLvl: ident, $Jmatter: ident) => {
        for lvl in 0..$CurLvl {
            for _ in 0..$Jmatter.ind_char_num(lvl) {
                $Dst.push($Jmatter.ind_char(lvl));
            }
        }
    };
}

macro_rules! impl_beauty_line_feed {
    ($Dst:ident, $IsParentObj:ident, $CurLvl:ident, $Jmatter:ident) => {
        if $IsParentObj {
            if $Jmatter.is_obj_line_feed($CurLvl) {
                $Dst.push('\n');
            }
        } else {
            if $Jmatter.is_arr_line_feed($CurLvl) {
                $Dst.push('\n');
            }
        }
    };
    ($JsonObj:ident, $Dst:ident, $IsParentObj:ident, $CurLvl:ident, $Jmatter:ident) => {
        if $JsonObj.len() > 0 {
            if $IsParentObj {
                if $Jmatter.is_obj_line_feed($CurLvl) {
                    $Dst.pop();
                }
            } else {
                if $Jmatter.is_arr_line_feed($CurLvl) {
                    $Dst.pop();
                }
            }
            $Dst.pop();
            impl_beauty_line_feed!($Dst, $IsParentObj, $CurLvl, $Jmatter);
        }
    }
}

impl Json {
    fn beauty(&self, dst: &mut String, is_parent_obj: bool, cur_lvl: usize, jmatter: &JsonFormatter) {
        match &self.entity {
            JsonEntityNone => {},
            JsonEntityNull(json) => {
                dst.push_str(format!("{}", json).as_str());
                dst.push(',');
            },
            JsonEntityBool(json) => {
                dst.push_str(format!("{}", json).as_str());
                dst.push(',');
            },
            JsonEntityNumber(json) => {
                dst.push_str(format!("{}", json).as_str());
                dst.push(',');
            },
            JsonEntityString(json) => {
                dst.push_str(format!("{}", json).as_str());
                dst.push(',');
            },
            JsonEntityArray(json) => {
                dst.push('[');
                if jmatter.is_arr_line_feed(cur_lvl) {
                    dst.push('\n');
                }
                let next_lvl = cur_lvl + 1;

                for ele in json.iter() {
                    impl_beauty_ind!(dst, next_lvl, jmatter);
                    ele.beauty(dst,false,next_lvl, jmatter);
                    impl_beauty_line_feed!(dst, is_parent_obj, next_lvl, jmatter);
                }

                impl_beauty_line_feed!(json, dst, is_parent_obj, next_lvl, jmatter);
                impl_beauty_ind!(dst, cur_lvl, jmatter);
                dst.push(']');
                dst.push(',');

                if jmatter.is_arr_line_feed(cur_lvl) {
                    match dst.pop() {
                        Some('\n') => dst.push('\n'),
                        Some(x) => dst.push(x),
                        None => {},
                    }
                }
            },
            JsonEntityObject(json) => {
                dst.push('{');

                if jmatter.is_obj_line_feed(cur_lvl) {
                    dst.push('\n');
                }
                let next_lvl = cur_lvl+1;

                for ele in json.iter() {
                    impl_beauty_ind!(dst, next_lvl, jmatter);
                    dst.push('"');
                    dst.push_str(ele.0.as_str());
                    dst.push_str(r#"": "#);
                    ele.1.beauty(dst, true, next_lvl, jmatter);
                    impl_beauty_line_feed!(dst, is_parent_obj, next_lvl, jmatter);
                }

                impl_beauty_line_feed!(json, dst, is_parent_obj, next_lvl, jmatter);
                impl_beauty_ind!(dst, cur_lvl, jmatter);
                dst.push('}');
                dst.push(',');
                if jmatter.is_obj_line_feed(cur_lvl) {
                    match dst.pop() {
                        Some('\n') => dst.push('\n'),
                        Some(x) => dst.push(x),
                        None => {},
                    }
                }
            }
        } // match
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
            },
            JsonEntityNone => {
                write!($Format, $FmtStr, "")
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

impl Encoder<&str, ()> for Json {
    type Output = Json;
    type Error = JsonError;

    fn encode(&self, _dst: (), src: &str) -> Result<Self::Output, Self::Error> {
        let src = src.trim();
        
        if src.is_empty() {
            Ok(Json::new())
        } else {
            match src.chars().next() {
                Some('n') => match src.parse::<JsonNull>() {
                    Ok(json) => {
                        Ok(Json::from(json))
                    },
                    Err(e) => Err(e),
                },
                Some('t') | Some('f') => match src.parse::<JsonBool>() {
                    Ok(json) => {
                        Ok(Json::from(json))
                    },
                    Err(e) => Err(e),
                },
                Some('"') => match src.parse::<JsonString>() {
                    Ok(json) => {
                        Ok(Json::from(json))
                    },
                    Err(e) => Err(e),
                }
                Some('{') => match src.parse::<JsonObject>() {
                    Ok(json) => {
                        Ok(Json::from(json))
                    },
                    Err(e) => Err(e),
                },
                Some('[') => match src.parse::<JsonArray>() {
                    Ok(json) => {
                        Ok(Json::from(json))
                    },
                    Err(e) => Err(e),
                },
                Some(x) if x == '-' || x.is_digit(10) => match src.parse::<JsonNumber>() {
                    Ok(json) => {
                        Ok(Json::from(json))
                    },
                    Err(e) => Err(e),
                },
                _ => {
                    Err(JsonError {
                        kind: JsonErrorKind::Other(String::from("invalid json text")),
                    })
                }
            }
        }
    }
}

impl Encoder<&str, &mut Json> for Json {
    type Output = ();
    type Error = JsonError;

    fn encode(&self, dst: &mut Json, src: &str) -> Result<Self::Output, Self::Error> {
        match self.encode((), src) {
            Ok(json) => {
                *dst = json;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }
}

impl Encoder<&str, &mut Vec<u8>> for Json {
    type Output = ();
    type Error = JsonError;

    fn encode(&self, dst: &mut Vec<u8>, src: &str) -> Result<Self::Output, Self::Error> {
        dst.clear();
        match self.encode((), src) {
            Ok(json) => {
                use std::io::Write;
                match write!(dst, "{}", json) {
                    Err(e) => Err(JsonError {
                        kind: JsonErrorKind::Other(format!("{}", e))
                    }),
                    Ok(..) => {
                        Ok(())
                    }
                }
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

impl Decoder<&Json, &mut String> for Json {
    type Output = ();
    type Error = std::fmt::Error;

    fn decode(&self, dst: &mut String, src: &Json) -> Result<Self::Output, Self::Error> {
        use std::fmt::Write;
        dst.clear();
        match write!(dst, "{}", src) {
            Ok(..) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

impl Decoder<&Json, &mut Vec<u8>> for Json {
    type Output = ();
    type Error = std::io::Error;

    fn decode(&self, dst: &mut Vec<u8>, src: &Json) -> Result<Self::Output, Self::Error> {
        use std::io::Write;
        dst.clear();
        match write!(dst, "{}", src) {
            Ok(..) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

impl Decoder<&JsonFormatter, &mut Vec<u8>> for Json {
    type Output = ();
    type Error = std::io::Error;

    fn decode(&self, dst: &mut Vec<u8>, src: &JsonFormatter) -> Result<Self::Output, Self::Error> {
        let mut s = String::with_capacity(2048);
        match self.decode(&mut s, src) {
            Ok(x) => {
                dst.clear();
                unsafe {
                    dst.append(s.as_mut_vec());
                }
                Ok(x)
            },
            Err(e) => Err(e),
        }
    }
}

impl Decoder<&JsonFormatter, &mut String> for Json {
    type Output = ();
    type Error = std::io::Error;

    fn decode(&self, dst: &mut String, src: &JsonFormatter) -> Result<Self::Output, Self::Error> {
        dst.clear();
        self.beauty(dst, false, 0, src);
        while let Some(x) = dst.pop() {
            if x != '\n' && x != ',' {
                dst.push(x);
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read};
    use crate::encoding::json::{Json};
    use crate::encoding::{Encoder, Decoder};

    #[test]
    fn json() {
        let mut file = std::fs::File::open("./src/encoding/json/testdata/code.json").unwrap();
        let mut data = String::with_capacity(2048);
        file.read_to_string(&mut data).unwrap();
        let j = Json::new();
        let json = j.encode((), data.as_str()).unwrap();
        // let mut file = std::fs::File::create("./src/encoding/json/testdata/code_by_json.json").unwrap();
        let json_str = format!("{}", json);
        // file.write_all(json_str.as_bytes()).unwrap();
        assert_eq!(data, json_str);
        let mut json_str = String::with_capacity(2048);
        json.decode(&mut json_str, &json).unwrap();
        assert_eq!(data, json_str);
    }
}
