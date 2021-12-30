pub use glam::*;

pub trait VecExt {
    fn step(self, other: Self) -> Self;
    fn step_f(self, other: f32) -> Self;
    fn sum(self) -> f32;
    fn new_axis(u: usize) -> Self;
}

impl VecExt for Vec3 {

    fn step_f(self, other: f32) -> Self {
        // TODO intrinsics
        vec3(
            if self.x > other { 0.0 } else { 1.0 },
            if self.y > other { 0.0 } else { 1.0 },
            if self.z > other { 0.0 } else { 1.0 },
        )
    }

    fn step(self, other: Self) -> Self {
        // TODO intrinsics
        vec3(
            if self.x > other.x { 0.0 } else { 1.0 },
            if self.y > other.y { 0.0 } else { 1.0 },
            if self.z > other.z { 0.0 } else { 1.0 },
        )
    }

    fn sum(self) -> f32 {
        self.dot(Self::splat(1.0))
    }

    fn new_axis(a: usize) -> Vec3 {
        if a == 0 {
            Vec3::new(1.0, 0.0, 0.0)
        } else if a == 1 {
            Vec3::new(0.0, 1.0, 0.0)
        } else if a == 2 {
            Vec3::new(0.0, 0.0, 1.0)
        } else {
            panic!()
        }
    }
}

impl VecExt for Vec2 {
    fn step(self, other: Self) -> Self {
        vec2(
            if self.x > other.x { 0.0 } else { 1.0 },
            if self.y > other.y { 0.0 } else { 1.0 },
        )
    }

    fn step_f(self, other: f32) -> Self {
        // TODO intrinsics
        vec2(
            if self.x > other { 0.0 } else { 1.0 },
            if self.y > other { 0.0 } else { 1.0 },
        )
    }

    fn sum(self) -> f32 {
        self.dot(Self::splat(1.0))
    }

    fn new_axis(a: usize) -> Vec2 {
        if a == 0 {
            Vec2::new(1.0, 0.0)
        } else if a == 1 {
            Vec2::new(0.0, 1.0)
        } else {
            panic!()
        }
    }
}

pub trait Vec2Ext {
    fn extend_axis(self, a: usize, s: f32) -> Vec3;
}

impl Vec2Ext for Vec2 {
    #[inline]
    fn extend_axis(self, a: usize, s: f32) -> Vec3 {
        if a == 0 {
            Vec3::new(s, self[0], self[1])
        } else if a == 1 {
            Vec3::new(self[0], s, self[1])
        } else if a == 2 {
            Vec3::new(self[0], self[1], s)
        } else {
            panic!()
        }
    }
}

pub trait Vec3Ext {
    fn omit_axis(self, a: usize) -> Vec2;
}

impl Vec3Ext for Vec3 {
    #[inline]
    fn omit_axis(self, a: usize) -> Vec2 {
        if a == 0 {
            self.yz()
        } else if a == 1 {
            self.xz()
        } else if a == 2 {
            self.xy()
        } else {
            panic!()
        }
    }
}

pub fn barycentric(a: Vec2, b: Vec2, c: Vec2, p: Vec2) -> Vec3 {
    let v0 = Vec3::new(c[0] - a[0], b[0] - a[0], a[0] - p[0]);
    let v1 = Vec3::new(c[1] - a[1], b[1] - a[1], a[1] - p[1]);
    let u = v0.cross(v1);
    return Vec3::new(1.0 - (u[0] + u[1]) / u[2], u[1] / u[2], u[0] / u[2]);
}
