use rand::SeedableRng;

const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
const FNV_PRIME: u64 = 1099511628211;

fn hash_fnv_1a(input: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in input {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[derive(Debug)]
pub struct RngMaster {
    master_seed: String,
}

impl RngMaster {
    pub fn new(seed: &str) -> RngMaster {
        RngMaster {
            master_seed: seed.into(),
        }
    }

    pub fn for_stage(&self, stage: &str) -> rand::rngs::ChaCha8Rng {
        let mut stage: Vec<u8> = stage.as_bytes().to_vec();
        stage.extend(self.master_seed.as_bytes());
        let stage_seed = hash_fnv_1a(&stage);
        rand::rngs::ChaCha8Rng::seed_from_u64(stage_seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use rstest::{fixture, rstest};

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
}
