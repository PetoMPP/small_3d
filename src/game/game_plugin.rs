use super::components::{GameCamera, GameLight};
use super::plugins::aiming_plugin::AimingPlugin;
use super::plugins::game_scene_plugin::{GameScenePlugin, SetGameLevel};
use super::plugins::player_plugin::PlayerPlugin;
use crate::resources::game_assets::GameLevel;
use crate::AppState;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_picking_rapier::bevy_rapier3d::prelude::*;
use bevy_picking_rapier::RapierBackend;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(60.0))
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default().in_fixed_schedule())
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_plugins(DefaultPickingPlugins)
            .add_plugins(RapierBackend)
            .add_plugins((AimingPlugin, PlayerPlugin, GameScenePlugin))
            .add_systems(OnEnter(AppState::InGame), start_game);
    }
}

fn start_game(
    mut commands: Commands,
    mut set_scene: EventWriter<SetGameLevel>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    // collision config
    rapier_config.gravity = Vec3::Z * -9.81;
    rapier_config.timestep_mode = TimestepMode::Fixed {
        dt: 1.0 / 60.0,
        substeps: 4,
    };
    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                shadows_enabled: true,
                range: 20.0,
                intensity: 66_000.0,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(GameLight);

    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(GameCamera::default());

    // game scene
    set_scene.send(SetGameLevel(GameLevel::Level1));
}
