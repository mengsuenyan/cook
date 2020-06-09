use crate::encoding::json::{Json, JsonError, JsonErrorKind};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Display};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::{DefaultHasher};
use std::str::FromStr;
use crate::encoding::json::json_array;

#[derive(Clone)]
enum JsonSubObject {
    UniqueKey(HashMap<String, Json>),
    RepeatKey(Vec<(u64, String, Json)>),
}

#[derive(Clone)]
pub struct JsonObject {
    obj: JsonSubObject,
    is_beauty: bool,
}

#[derive(Clone)]
pub struct JsonObjectIter<'a> {
    itr_uk: Option<std::collections::hash_map::Iter<'a, String, Json>>,
    itr_rk: Option<std::slice::Iter<'a, (u64, String, Json)>>,
}

macro_rules! impl_jsonobjecti_next_hint {
    ($self: ident) => {
        match &$self.itr_uk {
            Some(x) => {
                x.size_hint()
            },
            None => {
                $self.itr_rk.as_ref().unwrap().size_hint()
            }
        }
    };
}

impl<'a> Iterator for JsonObjectIter<'a> {
    type Item = (&'a String, &'a Json);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.itr_uk {
            Some(x) => {
                x.next()
            },
            None => {
                match self.itr_rk.as_mut().unwrap().next() {
                    Some(y) => {
                        Some((&y.1, &y.2))
                    },
                    None => None,
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        impl_jsonobjecti_next_hint!(self)
    }
}

pub struct JsonObjectIterMut<'a> {
    itr_uk: Option<std::collections::hash_map::IterMut<'a, String, Json>>,
    itr_rk: Option<std::slice::IterMut<'a, (u64, String, Json)>>,
}

impl<'a> Iterator for JsonObjectIterMut<'a> {
    type Item = (&'a String, &'a mut Json);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.itr_uk {
            Some(x) => {
                x.next()
            },
            None => {
                match self.itr_rk.as_mut().unwrap().next() {
                    Some(y) => {
                        Some((&y.1, &mut y.2))
                    },
                    None => None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        impl_jsonobjecti_next_hint!(self)
    }
}

pub struct JsonObjectIntoIter {
    itr_uk: Option<std::collections::hash_map::IntoIter<String, Json>>,
    itr_rk: Option<std::vec::IntoIter<(u64, String, Json)>>,
}

impl Iterator for JsonObjectIntoIter {
    type Item = (String, Json);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.itr_uk {
            Some(x) => {
                x.next()
            },
            None => {
                match self.itr_rk.as_mut().unwrap().next() {
                    Some(y) => {
                        Some((y.1, y.2))
                    },
                    None => None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        impl_jsonobjecti_next_hint!(self)
    }
}

impl JsonObject {
    pub fn new(is_unique_key: bool) -> JsonObject {
        JsonObject {
            obj: if is_unique_key {JsonSubObject::UniqueKey(HashMap::new())} else {JsonSubObject::RepeatKey(Vec::new())},
            is_beauty: false,
        }
    }
    
    pub fn iter(&self) -> JsonObjectIter {
        match &self.obj {
            JsonSubObject::UniqueKey(x) => {
                JsonObjectIter {
                    itr_uk: Some(x.iter()),
                    itr_rk: None,
                }
            },
            JsonSubObject::RepeatKey(y) => {
                JsonObjectIter {
                    itr_uk: None,
                    itr_rk: Some(y.iter()),
                }
            }
        }
    }
    
    pub fn iter_mut(&mut self) -> JsonObjectIterMut {
        match &mut self.obj {
            JsonSubObject::UniqueKey(x) => {
                JsonObjectIterMut {
                    itr_uk: Some(x.iter_mut()),
                    itr_rk: None,
                }
            },
            JsonSubObject::RepeatKey(y) => {
                JsonObjectIterMut {
                    itr_uk: None,
                    itr_rk: Some(y.iter_mut())
                }
            }
        }
    }
    
    pub fn is_empty(&self) -> bool {
        match &self.obj {
            JsonSubObject::UniqueKey(x) => x.is_empty(),
            JsonSubObject::RepeatKey(y) => y.is_empty(),
        }
    }
    
    pub fn is_unique_key(&self) -> bool {
        match &self.obj {
            JsonSubObject::UniqueKey(..) => true,
            JsonSubObject::RepeatKey(..) => false,
        }
    }
    
    pub fn is_beauty(&self) -> bool {
        self.is_beauty
    }
    
    pub fn set_is_beauty(&mut self, is_beauty: bool) -> &Self {
        self.is_beauty = is_beauty;
        &*self
    }
    
    pub fn clear(&mut self) -> &Self {
        match &mut self.obj {
            JsonSubObject::RepeatKey(x) => x.clear(),
            JsonSubObject::UniqueKey(y) => y.clear(),
        }
        &*self
    }
    
    pub fn len(&self) -> usize {
        match &self.obj {
            JsonSubObject::UniqueKey(x) => x.len(),
            JsonSubObject::RepeatKey(y) => y.len(),
        }
    }
    
    pub fn with_capacity(capacity: usize, is_unique_key: bool) -> JsonObject {
        JsonObject {
            obj: if is_unique_key {JsonSubObject::UniqueKey(HashMap::with_capacity(capacity))} else {JsonSubObject::RepeatKey(Vec::with_capacity(capacity))},
            is_beauty: false,
        }
    }
    
    fn hash_string(key: &String) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
    
    /// 当设置唯一键值时, 如果key已经存在, 那么返回旧值, 否认则None;  
    /// 当设置键值可重复时, 那么一直返回None;  
    pub fn insert(&mut self, key: String, val: Json) -> Option<Json> {
        match &mut self.obj {
            JsonSubObject::UniqueKey(x) => {
                x.insert(key, val)
            },
            JsonSubObject::RepeatKey(y) => {
                let hash = Self::hash_string(&key);
                y.push((hash, key, val));
                None
            }
        }
    }
    
    /// 返回最新的键值为key的value  
    pub fn get(&self, key: &String) -> Option<&Json> {
        match &self.obj {
            JsonSubObject::UniqueKey(x) => {
                x.get(key)
            },
            JsonSubObject::RepeatKey(y) => {
                let hash = Self::hash_string(key);
                for ele in y.iter().rev() {
                    if ele.0 == hash && &ele.1 == key{
                        return Some(&ele.2)
                    }
                }
                
                None
            }
        }
    }

    /// 返回最新的键值为key的value  
    pub fn get_mut(&mut self, key: &String) -> Option<&mut Json> {
        match &mut self.obj {
            JsonSubObject::UniqueKey(x) => {
                x.get_mut(key)
            },
            JsonSubObject::RepeatKey(y) => {
                let hash = Self::hash_string(key);
                for ele in y.iter_mut().rev() {
                    if ele.0 == hash && &ele.1 == key {
                        return Some(&mut ele.2)
                    }
                }
                
                None
            }
        }
    }
    
    pub fn contains_key(&self, key: &String) -> bool {
        match &self.obj {
            JsonSubObject::UniqueKey(x) => {
                x.contains_key(key)
            },
            JsonSubObject::RepeatKey(y)  => {
                let hash = Self::hash_string(key);
                for ele in y.iter() {
                    if ele.0 == hash && &ele.1 == key {
                        return true;
                    }
                }
                
                false
            }
        }
    }
    
    pub fn cvt_to_string(&self, buf: &mut String) {
        buf.push('{');
        if self.is_beauty() {
            #[cfg(target_os = "windows")]
            buf.push('\r');
            
            buf.push('\n');
        }
        
        for ele in self.iter() {
            let key = format!("\"{}\"{}", ele.0, if self.is_beauty() {": "} else {":"});
            buf.push_str(key.as_str());
            buf.push_str(ele.1.to_string().as_str());
            
            if self.is_beauty() {
                #[cfg(target_os = "windows")]
                    buf.push('\r');

                buf.push('\n');
            }
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
        let (mut itr, mut idx, mut is_need_comma) = (s.chars(), 0, false);
        
        loop {
            idx += 1;
            match itr.next() {
                Some('"') => {
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
                                    match json_array::find_value_auxiliary(x, &mut itr, is_need_comma, s, &mut idx) {
                                        Ok(json) => {
                                            obj.insert(String::from(sub_s), json);
                                        },
                                        Err(e) => {
                                            return Err(e);
                                        }
                                    }
                                }
                            }
                            
                            is_need_comma = true;
                        },
                        None => {
                            return Err(Self::err(idx, "object(can't find matched `\"`)"));
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
                    idx += 1;
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
        match self.obj {
            JsonSubObject::UniqueKey(x) => JsonObjectIntoIter {
                itr_uk: Some(x.into_iter()),
                itr_rk: None,
            },
            JsonSubObject::RepeatKey(y) => JsonObjectIntoIter {
                itr_uk: None,
                itr_rk: Some(y.into_iter()),
            }
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