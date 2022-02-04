use bevy::{
    math::Vec3,
    pbr::{PbrBundle, StandardMaterial},
    prelude::{
        AssetServer, Assets, Camera, Color, Commands, GlobalTransform, Res, ResMut, Transform, App, Handle, Shader,
    },
    render::{
        mesh::{shape, Indices, Mesh},
        render_resource::PrimitiveTopology,
    },
    utils::Instant, asset::AssetPath,
};
use common::{vec4, Mat4, Ray3, UVec2, shader::base_uniform::RayTracingViewInfo};

fn create_single_debug_cube(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    pos: Vec3,
    color: Color,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
        material: materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..Default::default()
        }),
        transform: Transform::from_translation(pos),
        ..Default::default()
    });
}

pub fn create_debug_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    create_single_debug_cube(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::ZERO,
        Color::WHITE,
    );
    create_single_debug_cube(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::X * 10.0,
        Color::RED,
    );
    create_single_debug_cube(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::Y * 10.0,
        Color::GREEN,
    );
    create_single_debug_cube(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::Z * 10.0,
        Color::BLUE,
    );
}

#[derive(Debug, Copy, Clone)]
pub struct RevertBox {
    pub min_x: f32,
    pub max_x: f32,

    pub min_y: f32,
    pub max_y: f32,

    pub min_z: f32,
    pub max_z: f32,
}

impl RevertBox {
    pub fn zero_with_size(size: Vec3) -> RevertBox {
        return RevertBox {
            min_x: 0.0,
            max_x: size.x,
            min_y: 0.0,
            max_y: size.y,
            min_z: 0.0,
            max_z: size.z,
        };
    }
    pub fn new(x_length: f32, y_length: f32, z_length: f32) -> RevertBox {
        RevertBox {
            max_x: x_length / 2.0,
            min_x: -x_length / 2.0,
            max_y: y_length / 2.0,
            min_y: -y_length / 2.0,
            max_z: z_length / 2.0,
            min_z: -z_length / 2.0,
        }
    }
}

impl Default for RevertBox {
    fn default() -> Self {
        RevertBox::new(1.0, 1.0, 1.0)
    }
}

impl From<RevertBox> for Mesh {
    fn from(sp: RevertBox) -> Self {
        let vertices = &mut [
            // Top
            ([sp.min_x, sp.min_y, sp.max_z], [0., 0., 1.0], [0., 0.]),
            ([sp.max_x, sp.min_y, sp.max_z], [0., 0., 1.0], [1.0, 0.]),
            ([sp.max_x, sp.max_y, sp.max_z], [0., 0., 1.0], [1.0, 1.0]),
            ([sp.min_x, sp.max_y, sp.max_z], [0., 0., 1.0], [0., 1.0]),
            // Bottom
            ([sp.min_x, sp.max_y, sp.min_z], [0., 0., -1.0], [1.0, 0.]),
            ([sp.max_x, sp.max_y, sp.min_z], [0., 0., -1.0], [0., 0.]),
            ([sp.max_x, sp.min_y, sp.min_z], [0., 0., -1.0], [0., 1.0]),
            ([sp.min_x, sp.min_y, sp.min_z], [0., 0., -1.0], [1.0, 1.0]),
            // Right
            ([sp.max_x, sp.min_y, sp.min_z], [1.0, 0., 0.], [0., 0.]),
            ([sp.max_x, sp.max_y, sp.min_z], [1.0, 0., 0.], [1.0, 0.]),
            ([sp.max_x, sp.max_y, sp.max_z], [1.0, 0., 0.], [1.0, 1.0]),
            ([sp.max_x, sp.min_y, sp.max_z], [1.0, 0., 0.], [0., 1.0]),
            // Left
            ([sp.min_x, sp.min_y, sp.max_z], [-1.0, 0., 0.], [1.0, 0.]),
            ([sp.min_x, sp.max_y, sp.max_z], [-1.0, 0., 0.], [0., 0.]),
            ([sp.min_x, sp.max_y, sp.min_z], [-1.0, 0., 0.], [0., 1.0]),
            ([sp.min_x, sp.min_y, sp.min_z], [-1.0, 0., 0.], [1.0, 1.0]),
            // Front
            ([sp.max_x, sp.max_y, sp.min_z], [0., 1.0, 0.], [1.0, 0.]),
            ([sp.min_x, sp.max_y, sp.min_z], [0., 1.0, 0.], [0., 0.]),
            ([sp.min_x, sp.max_y, sp.max_z], [0., 1.0, 0.], [0., 1.0]),
            ([sp.max_x, sp.max_y, sp.max_z], [0., 1.0, 0.], [1.0, 1.0]),
            // Back
            ([sp.max_x, sp.min_y, sp.max_z], [0., -1.0, 0.], [0., 0.]),
            ([sp.min_x, sp.min_y, sp.max_z], [0., -1.0, 0.], [1.0, 0.]),
            ([sp.min_x, sp.min_y, sp.min_z], [0., -1.0, 0.], [1.0, 1.0]),
            ([sp.max_x, sp.min_y, sp.min_z], [0., -1.0, 0.], [0., 1.0]),
        ];

        let mut positions = Vec::with_capacity(24);
        let mut normals = Vec::with_capacity(24);
        let mut uvs = Vec::with_capacity(24);

        for (position, normal, uv) in vertices.iter() {
            positions.push(*position);
            normals.push(*normal);
            uvs.push(*uv);
        }
        // compared to the original, the indexes are reverted by the second/third
        let indices = vec![
            // flip sided
            0, 2, 1, 2, 0, 3, // top
            4, 6, 5, 6, 4, 7, // bottom
            8, 10, 9, 10, 8, 11, // right
            12, 14, 13, 14, 12, 15, // left
            16, 18, 17, 18, 16, 19, // front
            20, 22, 21, 22, 20, 23, // back
            // original
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];
        let indices = Indices::U32(indices);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(indices));
        mesh
    }
}

pub fn center_ray((camera, transform): (&Camera, &GlobalTransform)) -> Ray3 {
    let matrix = transform.compute_matrix() * camera.projection_matrix.inverse();
    let near = matrix.project_point3(Vec3::new(0.0, 0.0, -1.0));
    let far = matrix.project_point3(Vec3::new(0.0, 0.0, 1.0));
    let dir = far - near;
    let ray = Ray3 {
        pos: near,
        dir: dir.normalize(),
    };
    ray
}

#[derive(Clone, Copy, PartialEq, Default)]
pub struct CameraProp {
    pub transform: GlobalTransform,
    pub view_projection: Mat4,
}
impl CameraProp {
    pub fn get_ray_tracing_uniform(&self, size: UVec2, time: f32, frame_index: u32) -> RayTracingViewInfo {
        let transform = self.transform.compute_matrix();
        let inverse_projection = self.view_projection.inverse();
        let mut top_left = inverse_projection * vec4(-1.0, 1.0, -1.0, 1.0);
        let bottom_right = inverse_projection * vec4(1.0, -1.0, -1.0, 1.0);
        let mut camera_h = (bottom_right - top_left) / (size.x as f32);
        camera_h.y = 0.0;
        let mut camera_v = (bottom_right - top_left) / (size.y as f32);
        camera_v.x = 0.0;
        top_left.w = 0.0;
        RayTracingViewInfo {
            camera_pos: self.transform.translation,
            camera_look: (transform * top_left).truncate(),
            camera_h: (transform * camera_h).truncate(),
            camera_v: (transform * camera_v).truncate(),
            time,
            frame_index,
            not_used: UVec2::ZERO,
        }
    }
}

pub fn load_shader<'a, P: Into<AssetPath<'a>>>(app: &mut App, p: P) -> Handle<Shader> {
    let world = &mut app.world;
    let asset_server = world.get_resource::<AssetServer>().unwrap();
    asset_server.load(p)
}
