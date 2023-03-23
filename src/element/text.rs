use bevy::{
    ecs::system::lifetimeless::SRes,
    prelude::{
        error, Bundle, Color, Commands, Component, Entity, IntoSystemAppConfig, IntoSystemConfig,
        Plugin, Query, ReflectComponent, Res, ResMut, Resource,
    },
    reflect::Reflect,
    render::{
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase,
        },
        render_resource::{CachedRenderPipelineId, MultisampleState, TextureFormat},
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        Extract, ExtractSchedule, RenderApp, RenderSet,
    },
};
use glyphon::{FontSystem, Metrics, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer};

use crate::{
    camera::{PhysicalViewportSize, UiPhaseItem},
    prelude::{AutoZUpdate, ColoredElement, Position, Size},
    property::{update::AutoVisibleRegionUpdate, VisibleRegion, ZLevel},
};

#[derive(Component, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct UiText {
    pub text: String,
    pub font_size: u32,
}

impl Default for UiText {
    fn default() -> Self {
        UiText {
            text: String::new(),
            font_size: 16,
        }
    }
}

#[derive(Bundle)]
pub struct UiTextBundle {
    pub text: UiText,
    pub colored_element: ColoredElement,

    pub position: Position,
    pub size: Size,

    pub z_level: ZLevel,
    pub auto_z_update: AutoZUpdate,

    pub visible_region: VisibleRegion,
    pub auto_visible_region_update: AutoVisibleRegionUpdate,
}

impl Default for UiTextBundle {
    fn default() -> Self {
        Self {
            text: Default::default(),
            colored_element: ColoredElement::new(Color::BLACK),

            position: Default::default(),
            size: Default::default(),

            z_level: Default::default(),
            auto_z_update: Default::default(),

            visible_region: Default::default(),
            auto_visible_region_update: Default::default(),
        }
    }
}

pub struct UiTextPlugin;

#[derive(Component)]
struct UiTextBuffer(glyphon::Buffer);

#[derive(Resource)]
struct TextRenderData {
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
}

impl TextRenderData {
    fn new(
        font_system: FontSystem,
        swash_cache: SwashCache,
        text_atlas: TextAtlas,
        text_renderer: TextRenderer,
    ) -> TextRenderData {
        TextRenderData {
            font_system,
            swash_cache,
            text_atlas,
            text_renderer,
        }
    }
}

#[derive(Resource)]
struct TextPipeline(CachedRenderPipelineId);

impl Plugin for UiTextPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        let mut text_atlas = TextAtlas::new(
            render_app.world.resource::<RenderDevice>().wgpu_device(),
            &render_app.world.resource::<RenderQueue>().0,
            TextureFormat::bevy_default(),
        );

        let text_renderer = TextRenderer::new(
            &mut text_atlas,
            render_app.world.resource::<RenderDevice>().wgpu_device(),
            MultisampleState {
                count: 4,
                ..Default::default()
            },
            None,
        );

        render_app
            .insert_resource(TextRenderData::new(
                font_system,
                swash_cache,
                text_atlas,
                text_renderer,
            ))
            .add_render_command::<UiPhaseItem, RenderTextCommand>()
            .add_system(extract_texts.in_schedule(ExtractSchedule))
            .add_system(prepare_texts.in_set(RenderSet::Prepare))
            .add_system(queue_texts.in_set(RenderSet::Queue));
    }
}

fn extract_texts(
    mut commands: Commands,
    texts: Extract<
        Query<(
            Entity,
            &UiText,
            &Position,
            &Size,
            &VisibleRegion,
            &ColoredElement,
        )>,
    >,
) {
    for (entity, text, position, size, visible_region, colored_element) in texts.iter() {
        commands.get_or_spawn(entity).insert((
            text.clone(),
            position.clone(),
            size.clone(),
            visible_region.clone(),
            colored_element.clone(),
        ));
    }
}

fn prepare_texts(
    mut commands: Commands,
    mut text_render_data: ResMut<TextRenderData>,
    mut texts: Query<(Entity, &Size, &UiText, Option<&mut UiTextBuffer>)>,
) {
    for (entity, size, text, buffer) in texts.iter_mut() {
        if let Some(mut buffer) = buffer {
            buffer.0.set_text(
                &mut text_render_data.font_system,
                &text.text,
                glyphon::Attrs::new(),
            );

            for line in &mut buffer.0.lines {
                line.set_align(Some(glyphon::cosmic_text::Align::Left));
            }

            buffer.0.set_size(
                &mut text_render_data.font_system,
                size.width as f32,
                size.height as f32,
            );

            buffer.0.set_metrics(
                &mut text_render_data.font_system,
                Metrics::new(text.font_size as f32, text.font_size as f32 + 4.0f32),
            );

            buffer
                .0
                .shape_until_scroll(&mut text_render_data.font_system);
        } else if let Some(mut commands) = commands.get_entity(entity) {
            let mut buffer = glyphon::Buffer::new(
                &mut text_render_data.font_system,
                Metrics::new(text.font_size as f32, text.font_size as f32 + 4.0f32),
            );

            buffer.set_text(
                &mut text_render_data.font_system,
                &text.text,
                glyphon::Attrs::new(),
            );

            for line in &mut buffer.lines {
                line.set_align(Some(glyphon::cosmic_text::Align::Left));
            }

            buffer.set_size(
                &mut text_render_data.font_system,
                size.width as f32,
                size.height as f32,
            );

            buffer.shape_until_scroll(&mut text_render_data.font_system);
            commands.insert(UiTextBuffer(buffer));
        } else {
            continue;
        }
    }
}

fn queue_texts(
    mut view_query: Query<(&PhysicalViewportSize, &mut RenderPhase<UiPhaseItem>)>,
    mut text_render_data: ResMut<TextRenderData>,
    texts: Query<(
        Entity,
        &UiTextBuffer,
        &Position,
        &VisibleRegion,
        &ColoredElement,
    )>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    draw_functions: Res<DrawFunctions<UiPhaseItem>>,
) {
    for (viewport_size, mut ui_phase) in view_query.iter_mut() {
        let Some(viewport_size) = viewport_size.0 else {
            continue;
        };

        let mut text_areas = Vec::new();
        let mut phase_entity = None;

        for (entity, text_buffer, position, visible_region, colored_element) in texts.iter() {
            text_areas.push(TextArea {
                buffer: &text_buffer.0,
                left: position.x as i32,
                top: position.y as i32,
                bounds: TextBounds {
                    left: visible_region.x as i32,
                    top: visible_region.y as i32,

                    right: visible_region.x as i32
                        + visible_region.width.min(i32::MAX as u32) as i32,
                    bottom: visible_region.y as i32
                        + visible_region.height.min(i32::MAX as u32) as i32,
                },
                default_color: glyphon::Color(colored_element.color.as_rgba_u32()),
            });

            phase_entity = Some(entity);
        }

        if let Some(phase_entity) = phase_entity {
            let TextRenderData {
                font_system,
                swash_cache,
                text_atlas,
                text_renderer,
            } = text_render_data.as_mut();

            if let Err(err) = text_renderer.prepare(
                device.wgpu_device(),
                &queue,
                font_system,
                text_atlas,
                glyphon::Resolution {
                    width: viewport_size.x,
                    height: viewport_size.y,
                },
                &text_areas,
                swash_cache,
            ) {
                error!("Error during preparing text renderer: {:?}", err);
            };

            let Some(draw_function_id) = draw_functions.read().get_id::<RenderTextCommand>() else {
                error!("Couldn't find text draw function id");
                continue;
            };

            ui_phase.add(UiPhaseItem {
                entity: phase_entity,
                batch_range: None,
                cached_render_pipeline_id: CachedRenderPipelineId::INVALID,
                z_index: 0,
                draw_function: draw_function_id,
            })
        }
    }
}

struct RenderTextCommand;

impl<P: PhaseItem> RenderCommand<P> for RenderTextCommand {
    type Param = SRes<TextRenderData>;
    type ItemWorldQuery = ();
    type ViewWorldQuery = ();

    fn render<'w>(
        _item: &P,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        _entity: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        param: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        let param = param.into_inner();

        if let Err(err) = param.text_renderer.render(&param.text_atlas, pass) {
            error!("Error during rendering text: {:?}", err);
            RenderCommandResult::Failure
        } else {
            RenderCommandResult::Success
        }
    }
}
