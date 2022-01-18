
use std::{
    path::{Path},
};

use bsoky_shader::{MySvtMut};
use common::math::svt::*;
use common::math::*;

use sdfu::{SDF};


pub fn debug_create_rsvo() -> MySvtMut {
    // download yourself here https://github.com/ephtracy/voxel-model/blob/master/svo/
    let rsvo = std::fs::read( Path::new(env!("CARGO_MANIFEST_DIR")).join("sibenik_8k.rsvo")).unwrap();
    MySvtMut::load_from_rsvo(&rsvo, 10)
}

pub fn debug_create_simple() -> MySvtMut {
    let mut svt = MySvtMut::new(0);
    svt.set(Usvt3::new(3, 3, 3), 1);
    //println!("{:?}", svt.debug_items());
    //println!("{:?}", mem[0..10].to_vec());
    svt
}
pub fn debug_create_sdf() -> MySvtMut {
    // 4,4 = 0.21
    // 2,8 = 0.11
    let mut svt = MySvtMut::new(0);
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
    svt.sample(&|v| {
        if sdf.dist(v.as_vec3a() / total_size) < 0.0 {
            1
        } else {
            0
        }
    });
    svt
}
