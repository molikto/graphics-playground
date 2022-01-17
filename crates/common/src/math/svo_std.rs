use super::svo::*;
use super::vec::*;

pub type SvoMut<const LEVEL_COUNT: usize> =
    Svo<Vec<usvo>, LEVEL_COUNT>;

use rand::Rng;

impl<const LEVEL_COUNT: usize> Svo<Vec<usvo>, LEVEL_COUNT> {
    pub fn init(material: usvo) -> Self {
        let mem = vec![0; (Self::BLOCK_SIZE as usize) * 10];
        let mut svo = Svo { mem };
        // the root pointer is 1, here nothing is allocated, so we set it to 1
        svo.mem[0] = svo.root_block_index() as usvo;
        svo.alloc_new_block(material);
        return svo;
    }

    pub fn block_count(&self) -> usize {
        return self.usvo_used() / (Self::BLOCK_SIZE as usize);
    }

    pub fn usvo_used(&self) -> usize {
        return self.mem.len();
    }

    // in bytes
    pub fn memory_used(&self) -> usize {
        return self.usvo_used() * if (usvo::MAX as u32) == u32::MAX { 4 } else { 2 };
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
    //     let usvo_used =  self.usvo_used();
    //     for i in 0..usvo_used {
    //         hasher.write_u16(self.mem[i]);
    //     }
    //     hasher.finish()
    // }

    // fn remove_blocks_at_index(&mut self, index: usize) {
    //     // TODO remove unused blocks
    // }

    fn alloc_new_block(&mut self, material: usvo) -> usvo {
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
    pub fn set_with_level_cap(&mut self, level_cap: usvo, position: Usvo3, material: usvo) {
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

    pub fn set(&mut self, position: Usvo3, material: usvo) {
        let mut first_block_index = self.root_block_index() * (Self::BLOCK_SIZE as usize);
        let mut level = 0;
        while level < LEVEL_COUNT as usvo {
            let level_position = self.level_position(position, level);
            let target_block_index = first_block_index + Self::encode(level_position);
            let target_block = self.mem[target_block_index];
            // the last level, get the block directly
            if level == LEVEL_COUNT as usvo - 1 {
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

    #[cfg(feature = "svo-vox")]
    pub fn load_from_vox(
        path: &Path,
    ) -> (SvoMut<LEVEL_COUNT>, Vec<Material>) {
        let mut svo: SvoMut<LEVEL_COUNT> = Svo::init(0);
        let data = dot_vox::load(path.to_str().unwrap()).unwrap();
        let mut materials: Vec<Material> = Vec::new();
        for model in data.models {
            for v in model.voxels {
                svo.set(
                    Usvo3::new(v.x as usvo, v.y as usvo, v.z as usvo),
                    (v.i + 1) as usvo,
                );
            }
        }
        (svo, materials)
    }

    fn set_from_rsvo(
        &mut self,
        level: usize,
        position: Usvo3,
    ) {
        let material = rand::thread_rng().gen_range(1..4);
        self.set_with_level_cap(level as usvo, position, material);
    }

    fn load_rsvo_rec(
        &mut self,
        rsvo: &[u8],
        level_count: usize,
        level_diff: usize,
        indexes: &mut [usize; 20],
        level: usize,
        position: Usvo3,
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
                let op = Usvo3::new(i / 4, (i % 4) / 2, i % 2);
                if bit != 0 {
                    self.load_rsvo_rec(
                        rsvo,
                        level_count,
                        level_diff,
                        indexes,
                        level + 1,
                        position
                            + op * (2u32.pow((level_count - (level as usize) - 1) as u32)) as usvo,
                    );
                }
            }
        }
    }

    pub fn load_from_rsvo(
        rsvo: &[u8],
        max_pow_2_level: usize,
    ) -> Self {
        let mut svo: Self = Svo::init(0);
        // TODO fix this transmute
        let rsvo32 = unsafe { std::mem::transmute::<&[u8], &[u32]>(rsvo) };
        let level_max = LEVEL_COUNT;
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
        svo.load_rsvo_rec(
            rsvo,
            max_pow_2_level,
            level_diff,
            &mut indexes,
            0,
            Usvo3::new(0, 0, 0),
        );
        svo
    }
}

#[cfg(test)]
mod tests {
    use crate::Ray3;

    use super::*;
    use image::{ImageBuffer, Rgb, RgbImage};
    use rand::Rng;

    #[test]
    fn simple_image_render() {
        const LEVEL: usize = 2;
        let mut svo = Svo::<Vec<usvo>, LEVEL>::init(0);
        let total = BLOCK_DIM.pow(LEVEL as u32);
        // svo.set(Usvo3(0, 0, 0), 1);
        svo.set(Usvo3::new(0, 2, 1), 2);
        svo.set(Usvo3::new(2, 0, 1), 3);
        svo.set(Usvo3::new(7, 3, 2), 8);
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
                svo.traverse_ray(100, ray, |_, out_info, block| {
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
        image.save("test_svo_simple.png").unwrap();
    }

    type MySvo = Svo<Vec<usvo>, 8>;

    #[test]
    fn simple_debug() {
        let mut svo = MySvo::init(0);
        let size = (MySvo::TOTAL_DIM - 10) as f32;
        let mut rng = rand::thread_rng();
        for i in 0..1000 {
            let v = vec3(rng.gen(), rng.gen(), rng.gen()) * size;
            let v = v.min(Vec3::splat(size)).max(Vec3::ZERO);
            svo.set(vec3_to_usvo3(v), i as usvo);
        }
        for _ in 0..100000 {
            let error_code = svo.traverse_ray(
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
}
