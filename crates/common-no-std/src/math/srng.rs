use num_traits::Float;
use super::vec::*;
// shader rng

#[derive(Clone, Default)]
pub struct SRng {
    pub seed: Vec2,
}


impl SRng {
    pub fn gen_signed(&mut self) -> f32 {
        let res = (self.seed.dot(vec2(12.9898, 78.233)).sin() * 43758.5453).fract();
        self.seed = vec2(
            (self.seed.x + res + 17.825) % 3718.0,
            (self.seed.y + res + 72.7859) % 1739.0,
        );
        res
    }

    pub fn gen(&mut self) -> f32 {
        self.gen_signed() * 0.5 + 0.5
    }
}
