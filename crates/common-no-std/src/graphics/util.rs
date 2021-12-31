
pub fn barycentric(a: Vec2, b: Vec2, c: Vec2, p: Vec2) -> Vec3 {
    let v0 = Vec3::new(c[0] - a[0], b[0] - a[0], a[0] - p[0]);
    let v1 = Vec3::new(c[1] - a[1], b[1] - a[1], a[1] - p[1]);
    let u = v0.cross(v1);
    return Vec3::new(1.0 - (u[0] + u[1]) / u[2], u[1] / u[2], u[0] / u[2]);
}
