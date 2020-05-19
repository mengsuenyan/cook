//! SHA

pub const SHA1_BLOCK_SIZE: usize = 64;
pub const SHA1_WORD_LEN: usize = 4;
pub const SHA1_DIGEST_SIZE: usize = 20;
pub const SHA1_DIGEST_WSIZE: usize = 5;

// pub const SHA224_BLOCK_SIZE: usize = 64;
// pub const SHA224_WORD_LEN: usize = 4;
// pub const SHA224_DIGEST_SIZE: usize = 28;
//
// pub const SHA256_BLOCK_SIZE: usize = 64;
// pub const SHA256_WORD_LEN: usize = 4;
// pub const SHA256_DIGEST_SIZE: usize = 32;
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