#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

use common::{math::*, shader::base_uniform::*, ifelse};
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;
use spirv_std::*;

pub trait Sdf {
    fn distance(self, p: Vec3) -> f32;
}

pub struct Sphere(pub f32);
impl Sdf for Sphere {
    fn distance(self, p: Vec3) -> f32 {
        p.length() - self.0
    }
}

pub struct Box(pub Vec3);
impl Sdf for Box {
    fn distance(self, p: Vec3) -> f32 {
        let q = p.abs() - self.0;
        q.max(Vec3::ZERO).length() + q.max_element().min(0.0)
    }
}

pub struct RoundBox {
    b: Vec3,
    r: f32,
}
impl Sdf for RoundBox {
    fn distance(self, p: Vec3) -> f32 {
        let q = p.abs() - self.b;
        q.max(Vec3::ZERO).length() + q.max_element().min(0.0) - self.r
    }
}

pub struct BoxFrame {
    pub b: Vec3,
    pub e: f32,
}

impl Sdf for BoxFrame {
    fn distance(self, p: Vec3) -> f32 {
        let p = p.abs() - self.b;
        let q = (p + self.e).abs() - self.e;
        fn helper(v: Vec3) -> f32 {
            v.max(Vec3::ZERO).length() + v.max_element().min(0.0)
        }
        helper(vec3(p.x, q.y, q.z))
            .min(helper(vec3(q.x, p.y, q.z)))
            .min(helper(vec3(q.x, q.y, p.z)))
    }
}

pub struct Torus(pub Vec2);

impl Sdf for Torus {
    fn distance(self, p: Vec3) -> f32 {
        let q = vec2(p.xz().length() - self.0.x, p.y);
        q.length() - self.0.y
    }
}

pub struct CappedTorus {
    sc: Vec2,
    ra: f32,
    rb: f32,
}
impl Sdf for CappedTorus {
    fn distance(self, mut p: Vec3) -> f32 {
        let CappedTorus { sc, ra, rb } = self;
        p.x = p.x.abs();
        let k = ifelse!(sc.y * p.x > sc.x * p.y, p.xy().dot(sc), p.xy().length());
        return (p.dot(p) + ra * ra - 2.0 * ra * k).sqrt() - rb;
    }
}

pub struct HexPrism {
    hx: f32,
    hy: f32,
}
impl Sdf for HexPrism {
    fn distance(self, mut p: Vec3) -> f32 {
        let HexPrism { hx, hy } = self;
        const K: Vec3 = const_vec3!([-0.8660254, 0.5, 0.57735]);
        p = p.abs();
        let mut pxy = p.xy();
        pxy -= 2.0 * K.xy().dot(pxy).min(0.0) * K.xy();
        let px = pxy.x;
        let py = pxy.y;
        let pz = p.z;
        let d = vec2(
            (pxy - vec2(px.clamp(-K.z * hx, K.z * hx), hx)).length() * (py - hx).signum(),
            pz - hy,
        );
        return d.x.max(d.y).min(0.0) + d.max(Vec2::ZERO).length();
    }
}

pub struct OctogonPrism {
    hx: f32,
    hy: f32
}
impl Sdf for OctogonPrism {
    fn distance(self, p: Vec3) -> f32 {
        todo!()
        // TODO
//         let k = vec3(-0.9238795325,   // sqrt(2+sqrt(2))/2 
//         0.3826834323,   // sqrt(2-sqrt(2))/2
//         0.4142135623 ); // sqrt(2)-1 
//         // reflections
//         let p = p.abs();
// p.xy -= 2.0*min(dot(vec2( k.x,k.y),p.xy),0.0)*vec2( k.x,k.y);
// p.xy -= 2.0*min(dot(vec2(-k.x,k.y),p.xy),0.0)*vec2(-k.x,k.y);
// // polygon side
// p.xy -= vec2(clamp(p.x, -k.z*r, k.z*r), r);
// vec2 d = vec2( length(p.xy)*sign(p.y), p.z-h );
// return min(max(d.x,d.y),0.0) + length(max(d,0.0));
    }
}

fn op_u(d1: Vec2, d2: Vec2) -> Vec2 {
    if d1.x < d2.x {
        d1
    } else {
        d2
    }
}
fn map(pos: Vec3) -> Vec2 {
    let mut res = vec2(1e10, 0.0);

    res = op_u(
        res,
        vec2(Sphere(0.25).distance(pos - vec3(-2.0, 0.25, 0.0)), 26.9),
    );

    // bounding box
    if Box(vec3(0.35, 0.3, 2.5)).distance(pos - vec3(0.0, 0.3, -1.0)) < res.x {
        // more primitives
        res = op_u(
            res,
            vec2(
                BoxFrame {
                    b: vec3(0.3, 0.25, 0.2),
                    e: 0.025,
                }
                .distance(pos - vec3(0.0, 0.25, 0.0)),
                16.9,
            ),
        );
        res = op_u( res, vec2( Torus(vec2(0.25,0.05)).distance((pos-vec3( 0.0,0.30, 1.0)).xzy()), 25.0 ) );
        // res = op_u( res, vec2( sdCone(        pos-vec3( 0.0,0.45,-1.0), vec2(0.6,0.8),0.45 ), 55.0 ) );
        // res = op_u( res, vec2( sdCappedCone(  pos-vec3( 0.0,0.25,-2.0), 0.25, 0.25, 0.1 ), 13.67 ) );
        // res = op_u( res, vec2( sdSolidAngle(  pos-vec3( 0.0,0.00,-3.0), vec2(3,4)/5.0, 0.4 ), 49.13 ) );
    }

    // // bounding box
    // if( sdBox( pos-vec3(1.0,0.3,-1.0),vec3(0.35,0.3,2.5) )<res.x )
    // {
    // // more primitives
    // res = op_u( res, vec2( sdCappedTorus((pos-vec3( 1.0,0.30, 1.0))*vec3(1,-1,1), vec2(0.866025,-0.5), 0.25, 0.05), 8.5) );
    // res = op_u( res, vec2( sdBox(         pos-vec3( 1.0,0.25, 0.0), vec3(0.3,0.25,0.1) ), 3.0 ) );
    // res = op_u( res, vec2( sdCapsule(     pos-vec3( 1.0,0.00,-1.0),vec3(-0.1,0.1,-0.1), vec3(0.2,0.4,0.2), 0.1  ), 31.9 ) );
    // res = op_u( res, vec2( sdCylinder(    pos-vec3( 1.0,0.25,-2.0), vec2(0.15,0.25) ), 8.0 ) );
    // res = op_u( res, vec2( sdHexPrism(    pos-vec3( 1.0,0.2,-3.0), vec2(0.2,0.05) ), 18.4 ) );
    // }

    // // bounding box
    // if( sdBox( pos-vec3(-1.0,0.35,-1.0),vec3(0.35,0.35,2.5))<res.x )
    // {
    // // more primitives
    // res = op_u( res, vec2( sdPyramid(    pos-vec3(-1.0,-0.6,-3.0), 1.0 ), 13.56 ) );
    // res = op_u( res, vec2( sdOctahedron( pos-vec3(-1.0,0.15,-2.0), 0.35 ), 23.56 ) );
    // res = op_u( res, vec2( sdTriPrism(   pos-vec3(-1.0,0.15,-1.0), vec2(0.3,0.05) ),43.5 ) );
    // res = op_u( res, vec2( sdEllipsoid(  pos-vec3(-1.0,0.25, 0.0), vec3(0.2, 0.25, 0.05) ), 43.17 ) );
    // res = op_u( res, vec2( sdRhombus(   (pos-vec3(-1.0,0.34, 1.0)).xzy, 0.15, 0.25, 0.04, 0.08 ),17.0 ) );
    // }

    // // bounding box
    // if( sdBox( pos-vec3(2.0,0.3,-1.0),vec3(0.35,0.3,2.5) )<res.x )
    // {
    // // more primitives
    // res = op_u( res, vec2( sdOctogonPrism(pos-vec3( 2.0,0.2,-3.0), 0.2, 0.05), 51.8 ) );
    // res = op_u( res, vec2( sdCylinder(    pos-vec3( 2.0,0.15,-2.0), vec3(0.1,-0.1,0.0), vec3(-0.2,0.35,0.1), 0.08), 31.2 ) );
    // res = op_u( res, vec2( sdCappedCone(  pos-vec3( 2.0,0.10,-1.0), vec3(0.1,0.0,0.0), vec3(-0.2,0.40,0.1), 0.15, 0.05), 46.1 ) );
    // res = op_u( res, vec2( sdRoundCone(   pos-vec3( 2.0,0.15, 0.0), vec3(0.1,0.0,0.0), vec3(-0.1,0.35,0.1), 0.15, 0.05), 51.7 ) );
    // res = op_u( res, vec2( sdRoundCone(   pos-vec3( 2.0,0.20, 1.0), 0.2, 0.1, 0.3 ), 37.0 ) );
    // }

    return res;
}

fn iBox(ro: Vec3, rd: Vec3, rad : Vec3) -> Vec2  {
    let m = 1.0/rd;
    let n = m*ro;
    let k = m.abs()*rad;
    let t1 = -n - k;
    let t2 = -n + k;
	return vec2( t1.x.max(t1.y ).max(t1.z ),
	             t2.x.min(t2.y ).min(t2.z ) );
}

fn ray_cast(ray: Ray3) -> Vec2 {
    let mut res = vec2(-1.0, -1.0);
    let mut tmin = 1.0;
    let mut tmax: f32 = 20.0;
    let tp1 = (-ray.pos.y) / ray.dir.y;
    if tp1 > 0.0 {
        tmax = tmax.min(tp1);
        res = vec2(tp1, 1.0);
    }
    let tb = iBox(ray.pos - vec3(0.0, 0.4,-0.5), ray.dir, vec3(2.5, 0.41, 3.0));
    if tb.x < tb.y && tb.y > 0.0 && tb.x < tmax {
        tmin = tb.x.max(tmin);
        tmax = tb.y.min(tmax);
        let mut t = tmin;
        let mut i = 0;
        while i < 70 && t < tmax {
            let h = map(ray.at(t));
            if h.x.abs() < (0.0001 * t) {
                res = vec2(t, h.y);
                break;
            }
            t += h.x;
            i+=1;
        }
    }
    res
}

fn cal_normal(pos: Vec3) -> Vec3 {
    let mut n = Vec3::ZERO;
    for i in 0..4 {
        let e = 0.5773*(2.0*vec3((((i+3)>>1)&1) as f32,((i>>1)&1) as f32,(i&1) as f32)-1.0);
        n += e * map(pos + 0.0005 * e).x;
    }
    n.normalize()
}

fn cal_soft_shadow(ray: Ray3, tmin: f32, mut tmax: f32) -> f32 {
    // bounding volume
    let tp = (0.8-ray.pos.y)/ray.dir.y;
    if tp>0.0 {
        tmax = tmax.min(tp );
    }

    let mut res: f32 = 1.0;
    let mut t = tmin;
    for i in 0 .. 24{
		let h = map( ray.pos + ray.dir *t ).x;
        let s = (8.0*h/t).clamp(0.0,1.0);
        res = res.min(s*s*(3.0-2.0*s) );
        t += h.clamp( 0.02, 0.2 );
        if res<0.004 || t>tmax {
            break;
        }
    }
    return res.clamp(0.0, 1.0 );
}

fn cal_ao(pos: Vec3, nor: Vec3) -> f32 {
    let mut occ = 0.0;
    let mut sca = 1.0;
    for i in 0..5 {
        let h = 0.01 + 0.12 * (i as f32) / 4.0;
        let d = map(pos + h * nor).x;
        occ += (h - d) * sca;
        sca *= 0.95;
        if occ > 0.35 {
            break;
        }
    }
    (1.0 - 3.0*occ).clamp(0.0, 1.0) * (0.5 + 0.5 * nor.y)
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t =((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn render(ray: Ray3) -> Vec3 {
    let mut col = vec3(0.7, 0.7, 0.9) - ray.dir.y.max(0.0) * 0.3;
    let res = ray_cast(ray);
    let t = res.x;
    let m = res.y;
    if m > -0.5 {
        let pos = ray.at(t);
        let nor = ifelse!(m < 1.5, vec3(0.0, 1.0, 0.0), cal_normal(pos));
        let ref_ = ray.dir.reflect(nor);
        col = 0.2 +  0.2 * (m * 2.0 + vec3(0.0, 1.0, 2.0)).sin();
        let mut ks: f32 = 1.0;
        if m < 1.5 {
            col = 0.15 + Vec3::splat(0.05);
            ks = 0.4;
        }
        let occ = cal_ao(pos, nor);
        let mut lin = Vec3::ZERO;
        {
            let lig = vec3(-0.5, 0.4, -0.6).normalize();
            let hal = (lig -ray.dir).normalize();
            let mut dif = lig.dot(nor).clamp(0.0, 1.0);
            dif *= cal_soft_shadow(Ray3 { pos: pos, dir: lig }, 0.02, 2.5);
            let mut spe = nor.dot(hal).clamp(0.0, 1.0).powf(16.0);
            spe *= dif;
            spe *= 0.04 + 0.96 * (1.0 - hal.dot(lig)).clamp(0.0, 1.0).powf(0.5);
            lin += col * 2.2*dif*vec3(1.3, 1.0, 0.7);
            lin += 5.0 * spe * vec3(1.3, 1.0, 0.7) * ks;
        }
        {
            let mut dif = (0.5+0.5*nor.y).clamp( 0.0, 1.0 ).sqrt();
            dif *= occ;
            let mut spe = smoothstep( -0.2, 0.2, ref_.y );
            spe *= dif;
            spe *= 0.04+0.96*(1.0+nor.dot(ray.dir)).clamp(0.0,1.0).powf(5.0);
            spe *= cal_soft_shadow( Ray3 { pos, dir: ref_ }, 0.02, 2.5 );
            lin += col*0.60*dif*vec3(0.40,0.60,1.15);
            lin +=     2.00*spe*vec3(0.40,0.60,1.30)*ks;
        }
        {
        	let mut dif = nor.dot(vec3(0.5,0.0,0.6).normalize()).clamp(  0.0, 1.0 )* (1.0-pos.y).clamp(0.0,1.0);
            dif *= occ;
        	lin += col*0.55*dif*vec3(0.25,0.25,0.25);
        }
        {
            let mut dif = (1.0 + nor.dot(ray.dir)).clamp(0.0, 1.0).powf(2.0);
            dif *= occ;
            lin += col * 0.25*dif*Vec3::ONE;
        }
        col = lin;
        col = col.lerp(vec3(0.7, 0.7, 0.9), 1.0 - (-0.0001*t*t*t).exp());
    }
    //col
    col.clamp(Vec3::ZERO, Vec3::ONE)
}

// A simple vert/frag shader to copy an image to the swapchain.
// from https://github.com/googlefonts/compute-shader-101
#[spirv(vertex)]
pub fn vert(
    #[spirv(vertex_index)] in_vertex_index: u32,
    #[spirv(instance_index)] in_instance_index: u32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
    out_tex_coord: &mut Vec2,
) {
    let x = ((in_vertex_index & 1) ^ in_instance_index) as f32;
    let y = ((in_vertex_index >> 1) ^ in_instance_index) as f32;
    *out_pos = vec4(x * 2. - 1., 1. - y * 2., 0., 1.);
    *out_tex_coord = vec2(x, y);
}

#[spirv(fragment)]
pub fn frag(
    #[spirv(uniform, descriptor_set = 0, binding = 1)] view_target_size: &UVec2,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] view: &RayTracingViewInfo,
    tex_coord: Vec2,
    output: &mut Vec4,
) {
    let frag_coord = tex_coord * view_target_size.as_vec2();
    let mut rng = SRng::new(common::math::zzz_deprecated_svt::float_bits_to_uint(
        frag_coord.x * (view_target_size.y as f32) + frag_coord.y + view.time
    ));
    const AA: u32 = 2;
    let mut total_color = Vec3::ZERO;
    for i in 0 .. AA {
        for j in 0..AA {
            let ray = view.get_ray(frag_coord + (uvec2(i, j).as_vec2() + 0.5) / (AA as f32));
            total_color += render(ray);
        }
    }
    let frag_color = (total_color / ((AA * AA) as f32)).extend(1.0);
    *output = frag_color;
}