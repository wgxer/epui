use std::ops::Range;

use bevy::{
    core_pipeline::upscaling::UpscalingNode,
    prelude::{
        Bundle, Camera, Color, Commands, Component, Entity, EventReader, EventWriter,
        GlobalTransform, IntoSystemAppConfig, IntoSystemConfig, OrthographicProjection, Plugin,
        Query, QueryState, UVec2, With, World,
    },
    render::{
        camera::{CameraRenderGraph, ExtractedCamera},
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::{Node, NodeRunError, RenderGraph, SlotInfo, SlotType},
        render_phase::{
            sort_phase_system, BatchedPhaseItem, CachedRenderPipelinePhaseItem, DrawFunctionId,
            DrawFunctions, PhaseItem, RenderPhase,
        },
        render_resource::{CachedRenderPipelineId, LoadOp, Operations, RenderPassDescriptor},
        view::{ExtractedView, ViewTarget, VisibleEntities},
        Extract, ExtractSchedule, RenderApp, RenderSet,
    },
    window::{RequestRedraw, WindowResized},
};

const GRAPH_NAME: &'static str = "ui_graph";
const UI_INPUT_NAME: &'static str = "view";

pub struct UiCameraPlugin;

impl Plugin for UiCameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(redraw_on_resize)
            .add_plugin(ExtractComponentPlugin::<UiCamera>::default());

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<DrawFunctions<UiPhaseItem>>()
            .add_system(extract_ui_camera_phases.in_schedule(ExtractSchedule))
            .add_system(sort_phase_system::<UiPhaseItem>.in_set(RenderSet::PhaseSort));

        let ui_pass_node = UiPassNode::new(&mut render_app.world);
        let upscaling_pass_node = UpscalingNode::new(&mut render_app.world);
        let mut world_render_graph = render_app.world.resource_mut::<RenderGraph>();

        let mut render_ui_graph = RenderGraph::default();

        let ui_node = render_ui_graph.add_node(UiPassNode::NAME, ui_pass_node);
        let upscaling_node = render_ui_graph.add_node("upscaling", upscaling_pass_node);

        let ui_input_node =
            render_ui_graph.set_input(vec![SlotInfo::new(UI_INPUT_NAME, SlotType::Entity)]);

        render_ui_graph.add_slot_edge(
            ui_input_node,
            UI_INPUT_NAME,
            ui_node,
            UiPassNode::IN_VIEW_ENTITY,
        );

        render_ui_graph.add_slot_edge(
            ui_input_node,
            UI_INPUT_NAME,
            upscaling_node,
            UpscalingNode::IN_VIEW,
        );

        render_ui_graph.add_node_edge(ui_node, upscaling_node);
        world_render_graph.add_sub_graph(GRAPH_NAME, render_ui_graph);
    }
}

fn redraw_on_resize(
    mut resize_reader: EventReader<WindowResized>,
    mut request_redraw_writer: EventWriter<RequestRedraw>,
) {
    let mut redraw = false; // Using a bool to consume the reader & doing a request once

    for _ in resize_reader.iter() {
        redraw = true;
    }

    if redraw {
        request_redraw_writer.send(RequestRedraw);
    }
}

#[derive(Component, ExtractComponent, Clone, Default)]
#[extract_component_filter(With<Camera>)]
pub struct UiCamera;

#[derive(Bundle)]
pub struct UiCameraBundle {
    pub camera: Camera,
    pub camera_graph: CameraRenderGraph,
    pub ui_camera: UiCamera,
    global_transform: GlobalTransform,
    visible_entities: VisibleEntities,
    projection: OrthographicProjection,
}

impl Default for UiCameraBundle {
    fn default() -> Self {
        UiCameraBundle {
            camera: Camera::default(),
            camera_graph: CameraRenderGraph::new(GRAPH_NAME),
            ui_camera: UiCamera,
            global_transform: GlobalTransform::default(),
            visible_entities: VisibleEntities::default(),
            projection: Default::default(),
        }
    }
}

pub struct UiPhaseItem {
    pub entity: Entity,
    pub z_index: u32,

    pub draw_function: DrawFunctionId,
    pub cached_render_pipeline_id: CachedRenderPipelineId,

    pub batch_range: Option<Range<u32>>,
}

impl PhaseItem for UiPhaseItem {
    type SortKey = u32;

    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }

    fn entity(&self) -> bevy::prelude::Entity {
        self.entity
    }

    fn sort_key(&self) -> Self::SortKey {
        self.z_index
    }

    fn sort(items: &mut [Self]) {
        items.sort_unstable_by_key(|item| item.z_index);
    }
}

impl CachedRenderPipelinePhaseItem for UiPhaseItem {
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.cached_render_pipeline_id
    }
}

impl BatchedPhaseItem for UiPhaseItem {
    fn batch_range(&self) -> &Option<Range<u32>> {
        &self.batch_range
    }

    fn batch_range_mut(&mut self) -> &mut Option<Range<u32>> {
        &mut self.batch_range
    }
}

#[derive(Component)]
pub struct PhysicalViewportSize(pub Option<UVec2>);

fn extract_ui_camera_phases(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &Camera), With<UiCamera>>>,
) {
    for (entity, camera) in cameras.iter() {
        if camera.is_active {
            commands.get_or_spawn(entity).insert((
                PhysicalViewportSize(camera.physical_viewport_size()),
                RenderPhase::<UiPhaseItem>::default(),
            ));
        }
    }
}

pub struct UiPassNode(
    QueryState<
        (
            &'static ExtractedCamera,
            &'static RenderPhase<UiPhaseItem>,
            &'static ViewTarget,
        ),
        (With<ExtractedView>, With<UiCamera>),
    >,
);

impl UiPassNode {
    const NAME: &'static str = "ui_pass_node";
    const IN_VIEW_ENTITY: &'static str = "view";

    pub fn new(world: &mut World) -> UiPassNode {
        UiPassNode(world.query_filtered())
    }
}

impl Node for UiPassNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(Self::IN_VIEW_ENTITY, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.0.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW_ENTITY)?;

        let Ok((camera, ui_phase, view_target)) = self.0.get_manual(world, view_entity) else {
            return Ok(());
        };

        {
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("ui_render_pass"),
                color_attachments: &[Some(view_target.get_color_attachment(Operations {
                    load: LoadOp::Clear(Color::WHITE.into()),
                    store: true,
                }))],
                depth_stencil_attachment: None,
            });

            if let Some(viewport) = &camera.viewport {
                render_pass.set_camera_viewport(viewport);
            }

            ui_phase.render(&mut render_pass, &world, view_entity);
        }

        Ok(())
    }
}
