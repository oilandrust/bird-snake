use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline},
        render_resource::{
            BlendState, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace,
            MultisampleState, PipelineCache, PolygonMode, PrimitiveState, RenderPipelineDescriptor,
            SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget, VisibleEntities},
        Extract, RenderApp, RenderStage,
    },
    sprite::{
        ColorMaterial, DrawMesh2d, Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform,
        SetMesh2dBindGroup, SetMesh2dViewBindGroup,
    },
    utils::FloatOrd,
};

#[derive(Component, Default)]
pub struct WaterMesh2d;

#[derive(Resource)]
pub struct WaterMesh2dPipeline {
    mesh2d_pipeline: Mesh2dPipeline,
    water_shader: Handle<Shader>,
}

impl FromWorld for WaterMesh2dPipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
            water_shader: world.resource::<WaterShader>().0.clone(),
        }
    }
}

impl SpecializedRenderPipeline for WaterMesh2dPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let formats = vec![VertexFormat::Float32x3];

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        let format = match key.contains(Mesh2dPipelineKey::HDR) {
            true => ViewTarget::TEXTURE_FORMAT_HDR,
            false => TextureFormat::bevy_default(),
        };

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: self.water_shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: self.water_shader.clone(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: Some(vec![
                self.mesh2d_pipeline.view_layout.clone(),
                self.mesh2d_pipeline.mesh_layout.clone(),
            ]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("water_mesh2d_pipeline".into()),
        }
    }
}

type DrawColoredMesh2d = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMesh2dBindGroup<1>,
    DrawMesh2d,
);

pub struct WaterMesh2dPlugin;

#[derive(Resource)]
struct WaterShader(Handle<Shader>);

impl Plugin for WaterMesh2dPlugin {
    fn build(&self, app: &mut App) {
        let asset_server = app.world.resource_mut::<AssetServer>();
        let water_shader = asset_server.load("water_shader.wgsl");
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .add_render_command::<Transparent2d, DrawColoredMesh2d>()
            .insert_resource(WaterShader(water_shader))
            .init_resource::<WaterMesh2dPipeline>()
            .init_resource::<SpecializedRenderPipelines<WaterMesh2dPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_water_mesh2d)
            .add_system_to_stage(RenderStage::Queue, queue_water_mesh2d);
    }
}

pub fn extract_water_mesh2d(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Extract<Query<(Entity, &ComputedVisibility), With<WaterMesh2d>>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, computed_visibility) in &query {
        if !computed_visibility.is_visible() {
            continue;
        }
        values.push((entity, WaterMesh2d));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

#[allow(clippy::too_many_arguments)]
pub fn queue_water_mesh2d(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    water_mesh2d_pipeline: Res<WaterMesh2dPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<WaterMesh2dPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    water_mesh2d: Query<(&Mesh2dHandle, &Mesh2dUniform), With<WaterMesh2d>>,
    mut views: Query<(
        &VisibleEntities,
        &mut RenderPhase<Transparent2d>,
        &ExtractedView,
    )>,
) {
    if water_mesh2d.is_empty() {
        return;
    }

    for (visible_entities, mut transparent_phase, view) in &mut views {
        let draw_colored_mesh2d = transparent_draw_functions
            .read()
            .get_id::<DrawColoredMesh2d>()
            .unwrap();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples)
            | Mesh2dPipelineKey::from_hdr(view.hdr);

        for visible_entity in &visible_entities.entities {
            if let Ok((mesh2d_handle, mesh2d_uniform)) = water_mesh2d.get(*visible_entity) {
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                    mesh2d_key |=
                        Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);
                }

                let pipeline_id =
                    pipelines.specialize(&mut pipeline_cache, &water_mesh2d_pipeline, mesh2d_key);

                let mesh_z = mesh2d_uniform.transform.w_axis.z;
                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    draw_function: draw_colored_mesh2d,
                    pipeline: pipeline_id,
                    sort_key: FloatOrd(mesh_z),
                    batch_range: None,
                });
            }
        }
    }
}
