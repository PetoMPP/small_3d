use super::{aiming_plugin::DragInfo, player_plugin::PLAYER_RADIUS};
use crate::{
    game::{
        components::{GameCamera, GameEntity, Player},
        plugins::player_plugin::spawn_player,
    },
    resources::game_assets::{GameAssets, GameLevel, GameScene},
    AppState,
};
use bevy::{input::keyboard::KeyboardInput, prelude::*};
use bevy_picking_rapier::bevy_rapier3d::prelude::*;

pub struct GameScenePlugin;

#[derive(Resource, Default, Clone, Deref, DerefMut)]
pub struct CurrentLevel(pub Option<GameLevel>);

#[derive(Event, Deref, DerefMut)]
pub struct SetGameLevel(pub GameLevel);

impl Plugin for GameScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentLevel>()
            .add_event::<SetGameLevel>()
            .add_systems(
                Update,
                (
                    set_game_scene,
                    initialize_game_scene,
                    reload_scene,
                    reload_on_bounds_collision,
                    reload_on_pass_through_goal,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (reset_state, spawn_game_scene)
                    .run_if(in_state(AppState::InGame))
                    .run_if(resource_changed::<CurrentLevel>),
            );
    }
}

fn set_game_scene(
    mut game_scene: ResMut<CurrentLevel>,
    mut set_game_scene: EventReader<SetGameLevel>,
) {
    for set_game_scene in set_game_scene.read() {
        **game_scene = Some(**set_game_scene);
    }
}

fn reload_scene(
    mut set_game_scene: EventWriter<SetGameLevel>,
    game_scene: Res<CurrentLevel>,
    mut key_input: EventReader<KeyboardInput>,
) {
    let Some(key_input) = key_input.read().next() else {
        return;
    };

    if key_input.state.is_pressed() && key_input.key_code == KeyCode::KeyR {
        if let Some(game_scene) = **game_scene {
            set_game_scene.send(SetGameLevel(game_scene));
        }
    }
}

fn reset_state(
    mut commands: Commands,
    mut camera: Query<&mut GameCamera>,
    mut drag_info: ResMut<DragInfo>,
    entities: Query<(Entity, &GameEntity)>,
) {
    println!("Clearing game entities");
    for (entity, _) in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Reset the camera orientation and distance
    if let Some(mut camera) = camera.iter_mut().next() {
        *camera = GameCamera::default();
    }

    // Reset drag info
    **drag_info = None;
}

fn spawn_game_scene(
    mut commands: Commands,
    game_scene: Res<CurrentLevel>,
    game_assets: Res<GameAssets>,
) {
    if let Some(game_level) = &game_scene.0 {
        commands
            .spawn(SceneBundle {
                scene: game_assets.get_scene(GameScene::Level(*game_level)),
                ..default()
            })
            .insert(GameEntity);
    }
}

enum GameLevelObjectType {
    Object,
    Bounds,
    Spawn,
    Goal(Vec2),
}

impl TryFrom<&Name> for GameLevelObjectType {
    type Error = ();

    fn try_from(value: &Name) -> Result<Self, ()> {
        match value.as_str() {
            s if s.starts_with("Object") => Ok(Self::Object),
            "Bounds" => Ok(Self::Bounds),
            "Spawn" => Ok(Self::Spawn),
            s if s.starts_with("Goal") => s
                .split_once('_')
                .map(|(_, d)| match &d[..2] {
                    "X+" => Some(Vec2::X),
                    "X-" => Some(-Vec2::X),
                    "Y+" => Some(Vec2::Y),
                    "Y-" => Some(-Vec2::Y),
                    _ => None,
                })
                .flatten()
                .map(|d| Self::Goal(d))
                .ok_or(()),
            _ => Err(()),
        }
    }
}

fn initialize_game_scene(
    mut commands: Commands,
    entities: Query<(Entity, &Name, Option<&Children>), Added<Name>>,
    meshes: Res<Assets<Mesh>>,
    mesh_entities: Query<&Handle<Mesh>>,
    transforms: Query<&Transform>,
    game_assets: Res<GameAssets>,
) {
    for (entity, name, children) in entities.iter() {
        let Ok(object_type) = GameLevelObjectType::try_from(name) else {
            continue;
        };

        match object_type {
            GameLevelObjectType::Object => {
                let Some(children) = children else {
                    continue;
                };
                for child in children.iter() {
                    let Some(collider) =
                        get_collider_from_mesh_entity(*child, &meshes, &mesh_entities)
                    else {
                        continue;
                    };
                    let mut entity_commands = commands.entity(*child);
                    entity_commands.insert(collider);
                }
            }
            GameLevelObjectType::Bounds => {
                let Some(children) = children else {
                    continue;
                };
                for child in children.iter() {
                    let Some(collider) =
                        get_collider_from_mesh_entity(*child, &meshes, &mesh_entities)
                    else {
                        continue;
                    };
                    let mut entity_commands = commands.entity(*child);
                    entity_commands.insert(collider);
                    entity_commands.insert((GameBounds, Visibility::Hidden));
                }
            }
            GameLevelObjectType::Spawn => {
                let Ok(transform) = transforms.get(entity) else {
                    continue;
                };
                spawn_player(
                    &mut commands,
                    &game_assets,
                    transform.translation + Vec3::Z * PLAYER_RADIUS,
                );
            }
            GameLevelObjectType::Goal(dir) => {
                let Some(children) = children else {
                    continue;
                };
                for child in children.iter() {
                    let Ok(mesh) = mesh_entities.get(*child) else {
                        continue;
                    };
                    let Some(mesh) = meshes.get(mesh) else {
                        continue;
                    };
                    let Some(collider) =
                        Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh)
                    else {
                        continue;
                    };
                    commands
                        .entity(*child)
                        .insert((collider, Sensor, GameGoal(dir)));
                }
            }
        }
    }
}

fn get_collider_from_mesh_entity(
    entity: Entity,
    meshes: &Res<Assets<Mesh>>,
    mesh_entities: &Query<&Handle<Mesh>>,
) -> Option<Collider> {
    let Ok(mesh) = mesh_entities.get(entity) else {
        return None;
    };
    let Some(mesh) = meshes.get(mesh) else {
        return None;
    };
    Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh)
}

#[derive(Component)]
struct GameBounds;

fn reload_on_bounds_collision(
    mut set_game_level: EventWriter<SetGameLevel>,
    mut ground_collisions: EventReader<CollisionEvent>,
    game_level: Res<CurrentLevel>,
    player: Query<Entity, With<Player>>,
    bounds: Query<Entity, With<GameBounds>>,
) {
    let Some(player_entity) = player.iter().next() else {
        return;
    };
    let Some(bounds_entity) = bounds.iter().next() else {
        return;
    };
    let Some(game_level) = **game_level else {
        return;
    };

    let entities = [player_entity, bounds_entity];
    for collision in ground_collisions.read() {
        if let CollisionEvent::Started(e1, e2, _) = collision {
            if entities.contains(e1) && entities.contains(e2) {
                set_game_level.send(SetGameLevel(game_level));
            }
        }
    }
}

#[derive(Component)]
struct GameGoal(Vec2);

fn reload_on_pass_through_goal(
    mut set_game_level: EventWriter<SetGameLevel>,
    game_level: Res<CurrentLevel>,
    player: Query<(Entity, &Transform), With<Player>>,
    goals: Query<(Entity, &GlobalTransform, &GameGoal)>,
    rapier_context: Res<RapierContext>,
    mut started: Local<Option<Entity>>,
) {
    let Some((player_entity, player_transform)) = player.iter().next() else {
        return;
    };
    let Some(game_level) = **game_level else {
        return;
    };

    let mut is_intersected = false;

    for (e1, e2, _overlap) in rapier_context.intersection_pairs_with(player_entity) {
        let Some(goal_entity) = goals
            .get(e1)
            .ok()
            .or_else(|| goals.get(e2).ok())
            .map(|(e, _, _)| e)
        else {
            continue;
        };
        is_intersected = true;
        if Some(goal_entity) == *started {
            continue;
        }
        *started = Some(goal_entity);
    }

    if is_intersected {
        return;
    }

    let Some(goal_entity) = *started else {
        return;
    };

    let Ok((_, goal_transform, goal)) = goals.get(goal_entity) else {
        return;
    };

    let movement =
        (player_transform.translation.xy() - goal_transform.translation().xy()).normalize();
    if movement.x * goal.0.x >= 0.0 && movement.y * goal.0.y >= 0.0 {
        set_game_level.send(SetGameLevel(game_level));
    }
    *started = None;
}
