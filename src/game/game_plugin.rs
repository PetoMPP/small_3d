use super::components::{GameCamera, GameLight, GameUiCamera};
use super::plugins::aiming_plugin::AimingPlugin;
use super::plugins::custom_tweening_plugin::CustomTweeningPlugin;
use super::plugins::game_scene_plugin::{GameScenePlugin, SetGameLevel};
use super::plugins::player_plugin::PlayerPlugin;
use crate::resources::game_assets::GameLevel;
use crate::AppState;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tweening::TweeningPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(60.0))
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default().in_fixed_schedule())
            // .add_plugins(RapierDebugRenderPlugin::default())
            .add_plugins(TweeningPlugin)
            .add_plugins((
                AimingPlugin,
                PlayerPlugin,
                GameScenePlugin,
                CustomTweeningPlugin,
            ))
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
    commands
        .spawn(Camera2dBundle {
            camera: Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 100.0),
            ..Default::default()
        })
        .insert(GameUiCamera);

    // game scene
    set_scene.send(SetGameLevel(GameLevel::Demo));
}
