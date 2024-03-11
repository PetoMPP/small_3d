use std::f32::consts::PI;

use super::movement_plugin::{Velocity, MOVE_TICK};
use crate::{
    game::plugin::{GameCamera, GameLight},
    AppState,
};
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (move_player, move_camera_and_light).run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Component, Default)]
pub struct Player;

pub fn spawn_player(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    pos: IVec2,
    dir: f32,
    bundle: Option<impl Bundle>,
) {
    const X: f32 = 0.3;
    const Y: f32 = 0.2;
    const Z: f32 = 0.2;

    let asset = Cuboid::new(X, Y, Z);
    let mut e = commands.spawn(PbrBundle {
        mesh: meshes.add(asset),
        material: materials.add(Color::RED),
        transform: Transform::from_translation(Vec3::new(pos.x as f32, pos.y as f32, Z / 2.0))
            .with_rotation(Quat::from_rotation_z(dir)),
        ..Default::default()
    });
    let e = e.insert((Player, Velocity::default()));
    if let Some(bundle) = bundle {
        e.insert(bundle);
    }
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player: Query<&mut Velocity, With<Player>>,
) {
    let Some(mut player) = player.iter_mut().next() else {
        return;
    };

    // rotation
    if keyboard_input.pressed(KeyCode::KeyA) {
        player.angular = Quat::from_rotation_z(PI / 2.0 * MOVE_TICK);
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        player.angular = Quat::from_rotation_z(-PI / 2.0 * MOVE_TICK);
    } else if player.angular != Quat::IDENTITY {
        player.angular = Quat::IDENTITY;
    }

    // movement
    if keyboard_input.pressed(KeyCode::KeyW) {
        // move toward current dir
        player.linear = 1.0;
    } else if keyboard_input.pressed(KeyCode::KeyS) {
        // move backward
        player.linear = -1.0;
    } else if player.linear != 0.0 {
        player.linear = 0.0;
    }
}

fn move_camera_and_light(
    moved_player: Query<
        &Transform,
        (
            With<Player>,
            (Changed<Transform>, Without<GameLight>, Without<GameCamera>),
        ),
    >,
    mut camera: Query<&mut Transform, (With<GameCamera>, Without<GameLight>)>,
    mut light: Query<&mut Transform, (With<GameLight>, Without<GameCamera>)>,
) {
    const CAMERA_DISTANCE: f32 = 3.0;
    const LIGHT_DISTANCE: f32 = 1.8;

    let Some(moved_player) = moved_player.iter().next() else {
        return;
    };
    println!("Player moved");

    let (px, py) = (moved_player.translation.x, moved_player.translation.y);
    let rotation = moved_player.rotation;

    let c_trans = Vec3 {
        z: CAMERA_DISTANCE,
        ..moved_player.translation
    } + rotation * -Vec3::X * CAMERA_DISTANCE;

    // move camera and light
    let mut camera = camera.single_mut();
    let mut light = light.single_mut();
    *camera = Transform {
        translation: c_trans,
        ..Default::default()
    }
    .looking_at(Vec3::new(px, py, 0.5), Vec3::Z);
    *light = Transform::from_xyz(px, py, LIGHT_DISTANCE);
}
