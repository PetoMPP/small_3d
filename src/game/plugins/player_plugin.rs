use super::aiming_plugin::DragInfo;
use crate::common::plugins::user_input_plugin::{UserInput, UserInputPosition};
use crate::game::plugins::aiming_plugin::spawn_arrow;
use crate::resources::Inputs;
use crate::{
    game::components::{GameCamera, GameEntity, GameLight, Player},
    AppState,
};
use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_picking_rapier::bevy_rapier3d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (move_camera_and_light, zoom_camera, rotate_camera).run_if(in_state(AppState::InGame)),
        );
    }
}

pub fn spawn_player(commands: &mut Commands, asset_server: &AssetServer, pos: Vec3) {
    const R: f32 = 0.2;

    let player_scene = asset_server.load("models/player.glb#Scene0");
    commands
        .spawn(SceneBundle {
            scene: player_scene,
            transform: Transform::from_translation(pos),
            ..Default::default()
        })
        .insert((
            Player::default(),
            Sleeping::default(),
            ExternalImpulse::default(),
            RigidBody::Dynamic,
            Collider::ball(R),
            Friction::coefficient(0.6),
            Restitution::new(0.3),
            Damping {
                linear_damping: 0.5,
                angular_damping: 0.5,
            },
            ColliderMassProperties::Mass(10.0),
            ActiveEvents::COLLISION_EVENTS,
            GameEntity,
        ));
    spawn_arrow(commands, asset_server, pos);
}

fn move_camera_and_light(
    moved_player: Query<Ref<Transform>, (With<Player>, Without<GameLight>, Without<GameCamera>)>,
    mut camera: Query<(&mut Transform, Ref<GameCamera>), Without<GameLight>>,
    mut light: Query<&mut Transform, (With<GameLight>, Without<GameCamera>)>,
) {
    const LIGHT_DISTANCE: f32 = 2.5;

    let Some(moved_player) = moved_player.iter().next() else {
        return;
    };

    let (mut camera_trans, camera_comp) = camera.single_mut();

    if !moved_player.is_changed() && !camera_comp.is_changed() {
        return;
    }

    // move camera and light
    let camera_dist = camera_comp.get_distance();
    let mut light = light.single_mut();

    let offset = camera_comp.get_offset();
    let rotation = Quat::from_rotation_z(offset.x) * Quat::from_rotation_x(offset.y);

    *camera_trans = Transform::from_translation(
        moved_player.translation + rotation.mul_vec3(Vec3::Z * camera_dist),
    )
    .with_rotation(rotation)
    .looking_at(moved_player.translation, Vec3::Z);

    *light =
        Transform::from_translation(moved_player.translation + Vec3::new(0.0, 0.0, LIGHT_DISTANCE));
}

fn zoom_camera(
    mut camera: Query<(&Transform, &mut GameCamera)>,
    mut scroll: EventReader<MouseWheel>,
    user_input: Res<Inputs<UserInput>>,
    user_input_position: Res<UserInputPosition>,
    mut current: Local<Option<((u64, u64), f32)>>,
) {
    let mut camera = camera.single_mut();

    for scroll in scroll.read() {
        camera.1.distance(-scroll.y.clamp(-1.0, 1.0) * 0.5);
    }

    let Some(((first, second), dist)) = current.or_else(|| {
        let mut iter = user_input.iter_pressed();
        let first = **iter.next()?;
        let second = **iter.next()?;
        let dist = user_input_position
            .get(first)?
            .distance_squared(user_input_position.get(second)?);
        Some(((first, second), dist))
    }) else {
        return;
    };

    if user_input.just_released(UserInput(first)) || user_input.just_released(UserInput(second)) {
        *current = None;
        return;
    }

    if user_input.pressed(UserInput(first)) && user_input.pressed(UserInput(second)) {
        let new_dist = user_input_position
            .get(first)
            .and_then(|first| {
                user_input_position
                    .get(second)
                    .map(|second| first.distance_squared(second))
            })
            .unwrap_or(dist);

        let delta = new_dist - dist;
        *current = Some(((first, second), new_dist));
        if delta.abs() > 4.0 {
            camera.1.distance(-delta.clamp(-1.0, 1.0) * 0.25);
        }
    }
}

fn rotate_camera(
    mut camera: Query<&mut GameCamera>,
    user_input: Res<Inputs<UserInput>>,
    user_input_position: Res<UserInputPosition>,
    drag_info: Res<DragInfo>,
    mut last_point: Local<Option<(u64, Vec2)>>,
) {
    let Some((id, last_pos)) = last_point
        .map(|l| (l.0, Some(l.1)))
        .or_else(|| user_input.iter_just_pressed().next().map(|u| (u.0, None)))
    else {
        return;
    };

    if user_input.just_released(UserInput(id)) || user_input.iter_pressed().count() > 1 {
        *last_point = None;
        return;
    }

    if drag_info.is_some() {
        *last_point = None;
        return;
    }

    let mut camera = camera.single_mut();

    let Some(cursor_position) = user_input_position.get(id) else {
        return;
    };

    if user_input.just_pressed(UserInput(id)) {
        *last_point = Some((id, cursor_position));
        return;
    }

    let Some(last_pos) = last_pos else {
        return;
    };

    if user_input.pressed(UserInput(id)) {
        let delta = cursor_position - last_pos;
        *last_point = Some((id, cursor_position));
        camera.offset(Vec2::new(delta.x, -delta.y) * 0.02);
    }
}
