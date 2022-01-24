use super::svt::*;
use super::vec::*;

pub type SvtMut<T, const BLOCK_DIM: usvt, const LEVEL_COUNT: usize> =
    Svt<T, Vec<usvt>, BLOCK_DIM, LEVEL_COUNT>;

use rand::Rng;

impl<T: SvtData, const BLOCK_DIM: usvt, const LEVEL_COUNT: usize> Clone for SvtMut<T, BLOCK_DIM, LEVEL_COUNT> {
    fn clone(&self) -> Self {
        Self::new_wrap(self.mem.clone())
    }
}

impl<T: SvtData, const BLOCK_DIM: usvt, const LEVEL_COUNT: usize> Svt<T, Vec<usvt>, BLOCK_DIM, LEVEL_COUNT> {
    pub fn new(material: T) -> Self {
        let mem = vec![0; (Self::BLOCK_SIZE as usize) * 10];
        let mut svt = Svt::new_wrap(mem);
        // the root pointer is 1, here nothing is allocated, so we set it to 1
        svt.mem[0] = svt.root_block_index() as usvt;
        svt.alloc_new_block(material);
        return svt;
    }

    #[inline]
    fn sample_single<C>(&mut self, closure: &mut C, level_cap: u32, level_size: u32, level_pos: Usvt3)
    where
        C: FnMut(UVec3) -> T,
    {
        let material = closure(level_pos);
        for i in 0..level_size {
            for j in 0..level_size {
                for k in 0..level_size {
                    let v = level_pos + uvec3(i, j, k);
                    let new_material= closure(v);
                    if new_material != material {
                        return;
                    }
                }
            }
        }
        self.set_with_level_cap(
            level_cap,
            level_pos,
            material,
        );
    }

    // TODO make near by blocks nearby in memory
    #[inline]
    pub fn sample<C>(&mut self, closure: &mut C)
    where
        C: FnMut(UVec3) -> T,
    {
        for level in 0..LEVEL_COUNT as usvt {
            let level_cap = level + 1;
            let level_dim = BLOCK_DIM.pow(level_cap as u32);
            let level_size = BLOCK_DIM.pow(LEVEL_COUNT as u32 - (level_cap as u32));
            for x in 0..level_dim {
                for y in 0..level_dim {
                    for z in 0..level_dim {
                        let level_pos = uvec3(x, y, z) * level_size;
                        self.sample_single(closure, level_cap, level_size, level_pos);
                    }
                }
            }
        }
    }

    pub fn block_count(&self) -> usize {
        return self.usvo_used() / (Self::BLOCK_SIZE as usize);
    }

    pub fn usvo_used(&self) -> usize {
        return self.mem.len();
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
    // }

    fn alloc_new_block(&mut self, material: T) -> usvt {
        let cur_top = self.block_count();
        // here we need to allocate extra block, some padding issues...
        while self.mem.len() < ((cur_top + 1) as usize) * (Self::BLOCK_SIZE as usize) {
            self.mem.push(material.into());
        }
        for i in 0..Self::BLOCK_SIZE {
            self.mem[(cur_top as usize) * (Self::BLOCK_SIZE as usize) + (i as usize)] =
                Self::new_block(material);
        }
        return cur_top as usvt;
    }

    // the position is a "representative" position
    pub fn set_with_level_cap(&mut self, level_cap: usvt, position: Usvt3, material: T) {
        let mut first_block_index = self.root_block_index() * (Self::BLOCK_SIZE as usize);
        let mut level = 0;
        while level < level_cap {
            let level_position = self.level_position(position, level);
            let target_block_index = first_block_index + Self::encode(level_position);
            let target_block = self.mem[target_block_index];
            if level == level_cap - 1 {
                self.mem[target_block_index] = Self::new_block(material);
                return;
            } else {
                let mut index = Self::block_index_data(target_block);
                if Self::is_terminal_block(target_block) {
                    let old_material: T = index.into();
                    if old_material == material {
                        // already is that
                        return;
                    }
                    index = self.alloc_new_block(old_material);
                    self.mem[target_block_index] = Self::new_leaf_block(index);
                }
                first_block_index = index as usize * (Self::BLOCK_SIZE as usize);
            }
            level += 1;
        }
        panic!("not allowed to be here");
    }

    pub fn set(&mut self, position: Usvt3, material: T) {
        let mut first_block_index = self.root_block_index() * (Self::BLOCK_SIZE as usize);
        let mut level = 0;
        while level < LEVEL_COUNT as usvt {
            let level_position = self.level_position(position, level);
            let target_block_index = first_block_index + Self::encode(level_position);
            let target_block = self.mem[target_block_index];
            // the last level, get the block directly
            if level == LEVEL_COUNT as usvt - 1 {
                self.mem[target_block_index] = Self::new_block(material);
            } else {
                let mut index = Self::block_index_data(target_block);
                if Self::is_terminal_block(target_block) {
                    let old_material: T = index.into();
                    if old_material == material {
                        // already is that
                        return;
                    }
                    index = self.alloc_new_block(old_material);
                    self.mem[target_block_index] = Self::new_leaf_block(index);
                }
                first_block_index = index as usize * (Self::BLOCK_SIZE as usize);
            }
            level += 1;
        }
    }

    #[cfg(feature = "svo-vox")]
    pub fn load_from_vox(path: &Path) -> (SvtMut<BLOCK_DIM, LEVEL_COUNT>, Vec<Material>) {
        let mut svt: SvtMut<BLOCK_DIM, LEVEL_COUNT> = Svt::init(0);
        let data = dot_vox::load(path.to_str().unwrap()).unwrap();
        let mut materials: Vec<Material> = Vec::new();
        for model in data.models {
            for v in model.voxels {
                svt.set(
                    Usvt3::new(v.x as usvt, v.y as usvt, v.z as usvt),
                    (v.i + 1) as usvt,
                );
            }
        }
        (svt, materials)
    }

    fn set_from_rsvo(&mut self, level: usize, position: Usvt3) {
        let material: T = rand::thread_rng().gen_range(1..4).into();
        if BLOCK_DIM == 2 {
            self.set_with_level_cap(level as usvt, position, material);
        } else if BLOCK_DIM == 4 {
            if level % 2 == 0 {
                self.set_with_level_cap((level / 2) as usvt, position, material);
            } else {
                let level = level / 2;
                let level_position_abs = self.level_position_abs(position, level as usvt);
                for x in 0..2 {
                    for y in 0..2 {
                        for z in 0..2 {
                            let level_position = Usvt3::new(
                                level_position_abs.x + x,
                                level_position_abs.y + y,
                                level_position_abs.z + z,
                            );
                            self.set_with_level_cap(
                                level as usvt,
                                self.level_position_abs_to_position(level_position, level as usvt),
                                material,
                            );
                        }
                    }
                }
            }
        } else if BLOCK_DIM == 8 {
            if level % 3 == 0 {
                self.set_with_level_cap((level / 3) as usvt, position, material);
            } else if level % 3 == 1 {
                let level = level / 3;
                let level_position_abs = self.level_position_abs(position, level as usvt);
                for x in 0..4 {
                    for y in 0..4 {
                        for z in 0..4 {
                            let level_position = Usvt3::new(
                                level_position_abs.x + x,
                                level_position_abs.y + y,
                                level_position_abs.z + z,
                            );
                            self.set_with_level_cap(
                                level as usvt,
                                self.level_position_abs_to_position(level_position, level as usvt),
                                material,
                            );
                        }
                    }
                }
            } else {
                let level = level / 3;
                let level_position_abs = self.level_position_abs(position, level as usvt);
                for x in 0..2 {
                    for y in 0..2 {
                        for z in 0..2 {
                            let level_position = Usvt3::new(
                                level_position_abs.x + x,
                                level_position_abs.y + y,
                                level_position_abs.z + z,
                            );
                            self.set_with_level_cap(
                                level as usvt,
                                self.level_position_abs_to_position(level_position, level as usvt),
                                material,
                            );
                        }
                    }
                }
            }
        } else {
            panic!();
        }
    }

    fn load_rsvo_rec(
        &mut self,
        rsvo: &[u8],
        level_count: usize,
        level_diff: usize,
        indexes: &mut [usize; 20],
        level: usize,
        position: Usvt3,
    ) {
        if level == level_count {
            self.set_from_rsvo(level + level_diff, position);
            return;
        }
        let children_mask = rsvo[indexes[level]];
        indexes[level] += 1;
        if children_mask == 0 {
            self.set_from_rsvo(level + level_diff, position);
        } else {
            for i in 0..8 {
                let bit = (children_mask >> i) & 1;
                let op = Usvt3::new(i / 4, (i % 4) / 2, i % 2);
                if bit != 0 {
                    self.load_rsvo_rec(
                        rsvo,
                        level_count,
                        level_diff,
                        indexes,
                        level + 1,
                        position
                            + op * (2u32.pow((level_count - (level as usize) - 1) as u32)) as usvt,
                    );
                }
            }
        }
    }

    pub fn println_debug(&self) {
        println!("total dim {}\nblock count {}\nmemory used {}\nmemory ratio {}", Self::TOTAL_DIM, self.block_count(), self.memory_used(), self.memory_ratio());
    }

    pub fn load_from_rsvo(rsvo: &[u8], max_pow_2_level: usize) -> Self {
        if BLOCK_DIM != 2 && BLOCK_DIM != 4 && BLOCK_DIM != 8 {
            panic!();
        }
        let mut svt: Self = Svt::new(T::EMPTY);
        // TODO fix this transmute
        let rsvo32 = unsafe { std::mem::transmute::<&[u8], &[u32]>(rsvo) };
        let level_max = if BLOCK_DIM == 2 {
            LEVEL_COUNT
        } else if BLOCK_DIM == 4 {
            LEVEL_COUNT * 2
        } else {
            LEVEL_COUNT * 3
        };
        if max_pow_2_level > level_max {
            panic!();
        }
        let level_count = rsvo32[4] as usize;
        if level_count < level_max {
            panic!();
        }
        let level_diff = level_max - max_pow_2_level;
        let mut indexes: [usize; 20] = [0; 20];
        indexes[0] = (level_count + 1 + 5) * 4;
        for i in 1..level_count {
            indexes[i] = indexes[i - 1] + rsvo32[i + 4] as usize;
        }
        svt.load_rsvo_rec(
            rsvo,
            max_pow_2_level,
            level_diff,
            &mut indexes,
            0,
            Usvt3::new(0, 0, 0),
        );
        svt
    }
}

#[cfg(test)]
mod tests {
    use crate::Ray3;

    use super::*;
    use image::{ImageBuffer, Rgb, RgbImage};
    use rand::Rng;
    use sdfu::SDF;

    #[test]
    fn simple_image_render() {
        const BLOCK_DIM: usvt = 2;
        const LEVEL: usize = 2;
        let mut svt = Svt::<usvt, Vec<usvt>, BLOCK_DIM, LEVEL>::new(0);
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
                        * (total as f32))
                        .normalize(),
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
                            vec3(0.3, 0.7, 0.5) * light_level.dot(out_info.mask.as_vec3().abs()) * 255.0;
                        image.put_pixel(i, j, Rgb([color.x as u8, color.y as u8, color.z as u8]));
                    }
                    return hit;
                });
            }
        }
        image.save("test_svt_simple.png").unwrap();
    }

    type MySvt = Svt<usvt, Vec<usvt>, 4, 4>;

    #[test]
    fn simple_debug() {
        let mut svt = MySvt::new(0);
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
                    dir: (vec3(rng.gen(), rng.gen(), rng.gen()) * size * 2.0 - Vec3::splat(size))
                        .try_normalize_or(Vec3::X),
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
        let mut svt = Svt::<usvt, Vec<usvt>, 4, 4>::new(0);
        let level_count = 4 as usvt;
        let block_size = 4 as usvt;
        let total_size = block_size.pow(level_count as u32) as f32;
        svt.sample(&mut |v| {
            if sdf.dist(v.as_vec3a() / total_size) < 0.0 {
                1
            } else {
                0
            }
        });
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
