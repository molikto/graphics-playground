
use std::{
    path::{Path},
};

use bsoky_no_std::{BLOCK_DIM, LEVEL_COUNT, MySvtMut};
use common::math::svt::*;
use common::math::*;

use rand::{thread_rng, Rng};
use sdfu::{SDF};


pub fn debug_create_rsvo() -> MySvtMut {
    // download yourself here https://github.com/ephtracy/voxel-model/blob/master/svo/
    let rsvo = std::fs::read( Path::new(env!("CARGO_MANIFEST_DIR")).join("sibenik_8k.rsvo")).unwrap();
    load_svt_from_rsvo(&rsvo, 6)
}

pub fn debug_create_simple() -> MySvtMut {
    let mut svt = MySvtMut::init(0);
    svt.set(Usvt3::new(3, 3, 3), 1);
    //println!("{:?}", svt.debug_items());
    //println!("{:?}", mem[0..10].to_vec());
    svt
}
pub fn debug_create_sdf() -> MySvtMut {
    // 4,4 = 0.21
    // 2,8 = 0.11
    let mut svt = MySvtMut::init(0);
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
        .subtract(sdfu::Box::new(Vec3A::new(1.5, 0.1, 0.1)).translate(Vec3A::new(0.0, 0.3, 0.0)))
        .subtract(sdfu::Box::new(Vec3A::new(0.2, 2.0, 0.2)))
        .scale(0.5)
        .translate(Vec3A::new(0.5, 0.5, 0.5));
    let total_size = MySvtMut::TOTAL_DIM as f32;
    for level in 0..LEVEL_COUNT as usvt {
        let level_cap = level + 1;
        let level_dim = BLOCK_DIM.pow(level_cap as u32);
        let level_size = BLOCK_DIM.pow(LEVEL_COUNT as u32 - (level_cap as u32));
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
                        thread_rng().gen_range(1..4)
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
    svt
}
