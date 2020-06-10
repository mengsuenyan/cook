use std::fmt::{Display, Formatter, Debug};
use crate::encoding::json::{Json, JsonError, JsonErrorKind, JsonNull, JsonBool, JsonString, JsonObject, JsonNumber};
use std::str::{FromStr, Chars};

#[derive(Clone)]
pub struct  JsonArray {
    arr: Vec<Json>,
}

#[derive(Clone)]
pub struct JsonArrayIter<'a> {
    itr: std::slice::Iter<'a, Json>
}

impl<'a> Iterator for JsonArrayIter<'a> {
    type Item = &'a Json;

    fn next(&mut self) -> Option<Self::Item> {
        self.itr.next()
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.itr.size_hint()
    }
}

pub struct JsonArrayIterMut<'a> {
    itr: std::slice::IterMut<'a, Json>
}

impl<'a> Iterator for JsonArrayIterMut<'a> {
    type Item = &'a mut Json; 

    fn next(&mut self) -> Option<Self::Item> {
        self.itr.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.itr.size_hint()
    }
}

pub struct JsonArrayIntoIter {
    itr: std::vec::IntoIter<Json>
}

impl Iterator for JsonArrayIntoIter {
    type Item = Json;

    fn next(&mut self) -> Option<Self::Item> {
        self.itr.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.itr.size_hint()
    }
}

impl JsonArray {
    pub fn new() -> JsonArray {
        JsonArray {
            arr: Vec::new(),
        }
    }
    
    pub fn clear(&mut self) -> &Self {
        self.arr.clear();
        &*self
    }
    
    pub fn iter(&self) -> JsonArrayIter {
        JsonArrayIter {
            itr: self.arr.iter()
        }
    }
    
    pub fn iter_mut(&mut self) -> JsonArrayIterMut {
        JsonArrayIterMut {
            itr: self.arr.iter_mut()
        }
    }
    
    pub fn with_capacity(capacity: usize) -> JsonArray {
        JsonArray {
            arr: Vec::with_capacity(capacity),
        }
    }
    
    pub fn push(&mut self, val: Json) {
        self.arr.push(val)
    }
    
    pub fn is_empty(&self) -> bool {
        self.arr.is_empty()
    }
    
    pub fn pop(&mut self) -> Option<Json> {
        self.arr.pop()
    }
    
    pub fn truncate(&mut self, len: usize) {
        self.arr.truncate(len)
    }
    
    pub fn capacity(&self) -> usize {
        self.arr.capacity()
    }
    
    pub fn len(&self) -> usize {
        self.arr.len()
    }
    
    fn cvt_to_string(&self, buf: &mut String) {
        buf.push('[');
        
        for ele in self.iter() {
            buf.push_str(ele.to_string().as_str());
            buf.push(',');
        }
        
        if buf.len() > 1 {
            buf.pop();
        }
        buf.push(']');
    }
    
    fn err(idx: usize, want: &str) -> JsonError {
        JsonError {
            kind: JsonErrorKind::ParseJsonArrayError {
                des: format!("trying transform to `{}` wrong in the position {}", want, idx),
            }
        }
    }
}

impl Default for JsonArray {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for JsonArray {
    type Item = Json;
    type IntoIter = JsonArrayIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        JsonArrayIntoIter {
            itr: self.arr.into_iter()
        }
    }
}

impl Display for JsonArray {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::with_capacity(self.len() << 5);
        self.cvt_to_string(&mut buf);
        f.write_str(buf.as_str())
    }
}

impl Debug for JsonArray {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::with_capacity(self.len() << 5);
        self.cvt_to_string(&mut buf);
        f.write_str(buf.as_str())
    }
}

impl From<Json> for JsonArray {
    fn from(val: Json) -> Self {
        JsonArray {
            arr: vec![val],
        }
    }
}

impl From<JsonNull> for JsonArray {
    fn from(val: JsonNull) -> Self {
        JsonArray::from(Json::from(val))
    }
}

impl From<JsonBool> for JsonArray {
    fn from(val: JsonBool) -> Self {
        JsonArray::from(Json::from(val))
    }
}

impl From<JsonNumber> for JsonArray {
    fn from(val: JsonNumber) -> Self {
        JsonArray::from(Json::from(val))
    }
}

impl From<JsonString> for JsonArray {
    fn from(val: JsonString) -> Self {
        JsonArray::from(Json::from(val))
    }
}

impl From<JsonObject> for JsonArray {
    fn from(val: JsonObject) -> Self {
        JsonArray::from(Json::from(val))
    }
}

impl FromStr for JsonArray {
    type Err = JsonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Ok(JsonArray::new());
        }

        if s.len() < 2 || (s.chars().next() != Some('[') || s.chars().last() != Some(']')){
            return Err(JsonError {
                kind: JsonErrorKind::ParseJsonArrayError {
                    des: format!("Only the pattern of `[(value), *]` can be transformed to JsonArray"),
                }
            })
        }

        let s = &s[1..(s.len()-1)];
        let s = s.trim();
        
        // 下一个要迭代字符的索引
        let mut idx = 0usize;
        let mut arr = JsonArray::new();
        
        if s.is_empty() {
            return Ok(arr);
        }

        let mut itr = s.chars();
        let mut is_need_comma = false;
        loop {
            match itr.next() {
                Some(x) => {
                    idx += x.len_utf8();
                    
                    if x.is_whitespace() {
                        continue;
                    } else if x == ',' {
                        if is_need_comma {
                            is_need_comma = false;
                        } else {
                            return Err(JsonError {
                                kind: JsonErrorKind::ParseJsonArrayError {
                                    des: String::from("doesn't match comma"),
                                }
                            });
                        }
                    } else {
                        match find_value_auxiliary(x, &mut itr, is_need_comma, s, &mut idx) {
                            Ok(json) => {
                                arr.push(json);
                                is_need_comma = true;
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                },
                None => {
                    break;
                },
            }
        } // loop
        
        if !is_need_comma {
            Err(JsonError {
                kind: JsonErrorKind::ParseJsonArrayError {
                    des: format!("unnessary , in the position `{}`", idx),
                }
            })
        } else {
            Ok(arr)
        }
    } // fn
}

// 在itr中寻找'"', 找到返回迭代经过的字符个数, 否则返回None
pub fn find_quote_auxiliary(itr: &mut Chars) -> Option<usize> {
    let mut len = 0;
    let mut backslach_cnt = 0; // 连续'\'的个数
    let mut is_quote = false;
    while let Some(x) = itr.next() {
        if x == '"' {
            if (backslach_cnt & 0x1) == 0 {
                is_quote = true;
                break;
            } else {
                backslach_cnt = 0;
            }
        } else if x == '\\' {
            backslach_cnt += 1;
        } else {
            backslach_cnt = 0;
        }
        len += x.len_utf8();
    }
    
    if is_quote {
        Some(len)
    } else {
        None
    }
}

// 在itr中寻找`]`or`}`, 找到返回经过的字符个数, 否则返回Err  
pub fn find_bracket_auxiliary(itr: &mut Chars, tgt: char, bracket: char, idx: usize) -> Result<usize, JsonError> {
    let mut len = 0;
    let mut is_bracket = false;
    
    let mut tgt_cnt = 0;
    while let Some(x) = itr.next() {
        // 字符串中可能会出现'}', 跳过字符串
        if x == '"' {
            match find_quote_auxiliary(itr) {
                Some(x_len) => {
                    len += x_len + 1;
                },
                None => {
                    return Err(JsonArray::err(idx+len, "object(doesn't match '\"')"));
                }
            }
        } else if x == tgt {
            tgt_cnt += 1;
        } else if x == bracket {
            if tgt_cnt > 0 {
                tgt_cnt -= 1;
            } else {
                is_bracket = true;
                break;
            }
        }
        len += x.len_utf8();
    }
    
    if is_bracket {
        Ok(len)
    } else {
        Err(JsonArray::err(idx, "object(doesn't match '} or ]'"))
    }
}

// 在itr中查找键值, 如果成功则返回(下一次是否需要查找`,`, 经过的字符个数, 本次解析的Json值),  
// 否则返回Err
pub fn find_value_auxiliary(cur_char: char, itr: &mut Chars, is_need_comma: bool, s: &str, idx: &mut usize) -> Result<Json, JsonError> {
    match cur_char {
        'n' => {
            if is_need_comma {
                return Err(JsonArray::err(*idx, "null(need , before null)"));
            }

            if itr.next() == Some('u') && itr.next() == Some('l') && itr.next() == Some('l') {
                *idx += 3;
                Ok(Json::from(JsonNull::new()))
            } else {
                Err(JsonArray::err(*idx, "null"))
            }
        },
        't' => {
            if is_need_comma {
                return Err(JsonArray::err(*idx, "true(need , before true)"));
            }

            if itr.next() == Some('r') && itr.next() == Some('u') && itr.next() == Some('e') {
                *idx += 3;
                Ok(Json::from(JsonBool::new(true)))
            } else {
                Err(JsonArray::err(*idx, "true"))
            }
        },
        'f' => {
            if is_need_comma {
                return Err(JsonArray::err(*idx, "false(need , before false)"));
            }

            if itr.next() == Some('a') && itr.next() == Some('l') && itr.next() == Some('s') && itr.next() == Some('e'){
                *idx += 4;
                Ok(Json::from(JsonBool::new(false)))
            } else {
                Err(JsonArray::err(*idx, "false"))
            }
        },
        '"' => {
            if is_need_comma {
                return Err(JsonArray::err(*idx, "string(need , before string)"));
            }

            match find_quote_auxiliary(itr) {
                Some(len) => {
                    let sub_str = &s[(*idx)..(*idx+len)];
                    *idx += len + 1;
                    Ok(Json::from(JsonString::from(sub_str)))
                },
                None => {
                    Err(JsonArray::err(*idx, "string"))
                }
            }
        },
        '[' => {
            if is_need_comma {
                return Err(JsonArray::err(*idx, "array(need , before array)"));
            }

            match find_bracket_auxiliary(itr, '[', ']', *idx) {
                Ok(len) => {
                    // [...]
                    let sub_s = &s[(*idx-1)..(*idx+len+1)];
                    match sub_s.parse::<JsonArray>() {
                        Ok(x) => {
                            *idx += len + 1;
                            Ok(Json::from(JsonArray::from(x)))
                        },
                        Err(e) => {
                            Err(e)
                        }
                    }
                },
                Err(e) => {
                    Err(e)
                }
            }
        },
        '{' => {
            if is_need_comma {
                return Err(JsonArray::err(*idx, "object(need , before object)"));
            }

            match find_bracket_auxiliary(itr, '{', '}', *idx) {
                Ok(len) => {
                    // {...}
                    let sub_s = &s[(*idx-1)..(*idx+len+1)];
                    match sub_s.parse::<JsonObject>() {
                        Ok(x) => {
                            *idx += len + 1;
                            Ok(Json::from(JsonObject::from(x)))
                        },
                        Err(e) => {
                            Err(e)
                        }
                    }
                },
                Err(e) => {
                    Err(e)
                }
            }
        },
        x if x == '-' || x.is_digit(10) => {
            if is_need_comma {
                return Err(JsonArray::err(*idx, "number(need , before number)"));
            }
            
            let mut len = 0;
            let mut itr_copy = itr.clone();
            while let Some(x) = itr_copy.next() {
                if x != '.' && !x.is_digit(10) {
                    break;
                } else {
                    itr.next();
                    len += x.len_utf8();
                }
            }

            let sub_s = &s[(*idx-1)..(*idx + len)];
            *idx += len;
            match sub_s.parse::<JsonNumber>() {
                Ok(json) => Ok(Json::from(json)),
                Err(e) => Err(e),
            }
        },
        x => {
            Err(JsonError {
                kind: JsonErrorKind::ParseJsonArrayError {
                    des: format!("unknown character '{}' in the position {}", x, *idx),
                }
            })
        },
    } // match
}

#[cfg(test)]
mod tests {
    use crate::encoding::json::JsonArray;

    #[test]
    fn json_array() {
        let cases = [
            "[[0]]",
            r#"["庄子", 3.1415926, null, true, false, ["<<逍遥游>>", "<<齐物论>>"]]"#,
            r#"["庄子", 3.1415926, null, true, false]"#,
            "[]",
            r#"["庄子", 3.1415926, null, true, false, ["<<逍遥游>>", "<<齐物论>>"], 
            {"逍遥游": "抟扶摇而上者九万里", "齐物论": "大智闲闲, 小智间间", "name": null, "is_exist": true, "year": 2020}]"#,
        ];
        
        for &data in cases.iter() {
            let json = data.parse::<JsonArray>();
            assert!(json.is_ok());
            json.unwrap();
            // println!("{}", json);
        }
    }
}
