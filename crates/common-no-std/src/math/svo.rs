use crate::Aabb3;

use super::ray::*;
use super::vec::*;

use num_traits::Float;

pub struct BlockInfo<const BLOCK_DIM: u32, const LEVEL_COUNT: usize> {
    pub level_position_abs: UVec3,
    pub level: u32,
    pub data: u32,
}

impl<const BLOCK_DIM: u32, const LEVEL_COUNT: usize> BlockInfo<BLOCK_DIM, LEVEL_COUNT> {
    pub fn size(&self) -> u32 {
        BLOCK_DIM.pow(LEVEL_COUNT as u32 - self.level - 1)
    }
}

pub struct Svo<'a, const BLOCK_DIM: u32, const LEVEL_COUNT: usize> {
    pub mem: &'a mut [u32], // each alloc pointer means if a 8 node sequence is vacant
}

impl<'a, const BLOCK_DIM: u32, const LEVEL_COUNT: usize> Svo<'a, BLOCK_DIM, LEVEL_COUNT> {
    const BLOCK_SIZE: u32 = BLOCK_DIM * BLOCK_DIM * BLOCK_DIM;
    //
    // memory management
    //

    pub fn total_dim() -> u32 {
        BLOCK_DIM.pow(LEVEL_COUNT as u32)
    }

    pub fn init(mem: &'a mut [u32], material: u32) -> Self {
        let mut svo = Svo { mem };
        // the root pointer is 1, here nothing is allocated, so we set it to 1
        svo.mem[0] = svo.root_block_index() as u32;
        svo.alloc_new_block(material);
        return svo;
    }

    // TODO memory allocation
    pub fn root_block_index(&self) -> usize {
        return 1;
    }

    pub fn mem_used(&self) -> usize {
        return self.mem[0] as usize - self.root_block_index();
    }

    pub fn block_count(&self) -> usize {
        return (self.mem[0] as usize - self.root_block_index()) / (Self::BLOCK_SIZE as usize);
    }

    pub fn memory_ratio(&self) -> f32 {
        let size = (self.mem[0] as usize - self.root_block_index()) * 4;
        let native_size = (BLOCK_DIM.pow(LEVEL_COUNT as u32)).pow(3);
        return size as f32 / (native_size as f32);
    }

    fn alloc_new_block(&mut self, material: u32) -> u32 {
        let cur_top = self.mem[0];
        self.mem[0] = cur_top + Self::BLOCK_SIZE;
        for i in 0u32..Self::BLOCK_SIZE {
            self.mem[(cur_top + i) as usize] = Self::new_block(true, material);
        }
        return cur_top;
    }

    //
    // helpers
    //

    #[inline]
    pub fn encode(v: UVec3) -> usize {
        return (v[0] * BLOCK_DIM * BLOCK_DIM + v[1] * BLOCK_DIM + v[2]) as usize;
    }

    //
    // block API
    #[inline]
    pub fn is_terminal_block(u: u32) -> bool {
        u % 2u32 == 0u32
    }

    #[inline]
    pub fn block_index_data(u: u32) -> u32 {
        u >> 1u32
    }

    #[inline]
    fn new_block(terminal: bool, index: u32) -> u32 {
        if terminal {
            return index << 1u32;
        } else {
            return index << 1u32 | 1u32;
        }
    }

    //
    // modification API
    //

    fn remove_blocks_at_index(&mut self, index: usize) {
        // TODO remove unused blocks
    }

    #[inline]
    pub fn level_position_abs(&self, position: UVec3, level: u32) -> UVec3 {
        return position / (BLOCK_DIM.pow(LEVEL_COUNT as u32 - 1 - level));
    }

    #[inline]
    pub fn level_position(&self, position: UVec3, level: u32) -> UVec3 {
        return self.level_position_abs(position, level) % BLOCK_DIM;
    }

    #[inline]
    pub fn traverse_ray<C>(&self, mut ray: Ray3, mut closure: C) ->  u32
    where
        C: FnMut(BlockInfo<BLOCK_DIM, LEVEL_COUNT>, Vec3, u32) -> bool,
    {
        // the max dim we can have is 8^8, otherwise it will not work because of floating point issue
        // https://itectec.com/matlab-ref/matlab-function-flintmax-largest-consecutive-integer-in-floating-point-format/
        ray.dir = ray.dir.normalize_or_zero();
        if ray.dir == Vec3::ZERO {
            return 3;
        }
        let ray_dir_signum = ray.dir.signum();
        let ray_dir_limit_mul = (ray_dir_signum + 1.0) / 2.0;
        let total_dim = Self::total_dim() as f32;
        let aabb = Aabb3::new(Vec3::ZERO, Vec3::splat(total_dim));
        let mut mask: Vec3;
        let mut position: Vec3;
        if aabb.inside(ray.pos) {
            mask = Vec3::ZERO;
            position = ray.pos;
        } else {
            let hit = aabb.hit(&ray, 0.0, 100000000000.0);
            mask = hit.nor;
            position = ray.at(hit.t + 0.0001);
        }
        let mut t = 0 as f32;
        let mut block_indexs = [0usize; LEVEL_COUNT];
        block_indexs[0] = self.root_block_index();

        let mut block_limits = [Vec3::ZERO; LEVEL_COUNT];
        block_limits[0] = ray_dir_limit_mul * total_dim;
        // there is a off by one error...
        let mut block_limit: Vec3;
        //
        // block aabbs is terminal block
        let mut level = 0u32;
        let mut position_up = UVec3::ZERO;
        loop {
            loop {
                // exit levels first
                let block_limit = block_limits[level as usize];
                let test = ((block_limit - position).signum() * ray_dir_signum).sum();
                if test == 3.0 {
                    break;
                } else if level == 0 {
                    return 0;
                } else {
                    level -= 1;
                }
            }
            // level = 0;
            // let mut block_limit = ray_dir_limit_mul * Self::total_dim() as f32;
            // let test = ((block_limit - position).signum() * ray_dir_signum).sum();
            // if test != 3.0 {
            //     return;
            // }
            // go inside levels
            let mut level_position_abs: UVec3;
            let mut index: u32;
            let position_u = position.as_uvec3();
            if position_u == position_up {
                return 4;
            }
            position_up = position_u;
            loop {
                level_position_abs = self.level_position_abs(position_u, level);
                let level_position = level_position_abs % BLOCK_DIM;
                let target_block_index =
                    block_indexs[level as usize] + Self::encode(level_position);
                let target_block = self.mem[target_block_index];
                index = Self::block_index_data(target_block);
                level += 1;
                let block_size = BLOCK_DIM.pow(LEVEL_COUNT as u32 - level) as f32;
                block_limit = (level_position_abs.as_vec3() + ray_dir_limit_mul) * block_size;
                if Self::is_terminal_block(target_block) {
                    break;
                }
                block_limits[level as usize] = block_limit;
                block_indexs[level as usize] = index as usize;
            }
            let ts = (block_limit - ray.pos) / ray.dir;
            let ts_min = ts.min_element();
            if ts_min < t {
                return 1;
            }
            t = ts_min;
            let ret = closure(
                BlockInfo {
                    level_position_abs,
                    level,
                    data: index,
                },
                mask,
                level,
            );
            if ret {
                return 0;
            }
            // set mask later, because we want to know the mask of the enter face
            mask = ts.step_f(t) * ray_dir_signum;
            if mask == Vec3::ZERO {
                return 2;
            }

            // this floating point position is because we use unsigned position_u, it's actually value doesn't mater that much
            // we add extra value so we don't step on the boundary. `position = ray.at(t + 0.01)` doesn't work
            position = ray.at(t) + mask / 2.0;

            // here we can always go out a level
            level -= 1;
        }
    }

    pub fn get(&mut self, position: UVec3) -> u32 {
        let mut level = 0;
        let mut first_block_index = self.root_block_index();
        // get the block at that position, create new blocks if needed
        while level < LEVEL_COUNT as u32 {
            let level_position = self.level_position(position, level);
            let target_block_index = first_block_index + Self::encode(level_position);
            let target_block = self.mem[target_block_index];
            let mut index = Self::block_index_data(target_block);
            if Self::is_terminal_block(target_block) {
                return index;
            }
            first_block_index = index as usize;
            level += 1;
        }
        panic!("not allowed to be here");
    }

    // the position is a "representative" position
    pub fn set_with_level_cap(&mut self, level_cap: u32, position: UVec3, material: u32) {
        let mut first_block_index = self.root_block_index();
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
                first_block_index = index as usize;
            }
            level += 1;
        }
        panic!("not allowed to be here");
    }

    pub fn set(&mut self, position: UVec3, material: u32) {
        let mut first_block_index = self.root_block_index();
        let mut level = 0;
        while level < LEVEL_COUNT as u32 {
            let level_position = self.level_position(position, level);
            let target_block_index = first_block_index + Self::encode(level_position);
            let target_block = self.mem[target_block_index];
            // the last level, get the block directly
            if level == LEVEL_COUNT as u32 - 1 {
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
                first_block_index = index as usize;
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

    pub trait SvoExtension {
        fn traverse_inner(
            &self,
            closure: &mut Vec<(u32, UVec3, u32, u32)>,
            level: u32,
            pos: UVec3,
            first_index: usize,
        );
        fn debug_items(&self) -> Vec<u32>;
        fn traverse<C>(&self, closure: C)
        where
            C: FnMut(u32, UVec3, u32, u32);
    }

    impl<'a, const BLOCK_DIM: u32, const LEVEL_COUNT: usize> SvoExtension
        for Svo<'a, BLOCK_DIM, LEVEL_COUNT>
    {
        fn debug_items(&self) -> Vec<u32> {
            self.mem[self.root_block_index()..self.mem[0] as usize].to_vec()
        }
        fn traverse_inner(
            &self,
            closure: &mut Vec<(u32, UVec3, u32, u32)>,
            level: u32,
            pos: UVec3,
            first_index: usize,
        ) {
            for x in 0..BLOCK_DIM {
                for y in 0..BLOCK_DIM {
                    for z in 0..BLOCK_DIM {
                        let pp = uvec3(x, y, z);
                        let index = first_index as usize + Self::encode(pp);
                        let block = self.mem[index];
                        let data = Self::block_index_data(block);
                        let block_size = BLOCK_DIM.pow(LEVEL_COUNT as u32 - level - 1);
                        let ppp = pos + pp * block_size;
                        if Self::is_terminal_block(block) {
                            closure.push((level, ppp, block_size, data));
                        } else {
                            self.traverse_inner(closure, level + 1, ppp, data as usize)
                        }
                    }
                }
            }
        }

        fn traverse<C>(&self, mut closure: C)
        where
            C: FnMut(u32, UVec3, u32, u32),
        {
            let mut cs = vec![];
            self.traverse_inner(&mut cs, 0, UVec3::ZERO, self.root_block_index() as usize);
            for c in cs {
                closure(c.0, c.1, c.2, c.3);
            }
        }
    }

    #[test]
    fn simple_image_render() {
        let mut mem0 = vec![0; 10000];
        let mem = &mut mem0;
        const BLOCK_DIM: u32 = 2;
        const LEVEL: usize = 2;
        let mut svo = Svo::<BLOCK_DIM, LEVEL>::init(mem, 0);
        let TOTAL = BLOCK_DIM.pow(LEVEL as u32);
        // svo.set(uvec3(0, 0, 0), 1);
        svo.set(uvec3(0, 2, 1), 2);
        svo.set(uvec3(2, 0, 1), 3);
        svo.set(uvec3(7, 3, 2), 8);
        let image_size = 1000;
        let mut image: RgbImage = ImageBuffer::new(image_size, image_size);
        for i in 0..image_size {
            for j in 0..image_size {
                let ray = Ray3 {
                    pos: Vec3::new(-100.0, -100.0, 0.0),
                    dir: Vec3::new(i as f32, image_size as f32, image_size as f32)
                        / (image_size as f32)
                        * (TOTAL as f32),
                };
                // let ray = Ray3 {
                //     pos: Vec3::new(i as f32 + 0.4, j as f32 + 0.4, 0.1) / (image_size as f32) * (TOTAL as f32),
                //     dir: Vec3::new(0.1, 0.1, 1.0),
                // };
                svo.traverse_ray(ray, |block, hit_mask, _| {
                    let hit = block.data != 0;
                    if hit {
                        let light_level = vec3(0.6, 0.75, 1.0);
                        let color = vec3(0.3, 0.7, 0.5) * light_level.dot(hit_mask.abs()) * 255.0;
                        image.put_pixel(
                            i,
                            j,
                            Rgb([color[0] as u8, color[1] as u8, color[2] as u8]),
                        );
                    }
                    return hit;
                });
            }
        }
        image.save("test_svo_simple.png").unwrap();
    }

    type MySvo<'a> = Svo<'a, 4, 4>;

    #[test]
    fn simple_debug() {
        println!("fds");
        let mut mem0 = vec![0; 1000000];
        let mem = &mut mem0;
        let mut svo = MySvo::init(mem, 0);
        let size = (MySvo::total_dim() - 10) as f32;
        let mut rng = rand::thread_rng();
        for i in 0..1000 {
            let v = vec3(rng.gen(), rng.gen(), rng.gen()) * size;
            let v = v.min(Vec3::splat(size)).max(Vec3::ZERO);
            svo.set(v.as_uvec3(), i as u32);
        }
        for i in 0..100000 {
            let mut count = 0;
            let error_code = svo.traverse_ray(
                Ray3 {
                    pos: vec3(rng.gen(), rng.gen(), rng.gen()) * size,
                    dir: vec3(rng.gen(), rng.gen(), rng.gen()) * size,
                },
                |block, mask, _| {
                    count += 1;
                    if count > 300 {
                        assert_eq!(0, 1);
                    }
                    return false;
                },
            );
            if error_code != 0 {
                println!("{}", error_code);
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
        let mut mem0 = vec![0; 10000000];
        let mem = &mut mem0;
        let mut svo = Svo::<4, 4>::init(mem, 0);
        let level_count = 4u32;
        let block_size = 4u32;
        let total_size = block_size.pow(level_count) as f32;
        for level in 0..level_count {
            let level_cap = level + 1;
            let level_dim = block_size.pow(level_cap);
            let level_size = block_size.pow(level_count - level_cap);
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
                        svo.set_with_level_cap(
                            level_cap,
                            uvec3(x * level_size, y * level_size, z * level_size),
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
                    dir: Vec3::new(0.1, 0.1, -1.0),
                };
                svo.traverse_ray(ray, |block, _, _| {
                    hit = block.data == 1;
                    if hit {
                        image.put_pixel(i, j, Rgb([255, 0, 0]));
                    }
                    return hit;
                });
            }
        }
        image.save("test_svo.png").unwrap();
    }
}
