//! SHA

pub const SHA1_BLOCK_SIZE: usize = 64;
pub const SHA1_WORD_LEN: usize = 4;
pub const SHA1_DIGEST_SIZE: usize = 20;
pub const SHA1_DIGEST_WSIZE: usize = 5;

pub const SHA224_BLOCK_SIZE: usize = 64;
// pub const SHA224_WORD_LEN: usize = 4;
pub const SHA224_DIGEST_SIZE: usize = 28;
pub const SHA224_DIGEST_WSIZE: usize = 7;

pub const SHA256_BLOCK_SIZE: usize = 64;
pub const SHA256_WORD_LEN: usize = 4;
pub const SHA256_DIGEST_SIZE: usize = 32;
pub const SHA256_DIGEST_WSIZE: usize = 8;
//
// pub const SHA384_BLOCK_SIZE: usize = 128;
// pub const SHA384_WORD_LEN: usize = 8;
// pub const SHA384_DIGEST_SIZE: usize = 48;
//
// pub const SHA512_BLOCK_SIZE: usize = 128;
// pub const SHA512_WORD_LEN: usize = 8;
// pub const SHA512_DIGEST_SIZE: usize = 64;
//
// pub const SHA512_224_BLOCK_SIZE: usize = 128;
// pub const SHA512_224_WORD_LEN: usize = 8;
// pub const SHA512_224_DIGEST_SIZE: usize = 28;
//
// pub const SHA512_256_BLOCK_SIZE: usize = 128;
// pub const SHA512_256_WORD_LEN: usize = 8;
// pub const SHA512_256_DIGEST_SIZE: usize = 32;

pub const SHA1_INIT: [u32; SHA1_DIGEST_WSIZE] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
pub const SHA1_K: [u32; 4] = [0x5A827999, 0x6ED9EBA1, 0x8F1BBCDC, 0xCA62C1D6];

pub const SHA224_INIT: [u32; SHA256_DIGEST_WSIZE] = [0xC1059ED8, 0x367CD507, 0x3070DD17, 0xF70E5939, 0xFFC00B31, 0x68581511, 0x64F98FA7, 0xBEFA4FA4,];

pub const SHA256_INIT: [u32; SHA256_DIGEST_WSIZE] = [0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19];
pub const SHA256_K: [u32; 64] = [
    0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
    0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
    0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
    0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
    0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
    0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
    0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
    0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2,
];

#[inline]
pub fn f_ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ ((!x) & z)
}

#[inline]
pub fn f_parity(x: u32, y: u32, z: u32) -> u32 {
    (x ^ y) ^ z
}

#[inline]
pub fn f_maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}
    
