use super::user_input_plugin::{UserInput, UserInputPosition};
use crate::resources::inputs::Inputs;
use bevy::{ecs::system::Command, prelude::*, ui::ui_focus_system, utils::HashSet};
use bevy_vector_shapes::{
    painter::{ShapeConfig, ShapePainter},
    shapes::{RectPainter, ThicknessType},
};

#[derive(Clone, Copy)]
pub struct UiPointerEventData {
    pub pos: Vec2,
    pub user_input: UserInput,
}

impl Default for UiPointerEventData {
    fn default() -> Self {
        Self {
            pos: Vec2::default(),
            user_input: UserInput(0),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct UiOnClick {
    pub command: UiCommand<UiPointerEventData>,
    // if true, the click will be handled on press
    pub eager_handle: bool,
    // if false, the click will not be handled once
    handle: bool,
}

impl UiOnClick {
    pub fn new(on_click: fn(&mut World, &UiCommandContext<UiPointerEventData>) -> ()) -> Self {
        Self {
            command: UiCommand::new(on_click),
            ..Default::default()
        }
    }
}

impl Default for UiOnClick {
    fn default() -> Self {
        Self {
            command: UiCommand::default(),
            eager_handle: false,
            handle: true,
        }
    }
}

#[derive(Clone, Copy)]
pub struct UiCommandContext<E: Clone + Copy + Default> {
    pub entity: Entity,
    pub event_data: Option<E>,
}

impl<E: Clone + Copy + Default> Default for UiCommandContext<E> {
    fn default() -> Self {
        Self {
            entity: Entity::from_raw(u32::MAX),
            event_data: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct UiCommand<E: Clone + Copy + Default> {
    command: fn(&mut World, &UiCommandContext<E>) -> (),
    context: UiCommandContext<E>,
}

impl<E: Clone + Copy + Default> UiCommand<E> {
    pub fn new(command: fn(&mut World, &UiCommandContext<E>) -> ()) -> Self {
        Self {
            command,
            ..Default::default()
        }
    }
}

impl<E: Clone + Copy + Default> Default for UiCommand<E> {
    fn default() -> Self {
        Self {
            command: |_, _| {},
            context: UiCommandContext::<E>::default(),
        }
    }
}

impl<E: Clone + Copy + Default + Send + Sync + 'static> Command for UiCommand<E> {
    fn apply(self, world: &mut World) {
        (self.command)(world, &self.context);
    }
}

#[derive(Bundle, Default)]
pub struct UiOnClickBundle {
    pub ui_on_click: UiOnClick,
    pub interaction: Interaction,
}

pub mod components {
    use super::*;
    use crate::resources::{
        game_assets::{GameAssets, GameColor},
        text_styles::{FontSize, FontType, TextStyles},
    };
    use bevy::ecs::system::{EntityCommands, SystemParam};

    pub trait UiComponent: Sized {
        fn spawn<'a>(&'a self, parent: &'a mut ChildBuilder) -> EntityCommands;
        fn new(builder: &mut UiBuilder, width: Val, height: Val, z: f32) -> Self;
        fn new_auto(builder: &mut UiBuilder, z: f32) -> Self {
            Self::new(builder, Val::Auto, Val::Auto, z)
        }
    }

    #[derive(SystemParam)]
    pub struct UiBuilder<'w, 's> {
        pub game_assets: Res<'w, GameAssets>,
        pub text_styles: Res<'w, TextStyles>,
        pub window: Query<'w, 's, &'static Window>,
        z: Local<'s, f32>,
    }

    impl<'w, 's> UiBuilder<'w, 's> {
        pub fn create<C: UiComponent>(&mut self, width: Val, height: Val) -> C {
            let z = self.get_next_z();
            C::new(self, width, height, z)
        }

        pub fn create_auto<C: UiComponent>(&mut self) -> C {
            let z = self.get_next_z();
            C::new_auto(self, z)
        }

        pub fn get_next_z(&mut self) -> f32 {
            *self.z += 1.0;
            *self.z
        }

        pub fn window(&self) -> &Window {
            self.window.single()
        }
    }

    pub struct UiBase {
        pub color: Color,
        pub style: Style,
    }

    impl UiBase {
        pub fn new(color: Color) -> Self {
            Self {
                color,
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
            }
        }

        pub fn spawn<'a>(&'a self, commands: &'a mut Commands) -> EntityCommands {
            commands.spawn((
                UiNode::container(
                    UiStyle {
                        color: self.color,
                        border_radius: 0.0,
                        border_width: 0.0,
                        ..Default::default()
                    },
                    0.0,
                ),
                NodeBundle {
                    style: self.style.clone(),
                    ..Default::default()
                },
            ))
        }
    }
    pub struct UiContainer {
        pub style: Style,
        pub ui_style: UiStyle,
        node: Box<dyn Fn(Style) -> NodeBundle>,
        ui_node: Box<dyn Fn(UiStyle) -> UiNode>,
    }

    impl UiComponent for UiContainer {
        fn new<'a>(builder: &mut UiBuilder, width: Val, height: Val, z: f32) -> Self {
            let window = &builder.window.single();
            Self {
                style: Style {
                    width,
                    height,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(32.0 * window.scale_factor())),
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                ui_style: UiStyle {
                    color: builder.game_assets.colors.get(GameColor::Primary),
                    border_color: Some(builder.game_assets.colors.get_content(GameColor::Primary)),
                    ..Default::default()
                },
                node: Box::new(|style| NodeBundle {
                    style,
                    ..Default::default()
                }),
                ui_node: Box::new(move |s| UiNode::container(s, z)),
            }
        }

        fn spawn<'a>(&'a self, parent: &'a mut ChildBuilder) -> EntityCommands {
            let node = (self.node)(self.style.clone());
            let ui_node = (self.ui_node)(self.ui_style);
            parent.spawn((node, ui_node))
        }
    }

    impl UiContainer {
        pub fn with_game_color(mut self, game_color: GameColor, ui_builder: &UiBuilder) -> Self {
            self.ui_style.color = ui_builder.game_assets.colors.get(game_color);
            self.ui_style.border_color =
                Some(ui_builder.game_assets.colors.get_content(game_color));
            self
        }
    }

    pub struct UiButton {
        pub text: String,
        pub style: Style,
        pub ui_style: UiStyle,
        pub text_style: TextStyle,
        pub on_click: UiOnClick,
        button:
            Box<dyn Fn(Style, UiStyle, UiOnClick) -> (NodeBundle, Button, UiNode, UiOnClickBundle)>,
        text_node: Box<dyn Fn(String, TextStyle) -> TextBundle>,
    }

    impl UiButton {
        pub fn with_text(mut self, text: impl Into<String>) -> Self {
            self.text = text.into();
            self
        }

        pub fn with_on_click(mut self, on_click: UiOnClick) -> Self {
            self.on_click = on_click;
            self
        }

        pub fn with_game_color(mut self, game_color: GameColor, ui_builder: &UiBuilder) -> Self {
            self.ui_style.color = ui_builder.game_assets.colors.get(game_color);
            self.ui_style.border_color =
                Some(ui_builder.game_assets.colors.get_content(game_color));
            self.text_style.color = ui_builder.game_assets.colors.get_content(game_color);
            self
        }
    }

    impl UiComponent for UiButton {
        fn new<'a>(builder: &mut UiBuilder, width: Val, height: Val, z: f32) -> Self {
            Self {
                text: Default::default(),
                style: Style {
                    width,
                    height,
                    padding: UiRect::all(Val::Px(12.0 * builder.window.single().scale_factor())),
                    margin: UiRect::all(Val::Px(12.0 * builder.window.single().scale_factor())),
                    min_width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                ui_style: UiStyle {
                    color: builder.game_assets.colors.get(GameColor::Secondary),
                    border_color: Some(
                        builder.game_assets.colors.get_content(GameColor::Secondary),
                    ),
                    ..Default::default()
                },
                text_style: builder.text_styles.get(
                    FontType::Regular,
                    FontSize::Medium,
                    builder.game_assets.colors.get_content(GameColor::Secondary),
                ),
                on_click: UiOnClick::default(),
                button: Box::new(move |style, ui_style, ui_on_click| {
                    (
                        NodeBundle {
                            style,
                            ..Default::default()
                        },
                        Button,
                        UiNode::container(ui_style, z),
                        UiOnClickBundle {
                            ui_on_click,
                            ..Default::default()
                        },
                    )
                }),
                text_node: Box::new(|text, style| TextBundle {
                    text: Text::from_section(text, style),
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            }
        }

        fn spawn<'a>(&'a self, parent: &'a mut ChildBuilder) -> EntityCommands {
            let mut spawn = parent.spawn((self.button)(
                self.style.clone(),
                self.ui_style,
                self.on_click,
            ));
            spawn.with_children(|parent| {
                parent.spawn((self.text_node)(self.text.clone(), self.text_style.clone()));
            });
            spawn
        }
    }

    pub struct UiText {
        pub text: String,
        pub style: Style,
        pub text_style: TextStyle,
        text_node: Box<dyn Fn(String, Style, TextStyle) -> TextBundle>,
    }

    impl UiText {
        pub fn with_text(mut self, text: impl Into<String>) -> Self {
            self.text = text.into();
            self
        }

        pub fn with_text_style(mut self, text_style: TextStyle) -> Self {
            self.text_style = text_style;
            self
        }
    }

    impl UiComponent for UiText {
        fn spawn<'a>(&'a self, parent: &'a mut ChildBuilder) -> EntityCommands {
            parent.spawn((self.text_node)(
                self.text.clone(),
                self.style.clone(),
                self.text_style.clone(),
            ))
        }

        fn new(builder: &mut UiBuilder, width: Val, height: Val, _z: f32) -> Self {
            Self {
                text: Default::default(),
                style: Style {
                    width,
                    height,
                    padding: UiRect::all(Val::Px(20.0 * builder.window.single().scale_factor())),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                text_style: builder.text_styles.get(
                    FontType::default(),
                    FontSize::default(),
                    Color::default(),
                ),
                text_node: Box::new(|text, style, text_style| TextBundle {
                    text: Text::from_section(text, text_style),
                    style,
                    ..Default::default()
                }),
            }
        }
    }
}

pub mod styles {
    use super::*;
    use std::default::Default;

    pub fn container(width: Val, height: Val) -> Style {
        Style {
            width,
            height,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        }
    }

    pub fn container_node(width: Val, height: Val) -> NodeBundle {
        NodeBundle {
            style: container(width, height),
            ..Default::default()
        }
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UiState>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    handle_on_click,
                    render_ui,
                    override_interactions.after(ui_focus_system),
                ),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..Default::default()
        },
        ..Default::default()
    });
}

fn override_interactions(
    mut interactions: Query<(
        &mut Interaction,
        &mut UiOnClick,
        &UiNode,
        &Node,
        &GlobalTransform,
    )>,
    user_input: Res<Inputs<UserInput>>,
    user_input_positions: Res<UserInputPosition>,
) {
    let Some((pos, user_input)) = user_input
        .iter_pressed()
        .next()
        .and_then(|u| user_input_positions.get(u.0).map(|p| (p, **u)))
    else {
        return;
    };
    for (mut interaction, mut on_click, ui_node, node, trans) in interactions
        .iter_mut()
        .filter(|i| *i.0 != Interaction::None)
    {
        let rect = node.logical_rect(trans);
        if !crate::utils::contains_point(rect, ui_node.corner_radius, pos) {
            *interaction = Interaction::None;
            on_click.handle = false;
        }

        on_click.command.context.event_data = Some(UiPointerEventData {
            pos,
            user_input: UserInput(user_input),
        });
    }
}

fn handle_on_click(
    mut commands: Commands,
    mut query: Query<(Entity, &mut UiOnClick, &Interaction), Changed<Interaction>>,
    mut presses: Local<HashSet<Entity>>,
) {
    for (entity, mut ui_on_click, interaction) in query.iter_mut() {
        if interaction == &Interaction::Pressed {
            if ui_on_click.eager_handle {
                commands.add(ui_on_click.command);
                continue;
            }
            presses.insert(entity);
        }
        if interaction != &Interaction::Pressed && presses.contains(&entity) {
            presses.remove(&entity);
            if !ui_on_click.handle {
                ui_on_click.handle = true;
                continue;
            }
            ui_on_click.command.context.entity = entity;
            commands.add(ui_on_click.command);
        }
    }
}

#[derive(Component)]
pub struct UiNode {
    pub paint: Box<dyn Fn(&mut ShapePainter, Vec3, &Interaction, u64) + Send + Sync>,
    pub corner_radius: f32,
    pub z: f32,
}

impl Default for UiNode {
    fn default() -> Self {
        Self {
            paint: Box::new(|_, _, _, _| {}),
            corner_radius: 0.0,
            z: 0.0,
        }
    }
}

#[derive(Component, Deref, DerefMut, Clone, Copy, Default, Reflect)]
pub struct UiState(pub u64);

#[derive(Debug, Clone, Copy)]
pub struct UiStyle {
    pub color: Color,
    pub border_width: f32,
    pub border_radius: f32,
    pub border_color: Option<Color>,
}

impl UiStyle {
    pub fn empty() -> Self {
        Self {
            color: Color::BLACK.with_a(0.0),
            border_width: 0.0,
            border_radius: 0.0,
            border_color: None,
        }
    }
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            color: Default::default(),
            border_width: 8.0,
            border_radius: 16.0,
            border_color: Default::default(),
        }
    }
}

impl UiNode {
    pub fn container(style: UiStyle, z: f32) -> Self {
        UiNode {
            paint: Box::new(move |painter, size, interaction, _state| {
                let color_mult = match interaction {
                    Interaction::Hovered => 1.35,
                    Interaction::Pressed => 0.7,
                    _ => 1.0,
                };
                painter.alpha_mode = AlphaMode::Blend;
                painter.color = style.color * color_mult;
                let z = size.z;
                let size = size.xy();
                if style.border_width < 1.0 {
                    painter.transform.translation.z = z;
                    painter.corner_radii = Vec4::splat(style.border_radius);
                    painter.rect(size);
                    return;
                }
                let inner_size = size - Vec2::splat(1.0);
                let ratio = inner_size / size;
                painter.transform.translation.z = z;
                painter.corner_radii =
                    Vec4::splat(style.border_radius) * ratio.extend(ratio.x).extend(ratio.y);
                painter.rect(size * ratio);
                painter.transform.translation.z = z * 1.5;
                painter.color = style.border_color.unwrap_or(Color::BLACK) * color_mult;
                painter.corner_radii = Vec4::splat(style.border_radius);
                painter.hollow = true;
                painter.thickness_type = ThicknessType::Pixels;
                painter.thickness = style.border_width;
                painter.rect(size);
            }),
            corner_radius: style.border_radius,
            z,
        }
    }
}

pub fn render_ui(
    nodes: Query<(
        &UiNode,
        &Node,
        &Style,
        &GlobalTransform,
        Option<&Interaction>,
        Option<&UiState>,
        Option<&Parent>,
    )>,
    ui_states: Query<&UiState>,
    window: Query<&Window>,
    mut painter: ShapePainter,
) {
    let window = window.single();
    for (ui_node, node, style, gt, int, state, parent) in nodes.iter() {
        if style.display == Display::None {
            continue;
        }
        let rect = &node.logical_rect(gt);
        let mut pos_2d = rect.center();
        pos_2d.x -= window.width() / 2.0;
        pos_2d.y = window.height() / 2.0 - pos_2d.y;
        let config = ShapeConfig {
            transform: Transform::from_translation(pos_2d.extend(0.0)),
            ..ShapeConfig::default_2d()
        };
        painter.set_config(config);
        (ui_node.paint)(
            &mut painter,
            rect.size().extend(ui_node.z),
            int.unwrap_or(&Interaction::None),
            state.map(|s| s.0).unwrap_or_else(|| {
                parent
                    .and_then(|p| ui_states.get(**p).ok().map(|s| s.0))
                    .unwrap_or_default()
            }),
        );
    }
}
