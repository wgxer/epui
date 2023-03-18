use bevy::{
    asset::{load_internal_asset, HandleId},
    ecs::system::lifetimeless::SRes,
    prelude::{
        error, Bundle, Color, Commands, Component, Entity, Handle, IntoSystemAppConfig,
        IntoSystemConfig, Plugin, Query, Res, ResMut, Resource, Shader, Vec2, Vec4, With,
    },
    render::{
        render_phase::{
            AddRenderCommand, BatchedPhaseItem, DrawFunctions, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline,
        },
        render_resource::{
            BlendState, BufferUsages, BufferVec, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, FragmentState, FrontFace, MultisampleState, PipelineCache, PolygonMode,
            PrimitiveState, PrimitiveTopology, RenderPipelineDescriptor, TextureFormat,
            VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        Extract, ExtractSchedule, RenderApp, RenderSet,
    },
};
use bytemuck_derive::{Zeroable, Pod};

use crate::{
    camera::{PhysicalViewportSize, UiPhaseItem},
    property::{ColoredElement, CornersRoundness, Position, Size},
    r#box,
};

pub struct UiBoxPlugin;

impl Plugin for UiBoxPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let shader_handle = HandleId::random::<Shader>();
        load_internal_asset!(app, shader_handle, "box.wgsl", Shader::from_wgsl);

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .insert_resource(r#box::BoxShader(Handle::weak(shader_handle)))
            .init_resource::<r#box::BoxPipeline>()
            .init_resource::<r#box::BoxBuffers>()
            .add_render_command::<UiPhaseItem, RenderBoxCommand>()
            .add_system(r#box::extract_boxes.in_schedule(ExtractSchedule))
            .add_system(r#box::queue_boxes.in_set(RenderSet::Queue));
    }
}

#[derive(Component, Default)]
pub struct UiBox;

#[derive(Bundle, Default)]
pub struct UiBoxBundle {
    pub ui_box: UiBox,
    pub position: Position,
    pub size: Size,
    pub color: ColoredElement,
}

#[derive(Resource, Default)]
struct BoxPipeline(Option<CachedRenderPipelineId>);

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct InstanceData {
    vertices: [[f32; 2]; 4], // Vertices (Vec2[4])
    color: [f32; 4],         // Color (RGBA)

    corner_center: [f32; 2], // Vec4: Corner Center (Vec2)
    corner_whd: f32,         // + Corner Width & Height Difference (f32)
    half_min_axis: f32,      // + Half Minimum Axis (f32)

    corners_roundness: [f32; 4],
}

impl InstanceData {
    fn new(
        vertices: [Vec2; 4],
        color: Color,
        corner_center: Vec2,
        corner_whd: f32,
        half_min_axis: f32,
        corners_roundness: Vec4,
    ) -> InstanceData {
        InstanceData {
            vertices: [
                vertices[0].into(),
                vertices[1].into(),
                vertices[2].into(),
                vertices[3].into(),
            ],
            color: color.into(),

            corner_center: corner_center.into(),
            corner_whd,
            half_min_axis,

            corners_roundness: corners_roundness.into(),
        }
    }
}

#[derive(Resource)]
struct BoxBuffers {
    instances: BufferVec<InstanceData>,
}

impl Default for BoxBuffers {
    fn default() -> Self {
        BoxBuffers {
            instances: BufferVec::new(BufferUsages::VERTEX),
        }
    }
}

#[derive(Resource)]
struct BoxShader(Handle<Shader>);

fn extract_boxes(
    mut commands: Commands,
    boxes: Extract<
        Query<
            (
                Entity,
                &Position,
                &Size,
                &ColoredElement,
                Option<&CornersRoundness>,
            ),
            With<UiBox>,
        >,
    >,
) {
    for (entity, position, size, colored_element, corners_roundness) in boxes.iter() {
        let mut entity_commands = commands.get_or_spawn(entity);

        entity_commands.insert((
            UiBox,
            position.clone(),
            size.clone(),
            colored_element.clone(),
        ));

        if let Some(corners_roundness) = corners_roundness {
            entity_commands.insert(corners_roundness.clone());
        }
    }
}

fn queue_boxes(
    mut box_pipeline: ResMut<BoxPipeline>,
    pipeline_cache: Res<PipelineCache>,

    mut view_query: Query<(&PhysicalViewportSize, &mut RenderPhase<UiPhaseItem>)>,
    mut box_buffers: ResMut<BoxBuffers>,
    boxes: Query<
        (
            Entity,
            &Position,
            &Size,
            &ColoredElement,
            Option<&CornersRoundness>,
        ),
        With<UiBox>,
    >,
    draw_functions: Res<DrawFunctions<UiPhaseItem>>,

    box_shader: Res<BoxShader>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let pipeline = box_pipeline.0.get_or_insert_with(|| {
        pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("box_pipeline_desc".into()),
            layout: vec![],
            vertex: VertexState {
                entry_point: "vs_main".into(),
                shader: box_shader.0.clone(),
                shader_defs: vec![],
                buffers: vec![VertexBufferLayout {
                    step_mode: VertexStepMode::Instance,
                    attributes: vec![
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: std::mem::size_of::<[f32; 2]>() as u64,
                            shader_location: 1,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: std::mem::size_of::<[[f32; 2]; 2]>() as u64,
                            shader_location: 2,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: std::mem::size_of::<[[f32; 2]; 3]>() as u64,
                            shader_location: 3,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: std::mem::size_of::<[[f32; 2]; 4]>() as u64,
                            shader_location: 4,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: std::mem::size_of::<[[f32; 2]; 6]>() as u64, // f32x4 = f32x2 * 2
                            shader_location: 5,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: std::mem::size_of::<[[f32; 2]; 8]>() as u64, // f32x4 = f32x2 * 2
                            shader_location: 6,
                        },
                    ],
                    array_stride: std::mem::size_of::<InstanceData>() as u64,
                }],
            },
            fragment: Some(FragmentState {
                entry_point: "fs_main".into(),
                shader: box_shader.0.clone(),
                shader_defs: vec![],
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            push_constant_ranges: vec![],
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                cull_mode: None,
                front_face: FrontFace::Ccw,
                strip_index_format: None,
                unclipped_depth: false,
            },
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            depth_stencil: None,
        })
    });

    let draw_function_id = draw_functions.read().id::<RenderBoxCommand>();

    for (viewport_size, mut ui_phase) in view_query.iter_mut() {
        let Some(viewport_size) = viewport_size.0 else { continue; };

        let x_pixel_unit = 2.0 / viewport_size.x as f32;
        let y_pixel_unit = 2.0 / viewport_size.y as f32;

        let mut instance = 0;
        box_buffers.instances.clear();

        for (box_entity, position, size, colored_element, corners_roundness) in boxes.iter() {
            let left_top_corner = Vec2::new(
                (x_pixel_unit * position.x as f32) - 1.0,
                1.0 - (y_pixel_unit * position.y as f32),
            );

            let right_top_corner = Vec2::new(
                left_top_corner.x + (x_pixel_unit * size.width as f32),
                left_top_corner.y,
            );

            let left_bottom_corner = Vec2::new(
                left_top_corner.x,
                left_top_corner.y - (y_pixel_unit * size.height as f32),
            );

            let right_bottom_corner = Vec2::new(right_top_corner.x, left_bottom_corner.y);

            let corner_center = Vec2::new(
                (size.width as f32 / 2.0) + position.x as f32,
                (size.height as f32 / 2.0) + position.y as f32,
            );

            let corners_roundness = Vec4::from(corners_roundness.cloned().unwrap_or_default());
            let min_half_unit = u32::min(size.width, size.height) as f32 / 2.0;
            let corner_whd = size.width as f32 - size.height as f32; // Positive = Width > Height, Negative = Width < Height

            box_buffers.instances.push(InstanceData::new(
                [
                    left_top_corner,
                    right_top_corner,
                    left_bottom_corner,
                    right_bottom_corner,
                ],
                colored_element.color,
                corner_center,
                corner_whd,
                min_half_unit,
                (1.0 - corners_roundness) * min_half_unit,
            ));

            ui_phase.add(UiPhaseItem {
                entity: box_entity,
                z_index: 0,

                draw_function: draw_function_id,
                cached_render_pipeline_id: *pipeline,

                batch_range: Some(instance..instance + 1),
            });

            instance += 1;
        }
    }

    box_buffers
        .instances
        .write_buffer(&render_device, &render_queue);
}

type RenderBoxCommand = (SetItemPipeline, DrawBox);

struct DrawBox;
impl<P: BatchedPhaseItem> RenderCommand<P> for DrawBox {
    type Param = SRes<BoxBuffers>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = ();

    fn render<'w>(
        item: &P,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        _entity: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        box_buffers: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        {
            let vertex_buffer = match box_buffers.into_inner().instances.buffer() {
                Some(buffer) => buffer,
                None => {
                    error!("Couldn't set vertex buffer because it's not present");
                    return RenderCommandResult::Failure;
                }
            };

            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        }

        {
            let instances_range = item
                .batch_range()
                .clone()
                .expect("Batch range isn't present");

            pass.draw(0..6, instances_range);
        }

        RenderCommandResult::Success
    }
}
