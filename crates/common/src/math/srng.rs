use super::vec::*;
// shader rng
#[repr(transparent)]
pub struct SRng {
    state: u32,
}

impl SRng {
    const PCG_DEFAULT_MULTIPLIER_32: u32 = 747796405;
    const PCG_DEFAULT_INCREMENT_32: u32 = 2891336453;

    fn pcg_oneseq_32_step_r(&mut self) {
        self.state = self
            .state
            .wrapping_mul(Self::PCG_DEFAULT_MULTIPLIER_32)
            .wrapping_add(Self::PCG_DEFAULT_INCREMENT_32);
    }

    fn pcg_output_rxs_m_xs_32_32(state: u32) -> u32 {
        let word = ((state >> ((state >> 28).wrapping_add(4))) ^ state).wrapping_mul(277803737);
        (word >> 22) ^ word
    }

    pub fn new(seed: u32) -> Self {
        let mut rng = Self { state: seed };
        rng.pcg_oneseq_32_step_r();
        rng.state = rng.state.wrapping_add(seed);
        rng.pcg_oneseq_32_step_r();
        rng
    }

    pub fn gen_u32(&mut self) -> u32 {
        let old_state = self.state;
        self.pcg_oneseq_32_step_r();
        Self::pcg_output_rxs_m_xs_32_32(old_state)
    }

    pub fn gen(&mut self) -> f32 {
        let float_size = core::mem::size_of::<f32>() as u32 * 8;
        let precision = 23 + 1;
        let scale = 1.0 / ((1 << precision) as f32);

        let value = self.gen_u32();
        let value = value >> (float_size - precision);
        scale * value as f32
    }

    pub fn gen_range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.gen()
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
