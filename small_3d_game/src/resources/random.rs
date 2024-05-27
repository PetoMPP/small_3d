use bevy::prelude::{Deref, DerefMut};
use rand::{rngs::StdRng, SeedableRng};
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Deref, DerefMut)]
pub struct Random {
    rng: StdRng,
}

impl Random {
    pub fn new(seed: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let rng = StdRng::seed_from_u64(hasher.finish());
        Self { rng }
    }

    pub fn reset(&mut self, seed: impl Hash) {
        *self = Self::new(seed);
    }
}

impl Default for Random {
    fn default() -> Self {
        Self::new(0)
    }
}
