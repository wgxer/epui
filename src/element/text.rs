use bevy::{
    ecs::system::lifetimeless::{Read, SRes},
    prelude::{
        error, Bundle, Changed, Color, Commands, Component, Entity, IntoSystemConfigs, Or, Plugin,
        Query, ReflectComponent, Res, ResMut, Resource, Without,
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
        Extract, ExtractSchedule, MainWorld, Render, RenderApp, RenderSet,
    },
    utils::HashMap,
};
use glyphon::{FontSystem, Metrics, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer};

use crate::{
    camera::{PhysicalViewportSize, UiPhaseItem},
    prelude::{AutoZUpdate, ColoredElement, Position, Size},
    property::{
        state::{Active, ActiveOptionExt},
        update::AutoVisibleRegionUpdate,
        VisibleRegion, ZLevel,
    },
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

#[derive(Component)]
pub struct PreparedText;

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
            render_app.world.resource::<RenderDevice>(),
            &render_app.world.resource::<RenderQueue>().0,
            TextureFormat::bevy_default(),
        );

        render_app
            .insert_resource(TextRenderData::new(font_system, swash_cache, text_atlas))
            .init_resource::<TextCachedUiPhases>()
            .add_render_command::<UiPhaseItem, RenderTextCommand>()
            .add_systems(ExtractSchedule, extract_texts)
            .add_systems(
                Render,
                (
                    prepare_texts.in_set(RenderSet::Prepare),
                    queue_texts.in_set(RenderSet::Queue),
                ),
            );
    }
}

fn extract_texts(
    main_world: Res<MainWorld>,
    mut commands: Commands,
    texts: Extract<
        Query<
            (
                Entity,
                &UiText,
                &FontSize,
                &Position,
                &Size,
                &VisibleRegion,
                &ColoredElement,
                Option<&ZLevel>,
                Option<&Active<UiText>>,
                Option<&Active<FontSize>>,
                Option<&Active<Position>>,
                Option<&Active<Size>>,
                Option<&Active<VisibleRegion>>,
                Option<&Active<ColoredElement>>,
                Option<&Active<ZLevel>>,
            ),
            Or<(
                Changed<UiText>,
                Changed<FontSize>,
                Changed<Position>,
                Changed<Size>,
                Changed<VisibleRegion>,
                Changed<ColoredElement>,
                Changed<ZLevel>,
                Changed<Active<UiText>>,
                Changed<Active<FontSize>>,
                Changed<Active<Position>>,
                Changed<Active<Size>>,
                Changed<Active<VisibleRegion>>,
                Changed<Active<ColoredElement>>,
                Changed<Active<ZLevel>>,
            )>,
        >,
    >,
) {
    for (
        entity,
        base_text,
        base_font_size,
        base_position,
        base_size,
        base_visible_region,
        base_colored_element,
        base_z_level,
        text,
        font_size,
        position,
        size,
        visible_region,
        colored_element,
        z_level,
    ) in texts.iter()
    {
        bevy::log::info!("text_update");
        let entity_ref = main_world.entity(entity);

        let (text, font_size, position, size, visible_region, colored_element, z_level) = (
            text.active_or_base(&entity_ref, base_text),
            font_size.active_or_base(&entity_ref, base_font_size),
            position.active_or_base(&entity_ref, base_position),
            size.active_or_base(&entity_ref, base_size),
            visible_region.active_or_base(&entity_ref, base_visible_region),
            colored_element.active_or_base(&entity_ref, base_colored_element),
            z_level
                .map(|z_level| z_level.active(&entity_ref))
                .or(base_z_level)
                .cloned(),
        );

        commands
            .get_or_spawn(entity)
            .insert((
                text.clone(),
                font_size.clone(),
                position.clone(),
                z_level.unwrap_or_default().clone(),
                size.clone(),
                visible_region.clone(),
                colored_element.clone(),
            ))
            .remove::<PreparedText>();
    }
}

fn prepare_texts(
    mut commands: Commands,
    mut text_render_data: ResMut<TextRenderData>,
    mut text_cached_ui_phases: ResMut<TextCachedUiPhases>,
    mut texts: Query<
        (Entity, &Size, &UiText, &FontSize, Option<&mut UiTextBuffer>),
        Without<PreparedText>,
    >,
) {
    if texts.is_empty() {
        return;
    }

    text_cached_ui_phases.phases.clear();

    for (entity, size, text, font_size, buffer) in texts.iter_mut() {
        if let Some(mut buffer) = buffer {
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

            commands.entity(entity).insert(PreparedText);
        } else if let Some(mut commands) = commands.get_entity(entity) {
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
            commands.insert((UiTextBuffer(buffer), PreparedText));
        } else {
            continue;
        }
    }
}

#[derive(Resource, Default)]
struct TextCachedUiPhases {
    phases: HashMap<u32, UiPhaseItem>,
}

fn queue_texts(
    mut commands: Commands,
    mut view_query: Query<(&PhysicalViewportSize, &mut RenderPhase<UiPhaseItem>)>,
    mut text_render_data: ResMut<TextRenderData>,
    mut text_cached_ui_phases: ResMut<TextCachedUiPhases>,
    texts: Query<(
        Entity,
        &UiTextBuffer,
        &Position,
        &ZLevel,
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

        if !text_cached_ui_phases.phases.is_empty() {
            for (_, phase) in &text_cached_ui_phases.phases {
                ui_phase.add(phase.clone());

                commands
                    .get_or_spawn(phase.entity)
                    .insert(ZLevel(phase.z_index));
            }

            continue;
        }

        let mut text_areas_map = HashMap::new();

        for (entity, text_buffer, position, z_level, visible_region, colored_element) in
            texts.iter()
        {
            let text_areas_vec =
                if let Some((_, text_areas_vec)) = text_areas_map.get_mut(&z_level.0) {
                    text_areas_vec
                } else {
                    text_areas_map.insert(&z_level.0, (entity, Vec::new()));

                    let Some((_, text_areas_vec)) = text_areas_map.get_mut(&z_level.0) else {
                        error!("Getting the value of the inserted z {} failed", z_level.0);
                        continue;
                    };

                    text_areas_vec
                };

            let [r, g, b, a] = colored_element
                .color
                .as_rgba_f32()
                .map(|x| (x * 255.0f32) as u8);

            bevy::log::info!("TextBufferLen: {:#?}", text_buffer.0.lines.len());

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

        for (z_index, (phase_entity, text_areas_vec)) in text_areas_map {
            let text_renderer = if let Some(text_renderer) = text_renderers.get_mut(z_index) {
                text_renderer
            } else {
                text_renderers
                    .insert_unique_unchecked(
                        *z_index,
                        TextRenderer::new(
                            text_atlas,
                            &device,
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
                &device,
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
                entity: phase_entity,
                batch_range: None,
                cached_render_pipeline_id: CachedRenderPipelineId::INVALID,
                z_index: *z_index,
                draw_function: draw_function_id,
            };

            text_cached_ui_phases.phases.insert(*z_index, phase.clone());
            ui_phase.add(phase);
        }
    }
}

struct RenderTextCommand;

impl<P: PhaseItem> RenderCommand<P> for RenderTextCommand {
    type Param = SRes<TextRenderData>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<ZLevel>;

    fn render<'w>(
        _item: &P,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        z_level: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        param: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        let span = bevy::log::debug_span!("RenderCommands: RenderTextCommand");

        span.in_scope(|| {
            let param = param.into_inner();

            let Some(text_renderer) = param.text_renderers.get(&z_level.0) else {
                error!("Couldn't find a text renderer for z level: {}", z_level.0);
                return RenderCommandResult::Failure;
            };

            if let Err(err) = text_renderer.render(&param.text_atlas, pass) {
                error!("Error during rendering text: {:?}", err);
                RenderCommandResult::Failure
            } else {
                RenderCommandResult::Success
            }
        })
    }
}
