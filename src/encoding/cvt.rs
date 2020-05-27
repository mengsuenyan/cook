//! 各种类型转换

use crate::encoding::Bytes;

pub struct Cvt;

impl Cvt {
    pub fn cvt_bytes_to_str(b: &[u8]) -> String {
        Bytes::cvt_bytes_to_str(b)
    }
    
    pub fn reinterpret_cast<T, U>(v: &T) -> &U
    {
        unsafe {
            std::mem::transmute(v)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding::Cvt;

    #[test]
    fn cvt_vec_u64_to_i64() {
        let vt = vec![u32::max_value(), u32::max_value()];
        let vu = vec![-1i32, -1i32];
        let x: &Vec<i32> = Cvt::reinterpret_cast(&vt);
        assert_eq!(x, &vu);
    }
}
