use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
    },
    sprite::Material2d,
};

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct WaterMaterial {
    #[uniform(0)]
    color: Color,

    #[uniform(0)]
    pub time: f32,
}

impl Material2d for WaterMaterial {
    fn vertex_shader() -> ShaderRef {
        "water_shader.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "water_shader.wgsl".into()
    }
}

impl From<Color> for WaterMaterial {
    fn from(color: Color) -> Self {
        WaterMaterial { color, time: 0.0 }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WaterMeshBuilder {
    subdivisions: i32,
    water_start: f32,
    water_end: f32,
}

impl WaterMeshBuilder {
    pub fn new(subdivisions: i32, begin: f32, end: f32) -> Self {
        Self {
            subdivisions,
            water_start: begin,
            water_end: end,
        }
    }

    pub fn build(&self) -> Mesh {
        let mut vertices: Vec<Vec3> = Vec::with_capacity(2 * self.subdivisions as usize);
        for i in 0..self.subdivisions + 1 {
            let x = self.water_start
                + i as f32 * (self.water_end - self.water_start) / self.subdivisions as f32;
            vertices.push(Vec3::new(x, 100.0, 0.0));
            vertices.push(Vec3::new(x, -200.0, 0.0));
        }

        let mut indices: Vec<u16> = Vec::with_capacity(6 * (self.subdivisions - 1) as usize);
        for i in 0..self.subdivisions as u16 {
            indices.push(2 * i);
            indices.push(2 * i + 3);
            indices.push(2 * i + 2);

            indices.push(2 * i + 1);
            indices.push(2 * i + 3);
            indices.push(2 * i);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U16(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh
    }
}
