use std::fmt::{Display, Formatter, Debug};
use crate::encoding::json::{Json, JsonError, JsonErrorKind, JsonNull, JsonBool, JsonString, JsonObject};
use std::str::{FromStr, Chars};

const LINE_FEED_LEN: usize = 80;

#[derive(Clone)]
pub struct  JsonArray {
    arr: Vec<Json>,
    is_space: bool,
    line_feed_len: usize,
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
            is_space: false,
            line_feed_len: LINE_FEED_LEN,
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
            is_space: false,
            line_feed_len: LINE_FEED_LEN,
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
    
    pub fn set_is_space(&mut self, is_space: bool) -> &Self {
        self.is_space = is_space;
        &*self
    }
    
    pub fn set_line_feed_len(&mut self, line_feed_len: usize) -> &Self {
        self.line_feed_len = if line_feed_len < LINE_FEED_LEN { LINE_FEED_LEN } else { line_feed_len };
        &*self
    }
    
    pub fn is_space(&self) -> bool {
        self.is_space
    }
    
    pub fn line_feed_len(&self) -> usize {
        self.line_feed_len
    }
    
    fn cvt_to_string(&self, buf: &mut String) {
        buf.push('[');
        
        for ele in self.iter() {
            buf.push_str(ele.to_string().as_str());
            buf.push(',');
            if self.is_space() {
                buf.push(' ');
            }
            if buf.len() > self.line_feed_len() {
                #[cfg(target_os = "windows")]
                buf.push('\r');
                
                buf.push('\n');
            }
        }
        
        if !buf.is_empty() {
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

        let mut itr = s.chars();
        let mut is_need_comma = false;
        loop {
            idx += 1;
            match itr.next() {
                Some(x) => {
                    if x.is_whitespace() {
                        idx += 1;
                    } else if x == ',' {
                        if is_need_comma {
                            is_need_comma = false;
                            idx += 1;
                        } else {
                            return Err(JsonError {
                                kind: JsonErrorKind::ParseJsonArrayError {
                                    des: String::from("doesn't matched comma"),
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
        len += 1;
    }
    
    if is_quote {
        Some(len)
    } else {
        None
    }
}

// 在itr中寻找`]`or`}`, 找到返回经过的字符个数, 否则返回Err  
pub fn find_bracket_auxiliary(itr: &mut Chars, bracket: char, idx: usize) -> Result<usize, JsonError> {
    let mut len = 0;
    let mut is_bracket = false;
    while let Some(x) = itr.next() {
        len += 1;
        // 字符串中可能会出现'}', 跳过字符串
        if x == '"' {
            match find_quote_auxiliary(itr) {
                Some(x_len) => {
                    len += x_len + 1;
                },
                None => {
                    return Err(JsonArray::err(idx+len, "object(doesn't matched '\"')"));
                }
            }
        } else if x == bracket {
            is_bracket = true;
            break;
        }
    }
    
    if is_bracket {
        Ok(len)
    } else {
        Err(JsonArray::err(idx, "object(doesn't matched '} or ]'"))
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
                    let sub_str = &s[*idx..(*idx+len)];
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

            match find_bracket_auxiliary(itr, ']', *idx) {
                Ok(len) => {
                    let sub_s = &s[(*idx-1)..(*idx+len)];
                    match sub_s.parse::<JsonArray>() {
                        Ok(x) => {
                            *idx += len;
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

            match find_bracket_auxiliary(itr, '}', *idx) {
                Ok(len) => {
                    let sub_s = &s[(*idx-1)..(*idx+len)];
                    match sub_s.parse::<JsonObject>() {
                        Ok(x) => {
                            *idx += len;
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
        x => {
            Err(JsonError {
                kind: JsonErrorKind::ParseJsonArrayError {
                    des: format!("unknown character '{}'", x),
                }
            })
        },
    } // match
}
