use crate::{
    game::{plugin::GameCamera, plugins::player_plugin::spawn_player},
    AppState,
};
use bevy::{input::keyboard::KeyboardInput, prelude::*};
use bevy_picking_rapier::bevy_rapier3d::prelude::*;

use super::player_plugin::Player;
pub struct GameScenePlugin;

#[derive(Resource, Default, Clone)]
pub struct GameScene(pub Option<GameSceneData>);

#[derive(Resource, Default, Clone)]
pub struct GameSceneData(pub GameLevel);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GameLevel {
    #[default]
    Level1,
    Level2,
}

#[derive(Event)]
pub struct SetGameScene(pub GameSceneData);

#[derive(Component, Clone, Copy)]
pub struct SceneEntity;

#[derive(Component, Deref, DerefMut)]
pub struct Ground(pub Timer);

impl Default for Ground {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.75, TimerMode::Once);
        timer.pause();
        Self(timer)
    }
}

impl Plugin for GameScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameScene>()
            .add_event::<SetGameScene>()
            .add_systems(
                Update,
                (set_game_scene, reload_scene, reload_on_ground_collision)
                    .run_if(in_state(AppState::InGame)),
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

    if key_input.state.is_pressed() && key_input.key_code == KeyCode::KeyR {
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
    mut camera: Query<&mut GameCamera>,
) {
    println!("Clearing game scene");
    for (entity, _) in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    println!("Spawning game scene");
    if let Some(game_scene) = &game_scene.0 {
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 1.0, 0.1)),
                material: materials.add(Color::BLUE),
                ..Default::default()
            })
            .insert(SceneEntity)
            .insert((
                Collider::cuboid(0.5, 0.5, 0.05),
                Friction::coefficient(1.0),
                ColliderMassProperties::Mass(1000.0),
            ));
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Circle::new(500.0)),
                material: materials.add(Color::CRIMSON),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
                ..Default::default()
            })
            .insert((SceneEntity, Ground::default()))
            .insert((
                Collider::cuboid(500.0, 500.0, 0.0),
                Friction::coefficient(1000.0),
                ColliderMassProperties::Mass(100000.0),
                ActiveEvents::COLLISION_EVENTS,
            ));
        println!("Spawning player");
        spawn_player(&mut commands, meshes, materials, Vec3::Z, Some(SceneEntity));
    }

    if let Some(mut camera) = camera.iter_mut().next() {
        *camera = GameCamera::default();
    }
}

fn reload_on_ground_collision(
    mut set_game_scene: EventWriter<SetGameScene>,
    mut ground_collisions: EventReader<CollisionEvent>,
    game_scene: Res<GameScene>,
    player: Query<(Entity, Ref<Sleeping>), With<Player>>,
    mut ground: Query<(Entity, &mut Ground), Without<Player>>,
    time: Res<Time>,
    mut initialized: Local<bool>,
) {
    let Some((player_entity, player_sleep)) = player.iter().next() else {
        return;
    };
    let Some((ground_entity, mut ground)) = ground.iter_mut().next() else {
        return;
    };

    // Progress the ground timer if unpaused
    ground.tick(time.delta());
    if ground.just_finished() {
        if let Some(game_scene) = &game_scene.0 {
            set_game_scene.send(SetGameScene(game_scene.clone()));
        }
        return;
    }

    // After first ground collision, wait for player to sleep before resetting
    if *initialized {
        if player_sleep.is_changed() {
            if player_sleep.sleeping {
                *initialized = false;
                ground.unpause();
                ground.reset();
            }
        }
    }

    let entities = vec![player_entity, ground_entity];
    for collision in ground_collisions.read() {
        match collision {
            // Set initialized to true if player and ground collide
            CollisionEvent::Started(e1, e2, _) => {
                if entities.contains(&e1) && entities.contains(&e2) {
                    *initialized = true;
                }
            }
            // Pause the ground timer if player and ground stop colliding
            CollisionEvent::Stopped(e1, e2, _) => {
                if entities.contains(&e1) && entities.contains(&e2) {
                    ground.pause();
                }
            }
        }
    }
}
