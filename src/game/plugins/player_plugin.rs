use super::user_input_plugin::{Pressed, UserInput, UserInputPosition};
use crate::{
    game::components::{GameCamera, GameEntity, GameLight, Player},
    log, AppState,
};
use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_mod_picking::prelude::*;
use bevy_picking_rapier::bevy_rapier3d::prelude::*;
use std::f32::{consts::PI, EPSILON};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                move_player,
                move_camera_and_light,
                zoom_camera,
                rotate_camera,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

pub fn spawn_player(commands: &mut Commands, asset_server: &AssetServer, pos: Vec3) {
    const R: f32 = 0.2;

    let scene = asset_server.load("models/player.glb#Scene0");
    commands
        .spawn(SceneBundle {
            scene,
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
}

#[derive(Default, Clone, Copy)]
struct DragInfo {
    start: Vec3,
    normal: Vec3,
    end: Vec3,
}

fn move_player(
    mut gizmos: Gizmos,
    mut player: Query<(&Transform, Entity, &mut Player, &mut ExternalImpulse)>,
    camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    user_input_position: Res<UserInputPosition>,
    user_input: Res<ButtonInput<UserInput>>,
    mut presses: EventReader<Pointer<Pressed>>,
    mut current: Local<Option<DragInfo>>,
) {
    let (camera, camera_transform) = camera.single();

    let Some((transform, player_entity, mut player, mut impulse)) = player.iter_mut().next() else {
        return;
    };

    for press in presses.read() {
        if let (PointerButton::Primary, Some(position), Some(normal)) =
            (press.button, press.hit.position, press.hit.normal)
        {
            log!("recieved!");
            if press.target == player_entity && player.shots > 0 {
                log!("player recieved!");
                player.shots -= 1;
                *current = Some(DragInfo {
                    start: position,
                    normal,
                    end: position,
                });
            }
        }
    }

    if let Some(cursor_position) = **user_input_position {
        let Some(drag_info) = &mut *current else {
            log!("no drag_info!");
            return;
        };
        log!("no cursor!");
        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            log!("no ray!");
            return;
        };

        let Some(distance) = ray.intersect_plane(
            transform
                .translation
                .lerp(camera_transform.translation(), 0.5),
            Plane3d::new(drag_info.normal),
        ) else {
            log!("no distance!");
            return;
        };

        let point = ray.get_point(distance);
        drag_info.end = point;
    };

    if user_input.just_released(UserInput(PointerButton::Primary)) {
        if let Some(drag_info) = *current {
            let push = (drag_info.start - drag_info.end) * 100.0;
            log!("Pushing with {:?}", push);
            *impulse = ExternalImpulse::at_point(push, drag_info.start, transform.translation);
            *current = None;
        }
    }

    if user_input.pressed(UserInput(PointerButton::Primary)) {
        if impulse.is_changed() {
            impulse.bypass_change_detection();
            impulse.reset();
        }
        if let Some(drag_info) = *current {
            gizmos.line(drag_info.start, drag_info.end, Color::WHITE);
        }
    }
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
    let camera_dist = camera_comp.distance;
    let mut light = light.single_mut();

    let ang_x = camera_comp.offset.x;
    let ang_z = camera_comp.offset.y;

    let rotation = Quat::from_rotation_z(ang_x)
        * Quat::from_rotation_x(ang_z.clamp(0.0 + EPSILON, PI - EPSILON));

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
) {
    let mut camera = camera.single_mut();

    for scroll in scroll.read() {
        camera.1.distance -= scroll.y.clamp(-1.0, 1.0) * 0.5;
        camera.1.distance = camera.1.distance.clamp(1.0, 25.0);
    }
}

fn rotate_camera(
    mut camera: Query<(&Transform, &mut GameCamera)>,
    windows: Query<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut last_point: Local<Vec2>,
) {
    let mut camera = camera.single_mut();

    let single = windows.single();
    let Some(cursor_position) = single.cursor_position() else {
        return;
    };

    if mouse_input.just_pressed(MouseButton::Middle) {
        *last_point = cursor_position;
        return;
    }

    if mouse_input.pressed(MouseButton::Middle) {
        let delta = cursor_position - *last_point;
        *last_point = cursor_position;
        camera.1.offset.x += delta.x * 0.02;
        camera.1.offset.y -= delta.y * 0.02;
    }
}
