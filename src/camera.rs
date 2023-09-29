use std::ops::Range;

use bevy::{
    core_pipeline::upscaling::UpscalingNode,
    ecs::query::QueryItem,
    prelude::{
        Bundle, Camera, Color, Commands, Component, Entity, EventReader, EventWriter, FromWorld,
        GlobalTransform, IntoSystemConfigs, OrthographicProjection, Plugin, Query, UVec2, Update,
        With, World,
    },
    render::{
        camera::{CameraRenderGraph, ExtractedCamera},
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::{NodeRunError, RenderGraph, ViewNode, ViewNodeRunner},
        render_phase::{
            sort_phase_system, BatchedPhaseItem, CachedRenderPipelinePhaseItem, DrawFunctionId,
            DrawFunctions, PhaseItem, RenderPhase,
        },
        render_resource::{CachedRenderPipelineId, LoadOp, Operations, RenderPassDescriptor},
        view::{ViewTarget, VisibleEntities},
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
    window::{RequestRedraw, WindowResized},
};

const GRAPH_NAME: &'static str = "ui_graph";

pub struct UiCameraPlugin;

impl Plugin for UiCameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, redraw_on_resize)
            .add_plugins(ExtractComponentPlugin::<UiCamera>::default());
    }

    fn finish(&self, app: &mut bevy::prelude::App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<DrawFunctions<UiPhaseItem>>()
            .add_systems(ExtractSchedule, extract_ui_camera_phases)
            .add_systems(
                Render,
                sort_phase_system::<UiPhaseItem>.in_set(RenderSet::PhaseSort),
            );

        let ui_pass_node = ViewNodeRunner::new(UiPassNode, &mut render_app.world);

        let upscaling_pass_node = ViewNodeRunner::new(
            UpscalingNode::from_world(&mut render_app.world),
            &mut render_app.world,
        );
        let mut world_render_graph = render_app.world.resource_mut::<RenderGraph>();

        let mut render_ui_graph = RenderGraph::default();

        let ui_node = render_ui_graph.add_node(UiPassNode::NAME, ui_pass_node);
        let upscaling_node = render_ui_graph.add_node("upscaling", upscaling_pass_node);

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

pub struct UiPassNode;

impl UiPassNode {
    const NAME: &'static str = "ui_pass_node";
}

impl ViewNode for UiPassNode {
    type ViewQuery = (
        &'static ExtractedCamera,
        &'static RenderPhase<UiPhaseItem>,
        &'static ViewTarget,
    );

    fn run(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        (camera, ui_phase, view_target): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.view_entity();

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
