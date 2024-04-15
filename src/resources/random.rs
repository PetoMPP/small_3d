use bevy::prelude::{Deref, DerefMut};
use rand::{rngs::StdRng, SeedableRng};

#[derive(Deref, DerefMut)]
pub struct Random {
    rng: StdRng,
}

impl Random {
    pub fn new(seed: u64) -> Self {
        let mut x = Vec::new();
        x.extend_from_slice(&seed.to_ne_bytes());
        x.extend_from_slice([0u8; 24].as_ref());
        let seed: [u8; 32] = x.try_into().unwrap();
        let rng = StdRng::from_seed(seed);
        Self { rng }
    }

    pub fn reset(&mut self, seed: u64) {
        *self = Self::new(seed);
    }
}

impl Default for Random {
    fn default() -> Self {
        Self::new(0)
    }
}
