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

    pub fn gen_vec2(&mut self) -> Vec2 {
        vec2(self.gen(), self.gen())
    }

    pub fn gen_in_unit_sphere(&mut self) -> Vec3 {
        let mut p;
        loop {
            let rv = vec3(self.gen(), self.gen(), self.gen());
            p = 2.0 * rv - Vec3::ONE;
            let p_len2 = p.length_squared();
            if p_len2 < 1.0 {
                break p;
            }
        }
    }

    pub fn gen_in_hemisphere(&mut self, nor: Vec3) -> Vec3 {
        let sp = self.gen_in_unit_sphere();
        if sp.dot(nor) > 0.0 {
            sp
        } else {
            -sp
        }
    }
}
