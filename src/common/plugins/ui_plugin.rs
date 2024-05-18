use super::user_input_plugin::{UserInput, UserInputPosition};
use crate::resources::inputs::Inputs;
use bevy::{ecs::system::Command, prelude::*, ui::ui_focus_system, utils::HashSet};
use bevy_vector_shapes::{
    painter::{ShapeConfig, ShapePainter},
    shapes::{RectPainter, ThicknessType},
};

#[derive(Component)]
pub struct UiOnClick {
    pub command: UiCommand,
    // if true, the click will be handled on press
    pub eager_handle: bool,
    // if false, the click will not be handled once
    handle: bool,
}

impl UiOnClick {
    pub fn new(on_click: fn(&mut World, Option<(&UserInput, Vec2)>) -> ()) -> Self {
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
pub struct UiCommand {
    command: fn(&mut World, Option<(&UserInput, Vec2)>) -> (),
    user_input: Option<UserInput>,
    pos: Option<Vec2>,
}

impl UiCommand {
    pub fn new(command: fn(&mut World, Option<(&UserInput, Vec2)>) -> ()) -> Self {
        Self {
            command,
            ..Default::default()
        }
    }
}

impl Default for UiCommand {
    fn default() -> Self {
        Self {
            command: |_, _| {},
            user_input: None,
            pos: None,
        }
    }
}

impl Command for UiCommand {
    fn apply(self, world: &mut World) {
        (self.command)(
            world,
            self.user_input
                .as_ref()
                .and_then(|u| self.pos.map(|p| (u, p))),
        );
    }
}

#[derive(Bundle, Default)]
pub struct UiOnClickBundle {
    pub ui_on_click: UiOnClick,
    pub interaction: Interaction,
}

pub mod styles {
    use bevy::ui::{node_bundles::NodeBundle, AlignItems, JustifyContent, Style, Val};
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
        app.register_type::<UiState>().add_systems(
            Update,
            (
                handle_on_click,
                render_bg,
                override_interactions.after(ui_focus_system),
            ),
        );
    }
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
        .or(user_input_positions.get(0).map(|p| (p, 0)))
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
        on_click.command.user_input = Some(UserInput(user_input));
        on_click.command.pos = Some(pos);
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
            commands.add(ui_on_click.command);
        }
    }
}

#[derive(Component)]
pub struct UiNode {
    pub paint: Box<dyn Fn(&mut ShapePainter, Vec2, &Interaction, u64) + Send + Sync>,
    pub corner_radius: f32,
}

impl Default for UiNode {
    fn default() -> Self {
        Self {
            paint: Box::new(|_, _, _, _| {}),
            corner_radius: 0.0,
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
    pub fn container(style: UiStyle) -> Self {
        UiNode {
            paint: Box::new(move |painter, size, interaction, _state| {
                let color_mult = match interaction {
                    Interaction::Hovered => 1.35,
                    Interaction::Pressed => 0.7,
                    _ => 1.0,
                };
                painter.alpha_mode = AlphaMode::Blend;
                painter.color = style.color * color_mult;
                if style.border_width < 1.0 {
                    painter.corner_radii = Vec4::splat(style.border_radius);
                    painter.rect(size);
                    return;
                }
                let inner_size = size - Vec2::splat(1.0);
                let ratio = inner_size / size;
                painter.corner_radii =
                    Vec4::splat(style.border_radius) * ratio.extend(ratio.x).extend(ratio.y);
                painter.rect(size * ratio);
                painter.color = style.border_color.unwrap_or(Color::BLACK) * color_mult;
                painter.corner_radii = Vec4::splat(style.border_radius);
                painter.hollow = true;
                painter.thickness_type = ThicknessType::Pixels;
                painter.thickness = style.border_width;
                painter.rect(size);
            }),
            corner_radius: style.border_radius,
        }
    }
}

fn render_bg(
    nodes: Query<(
        &UiNode,
        &Node,
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
    let mut nodes = nodes.iter().collect::<Vec<_>>();
    nodes.reverse();
    for (ui_node, node, gt, int, state, parent) in nodes {
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
            rect.size(),
            int.unwrap_or(&Interaction::None),
            state.map(|s| s.0).unwrap_or_else(|| {
                parent
                    .and_then(|p| ui_states.get(**p).ok().map(|s| s.0))
                    .unwrap_or_default()
            }),
        );
    }
}
