# cook

目标: Rust通用库

<span id='toc'></span>
[TOC]

## [crypto](#toc)

- Cipher: 加密算法需要实现的Trait;
- DesCipher: DES加密算法;
- Md5Cipher: MD5消息摘要算法;
- Sha1Digest: SHA-1安全散列算法;
- Sha256Digest: SHA-256安全散列算法;
- Sha224Digest: SHA-224安全散列算法;
- Sha512Digest: SHA-512安全散列算法;
- Sha512T224Digest: SHA-512/224安全散列算法;
- Sha512T256Digest: SHA-512/256安全散列算法;
- Sha512T384Digest: SHA-512/384安全散列算法;
- Sha512Digest::generate_sha512t: SHA-512/t384安全散列算法;
- Aes128Cipher/Aes192Cipher/Aes256Cipher: AES加密;  
- rand::CryptoRng/rand::CryptoRand: 加密模块随机数trait, 及提供的加密模块默认随机数生成器;
- rand::prime: 随机选择一个指定位数的质数;
- PrivateKey/PublicKey: RSA私钥/公钥;
- PKCS/PKCSType: PKCS RSA加密标准;

## [hash](#toc)

- GenericHasher: 通用Hasher Trait;
- Adler32: Adler32算法哈希值生成器;
- Fnv: Fnv算法哈希值生成器;
- Crc32/Crc64: Crc校验器;

## [gds](#toc)

通用数据结构;

- LinearBuf: 线性缓存;  
- Stack: 栈;  
- LinkedList: 双向链表;  
- BNode: 二叉树节点;  
- BSTree: 二叉搜索树;  
- BHeap: 二叉堆;  
- RBTree: 红黑树;  

## [ext_macro](#toc)

扩展宏

- cfg_if: 类似C语言if/elif/else宏定义;

## [math](#toc)

### [big](#toc)

- Nat: 任意长度的自然数;  
- BigInt: 任意长度的整数;  
- BigFloat: 任意精度的浮点数(**待测试**);  

### [complex](#toc)

- Complex: 复数;

### [rand](#toc)

- Seed: 随机数生成器seed种子trait;
- Source: 随机数生成器trait;
- RngSource: 随机数生成器;
- NormalDistribution: 正态分布随机数;
- UniformDistribution: 均匀分布随机苏;

## [encoding](#toc)

- Bytes: 字节序列相关辅助功能;