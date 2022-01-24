use super::vec::*;

pub trait Grid3<T> {
    fn get(pos: UVec3) -> T;
}

