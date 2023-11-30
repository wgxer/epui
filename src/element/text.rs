use std::ops::{Deref, DerefMut};

use bevy::{
    ecs::system::lifetimeless::SRes,
    log::error,
    prelude::{
        Bundle, Color, Component, Entity, IntoSystemConfigs, Plugin, Query, ReflectComponent, Res,
        ResMut, Resource,
    },
    reflect::Reflect,
    render::{
        render_phase::{
            AddRenderCommand, DrawFunctions, RenderCommand, RenderCommandResult, RenderPhase,
        },
        render_resource::{CachedRenderPipelineId, MultisampleState, TextureFormat},
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
    utils::HashMap,
};
use glyphon::{FontSystem, Metrics, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer};

use crate::{
    camera::{PhysicalViewportSize, UiPhaseItem},
    prelude::{AutoZUpdate, ColoredElement, Position, Size},
    property::{state::CurrentlyActive, update::AutoVisibleRegionUpdate, VisibleRegion, ZLevel},
};

#[derive(Component, Debug, Default, Clone, Hash, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct UiText(pub String);

#[derive(Component, Debug, Clone, Hash, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct FontSize(pub u32);

impl Default for FontSize {
    fn default() -> Self {
        FontSize(16)
    }
}

#[derive(Bundle)]
pub struct UiTextBundle {
    pub text: UiText,
    pub font_size: FontSize,
    pub color: ColoredElement,

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
            font_size: Default::default(),
            color: ColoredElement::new(Color::BLACK),

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
    text_renderers: HashMap<u32, TextRenderer>,
}

impl TextRenderData {
    fn new(
        font_system: FontSystem,
        swash_cache: SwashCache,
        text_atlas: TextAtlas,
    ) -> TextRenderData {
        TextRenderData {
            font_system,
            swash_cache,
            text_atlas,
            text_renderers: HashMap::new(),
        }
    }
}

#[derive(Resource)]
struct TextPipeline(CachedRenderPipelineId);

impl Plugin for UiTextPlugin {
    fn build(&self, _app: &mut bevy::prelude::App) {}

    fn finish(&self, app: &mut bevy::prelude::App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        let text_atlas = TextAtlas::new(
            render_app.world.resource::<RenderDevice>().wgpu_device(),
            &render_app.world.resource::<RenderQueue>().0,
            TextureFormat::bevy_default(),
        );

        render_app
            .insert_resource(TextRenderData::new(font_system, swash_cache, text_atlas))
            .init_resource::<ExtractedTexts>()
            .add_render_command::<UiPhaseItem, RenderTextCommand>()
            .add_systems(ExtractSchedule, extract_texts)
            .add_systems(
                Render,
                (
                    prepare_texts.in_set(RenderSet::PrepareAssets),
                    queue_texts.in_set(RenderSet::Queue),
                ),
            );
    }
}

struct TextInstance {
    text: UiText,
    font_size: FontSize,

    position: Position,
    size: Size,

    z_level: ZLevel,
    visible_region: VisibleRegion,

    color: ColoredElement,
    text_buffer: Option<UiTextBuffer>,
}

#[derive(Resource, Default)]
struct ExtractedTexts(Vec<TextInstance>);

impl Deref for ExtractedTexts {
    type Target = Vec<TextInstance>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ExtractedTexts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn extract_texts(
    texts: Extract<
        Query<(
            &CurrentlyActive<UiText>,
            &CurrentlyActive<FontSize>,
            &CurrentlyActive<Position>,
            &CurrentlyActive<Size>,
            &CurrentlyActive<VisibleRegion>,
            &CurrentlyActive<ColoredElement>,
            Option<&CurrentlyActive<ZLevel>>,
        )>,
    >,
    mut extracted_texts: ResMut<ExtractedTexts>,
) {
    extracted_texts.clear();

    let texts_count = texts.iter().len();
    if texts_count > extracted_texts.capacity() {
        let additional_slots = texts_count - extracted_texts.len() + 2;

        extracted_texts.reserve_exact(additional_slots);
    }

    if extracted_texts.capacity().abs_diff(extracted_texts.len()) >= 5 {
        extracted_texts.shrink_to_fit();
    }

    for (text, font_size, position, size, visible_region, colored_element, z_level) in texts.iter()
    {
        extracted_texts.push(TextInstance {
            text: text.clone(),
            font_size: font_size.clone(),

            position: position.clone(),
            size: size.clone(),

            z_level: z_level.cloned().unwrap_or_default(),
            visible_region: visible_region.clone(),

            color: colored_element.clone(),
            text_buffer: None,
        });
    }
}

fn prepare_texts(
    mut text_render_data: ResMut<TextRenderData>,
    mut extracted_texts: ResMut<ExtractedTexts>,
) {
    for TextInstance {
        size,
        text,
        font_size,
        text_buffer,
        ..
    } in extracted_texts.iter_mut()
    {
        if let Some(buffer) = text_buffer.as_mut() {
            buffer.0.set_text(
                &mut text_render_data.font_system,
                &text.0,
                glyphon::Attrs::new(),
                glyphon::Shaping::Advanced,
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
                Metrics::new(font_size.0 as f32, font_size.0 as f32 + 4.0f32),
            );

            buffer
                .0
                .shape_until_scroll(&mut text_render_data.font_system);
        } else {
            let mut buffer = glyphon::Buffer::new(
                &mut text_render_data.font_system,
                Metrics::new(font_size.0 as f32, font_size.0 as f32 + 4.0f32),
            );

            buffer.set_text(
                &mut text_render_data.font_system,
                &text.0,
                glyphon::Attrs::new(),
                glyphon::Shaping::Advanced,
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
            *text_buffer = Some(UiTextBuffer(buffer));
        }
    }
}

fn queue_texts(
    mut view_query: Query<(Entity, &PhysicalViewportSize, &mut RenderPhase<UiPhaseItem>)>,
    mut text_render_data: ResMut<TextRenderData>,
    extracted_texts: Res<ExtractedTexts>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    draw_functions: Res<DrawFunctions<UiPhaseItem>>,
) {
    for (view_entity, viewport_size, mut ui_phase) in view_query.iter_mut() {
        let Some(viewport_size) = viewport_size.0 else {
            continue;
        };

        let mut text_areas_map = HashMap::new();

        for TextInstance {
            text_buffer,
            position,
            z_level,
            visible_region,
            color,
            ..
        } in extracted_texts.iter()
        {
            let Some(text_buffer) = text_buffer else {
                continue;
            };

            let text_areas_vec = if let Some(text_areas_vec) = text_areas_map.get_mut(&z_level.0) {
                text_areas_vec
            } else {
                text_areas_map.insert(&z_level.0, Vec::new());

                let Some(text_areas_vec) = text_areas_map.get_mut(&z_level.0) else {
                    error!("Getting the value of the inserted z {} failed", z_level.0);
                    continue;
                };

                text_areas_vec
            };

            let [r, g, b, a] = color.color.as_rgba_f32().map(|x| (x * 255.0f32) as u8);

            text_areas_vec.push(TextArea {
                buffer: &text_buffer.0,
                left: position.x as f32,
                top: position.y as f32,
                scale: 1.0f32,
                bounds: TextBounds {
                    left: visible_region.x as i32,
                    top: visible_region.y as i32,

                    right: visible_region.x as i32
                        + visible_region.width.min(i32::MAX as u32) as i32,
                    bottom: visible_region.y as i32
                        + visible_region.height.min(i32::MAX as u32) as i32,
                },
                default_color: glyphon::Color::rgba(r, g, b, a),
            });
        }

        let TextRenderData {
            font_system,
            swash_cache,
            text_atlas,
            text_renderers,
        } = text_render_data.as_mut();

        for (z_index, text_areas_vec) in text_areas_map {
            let text_renderer = if let Some(text_renderer) = text_renderers.get_mut(z_index) {
                text_renderer
            } else {
                text_renderers
                    .insert_unique_unchecked(
                        *z_index,
                        TextRenderer::new(
                            text_atlas,
                            device.wgpu_device(),
                            MultisampleState {
                                count: 4,
                                ..Default::default()
                            },
                            None,
                        ),
                    )
                    .1
            };

            if let Err(err) = text_renderer.prepare(
                device.wgpu_device(),
                &queue,
                font_system,
                text_atlas,
                glyphon::Resolution {
                    width: viewport_size.x,
                    height: viewport_size.y,
                },
                text_areas_vec,
                swash_cache,
            ) {
                error!("Error during preparing text renderer: {:?}", err);
            };

            let Some(draw_function_id) = draw_functions.read().get_id::<RenderTextCommand>() else {
                error!("Couldn't find text draw function id");
                continue;
            };

            let phase = UiPhaseItem {
                entity: view_entity,
                z_index: *z_index,

                draw_function: draw_function_id,
                cached_render_pipeline_id: CachedRenderPipelineId::INVALID,

                batch_range: 0..1,
                dynamic_offset: None,
            };

            ui_phase.add(phase);
        }
    }
}

struct RenderTextCommand;

impl RenderCommand<UiPhaseItem> for RenderTextCommand {
    type Param = SRes<TextRenderData>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = ();

    fn render<'w>(
        item: &UiPhaseItem,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        _entity: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        param: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        let span = bevy::log::debug_span!("RenderCommands: RenderTextCommand");

        span.in_scope(|| {
            let param = param.into_inner();

            let Some(text_renderer) = param.text_renderers.get(&item.z_index) else {
                error!(
                    "Couldn't find a text renderer for z level: {}",
                    item.z_index
                );

                return RenderCommandResult::Failure;
            };

            if let Err(err) = text_renderer.render(&param.text_atlas, pass.wgpu_pass()) {
                error!("Error during rendering text: {:?}", err);
                RenderCommandResult::Failure
            } else {
                RenderCommandResult::Success
            }
        })
    }
}
