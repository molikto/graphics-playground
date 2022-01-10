pub use common_no_std::svo::*;
use rand::Rng;
use std::path::Path;

use common_no_std::material::Material;



#[cfg(feature="svo-vox")]
pub fn load_svo_from_vox<const BLOCK_DIM: usvo, const LEVEL_COUNT: usize>(path: &Path) -> (SvoMut<BLOCK_DIM, LEVEL_COUNT>, Vec<Material>) {
    let mut svo: SvoMut<BLOCK_DIM, LEVEL_COUNT> = Svo::init(0);
    let data = dot_vox::load(path.to_str().unwrap()).unwrap();
    let mut materials: Vec<Material> = Vec::new();
    for model in data.models {
        for v in model.voxels {
            svo.set(Usvo3::new(v.x as usvo, v.y as usvo, v.z as usvo), (v.i + 1) as usvo);
        }
    }
    (svo, materials)
}

fn set_from_rsvo<const BLOCK_SIZE: usvo, const LEVEL_COUNT: usize>(
    mem: &mut SvoMut<BLOCK_SIZE, LEVEL_COUNT>,
    level: usize,
    position: Usvo3,
)  {
    let material = rand::thread_rng().gen_range(1..4);
    if BLOCK_SIZE == 2 {
        mem.set_with_level_cap(level as usvo, position, material);
    } else if BLOCK_SIZE == 4 {
        if level % 2 == 0 {
            mem.set_with_level_cap((level / 2) as usvo, position, material);
        } else {
            let level = level / 2;
            let level_position_abs = mem.level_position_abs(position, level as usvo);
            for x in 0..2 {
                for y in 0..2 {
                    for z in 0..2 {
                        let level_position = Usvo3::new(level_position_abs.x + x, level_position_abs.y + y, level_position_abs.z + z);
                        mem.set_with_level_cap(level as usvo, mem.level_position_abs_to_position(level_position, level as usvo), material);
                    }
                }
            }
        }
    } else if BLOCK_SIZE == 8 {
        if level % 3 == 0 {
            mem.set_with_level_cap((level / 3) as usvo, position, material);
        } else if level % 3 == 1 {
            let level = level / 3;
            let level_position_abs = mem.level_position_abs(position, level as usvo);
            for x in 0..4 {
                for y in 0..4 {
                    for z in 0..4 {
                        let level_position = Usvo3::new(level_position_abs.x + x, level_position_abs.y + y, level_position_abs.z + z);
                        mem.set_with_level_cap(level as usvo, mem.level_position_abs_to_position(level_position, level as usvo), material);
                    }
                }
            }
        } else {
            let level = level / 3;
            let level_position_abs = mem.level_position_abs(position, level as usvo);
            for x in 0..2 {
                for y in 0..2 {
                    for z in 0..2 {
                        let level_position = Usvo3::new(level_position_abs.x + x, level_position_abs.y + y, level_position_abs.z + z);
                        mem.set_with_level_cap(level as usvo, mem.level_position_abs_to_position(level_position, level as usvo), material);
                    }
                }
            }
        }
    } else {
        panic!();
    }
}

fn load_rsvo_rec<const BLOCK_SIZE: usvo, const LEVEL_COUNT: usize>(
    rsvo: &[u8],
    level_count: usize,
    level_diff: usize,
    mem: &mut SvoMut<BLOCK_SIZE, LEVEL_COUNT>,
    indexes: &mut [usize; 20],
    level: usize,
    position: Usvo3,
)  {
    if level == level_count {
        set_from_rsvo(mem, level + level_diff, position);
        return;
    }
    let children_mask = rsvo[indexes[level]];
    indexes[level] += 1;
    if children_mask == 0 {
        set_from_rsvo(mem, level + level_diff, position);
    } else {
        for i in 0..8 {
            let bit = (children_mask >> i) & 1;
            let op = Usvo3::new(i / 4, (i % 4) / 2, i % 2);
            if bit != 0 {
                load_rsvo_rec(rsvo, level_count, level_diff, mem, indexes, level + 1, position + op * (2u32.pow((level_count - (level as usize) - 1) as u32)) as usvo);
            }
        }
    }
}

pub fn load_svo_from_rsvo<const BLOCK_SIZE: usvo, const LEVEL_COUNT: usize>(rsvo: &[u8], max_pow_2_level: usize) -> SvoMut<BLOCK_SIZE, LEVEL_COUNT> {
    if BLOCK_SIZE != 2 && BLOCK_SIZE != 4 && BLOCK_SIZE != 8 {
        panic!();
    }
    let mut svo: SvoMut<BLOCK_SIZE, LEVEL_COUNT> = Svo::init(0);
    // TODO fix this transmute
    let rsvo32 = unsafe { std::mem::transmute::<&[u8], &[u32]>(rsvo) };
    let level_max = if BLOCK_SIZE == 2 { LEVEL_COUNT } else if BLOCK_SIZE == 4 { LEVEL_COUNT * 2 } else { LEVEL_COUNT * 3 };
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
    load_rsvo_rec(rsvo, max_pow_2_level, level_diff, &mut svo, &mut indexes, 0, Usvo3::new(0, 0, 0));
    svo
}
