use crate::encoding::json::{Json, JsonError, JsonErrorKind};
use std::fmt::{Debug, Formatter, Display};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::{DefaultHasher};
use std::str::FromStr;
use crate::encoding::json::json_array;
use std::convert::TryFrom;

#[derive(Clone)]
pub struct JsonObject {
    obj: Vec<(u64, String, Json)>,
    is_unique_key: bool,
}

#[derive(Clone)]
pub struct JsonObjectIter<'a> {
    obj: std::slice::Iter<'a, (u64, String, Json)>,
}

impl<'a> Iterator for JsonObjectIter<'a> {
    type Item = (&'a String, &'a Json);

    fn next(&mut self) -> Option<Self::Item> {
        match self.obj.next() {
            Some(v) => {
                Some((&v.1, &v.2))
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.obj.size_hint()
    }
}

pub struct JsonObjectIterAll<'a> {
    obj: std::slice::Iter<'a, (u64, String, Json)>,
}

impl<'a> Iterator for JsonObjectIterAll<'a> {
    type Item = (&'a u64, &'a String, &'a Json);

    fn next(&mut self) -> Option<Self::Item> {
        match self.obj.next() {
            Some(v) => {
                Some((&v.0, &v.1, &v.2))
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.obj.size_hint()
    }
}

pub struct JsonObjectIterAllMut<'a> {
    obj: std::slice::IterMut<'a, (u64, String, Json)>,
}

impl<'a> Iterator for JsonObjectIterAllMut<'a> {
    type Item = (&'a u64, &'a String, &'a mut Json);

    fn next(&mut self) -> Option<Self::Item> {
        match self.obj.next() {
            Some(v) => {
                Some((&v.0 , &v.1, &mut v.2))
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.obj.size_hint()
    }
}

pub struct JsonObjectIterMut<'a> {
    obj: std::slice::IterMut<'a, (u64, String, Json)>,
}

impl<'a> Iterator for JsonObjectIterMut<'a> {
    type Item = (&'a String, &'a mut Json);

    fn next(&mut self) -> Option<Self::Item> {
        match self.obj.next() {
            Some(v) => {
                Some((&mut v.1, &mut v.2))
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.obj.size_hint()
    }
}

pub struct JsonObjectIntoIter {
    obj: std::vec::IntoIter<(u64, String, Json)>,
}

impl Iterator for JsonObjectIntoIter {
    type Item = (String, Json);

    fn next(&mut self) -> Option<Self::Item> {
        match self.obj.next() {
            Some(v) => {
                Some((v.1, v.2))
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.obj.size_hint()
    }
}

impl JsonObject {
    pub fn new(is_unique_key: bool) -> JsonObject {
        JsonObject {
            obj: Vec::new(),
            is_unique_key,
        }
    }
    
    pub fn iter(&self) -> JsonObjectIter {
        JsonObjectIter {
            obj: self.obj.iter(),
        }
    }
    
    pub fn iter_mut(&mut self) -> JsonObjectIterMut {
        JsonObjectIterMut {
            obj: self.obj.iter_mut()
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.obj.is_empty()
    }
    
    pub fn is_unique_key(&self) -> bool {
        self.is_unique_key
    }
    
    pub fn clear(&mut self) -> &Self {
        self.obj.clear();
        &*self
    }
    
    pub fn len(&self) -> usize {
        self.obj.len()
    }
    
    pub fn with_capacity(capacity: usize, is_unique_key: bool) -> JsonObject {
        JsonObject {
            obj: Vec::with_capacity(capacity),
            is_unique_key,
        }
    }
    
    fn hash_string(key: &String) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
    
    fn iter_all(&self) -> JsonObjectIterAll {
        JsonObjectIterAll {
            obj: self.obj.iter(),
        }
    }

    fn iter_all_mut(&mut self) -> JsonObjectIterAllMut {
        JsonObjectIterAllMut {
            obj: self.obj.iter_mut(),
        }
    }
    
    fn find_mut(&mut self, hash: &u64, key: &String) -> Option<&mut Json> {
        match self.iter_all_mut().find(|x| {
            x.0 == hash && key == x.1
        }) {
            Some(k) => {
                Some(k.2)
            },
            None => None,
        }
    }
    
    fn find(&self, hash: &u64, key: &String) -> Option<&Json> {
        match self.iter_all().find(|x| {
            x.0 == hash && key == x.1
        }) {
            Some(k) => {
                Some(k.2)
            },
            None => None,
        }
    }
    
    /// 当设置唯一键值时, 如果key已经存在, 那么返回旧值, 否认则None;  
    /// 当设置键值可重复时, 那么一直返回None;  
    pub fn insert(&mut self, key: String, val: Json) -> Option<Json> {
        let hash = Self::hash_string(&key);
        if self.is_unique_key {
            match self.find_mut(&hash, &key) {
                Some(x) => {
                    Some(std::mem::replace(x, val))
                },
                _ => {
                    self.obj.push((hash, key, val));
                    None
                }
            }
        } else {
            self.obj.push((hash, key, val));
            None
        }
    }
    
    pub fn get(&self, key: &String) -> Option<&Json> {
        let hash = Self::hash_string(&key);
        self.find(&hash, key)
    }

    pub fn get_mut(&mut self, key: &String) -> Option<&mut Json> {
        let hash = Self::hash_string(&key);
        self.find_mut(&hash, key)
    }
    
    pub fn contains_key(&self, key: &String) -> bool {
        self.get(key).is_some()
    }
    
    pub fn cvt_to_string(&self, buf: &mut String) {
        buf.push('{');
        
        for ele in self.iter() {
            buf.push('"');
            buf.push_str(ele.0.as_str());
            buf.push('"');
            buf.push(':');
            buf.push_str(ele.1.to_string().as_str());
            buf.push(',');
        }
        
        if buf.len() > 1 {
            buf.pop();
        }
        
        buf.push('}');
    }

    fn err(idx: usize, want: &str) -> JsonError {
        JsonError {
            kind: JsonErrorKind::ParseJsonArrayError {
                des: format!("trying transform to `{}` wrong in the position {}", want, idx),
            }
        }
    }

    pub fn parse_from_str(s: &str, is_unique_key: bool) -> Result<Self, JsonError> {
        let s = s.trim();
        if s.is_empty() {
            return Ok(JsonObject::new(is_unique_key));
        }

        if s.len() < 2 || s.chars().next() != Some('{') || s.chars().last() != Some('}') {
            return Err(JsonError {
                kind: JsonErrorKind::ParseJsonObjectError {
                    des: format!("Only the pattern of `{{(key: value), *}}` can be transformed to JsonObject"),
                }
            })
        }

        let mut obj = JsonObject::new(is_unique_key);
        let s = &s[1..(s.len()-1)];
        let s = s.trim();
        if s.is_empty() {
            return Ok(obj);
        }
        
        let (mut itr, mut idx, mut is_need_comma) = (s.chars(), 0, false);
        
        loop {
            match itr.next() {
                Some('"') => {
                    idx += 1;
                    match json_array::find_quote_auxiliary(&mut itr) {
                        Some(len) => {
                            // find key
                            let sub_s = &s[idx..(idx+len)];
                            idx += len + 1;

                            let mut is_find_colon = false;
                            while let Some(x) = itr.next() {
                                idx += 1;
                                if x == ':' {
                                    if !is_find_colon {
                                        is_find_colon = true;
                                    } else {
                                        return Err(Self::err(idx, "object(find more than one `:` between key and value)"));
                                    }
                                } else if x.is_whitespace() {
                                    continue;
                                } else {
                                    if is_find_colon {
                                        match json_array::find_value_auxiliary(x, &mut itr, is_need_comma, s, &mut idx) {
                                            Ok(json) => {
                                                obj.insert(String::from(sub_s), json);
                                                break;
                                            },
                                            Err(e) => {
                                                return Err(e);
                                            }
                                        }
                                    } else {
                                        return Err(Self::err(idx, "object(not find `:` between key and value)"));
                                    }
                                }
                            }
                            
                            is_need_comma = true;
                        },
                        None => {
                            return Err(Self::err(idx, "object(can't find match `\"`)"));
                        }
                    }
                }, // "key": value
                Some(',') => {
                    if is_need_comma {
                        is_need_comma = false;
                        idx += 1;
                    } else {
                        return Err(Self::err(idx, "object(unnecessary comma `,`)"));
                    }
                },
                Some(x) if x.is_whitespace() => {
                    idx += x.len_utf8();
                },
                Some(x) => {
                    return Err(JsonError {
                        kind: JsonErrorKind::ParseJsonObjectError {
                            des: format!("Unknown character `{}` in the position {}", x, idx),
                        }
                    });
                },
                None => {
                    break;
                }
            } // match
        } // loop
        

        Ok(obj)
    } // fn
}

impl Default for JsonObject {
    /// 默认键值唯一  
    fn default() -> Self {
        Self::new(true)
    }
}


impl IntoIterator for JsonObject {
    type Item = (String, Json);
    type IntoIter = JsonObjectIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        JsonObjectIntoIter {
            obj: self.obj.into_iter()
        }
    }
}

impl Debug for JsonObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::with_capacity(self.len() << 5);
        self.cvt_to_string(&mut buf);
        f.write_str(buf.as_str())
    }
}

impl Display for JsonObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::with_capacity(self.len() << 5);
        self.cvt_to_string(&mut buf);
        f.write_str(buf.as_str())
    }
}

impl FromStr for JsonObject {
    type Err = JsonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JsonObject::parse_from_str(s, false)
    }
}

impl TryFrom<Json> for JsonObject {
    type Error = JsonError;

    fn try_from(value: Json) -> Result<Self, Self::Error> {
        value.to_json_object()
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding::json::JsonObject;

    #[test]
    fn json_object() {
        let cases = [
            "{}",
            r#"{"nu": {}}"#,
            r#"
{
  "abi-blacklist": [
    "stdcall",
    "fastcall",
    "vectorcall",
    "thiscall",
    "win64",
    "sysv64"
  ],
  "arch": "arm",
  "atomic-cas": false,
  "cpu": "arm7tdmi",
  "data-layout": "e-m:e-p:32:32-i64:64-v128:64:128-a:0:32-n32-S64",
  "emit-debug-gdb-scripts": false,
  "env": "agb",
  "executables": true,
  "features": "+soft-float,+strict-align",
  "linker": "arm-none-eabi-ld",
  "linker-flavor": "ld",
  "linker-is-gnu": true,
  "llvm-target": "thumbv4-none-agb",
  "os": "none",
  "panic-strategy": "abort",
  "pre-link-args": {
    "ld": [
      "-Tlinker.ld"
    ]
  },
  "relocation-model": "static",
  "target-c-int-width": "32",
  "target-endian": "little",
  "target-pointer-width": "32",
  "vendor": "nintendo"
}
            "#
        ];

        for &ele in cases.iter() {
            let json = ele.parse::<JsonObject>();
            assert!(json.is_ok());
            json.unwrap();
            // println!("{}", json);
        }
    }
}
