use glam::Vec3;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Axis2 {
    X,
    Y,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Axis3 {
    X,
    Y,
    Z
}

impl Axis3 {
    pub fn from_raw(u: u32) -> Axis3 {
        if u == 0 {
            Axis3::X
        } else if u == 1 {
            Axis3::Y
        } else {
            Axis3::Z
        }
    }

    pub fn as_vec3(self) -> Vec3 {
        match self {
            Axis3::X => Vec3::X,
            Axis3::Y => Vec3::Y,
            Axis3::Z => Vec3::Z,
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Face3 {
    pub axis: Axis3,
    pub p: bool
}

impl Face3 {
    pub fn as_vec3(self) -> Vec3 {
        if self.p {
            self.axis.as_vec3()
        } else {
            Vec3::ZERO
        }
    }
}