//! 用于crypto随机数生成  

use std::io::{Read, ErrorKind};

#[cfg(target_os = "windows")]
mod gr_windows {
    use std::os::raw::c_ulong;
    #[cfg(target_vendor = "uwp")]
    use std::os::raw::c_long;


    extern "system" {
        #[cfg(not(target_vendor = "uwp"))]
        #[link_name = "SystemFunction036"]
        fn RtlGenRandom(RandomBuffer: *mut u8, RandomBufferLength: c_ulong) -> u8;

        #[cfg(target_vendor = "uwp")]
        pub fn BCryptGenRandom(hAlgorithm: std::ffi::c_void, pBuffer: *mut u8,
                               cbBuffer: c_ulong, dwFlags: c_ulong) -> c_long;       
    }
    
    #[cfg(not(target_vendor = "uwp"))]
    pub fn get_random(r: &mut [u8]) -> bool {
        let ret = unsafe {
            RtlGenRandom(r.as_mut_ptr(), r.len() as c_ulong)
        };
        
        ret != 0
    }

    #[cfg(target_vendor = "uwp")]
    const BCRYPT_USE_SYSTEM_PREFERRED_RNG: c_ulong = 0x00000002;
    
    #[cfg(target_vendor = "uwp")]
    pub fn get_random(r: &mut [u8]) -> bool {
        let ret = unsafe {
            BCryptGenRandom(std::ptr::null_mut(), r.as_mut_ptr(), r.len() as c_long, BCRYPT_USE_SYSTEM_PREFERRED_RNG)
        };
        
        ret == 0
    }
}

#[cfg(target_os = "linux")]
mod gr_linux {
    use std::os::raw::{c_long, c_uint};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::fs::File;
    use std::io::Read;

    #[cfg(target_arch = "x86_64")]
    const __X32_SYSCALL_BIT: c_long = 0x40000000;
    #[cfg(target_arch = "x86_64")]
    const SYS_getrandom: c_long = __X32_SYSCALL_BIT + 318;
    
    #[cfg(target_arch = "x86")]
    const SYS_getrandom: c_long = 355;
    
    const GRND_NONBLOCK :c_uint = 0x0001;

    extern "C" {
        fn syscall(num: c_long, ...) -> c_long;
    }
    
    fn sys_getrandom(buf: &mut [u8]) -> c_long {
        unsafe {
            syscall(SYS_getrandom, buf.as_mut_ptr(), buf.len(), GRND_NONBLOCK)
        }
    }
    
    fn getrandom_fill_bytes(v: &mut [u8]) -> bool {
        static GETRANDOM_UNAVAILABLE: AtomicBool = AtomicBool::new(false);
        if GETRANDOM_UNAVAILABLE.load(Ordering::Relaxed) {
            return false;
        }
        
        let mut read = 0;
        // note 这里需要改进, 未处理错误类型
        while read < v.len() {
            let result = sys_getrandom(&mut v[read..]);
            if result == -1 {
                return false;
            } else {
                read += result as usize;
            }
        }
        
        true
    }
    
    pub fn get_random(r: &mut [u8]) -> bool {
        match getrandom_fill_bytes(r) { 
            false => {
                let mut file = File::open("/dev/urandom").expect("failed to open /dev/urandom");
                if file.read_exact(r).is_ok() {
                    true
                } else {
                    false
                }
            },
            _ => true,
        }
    }
}

#[cfg(target_os = "windows")]
use gr_windows::get_random;

#[cfg(target_os = "linux")]
use gr_linux::get_random;

use crate::math::big::Nat;

pub trait CryptoRng {}

pub struct CryptoRand;

impl CryptoRand {
    pub fn new() -> CryptoRand {
        CryptoRand {}
    }
}

impl CryptoRng for CryptoRand {}

impl Default for CryptoRand {
    fn default() -> Self {
        Self::new()
    }
}

impl Read for CryptoRand {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if get_random(buf) {
            Ok(buf.len())
        } else {
            Err(std::io::Error::new(ErrorKind::Other, "cannot get_random"))
        }
    }
}

const SMALL_PRIMES: [u8; 15] = [
    3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53,
];

const SMALL_RIMES_PRODUCT: u64 = 16294579238595022365u64;

/// 获取一个位长度为bits的质数  
pub fn prime<Rand>(bits: usize) -> Result<Nat, &'static str> 
    where Rand: CryptoRng + Read + Default
{
    if bits < 2 {
        return Err("crypto/rand: prime size must be at least 2-bit")
    }

    let small_prime_product = Nat::from_u64(SMALL_RIMES_PRODUCT);
    let mut rng = Rand::default();

    let b = match bits % 8 {
        0 => 8,
        x => x,
    } as u8;
    
    let mut bytes: Vec<u8> = Vec::new();
    bytes.resize((bits + 7) >> 3, 0);
    
    loop {
        match rng.read_exact(bytes.as_mut_slice()) {
            Err(_e) => return Err("read random number failed"),
            _ => {},
        };
        
        // 获取的随机数位长度大于bits, 清除最高位
        let bytes_last = bytes.last_mut().unwrap();
        *bytes_last &= ((1u32 << (b as u32)) - 1) as u8;
        // 移除了bytes超出bits位长度的高位, 但bits位长度的高位可能是0, 为了不让bytes太小
        // 高位填充位1
        if b >= 2 {
            *bytes_last |= 3 << (b - 2);
        } else {
            *bytes_last |= 1;
            let len = bytes.len();
            if len > 1 {
                bytes[len - 2] |= 0x80;
            }
        }
        
        // 保证bytes是奇数
        bytes[0] |= 0x1;
        
        let mut p = Nat::from_vec(&bytes);
        let bigmod = &p % &small_prime_product;
        let modulus: u64 = bigmod.to_u64().expect("Cannot convert NaN to u64's number");
        let mut delta = 0u64;
        'nextdelta: while delta < (1 << 20) {
            let m = modulus + delta;
            for &prime in SMALL_PRIMES.iter() {
                if (m % (prime as u64) == 0) && (bits > 6 || m != (prime as u64)) {
                    delta += 2;
                    continue 'nextdelta;
                }
            }
            
            if delta > 0 {
                p += &Nat::from_u64(delta);
            }
            
            break;
        }
        
        if p.probably_prime(20) && p.bits_len() == bits {
        // if p.probably_prime(20) {
             return Ok(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::rand::CryptoRand;
    use std::time::SystemTime;

    #[test]
    fn rand_prime() {
        let his0 = SystemTime::now();
        for i in 2..100 {
            let his = SystemTime::now();
            let nat = super::prime::<CryptoRand>(i);
            assert!(nat.is_ok());
            let nat = nat.unwrap();
            println!("time: {:?}, case=>i{}->nat:{}:{}", SystemTime::now().duration_since(his), i, nat, nat.bits_len());
        }
        println!("total time: {:?}", SystemTime::now().duration_since(his0));

        let cases = [
            512,
            1024,
            // 2048,
        ];
        let his0 = SystemTime::now();
        for &i in cases.iter() {
            let his = SystemTime::now();
            let nat = super::prime::<CryptoRand>(i);
            assert!(nat.is_ok());
            let nat = nat.unwrap();
            println!("time: {:?}, case=>i{}->nat:{}:{}", SystemTime::now().duration_since(his), i, nat, nat.bits_len());
        }
        println!("total time: {:?}", SystemTime::now().duration_since(his0));
    }
}
