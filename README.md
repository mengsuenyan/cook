# cook

目标: Rust通用库

<span id='toc'></span>
[TOC]

## [hash](#toc)

- GenericHasher: 通用Hasher;
- Adler32: Adler32算法哈希值生成器;
- Fnv: Fnv算法哈希值生成器;

## [gds](#toc)

通用数据结构;

- LinearBuf: 线性缓存;  
- Stack: 栈;  
- LinkedList: 双向链表;  
- BNode: 二叉树节点;  
- BSTree: 二叉搜索树;  

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
