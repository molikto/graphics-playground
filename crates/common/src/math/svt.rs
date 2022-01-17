use core::ops::Deref;

use super::ray::*;
use super::vec::*;

#[allow(non_camel_case_types)]
// pub type usvt = u16;
// pub type Usvt3 = SUVec3;
pub type usvt = u32;
pub type Usvt3 = UVec3;

const MASK_IS_LEAF: usvt = 1u32 << (usvt::BITS - 1);
const MASK_DATA: usvt = !MASK_IS_LEAF;

pub fn vec3_to_usvt3(v: Vec3) -> Usvt3 {
    Usvt3::new(v.x as usvt, v.y as usvt, v.z as usvt)
}

pub struct BlockRayIntersectionInfo {
    pub mask: Vec3,
    pub t: f32,
}

pub struct BlockInfo<const BLOCK_DIM: usvt, const LEVEL_COUNT: usize> {
    pub level: usvt,
    pub data: usvt,
}

impl<const BLOCK_DIM: usvt, const LEVEL_COUNT: usize> BlockInfo<BLOCK_DIM, LEVEL_COUNT> {
    pub fn size(&self) -> usvt {
        BLOCK_DIM.pow((LEVEL_COUNT as usvt - self.level - 1) as u32)
    }
}

#[repr(transparent)]
pub struct Svt<REF: Deref<Target = [usvt]>, const BLOCK_DIM: usvt, const LEVEL_COUNT: usize> {
    pub mem: REF,
}

impl<REF: Deref<Target = [usvt]>, const BLOCK_DIM: usvt, const LEVEL_COUNT: usize>
    Svt<REF, BLOCK_DIM, LEVEL_COUNT>
{
    pub const BLOCK_SIZE: usvt = BLOCK_DIM * BLOCK_DIM * BLOCK_DIM;
    pub const TOTAL_DIM: usvt = BLOCK_DIM.pow(LEVEL_COUNT as usvt);

    // TODO memory allocation
    pub fn root_block_index(&self) -> usize {
        return 0;
    }

    //
    // helpers
    //

    #[inline]
    pub fn encode(v: Usvt3) -> usize {
        return (v.x * BLOCK_DIM * BLOCK_DIM + v.y * BLOCK_DIM + v.z) as usize;
    }

    //
    // block API
    #[inline]
    pub fn is_terminal_block(u: usvt) -> bool {
        u & MASK_IS_LEAF == (0 as usvt)
    }

    #[inline]
    pub fn block_index_data(u: usvt) -> usvt {
        u & MASK_DATA
    }

    #[inline]
    pub fn new_block(terminal: bool, index: usvt) -> usvt {
        if terminal {
            return index;
        } else {
            return index | MASK_IS_LEAF;
        }
    }

    #[inline]
    pub fn level_position_abs(&self, position: Usvt3, level: usvt) -> Usvt3 {
        return position / (BLOCK_DIM.pow(LEVEL_COUNT as u32 - 1 - (level as u32)));
    }

    #[inline]
    pub fn level_position_abs_to_position(&self, position: Usvt3, level: usvt) -> Usvt3 {
        return position * (BLOCK_DIM.pow(LEVEL_COUNT as u32 - 1 - (level as u32)));
    }

    #[inline]
    pub fn level_position(&self, position: Usvt3, level: usvt) -> Usvt3 {
        return self.level_position_abs(position, level) % BLOCK_DIM;
    }

    pub fn debug_error_code_colors(error_code: i32) -> Vec3 {
        if error_code == -1 {
            return vec3(0.0, 1.0, 0.0);
        } else if error_code == -2 {
            return vec3(1.0, 0.0, 0.0);
        } else if error_code == -3 {
            return vec3(0.0, 0.0, 1.0);
        } else if error_code == -4 {
            return vec3(0.0, 1.0, 1.0);
        } else if error_code == -5 {
            return vec3(1.0, 0.0, 1.0);
        } else if error_code == -6 {
            return vec3(1.0, 1.0, 0.0);
        } else {
            panic!();
        }
    }

    const EPS: f32 = 3.552713678800501e-15;

    // from https://github.com/AdamYuan/SparseVoxelOctree
    #[inline]
    pub fn traverse_ray_oct(&self, mut o: Vec3, mut d: Vec3) -> i32
    {
        const STACK_SIZE: usize = 23;
        let mut stack_node = [0 as u32; STACK_SIZE];
        let mut stack_t_max = [0.0 as f32; STACK_SIZE];
        let mut iter = 0;
        // The octree is assumed to reside at coordinates [1, 2].
        o = o / (Self::TOTAL_DIM as f32) + 1.0;
        d = d.de_eps(Self::EPS);
        // Precompute the coefficients of tx(x), ty(y), and tz(z).
        let t_coef = 1.0 / -d.abs();
        let mut t_bias = t_coef * o;

        let mut oct_mask = 0u32;
        if d.x > 0.0 {
            oct_mask ^= 1;
            t_bias.x = 3.0 * t_coef.x - t_bias.x;
        }
        if d.y > 0.0 {
            oct_mask ^= 2;
            t_bias.y = 3.0 * t_coef.y - t_bias.y;
        }
        if d.z > 0.0 {
            oct_mask ^= 4;
            t_bias.z = 3.0 * t_coef.z - t_bias.z;
        }

        // Initialize the active span of t-values.
        let mut t_min = (2.0 * t_coef.x - t_bias.x)
            .max(2.0 * t_coef.y - t_bias.y)
            .max(2.0 * t_coef.z - t_bias.z);
        let mut t_max = (t_coef.x - t_bias.x)
            .min(t_coef.y - t_bias.y)
            .min(t_coef.z - t_bias.z);
        t_min = t_min.max(0.0);
        let mut h = t_max;

        let mut parent = 0u32;
        let mut cur = 0u32;
        let mut pos = Vec3::splat(1.0);
        let mut idx = 0u32;
        if 1.5 * t_coef.x - t_bias.x > t_min {
            idx ^= 1;
            pos.x = 1.5;
        }
        if 1.5 * t_coef.y - t_bias.y > t_min {
            idx ^= 2;
            pos.y = 1.5;
        }
        if 1.5 * t_coef.z - t_bias.z > t_min {
            idx ^= 4;
            pos.z = 1.5;
        }

        let mut scale = STACK_SIZE - 1;
        let mut scale_exp2 = 0.5; // exp2( scale - STACK_SIZE )

        while scale < STACK_SIZE {
            iter += 1;
            if cur == 0u32 {
                cur = self.mem[(parent + (idx ^ oct_mask)) as usize];
            }
            // Determine maximum t-value of the cube by evaluating
            // tx(), ty(), and tz() at its corner.

            let t_corner = pos * t_coef - t_bias;
            let tc_max = t_corner.min_element();

            if cur != 0 && t_min <= t_max {
                // INTERSECT
                let tv_max = t_max.min(tc_max);
                let half_scale_exp2 = scale_exp2 * 0.5f32;
                let t_center = half_scale_exp2 * t_coef + t_corner;

                if t_min <= tv_max {
                    if (cur & 0x80000000u32) == 0 {
                        break;
                    }

                    // PUSH
                    if tc_max < h {
                        stack_node[scale] = parent;
                        stack_t_max[scale] = t_max;
                    }
                    h = tc_max;

                    parent = Self::block_index_data(cur) << 3u32;

                    idx = 0u32;
                    scale -= 1;
                    scale_exp2 = half_scale_exp2;
                    if t_center.x > t_min {
                        idx ^= 1u32;
                        pos.x += scale_exp2;
                    }
                    if t_center.y > t_min {
                        idx ^= 2u32;
                        pos.y += scale_exp2;
                    }
                    if t_center.z > t_min {
                        idx ^= 4u32;
                        pos.z += scale_exp2;
                    }

                    cur = 0;
                    t_max = tv_max;

                    continue;
                }
            }

            // ADVANCE
            let mut step_mask = 0u32;
            if t_corner.x <= tc_max {
                step_mask ^= 1u32;
                pos.x -= scale_exp2;
            }
            if t_corner.y <= tc_max {
                step_mask ^= 2u32;
                pos.y -= scale_exp2;
            }
            if t_corner.z <= tc_max {
                step_mask ^= 4u32;
                pos.z -= scale_exp2;
            }

            // Update active t-span and flip bits of the child slot index.
            t_min = tc_max;
            idx ^= step_mask;

            // Proceed with pop if the bit flips disagree with the ray direction.
            if (idx & step_mask) != 0 {
                // POP
                // Find the highest differing bit between the two positions.
                let mut differing_bits = 0u32;
                if (step_mask & 1) != 0 {
                    differing_bits |= floatBitsToUint(pos.x) ^ floatBitsToUint(pos.x + scale_exp2);
                }
                if (step_mask & 2) != 0 {
                    differing_bits |= floatBitsToUint(pos.y) ^ floatBitsToUint(pos.y + scale_exp2);
                }
                if (step_mask & 4) != 0 {
                    differing_bits |= floatBitsToUint(pos.z) ^ floatBitsToUint(pos.z + scale_exp2);
                }
                scale = findMSB(differing_bits);
                scale_exp2 = uintBitsToFloat(((scale - STACK_SIZE + 127usize) as u32) << 23u32); // exp2f(scale - s_max)

                // Restore parent voxel from the stack.
                parent = stack_node[scale];
                t_max = stack_t_max[scale];

                // Round cube position and extract child slot index.
                let shx = floatBitsToUint(pos.x) >> scale;
                let shy = floatBitsToUint(pos.y) >> scale;
                let shz = floatBitsToUint(pos.z) >> scale;
                pos.x = uintBitsToFloat(shx << scale);
                pos.y = uintBitsToFloat(shy << scale);
                pos.z = uintBitsToFloat(shz << scale);
                idx = (shx & 1) | ((shy & 1) << 1u32) | ((shz & 1) << 2u32);

                // Prevent same parent from being stored again and invalidate cached
                // child descriptor.
                h = 0.0f32;
                cur = 0;
            }
        }
        iter
    }

    #[inline]
    pub fn traverse_ray(&self, max_count: u32, ray: Ray3) -> i32
    {
        let mut count: u32 = 0;
        // TODO there are still cases where it infinite loop..
        // the max dim we can have is 8^8, otherwise it will not work because of floating point issue
        // https://itectec.com/matlab-ref/matlab-function-flintmax-largest-consecutive-integer-in-floating-point-format/
        // de_eps(&mut ray.dir);
        if ray.dir == Vec3::ZERO {
            return -3;
        }
        let ray_dir_inv = 1.0 / ray.dir;
        let ray_pos_div_ray_dir = ray.pos / ray.dir;
        let ray_dir_signum = ray.dir.signum();
        let ray_dir_limit_mul = (ray_dir_signum + 1.0) / 2.0;

        let mut mask: Vec3;
        let mut position: Vec3;

        // use crate::Aabb3;
        // let aabb = Aabb3::new(Vec3::ZERO, Vec3::splat(Self::TOTAL_DIM as f32));
        // if aabb.inside(ray.pos) {
        mask = Vec3::ZERO;
        position = ray.pos;
        // } else {
        //     let hit = aabb.hit(&ray, 0.0, 100000000000.0);
        //     if hit.is_hit {
        //         mask = hit.nor;
        //         position = ray.at(hit.t + 0.0001);
        //     } else {
        //         return 0;
        //     }
        // }
        let mut t = 0 as f32;
        let mut block_indexs = [0usize; LEVEL_COUNT];
        block_indexs[0] = self.root_block_index() * (Self::BLOCK_SIZE as usize);

        let block_limit = ray_dir_limit_mul * (Self::TOTAL_DIM as f32);
        let ts = block_limit * ray_dir_inv - ray_pos_div_ray_dir;
        let ts_min = ts.x.min(ts.y).min(ts.z);
        let mut block_limits = [ts_min; LEVEL_COUNT];
        // there is a off by one error...
        //
        // block aabbs is terminal block
        let mut level: usvt = 0;
        let mut level_dim_div = Self::TOTAL_DIM / BLOCK_DIM;
        // go inside levels
        let mut level_position_abs: Usvt3;
        let mut index: usvt;
        loop {
            let position_u = vec3_to_usvt3(position);
            loop {
                level_position_abs = position_u / level_dim_div;
                let level_position = level_position_abs % BLOCK_DIM;
                let target_block_index =
                    block_indexs[level as usize] + Self::encode(level_position);
                let target_block = self.mem[target_block_index];
                index = Self::block_index_data(target_block);
                let block_limit =
                    (level_position_abs.as_vec3() + ray_dir_limit_mul) * (level_dim_div as f32);
                let ts = block_limit * ray_dir_inv - ray_pos_div_ray_dir;
                let ts_min = ts.x.min(ts.y).min(ts.z);
                if Self::is_terminal_block(target_block) {
                    let incident = BlockRayIntersectionInfo { t, mask };
                    // TODO this fixes some problem of inifinite looping
                    // set mask later, because we want to know the mask of the enter face
                    // t = if ts_min < t { t + 0.0001 } else { ts_min };
                    t = ts_min;
                    mask = ts.step_f(t) * ray_dir_signum;
                    if mask == Vec3::ZERO {
                        return -2;
                    }
                    break;
                } else {
                    level += 1;
                    level_dim_div /= BLOCK_DIM; // must be here or we get 1 / N
                    block_limits[level as usize] = ts_min;
                    block_indexs[level as usize] = index as usize * (Self::BLOCK_SIZE as usize);
                }
            }
            // we add extra value so we don't step on the boundary. `position = ray.at(t + 0.01)` doesn't work
            let position_new = ray.at(t) + mask * 0.5;
            // comparing `t = if ts_min < t { t + 0.0001 } else { ts_min };` doesn't work!! needs to do it like bellow
            position.x = if ray_dir_signum.x > 0.0 {
                position.x.max(position_new.x)
            } else {
                position.x.min(position_new.x)
            };
            position.y = if ray_dir_signum.y > 0.0 {
                position.y.max(position_new.y)
            } else {
                position.y.min(position_new.y)
            };
            position.z = if ray_dir_signum.z > 0.0 {
                position.z.max(position_new.z)
            } else {
                position.z.min(position_new.z)
            };
            count += 1;
            if count > max_count {
                return -1;
            }
            loop {
                let block_limit = block_limits[level as usize];
                if t < block_limit {
                    break;
                } else if level == 0 {
                    return count as i32;
                } else {
                    level -= 1;
                    level_dim_div *= BLOCK_DIM;
                }
            }
        }
    }

    pub fn get(&self, position: Usvt3) -> usvt {
        let mut level = 0;
        let mut first_block_index = self.root_block_index() * (Self::BLOCK_SIZE as usize);
        // get the block at that position, create new blocks if needed
        while level < LEVEL_COUNT as usvt {
            let level_position = self.level_position(position, level);
            let target_block_index = first_block_index + Self::encode(level_position);
            let target_block = self.mem[target_block_index];
            let index = Self::block_index_data(target_block);
            if Self::is_terminal_block(target_block) {
                return index;
            }
            first_block_index = index as usize * (Self::BLOCK_SIZE as usize);

            level += 1;
        }
        panic!("not allowed to be here");
    }
}

fn findMSB(bits: u32) -> usize {
    #[cfg(target_arch = "spirv")]
    {
        let result;
        unsafe {
            asm!(
                "%glsl = OpExtInstImport \"GLSL.std.450\"",
                "%uint = OpTypeInt 32 0",
                // 75=FindUMsb
                "{result} = OpExtInst %uint %glsl 75 {bits}",
                bits = in(reg) bits,
                result = out(reg) result,
            )
        }
        result
    }
    #[cfg(not(target_arch = "spirv"))]
    todo!()
}

fn uintBitsToFloat(bits: u32) -> f32 {
    #[cfg(target_arch = "spirv")]
    {
        let result;
        unsafe {
            asm!(
                "%float = OpTypeFloat 32",
                "{result} = OpBitcast %float {bits}",
                bits = in(reg) bits,
                result = out(reg) result,
            )
        }
        result
    }
    #[cfg(not(target_arch = "spirv"))]
    unsafe {
        std::mem::transmute::<u32, f32>(bits)
    }
}

fn floatBitsToUint(bits: f32) -> u32 {
    #[cfg(target_arch = "spirv")]
    {
        let result;
        unsafe {
            asm!(
                "%uint = OpTypeInt 32 0",
                "{result} = OpBitcast %uint {bits}",
                bits = in(reg) bits,
                result = out(reg) result,
            )
        }
        result
    }
    #[cfg(not(target_arch = "spirv"))]
    unsafe {
        std::mem::transmute::<f32, u32>(bits)
    }
}

#[cfg(not(target_arch = "spirv"))]
pub use super::svt_std::*;
