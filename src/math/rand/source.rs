//! 随机数生成器需要实现的trait

pub trait Seed<Sd> 
    where Sd: Copy + PartialEq + PartialOrd {
    
    /// set seed value to initialize the pseudo random number generator
    fn set_seed(&mut self, sd: Sd);
}

pub trait Source<Rd, Sd>: Seed<Sd>
    where Rd: Copy,
        Sd: Copy + PartialOrd + PartialEq { 
    
    // generate pseudo random number
    fn generate_prn(&mut self) -> Rd;
}

