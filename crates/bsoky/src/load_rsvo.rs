use bevy::math::UVec3;
use common::math::svo::Svo;

pub fn load_rsvo_rec<const LEVEL_COUNT: usize>(
    rsvo: &[u8],
    mem: &mut Svo<2, LEVEL_COUNT>,
    indexes: &mut [usize; LEVEL_COUNT],
    level: usize,
    position: UVec3,
)  {
    if level == LEVEL_COUNT {
        mem.set_with_level_cap(level as u32, position, 1);
        return;
    }
    let children_mask = rsvo[indexes[level]];
    indexes[level] += 1;
    if children_mask == 0 {
        mem.set_with_level_cap(level as u32, position, 1);
    } else {
        for i in 0..8 {
            let bit = (children_mask >> i) & 1;
            let op = UVec3::new(i / 4, (i % 4) / 2, i % 2);
            if bit != 0 {
                load_rsvo_rec(rsvo, mem, indexes, level + 1, position + op * 2u32.pow((LEVEL_COUNT - level - 1) as u32));
            }
        }
    }
}

pub fn load_rsvo<const LEVEL_COUNT: usize>(rsvo: &[u8], svo: &mut Svo<2, LEVEL_COUNT>) {
    let rsvo32 = unsafe { std::mem::transmute::<&[u8], &[u32]>(rsvo) };
    let level_count = rsvo32[4] as usize;
    if level_count != LEVEL_COUNT {
        panic!("wroung size {}", level_count);
    }
    let mut indexes: [usize; LEVEL_COUNT] = [0; LEVEL_COUNT];
    indexes[0] = (level_count + 1 + 5) * 4;
    for i in 1..level_count {
        indexes[i] = indexes[i - 1] + rsvo32[i + 4] as usize;
    }
    load_rsvo_rec(rsvo, svo, &mut indexes, 0, UVec3::new(0, 0, 0));
}
