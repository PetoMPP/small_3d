use crate::{game::plugins::player_plugin::spawn_player, AppState};
use bevy::{input::keyboard::KeyboardInput, prelude::*};

pub struct GameScenePlugin;

#[derive(Resource, Default, Clone)]
pub struct GameScene(pub Option<GameSceneData>);

#[derive(Resource, Default, Clone)]
pub struct GameSceneData {
    pub name: String,
    pub tiles: Vec<Vec<Tile>>,
    pub start_pos: IVec2,
    pub start_dir: f32,
}

#[derive(Event)]
pub struct SetGameScene(pub GameSceneData);

#[derive(Clone, Copy, PartialEq, Default)]
pub struct Tile {
    pub color: Color,
    pub position: IVec2,
}

#[derive(Component)]
pub struct SceneEntity;

impl Plugin for GameScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameScene>()
            .add_event::<SetGameScene>()
            .add_systems(
                Update,
                (set_game_scene, reload_scene).run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                spawn_game_scene
                    .run_if(in_state(AppState::InGame))
                    .run_if(resource_changed::<GameScene>),
            );
    }
}

fn set_game_scene(
    mut game_scene: ResMut<GameScene>,
    mut set_game_scene: EventReader<SetGameScene>,
) {
    for set_game_scene in set_game_scene.read() {
        game_scene.0 = Some(set_game_scene.0.clone());
    }
}

fn reload_scene(
    mut set_game_scene: EventWriter<SetGameScene>,
    game_scene: Res<GameScene>,
    mut key_input: EventReader<KeyboardInput>,
) {
    let Some(key_input) = key_input.read().next() else {
        return;
    };

    if key_input.key_code == KeyCode::KeyR {
        if let Some(game_scene) = &game_scene.0 {
            set_game_scene.send(SetGameScene(game_scene.clone()));
        }
    }
}

fn spawn_game_scene(
    mut commands: Commands,
    game_scene: Res<GameScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    entities: Query<(Entity, &SceneEntity)>,
) {
    println!("Clearing game scene");
    for (entity, _) in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    println!("Spawning game scene");
    if let Some(game_scene) = &game_scene.0 {
        for (y, row) in game_scene.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Rectangle::new(1.0, 1.0)),
                        material: materials.add(tile.color),
                        transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 0.0)),
                        ..Default::default()
                    })
                    .insert(SceneEntity);
            }
        }
        println!("Spawning player");
        spawn_player(
            &mut commands,
            meshes,
            materials,
            game_scene.start_pos,
            game_scene.start_dir,
            Some(SceneEntity),
        );
    }
}
