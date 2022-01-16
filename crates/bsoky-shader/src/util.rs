use common::math::*;

pub fn frag_world_position_from_face(map_size: Vec3, face: u32, uv: Vec2) -> Vec3 {
    // TODO don't branch that much
    let mut frag_world_position = vec3(0.0, 0.0, 0.0);
    if face == 0 {
        // max z side
        frag_world_position = vec3(uv.x * map_size.x, uv.y * map_size.y, map_size.z);
    } else if face == 1 {
        // min z side
        frag_world_position = vec3((1.0 - uv.x) * map_size.x, (1.0 - uv.y) * map_size.y, 0.0);
    } else if face == 2 {
        // max x side
        frag_world_position = vec3(map_size.x, uv.x * map_size.y, uv.y * map_size.z);
    } else if face == 3 {
        // min x side
        frag_world_position = vec3(0.0, (1.0 - uv.x) * map_size.y, (1.0 - uv.y) * map_size.z);
    } else if face == 4 {
        // max y side
        frag_world_position = vec3(uv.x * map_size.x, map_size.y, uv.y * map_size.z);
    } else if face == 5 {
        // min y side
        frag_world_position = vec3((1.0 - uv.x) * map_size.x, 0.0, (1.0 - uv.y) * map_size.z);
    }
    return frag_world_position;
}
