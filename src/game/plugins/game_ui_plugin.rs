use bevy::{ecs::system::Command, prelude::*, utils::HashMap};
use bevy_vector_shapes::{
    painter::{BuildShapeChildren, ShapeConfig},
    shapes::RectangleSpawner,
};

use crate::{
    common::plugins::user_input_plugin::{UserInput, UserInputPosition},
    game::{components::GameUiCamera, game_plugin::GameRunningState},
    log,
    resources::inputs::Inputs,
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_ui_clicks);
    }
}

#[derive(Component)]
pub struct UiElement {
    pub size: Vec2,
}

// This is a component with command that will be called when the entity is clicked and released
#[derive(Component, Deref, DerefMut)]
pub struct UiClickable(UiCommand);

trait UiCommandExt {
    fn do_ui_command(self, ui_command: UiCommand);
}

impl UiCommandExt for &mut Commands<'_, '_> {
    fn do_ui_command(self, ui_command: UiCommand) {
        self.add(ui_command);
    }
}

#[derive(Clone)]
pub struct UiCommand(fn(&mut World) -> ());

impl Command for UiCommand {
    fn apply(self, world: &mut World) {
        (self.0)(world);
    }
}

impl UiCommand {
    pub fn toggle_pause() -> Self {
        Self(|world| {
            let game_state = *world.resource::<State<GameRunningState>>().get();
            world
                .resource_mut::<NextState<GameRunningState>>()
                .set(GameRunningState(!*game_state));
        })
    }
}

fn handle_ui_clicks(
    mut commands: Commands,
    user_input: Res<Inputs<UserInput>>,
    user_input_position: Res<UserInputPosition>,
    camera: Query<(&Camera, &GlobalTransform), With<GameUiCamera>>,
    mut query: Query<(Entity, &mut UiClickable, &UiElement, &GlobalTransform)>,
    mut clicks: Local<HashMap<UserInput, Entity>>,
) {
    let Some((camera, camera_transform)) = camera.iter().next() else {
        return;
    };

    for input in user_input.iter_just_pressed() {
        let Some(pos) = user_input_position.get(**input) else {
            continue;
        };

        let Some(pos) = camera.viewport_to_world_2d(camera_transform, pos) else {
            continue;
        };

        for (ui_entity, _, element, transform) in query.iter() {
            let rect = Rect::from_center_size(transform.translation().xy(), element.size);
            if rect.contains(pos) {
                clicks.insert(*input, ui_entity);
            }
        }
    }

    for (input, entity) in clicks
        .clone()
        .into_iter()
        .filter(|(input, _)| user_input.just_released(*input))
    {
        clicks.remove(&input);
        if let Ok(on_click) = query.get_mut(entity).map(|q| (*q.1).clone()) {
            commands.do_ui_command(on_click);
        }
    }
}

pub fn spawn_ui(commands: &mut Commands, window: &Window) {
    spawn_pause_button(commands, window);
}

fn spawn_pause_button(commands: &mut Commands, window: &Window) {
    // Spawn pause button
    let a = window.height().min(window.width()) / 5.0;
    let x = window.width() / 2.0 - a / 2.0 - 10.0;
    let y = window.height() / 2.0 - a / 2.0 - 10.0;
    let mut shapes_config = ShapeConfig::default_2d();
    shapes_config.color = Color::YELLOW_GREEN;
    shapes_config.corner_radii = Vec4::splat(a / 6.0);
    log!("x: {}, y: {}", x, y);

    commands
        .spawn((
            TransformBundle {
                local: Transform::from_xyz(x, y, 0.0),
                ..Default::default()
            },
            UiElement {
                size: Vec2::splat(a),
            },
            UiClickable(UiCommand::toggle_pause()),
        ))
        .with_shape_children(&shapes_config, |builder| {
            builder.rect(Vec2::splat(a));
        });
}
