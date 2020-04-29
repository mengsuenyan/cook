//! 均匀分布随机数生成

use crate::math::rand::rngsource::RngSource;
use crate::math::rand::source::Source;

pub struct UniformDistribution{
    min: i64,
    max: i64,
    source: RngSource,
}

impl UniformDistribution {
    /// [min, max)
    pub fn new(seed: i64, min: i64, max: i64) -> Option<UniformDistribution> {
        if min >= max {
            None
        } else {
            Some(UniformDistribution {
                min,
                max,
                source: RngSource::new(seed),
            })
        }
    }
    
    pub fn rng(&mut self) -> i64 {
        let n = self.max - self.min;
        let val = if n & (n-1) == 0 {
            let r: i64 = self.source.rng();
            r & (n - 1)
        } else {
            let max: i64 = std::i64::MAX - (((1u64 << 63) % (n as u64)) as i64);
            let mut r: i64 = self.source.rng();
            while r > max {
                r = self.source.rng()
            }
            r % n
        };
        
        val + self.min
    }
}

#[cfg(test)]
mod tests {
    use crate::math::rand::uniform_distribution::UniformDistribution;

    #[test]
    fn test_uniform_distribution() {
        let ud = UniformDistribution::new(1, 0, 100);
        assert!(ud.is_some());
        let mut ud = ud.unwrap();
        for _ in 0..1000 {
            let r = ud.rng();
            assert!(r < 100 && r >=0 );
        }
    }
}