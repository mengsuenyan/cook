//! 
//! ECMA-404: The JSON Data Interchange Format(2ed Edition December 2017);
//! 

mod json_array;
mod json_object;
mod json_number;
mod json_bool;
mod json_null;
mod json_string;
mod json;

pub use json_array::{JsonArray, JsonArrayIter, JsonArrayIterMut, JsonArrayIntoIter};
pub use json_object::{JsonObject, JsonObjectIter, JsonObjectIterMut, JsonObjectIntoIter};
pub use json_number::JsonNumber;
pub use json_bool::JsonBool;
pub use json_null::JsonNull;
pub use json_string::JsonString;
pub use json::Json;
use std::fmt::{Debug, Formatter, Display};

#[derive(Clone)]
enum JsonErrorKind {
    ParseJsonNullError{des: String},
    ParseJsonBoolError{des: String},
    ParseJsonNumberError{des: String},
    ParseJsonStringError{des: String},
    ParseJsonArrayError{des: String},
    ParseJsonObjectError{des: String},
    TryFromErr(String),
    Other(String),
}

#[derive(Clone)]
pub struct JsonError {
    kind: JsonErrorKind
}

impl JsonError {
    fn description(&self) -> &str {
        match &self.kind {
            JsonErrorKind::ParseJsonNullError {des, ..} => des.as_str(),
            JsonErrorKind::ParseJsonBoolError {des, ..} => des.as_str(),
            JsonErrorKind::ParseJsonNumberError {des, ..} => des.as_str(),
            JsonErrorKind::ParseJsonArrayError {des, ..} => des.as_str(),
            JsonErrorKind::ParseJsonObjectError {des, ..} => des.as_str(),
            JsonErrorKind::ParseJsonStringError {des, ..} => des.as_str(),
            JsonErrorKind::TryFromErr(des) => des.as_str(),
            JsonErrorKind::Other(des) => des.as_str(),
        }
    }
}

impl Debug for JsonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description())
    }
}

impl Display for JsonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description())
    }
}


