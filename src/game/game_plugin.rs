use super::components::{GameCamera, GameUiCamera};
use super::plugins::aiming_plugin::AimingPlugin;
use super::plugins::custom_tweening_plugin::CustomTweeningPlugin;
use super::plugins::game_scene_plugin::{GameScenePlugin, SetGameLevel};
use super::plugins::game_ui_plugin::{spawn_ui, GameUiPlugin};
use super::plugins::player_plugin::PlayerPlugin;
use crate::resources::game_assets::GameLevel;
use crate::AppState;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tweening::TweeningPlugin;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, States, Deref, DerefMut)]
pub struct GameRunningState(pub bool);

impl Default for GameRunningState {
    fn default() -> Self {
        Self(true)
    }
}

pub struct GamePlugin;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct PhysicsSchedule;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(PhysicsSchedule)
            .init_state::<GameRunningState>()
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .add_systems(
                FixedUpdate,
                run_physics_schedule.run_if(in_state(GameRunningState(true))),
            )
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default().in_schedule(PhysicsSchedule))
            // .add_plugins(RapierDebugRenderPlugin::default())
            .add_plugins(TweeningPlugin)
            .add_plugins((
                AimingPlugin,
                PlayerPlugin,
                GameScenePlugin,
                CustomTweeningPlugin,
                GameUiPlugin,
            ))
            .add_systems(OnEnter(AppState::InGame), start_game);
    }
}

pub fn run_physics_schedule(world: &mut World) {
    world.schedule_scope(PhysicsSchedule, |world, schedule| {
        schedule.run(world);
    });
}

fn start_game(
    mut commands: Commands,
    mut set_scene: EventWriter<SetGameLevel>,
    mut rapier_config: ResMut<RapierConfiguration>,
    window: Query<&Window>,
) {
    // collision config
    rapier_config.gravity = Vec3::Z * -9.81;
    rapier_config.timestep_mode = TimestepMode::Fixed {
        dt: 1.0 / 60.0,
        substeps: 4,
    };

    // ui
    spawn_ui(&mut commands, window.single());

    // light
    commands.spawn(SpotLightBundle {
        spot_light: SpotLight {
            color: Color::rgb(1.0, 0.95, 0.9),
            intensity: 180_000_000.0,
            shadows_enabled: true,
            range: 200.0,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 30.0, 40.0)
            .looking_at(Vec3::new(10.0, 0.0, 0.0), Vec3::Z),
        ..default()
    });

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
