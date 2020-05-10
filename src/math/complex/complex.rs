//! 复数  

use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Neg};
use std::fmt;
use std::fmt::Debug;

#[derive(Default, Clone)]
pub struct Complex<T> 
{
    real: T,
    image: T,
}

impl<T> Complex<T> {
    /// real + image * i  
    pub fn new(real: T, image: T) -> Self {
        Complex {
            real,
            image,
        }
    }
    
    pub fn to_complex<U>(&self) -> Complex<U>
        where U: From<T>, T: Clone
    {
        Complex::new(U::from(self.real.clone()), U::from(self.image.clone()))
    }
    
    /// 实部  
    pub fn real(&self) -> &T {
        &self.real
    }
    
    pub fn real_mut(&mut self) -> &mut T {
        &mut self.real
    }
    
    /// 虚部  
    pub fn image(&self) -> &T {
        &self.image
    }
    
    pub fn image_mut(&mut self) -> &mut T {
        &mut self.image
    }
}

macro_rules! complex_fuc_pow_macro {
    ($SelfName: ident, $Fuc: expr, $Argument1: ident) => {
        let (r, theta) = ($SelfName.abs(), $SelfName.arg());
        let r = $Fuc(r);
        let theta = theta * ($Argument1 as f64);
        let (real, image) = (r * theta.cos(), r * theta.sin());
        return Complex::new(real, image);
    };
}

impl<T> Complex<T> 
    where T: Into<f64> + Mul<Output=T> + Add<Output=T> + Clone
{
    /// 模值  
    pub fn abs(&self) -> f64 {
        let x = self.real.clone() * self.real.clone() + self.image.clone() * self.image.clone();
        let m = x.into();
        m.sqrt()
    }
    
    /// 主辐角  
    pub fn arg(&self) -> f64 {
        let (x, y) = (self.real.clone().into(), self.image.clone().into());
        y.atan2(x)
    }
    
    /// n次幂  
    pub fn powi(&self, n: i32) -> Complex<f64> {
        complex_fuc_pow_macro!(self, |val: f64| -> f64 {val.powi(n)}, n);
    }
    
    /// n次幂  
    pub fn powf(&self, n: f64) -> Complex<f64> {
        complex_fuc_pow_macro!(self, |val: f64| -> f64 {val.powf(n)}, n);
    }
    
    /// 平方根  
    pub fn sqrt(&self) -> Complex<f64> {
       let n = 0.5;
        complex_fuc_pow_macro!(self, |val: f64| -> f64 {val.sqrt()}, n);
    }
    
    /// 立方根
    pub fn cbrt(&self) -> Complex<f64> {
        let n = 1.0 / 3.0;
        complex_fuc_pow_macro!(self, |val: f64| -> f64 {val.cbrt()}, n);
    }
    
    /// ln(self)  
    pub fn ln(&self) -> Complex<f64> {
        let (r, theta) = (self.abs(), self.arg());
        let x1 = r.ln();
        let y1 = theta;
        Complex::new(x1, y1)
    }
    
    pub fn log(&self, base: f64) -> Complex<f64> {
        let (r, theta, lnb) = (self.abs(), self.arg(), base.ln());
        let x1 = r.ln() / lnb;
        let y1 = theta / lnb;
        Complex::new(x1, y1)
    }
    
    pub fn log2(&self) -> Complex<f64> {
        self.log(2.0f64)
    }

    pub fn log10(&self) -> Complex<f64> {
        self.log(10.0f64)
    }
}

impl<T> Complex<T> 
    where T: Neg<Output=T> + Clone
{
    pub fn conjugate(&self) -> Self {
        Complex::new(self.real.clone(), -self.image.clone())
    }
}

impl<T> fmt::Debug for Complex<T> 
    where T: Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = format!("{:?}+{:?}i", self.real, self.image);
        f.write_str(s.as_str())
    }
}

impl<T> Add for Complex<T> 
    where T: Add<Output=T>
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Complex::new(self.real + rhs.real, self.image + rhs.image)
    }
}

impl<T> AddAssign for Complex<T> 
    where  T: AddAssign 
{
    fn add_assign(&mut self, rhs: Self) {
        self.real += rhs.real;
        self.image += rhs.image;
    }
}

impl<T> Sub for Complex<T> 
    where T: Sub<Output=T>
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Complex::new(self.real - rhs.real, self.image - rhs.image)
    }
}

impl<T> SubAssign for Complex<T>
    where T:SubAssign
{
    fn sub_assign(&mut self, rhs: Self) {
        self.real -= rhs.real;
        self.image -= rhs.image;
    }
}

impl<T> Mul for Complex<T> 
    where  T: Mul<Output=T> + Add<Output=T> + Sub<Output=T> + Clone
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Complex::new(self.real.clone() * rhs.real.clone() - self.image.clone() * rhs.image.clone(),
            self.image * rhs.real + self.real * rhs.image)
    }
}

impl<T> MulAssign for Complex<T>
    where  T: Mul<Output=T> + Add<Output=T> + Sub<Output=T> + Clone
{
    fn mul_assign(&mut self, rhs: Self) {
        self.real = self.real.clone() * rhs.real.clone() - self.image.clone() * rhs.image.clone();
        self.image = self.image.clone() * rhs.real + self.real.clone() * rhs.image;
    }
}

impl<T> Div for Complex<T> 
    where T: Div<Output=T> + Mul<Output=T> + Sub<Output=T> + Add<Output=T> + Clone
{
    type Output = Self;
    
    fn div(self, rhs: Self) -> Self::Output {
        let deno = rhs.real.clone() * rhs.real.clone() + rhs.image.clone() * rhs.image.clone();
        let real = self.real.clone() * rhs.real.clone() + self.image.clone() * rhs.image.clone();
        let image = self.image * rhs.real - self.real * rhs.image;
        Complex::new(real / deno.clone(), image / deno)
    }
}

impl<T> DivAssign for Complex<T>
    where T: Div<Output=T> + Mul<Output=T> + Sub<Output=T> + Add<Output=T> + Clone
{
    fn div_assign(&mut self, rhs: Self) {
        let deno = rhs.real.clone() * rhs.real.clone() + rhs.image.clone() * rhs.image.clone();
        let real = self.real.clone() * rhs.real.clone() + self.image.clone() * rhs.image.clone();
        let image = self.image.clone() * rhs.real - self.real.clone() * rhs.image;
        self.real = real / deno.clone();
        self.image = image / deno.clone();
    }
}

impl<T> PartialEq for Complex<T> 
    where  T: PartialEq
{
    fn eq(&self, rhs: &Self) -> bool {
        (self.real == rhs.real) && (self.image == rhs.image)
    }
    
    fn ne(&self, rhs: &Self) -> bool {
        (self.real != rhs.real) || (self.image == rhs.image)
    }
}

impl<T> PartialEq<T> for Complex<T>
    where T: PartialEq + Default + Clone
{
    fn eq(&self, rhs: &T) -> bool {
        self == &Complex::new(rhs.clone(), T::default())
    }
    
    fn ne(&self, rhs: &T) -> bool {
        self != &Complex::new(rhs.clone(), T::default())
    }
}

macro_rules! impl_from_for_complex_macro {
    ($Type: ty) => {
        impl From<$Type> for Complex<$Type> {
            fn from(val: $Type) -> Self {
                Complex::new(val, <$Type>::default())
            }
        }
    };
}

impl_from_for_complex_macro!(u8);
impl_from_for_complex_macro!(u16);
impl_from_for_complex_macro!(u32);
impl_from_for_complex_macro!(usize);
impl_from_for_complex_macro!(u64);
impl_from_for_complex_macro!(u128);
impl_from_for_complex_macro!(i8);
impl_from_for_complex_macro!(i16);
impl_from_for_complex_macro!(i32);
impl_from_for_complex_macro!(isize);
impl_from_for_complex_macro!(i64);
impl_from_for_complex_macro!(i128);
impl_from_for_complex_macro!(f32);
impl_from_for_complex_macro!(f64);

impl<T> From<(T, T)> for Complex<T> {
    fn from(val: (T, T)) -> Self {
        Complex::new(val.0, val.1)
    }
}


impl<T> Into<(T,T)> for Complex<T> {
    fn into(self) -> (T, T) {
        (self.real, self.image)
    }
}


#[cfg(test)]
mod tests {
    use crate::math::complex::Complex;

    #[test]
    fn complex() {
        let cp = Complex::from((1,2));
        let tp: Complex<i32> = (1,2).into();
        assert_eq!(cp, tp);
    }
}

