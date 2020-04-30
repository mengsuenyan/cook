# cook

目标: Rust通用库

<span id='toc'></span>
[TOC]


## [hash](#toc)

- GenericHasher: 通用Hasher;
- Adler32: Adler32算法哈希值生成器;
- Fnv: Fnv算法哈希值生成器;

## [math](#toc)

### [big](#toc)

- Nat: 任意长度的自然数;  
- BigInt: 任意长度的整数;  
- BigFloat: 任意精度的浮点数(**待测试**);  

### [rand](#toc)

- Seed: 随机数生成器seed种子trait;
- Source: 随机数生成器trait;
- RngSource: 随机数生成器;
- NormalDistribution: 正态分布随机数;
- UniformDistribution: 均匀分布随机苏;