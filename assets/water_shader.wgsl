#import bevy_sprite::mesh2d_types
#import bevy_sprite::mesh2d_view_bindings

@group(2) @binding(0)
var<uniform> mesh: Mesh2d;

#import bevy_sprite::mesh2d_functions

struct WaterMaterial {
    color: vec4<f32>,
    time: f32,
}

@group(1) @binding(0)
var<uniform> material: WaterMaterial;

struct Vertex {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var y_offset = 10.0 * sin(0.03 * vertex.position.x + material.time);
    var position = vec3<f32>(vertex.position.x, vertex.position.y + y_offset, vertex.position.z);

    var out: VertexOutput;
    out.clip_position = mesh2d_position_local_to_clip(mesh.model, vec4<f32>(position, 1.0));
    out.color = material.color;
    return out;
}

struct FragmentInput {
    @location(0) color: vec4<f32>,
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    return in.color;
}