use core::ops::Deref;

use crate::Aabb3;

use super::ray::*;
use super::vec::*;

#[allow(non_camel_case_types)]
// pub type usvt = u16;
// pub type Usvt3 = SUVec3;
pub type usvt = u32;
pub type Usvt3 = UVec3;

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
        return 1;
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
        u % (2 as usvt) == (0 as usvt)
    }

    #[inline]
    pub fn block_index_data(u: usvt) -> usvt {
        u >> 1u32
    }

    #[inline]
    fn new_block(terminal: bool, index: usvt) -> usvt {
        if terminal {
            return index << (1 as usvt);
        } else {
            return index << (1 as usvt) | (1 as usvt);
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

    #[inline]
    pub fn traverse_ray<C>(&self, max_count: u32, ray: Ray3, mut closure: C) -> i32
    where
        C: FnMut(
            BlockRayIntersectionInfo,
            BlockRayIntersectionInfo,
            BlockInfo<BLOCK_DIM, LEVEL_COUNT>,
        ) -> bool,
    {
        let mut count: u32 = 0;
        // normaling ray here has some bad thing happens... if want to normalize, need the outside usage sync with it
        let ray_dir_inv = 1.0 / ray.dir;
        let ray_pos_div_ray_dir = ray.pos / ray.dir;
        let ray_dir_signum = ray.dir.signum();
        let ray_dir_limit_mul = (ray_dir_signum + 1.0) / 2.0;

        let mut mask: Vec3;
        let mut position: Vec3;
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

        // rollback buffers
        let mut block_indexs = [0usize; LEVEL_COUNT];
        let mut block_limits = [0.0; LEVEL_COUNT];

        let mut t = 0 as f32;
        let mut parent_index: usize = self.root_block_index() * (Self::BLOCK_SIZE as usize);

        let mut level_dim_div = Self::TOTAL_DIM / BLOCK_DIM;
        // pos in ESVO
        let mut block_limit_v = ray_dir_limit_mul * (Self::TOTAL_DIM as f32);
        let mut parent_block_limit =
            (block_limit_v * ray_dir_inv - ray_pos_div_ray_dir).min_element();
        let mut level: usvt = 0;
        // idx in ESVO
        // copied code same at loop end
        block_limit_v = ((position / (level_dim_div as f32)).floor() + ray_dir_limit_mul) * (level_dim_div as f32);
        let mut level_position = vec3_to_usvt3(position) / level_dim_div % BLOCK_DIM;
        loop {
            let target_block = self.mem[parent_index + Self::encode(level_position)];
            let block_data = Self::block_index_data(target_block);
            let ts = block_limit_v * ray_dir_inv - ray_pos_div_ray_dir;
            let ts_min = ts.min_element();
            if Self::is_terminal_block(target_block) {
                let incident = BlockRayIntersectionInfo { t, mask };
                // set mask later, because we want to know the mask of the enter face
                // t = if ts_min < t { t + 0.0001 } else { ts_min };
                t = ts_min;
                mask = ts.step_f(t) * ray_dir_signum;
                if mask == Vec3::ZERO {
                    return -2;
                }
                let block_info = BlockInfo {
                    level,
                    data: block_data,
                };
                let ret = closure(incident, BlockRayIntersectionInfo { t, mask }, block_info);
                if ret {
                    return count as i32;
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
                let over_limit = t >= parent_block_limit;
                if over_limit {
                    loop {
                        if level == 0 {
                            return count as i32;
                        } else {
                            count += 1; // keep same with ESVO paper for comparsion
                            level -= 1;
                            parent_block_limit = block_limits[level as usize];
                            parent_index = block_indexs[level as usize];
                            level_dim_div *= BLOCK_DIM;
                        }
                        if t < parent_block_limit {
                            break;
                        }
                    }
                } else {
                    // incremental case
                    level_position = (level_position.as_ivec3() + mask.as_ivec3()).as_uvec3();
                    block_limit_v += mask * (level_dim_div as f32);
                    continue;
                }
            } else {
                block_indexs[level as usize] = parent_index;
                block_limits[level as usize] = parent_block_limit;
                level += 1;
                parent_index = block_data as usize * (Self::BLOCK_SIZE as usize);
                level_dim_div /= BLOCK_DIM; // must be here or we get 1 / N
                parent_block_limit = ts_min;
            }
            block_limit_v = ((position / (level_dim_div as f32)).floor() + ray_dir_limit_mul) * (level_dim_div as f32);
            level_position = vec3_to_usvt3(position) / level_dim_div % BLOCK_DIM;
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

#[cfg(not(target_arch = "spirv"))]
pub type SvtMut<const BLOCK_DIM: usvt, const LEVEL_COUNT: usize> =
    Svt<Vec<usvt>, BLOCK_DIM, LEVEL_COUNT>;

#[cfg(not(target_arch = "spirv"))]
impl<const BLOCK_DIM: usvt, const LEVEL_COUNT: usize> Svt<Vec<usvt>, BLOCK_DIM, LEVEL_COUNT> {
    pub fn init(material: usvt) -> Self {
        let mem = vec![0; (Self::BLOCK_SIZE as usize) * 10];
        let mut svt = Svt { mem };
        // the root pointer is 1, here nothing is allocated, so we set it to 1
        svt.mem[0] = svt.root_block_index() as usvt;
        svt.alloc_new_block(material);
        return svt;
    }

    pub fn block_count(&self) -> usize {
        return self.usvo_used() / (Self::BLOCK_SIZE as usize);
    }

    pub fn usvo_used(&self) -> usize {
        return self.mem.len()
    }

    // in bytes
    pub fn memory_used(&self) -> usize {
        return self.usvo_used() * if (usvt::MAX as u32) == u32::MAX { 4 } else { 2 };
    }

    // memory ratio assuming each block use a byte of memory.
    pub fn memory_ratio(&self) -> f32 {
        let size = self.memory_used() as f64;
        let native_size = ((BLOCK_DIM as f64).powf(LEVEL_COUNT as f64)).powf(3.0);
        return (size / native_size) as f32;
    }

    pub fn capacity(&self) -> usize {
        self.mem.capacity()
    }

    // pub fn checksum(&self) -> u64 {
    //        use std::{collections::hash_map::DefaultHasher, hash::Hasher, vec::*};
    //     let mut hasher = DefaultHasher::new();
    //     let usvt_used =  self.usvt_used();
    //     for i in 0..usvt_used {
    //         hasher.write_u16(self.mem[i]);
    //     }
    //     hasher.finish()
    // }

    // fn remove_blocks_at_index(&mut self, index: usize) {
    //     // TODO remove unused blocks
    // }

    fn alloc_new_block(&mut self, material: usvt) -> usvt {
        let cur_top = self.mem[0];
        self.mem[0] = cur_top + 1;
        // here we need to allocate extra block, some padding issues...
        while self.mem.len() < ((cur_top + 1) as usize) * (Self::BLOCK_SIZE as usize) {
            self.mem.push(material);
        }
        for i in 0..Self::BLOCK_SIZE {
            self.mem[(cur_top as usize) * (Self::BLOCK_SIZE as usize) + (i as usize)] =
                Self::new_block(true, material);
        }
        return cur_top;
    }

    // the position is a "representative" position
    pub fn set_with_level_cap(&mut self, level_cap: usvt, position: Usvt3, material: usvt) {
        let mut first_block_index = self.root_block_index() * (Self::BLOCK_SIZE as usize);
        let mut level = 0;
        while level < level_cap {
            let level_position = self.level_position(position, level);
            let target_block_index = first_block_index + Self::encode(level_position);
            let target_block = self.mem[target_block_index];
            if level == level_cap - 1 {
                self.mem[target_block_index] = Self::new_block(true, material);
                return;
            } else {
                let mut index = Self::block_index_data(target_block);
                if Self::is_terminal_block(target_block) {
                    if index == material {
                        // already is that
                        return;
                    }
                    index = self.alloc_new_block(index);
                    self.mem[target_block_index] = Self::new_block(false, index);
                }
                first_block_index = index as usize * (Self::BLOCK_SIZE as usize);
            }
            level += 1;
        }
        panic!("not allowed to be here");
    }

    pub fn set(&mut self, position: Usvt3, material: usvt) {
        let mut first_block_index = self.root_block_index() * (Self::BLOCK_SIZE as usize);
        let mut level = 0;
        while level < LEVEL_COUNT as usvt {
            let level_position = self.level_position(position, level);
            let target_block_index = first_block_index + Self::encode(level_position);
            let target_block = self.mem[target_block_index];
            // the last level, get the block directly
            if level == LEVEL_COUNT as usvt - 1 {
                self.mem[target_block_index] = Self::new_block(true, material);
            } else {
                let mut index = Self::block_index_data(target_block);
                if Self::is_terminal_block(target_block) {
                    if index == material {
                        // already is that
                        return;
                    }
                    index = self.alloc_new_block(index);
                    self.mem[target_block_index] = Self::new_block(false, index);
                }
                first_block_index = index as usize * (Self::BLOCK_SIZE as usize);
            }
            level += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb, RgbImage};
    use rand::Rng;
    use sdfu::SDF;

    #[test]
    fn simple_image_render() {
        const BLOCK_DIM: usvt = 2;
        const LEVEL: usize = 2;
        let mut svt = Svt::<Vec<usvt>, BLOCK_DIM, LEVEL>::init(0);
        let total = BLOCK_DIM.pow(LEVEL as u32);
        // svt.set(Usvt3(0, 0, 0), 1);
        svt.set(Usvt3::new(0, 2, 1), 2);
        svt.set(Usvt3::new(2, 0, 1), 3);
        svt.set(Usvt3::new(7, 3, 2), 8);
        let image_size = 1000;
        let mut image: RgbImage = ImageBuffer::new(image_size, image_size);
        for i in 0..image_size {
            for j in 0..image_size {
                let ray = Ray3 {
                    pos: Vec3::new(-100.0, -100.0, 0.0),
                    dir: (Vec3::new(i as f32, image_size as f32, image_size as f32)
                        / (image_size as f32)
                        * (total as f32)).normalize(),
                };
                // let ray = Ray3 {
                //     pos: Vec3::new(i as f32 + 0.4, j as f32 + 0.4, 0.1) / (image_size as f32) * (TOTAL as f32),
                //     dir: Vec3::new(0.1, 0.1, 1.0),
                // };
                svt.traverse_ray(100, ray, |_, out_info, block| {
                    let hit = block.data != 0;
                    if hit {
                        let light_level = vec3(0.6, 0.75, 1.0);
                        let color =
                            vec3(0.3, 0.7, 0.5) * light_level.dot(out_info.mask.abs()) * 255.0;
                        image.put_pixel(i, j, Rgb([color.x as u8, color.y as u8, color.z as u8]));
                    }
                    return hit;
                });
            }
        }
        image.save("test_svt_simple.png").unwrap();
    }

    type MySvt = Svt<Vec<usvt>, 4, 4>;

    #[test]
    fn simple_debug() {
        let mut svt = MySvt::init(0);
        let size = (MySvt::TOTAL_DIM - 10) as f32;
        let mut rng = rand::thread_rng();
        for i in 0..1000 {
            let v = vec3(rng.gen(), rng.gen(), rng.gen()) * size;
            let v = v.min(Vec3::splat(size)).max(Vec3::ZERO);
            svt.set(vec3_to_usvt3(v), i as usvt);
        }
        for _ in 0..100000 {
            let error_code = svt.traverse_ray(
                300,
                Ray3 {
                    pos: vec3(rng.gen(), rng.gen(), rng.gen()) * size,
                    dir: (vec3(rng.gen(), rng.gen(), rng.gen()) * size * 2.0 - Vec3::splat(size)).try_normalize_or(Vec3::X),
                },
                |_, _, _| {
                    return false;
                },
            );
            if error_code < 0 {
                panic!();
            }
        }
    }

    #[test]
    fn sdf1() {
        let sdf = sdfu::Sphere::new(0.45)
            .subtract(sdfu::Box::new(Vec3A::new(0.25, 0.25, 1.5)))
            .union_smooth(
                sdfu::Sphere::new(0.3).translate(Vec3A::new(0.3, 0.3, 0.0)),
                0.1,
            )
            .union_smooth(
                sdfu::Sphere::new(0.3).translate(Vec3A::new(-0.3, 0.3, 0.0)),
                0.1,
            )
            .subtract(
                sdfu::Box::new(Vec3A::new(0.125, 0.125, 1.5)).translate(Vec3A::new(-0.3, 0.3, 0.0)),
            )
            .subtract(
                sdfu::Box::new(Vec3A::new(0.125, 0.125, 1.5)).translate(Vec3A::new(0.3, 0.3, 0.0)),
            )
            .subtract(
                sdfu::Box::new(Vec3A::new(1.5, 0.1, 0.1)).translate(Vec3A::new(0.0, 0.3, 0.0)),
            )
            .subtract(sdfu::Box::new(Vec3A::new(0.2, 2.0, 0.2)))
            .translate(Vec3A::new(0.5, 0.5, 0.5));
        let mut svt = Svt::<Vec<usvt>, 4, 4>::init(0);
        let level_count = 4 as usvt;
        let block_size = 4 as usvt;
        let total_size = block_size.pow(level_count as u32) as f32;
        for level in 0..level_count {
            let level_cap = level + 1;
            let level_dim = block_size.pow(level_cap as u32);
            let level_size = block_size.pow((level_count - level_cap) as u32);
            for x in 0..level_dim {
                for y in 0..level_dim {
                    for z in 0..level_dim {
                        let mut count = 0;
                        for i in 0..level_size {
                            for j in 0..level_size {
                                for k in 0..level_size {
                                    let v = Vec3A::new(
                                        (x * level_size + i) as f32,
                                        (y * level_size + j) as f32,
                                        (z * level_size + k) as f32,
                                    ) / total_size;
                                    if sdf.dist(v) < 0.0 {
                                        count += 1;
                                    }
                                }
                            }
                        }
                        let material = if count > level_size * level_size * level_size / 2 {
                            1
                        } else {
                            0
                        };
                        svt.set_with_level_cap(
                            level_cap,
                            Usvt3::new(x * level_size, y * level_size, z * level_size),
                            material,
                        );
                    }
                }
            }
        }
        let mut image: RgbImage = ImageBuffer::new(100, 100);
        for i in 0..100 {
            for j in 0..100 {
                let mut hit = false;
                let ray = Ray3 {
                    pos: Vec3::new(i as f32, j as f32, 200.0) / 100.0 * 256.0,
                    dir: Vec3::new(0.1, 0.1, -1.0).normalize(),
                };
                svt.traverse_ray(100, ray, |_, _, block| {
                    hit = block.data == 1;
                    if hit {
                        image.put_pixel(i, j, Rgb([255, 0, 0]));
                    }
                    return hit;
                });
            }
        }
        image.save("test_svt.png").unwrap();
    }
}
