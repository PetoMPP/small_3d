use super::plugins::game_scene_plugin::{GameScenePlugin, SetGameScene, Tile};
use super::plugins::movement_plugin::MovementPlugin;
use super::plugins::player_plugin::PlayerPlugin;
use crate::game::plugins::game_scene_plugin::GameSceneData;
use crate::AppState;
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PlayerPlugin, GameScenePlugin, MovementPlugin))
            .add_systems(OnEnter(AppState::InGame), start_game);
    }
}

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct GameLight;

fn start_game(mut commands: Commands, mut set_scene: EventWriter<SetGameScene>) {
    // light
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
        .insert(GameCamera);

    // game scene
    set_scene.send(SetGameScene(GameSceneData {
        tiles: vec![vec![Tile {
            color: Color::WHITE,
            position: IVec2::new(0, 0),
        }]],
        start_pos: IVec2::new(0, 0),
        start_dir: 0.0,
        name: "Test".to_string(),
    }));
}
