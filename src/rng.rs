#![allow(dead_code)]
use rand::{RngExt, SeedableRng};

const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
const FNV_PRIME: u64 = 1099511628211;

#[must_use]
pub fn hash_fnv_1a(input: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in input {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[derive(Debug)]
pub struct RngMaster {
    master_seed: String,
}

impl RngMaster {
    #[must_use]
    pub fn new(seed: &str) -> RngMaster {
        RngMaster { master_seed: seed.into() }
    }

    #[must_use]
    pub fn for_stage(&self, stage: &str) -> rand::rngs::ChaCha8Rng {
        let mut stage: Vec<u8> = stage.as_bytes().to_vec();
        stage.extend(self.master_seed.as_bytes());
        let stage_seed = hash_fnv_1a(&stage);
        rand::rngs::ChaCha8Rng::seed_from_u64(stage_seed)
    }
}

pub trait Dice {
    #[must_use]
    fn d(&mut self, s: u32) -> u32;

    #[must_use]
    fn dn(&mut self, n: u32, s: u32) -> u32;
}

impl<T: RngExt + ?Sized> Dice for T {
    fn d(&mut self, s: u32) -> u32 {
        debug_assert!(s > 0, "Can't roll zero-sided die");
        self.random_range(1..=s)
    }

    fn dn(&mut self, n: u32, s: u32) -> u32 {
        (0..n).map(|_| self.d(s)).sum()
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use rstest::{fixture, rstest};

    use super::*;

    #[fixture]
    fn master() -> RngMaster {
        RngMaster::new("test master seed")
        // Grandmaster Flash's less-successful cousin?
    }

    #[rstest]
    fn test_staged_rng_same(master: RngMaster) {
        let mut rng1 = master.for_stage("first");
        let mut rng2 = master.for_stage("first");
        assert_eq!(rng1.next_u64(), rng2.next_u64());
    }

    #[rstest]
    fn test_staged_rng_different(master: RngMaster) {
        let mut rng1 = master.for_stage("first");
        let mut rng2 = master.for_stage("second");
        assert_ne!(rng1.next_u64(), rng2.next_u64());
    }

    #[rstest]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "zero-sided")]
    fn test_dice_zero_die_panics(master: RngMaster) {
        let mut rng = master.for_stage("dice");
        let _ = rng.d(0);
    }

    #[rstest]
    fn test_dice_zero_n(master: RngMaster) {
        let mut rng = master.for_stage("dice");
        assert_eq!(0, rng.dn(0, 6));
    }
}
