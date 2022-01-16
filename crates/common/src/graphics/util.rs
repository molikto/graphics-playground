
pub fn barycentric(a: Vec2, b: Vec2, c: Vec2, p: Vec2) -> Vec3 {
    let v0 = Vec3::new(c.x - a.x, b.x - a.x, a.x - p.x);
    let v1 = Vec3::new(c.y - a.y, b.y - a.y, a.y - p.y);
    let u = v0.cross(v1);
    return Vec3::new(1.0 - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z);
}
