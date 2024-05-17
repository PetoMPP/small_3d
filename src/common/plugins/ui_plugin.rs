use bevy::{prelude::*, utils::HashSet};
use bevy_vector_shapes::{
    painter::{ShapeConfig, ShapePainter},
    shapes::{RectPainter, ThicknessType},
};

#[derive(Component)]
pub struct UiOnClick(pub fn(&mut World) -> ());

#[derive(Bundle)]
pub struct UiOnClickBundle {
    pub ui_on_click: UiOnClick,
    pub interaction: Interaction,
}

impl Default for UiOnClickBundle {
    fn default() -> Self {
        UiOnClickBundle {
            ui_on_click: UiOnClick(|_| {}),
            interaction: Interaction::default(),
        }
    }
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
        app
        .register_type::<UiState>()
        .add_systems(Update, (handle_on_click, render_bg));
    }
}

fn handle_on_click(
    mut commands: Commands,
    query: Query<(Entity, &UiOnClick, &Interaction), Changed<Interaction>>,
    mut presses: Local<HashSet<Entity>>,
) {
    for (entity, ui_on_click, interaction) in query.iter() {
        if interaction == &Interaction::Pressed {
            presses.insert(entity);
        }
        if interaction != &Interaction::Pressed && presses.contains(&entity) {
            presses.remove(&entity);
            commands.add(ui_on_click.0);
        }
    }
}

#[derive(Component)]
pub struct UiNode {
    pub paint: Box<dyn Fn(&mut ShapePainter, Vec2, &Interaction, u64) + Send + Sync>,
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
