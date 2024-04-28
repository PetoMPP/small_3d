use super::{
    aiming_plugin::{spawn_circle, DragInfo},
    custom_tweening_plugin::GameTween,
    player_plugin::{Player, PLAYER_RADIUS},
};
use crate::{
    game::{
        components::{GameCamera, GameEntity},
        game_plugin::GameRunningState,
        plugins::{
            custom_tweening_plugin::{RelativeScale, RelativeScaleLens, Rotation, RotationLens},
            player_plugin::spawn_player,
        },
    },
    resources::{
        game_assets::{GameAnimationSource, GameAssets, GameLevel, GameScene},
        random::Random,
    },
    AppState,
};
use bevy::{input::keyboard::KeyboardInput, prelude::*, scene::SceneInstance};
use bevy_rapier3d::prelude::*;
use bevy_tweening::{Animator, EaseFunction, EaseMethod, RepeatCount, RepeatStrategy, Tween};
use rand::Rng;
use std::time::Duration;

pub struct GameScenePlugin;

#[derive(Resource, Default, Clone)]
pub struct GameData {
    pub level: Option<GameLevel>,
    pub shots: u32,
    pub points: i32,
}

#[derive(Event)]
struct LevelChanged;

#[derive(Event, Deref, DerefMut)]
pub struct SetGameLevel(pub GameLevel);

impl Plugin for GameScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameData>()
            .add_event::<SetGameLevel>()
            .add_event::<LevelChanged>()
            .add_systems(
                Update,
                (
                    set_game_scene,
                    initialize_game_scene,
                    initialize_game_scene_components,
                    reward_points_on_collision,
                    reload_scene,
                    reload_on_bounds_collision,
                    reload_on_pass_through_goal,
                )
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(GameRunningState(true))),
            )
            .add_systems(
                Update,
                (reset_state, spawn_game_scene)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(GameRunningState(true))),
            )
            .add_systems(OnEnter(GameRunningState(false)), pause_animation_players)
            .add_systems(OnEnter(GameRunningState(true)), resume_animation_players);
    }
}

fn set_game_scene(
    mut game_data: ResMut<GameData>,
    mut set_game_level: EventReader<SetGameLevel>,
    mut level_changed: EventWriter<LevelChanged>,
) {
    for set_game_level in set_game_level.read() {
        *game_data = set_game_level.0.into();
        level_changed.send(LevelChanged);
    }
}

fn reload_scene(
    mut set_game_scene: EventWriter<SetGameLevel>,
    game_data: Res<GameData>,
    mut key_input: EventReader<KeyboardInput>,
) {
    let Some(key_input) = key_input.read().next() else {
        return;
    };

    if key_input.state.is_pressed() && key_input.key_code == KeyCode::KeyR {
        if let Some(game_level) = game_data.level {
            set_game_scene.send(SetGameLevel(game_level));
        }
    }
}

fn reset_state(
    mut commands: Commands,
    mut camera: Query<&mut GameCamera>,
    mut drag_info: ResMut<DragInfo>,
    entities: Query<(Entity, &GameEntity)>,
    mut level_changed: EventReader<LevelChanged>,
) {
    if level_changed.read().next().is_none() {
        return;
    }

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

#[derive(Component)]
struct GameSceneScene;

#[derive(Component)]
struct GameSceneEntity;

#[derive(Component)]
pub struct GameSceneAnimationPlayer(pub GameLevel);

impl GameAnimationSource for GameSceneAnimationPlayer {
    fn get_animation_filename(&self) -> &str {
        self.0.get_filename()
    }
}

fn spawn_game_scene(
    mut commands: Commands,
    game_data: Res<GameData>,
    game_assets: Res<GameAssets>,
    mut rng: NonSendMut<Random>,
    mut level_changed: EventReader<LevelChanged>,
) {
    if level_changed.read().next().is_none() {
        return;
    }

    if let Some(game_level) = game_data.level {
        commands
            .spawn(SceneBundle {
                scene: game_assets.get_scene(GameScene::Level(game_level)),
                ..default()
            })
            .insert((GameSceneScene, GameEntity));
        rng.reset(game_level);
    }
}

fn initialize_game_scene_components(
    mut commands: Commands,
    spawned_game_scene_scene: Query<&Children, (Added<SceneInstance>, With<GameSceneScene>)>,
    mut new_animations: Query<(Entity, &Parent), Added<AnimationPlayer>>,
    game_data: Res<GameData>,
) {
    let mut game_scene_entities = Vec::new();
    for children in spawned_game_scene_scene.iter() {
        for child in children.iter() {
            commands.entity(*child).try_insert(GameSceneEntity);
            game_scene_entities.push(*child);
        }
    }

    for (entity, parent) in new_animations.iter_mut() {
        if game_scene_entities.contains(&**parent) {
            commands
                .entity(entity)
                .try_insert(GameSceneAnimationPlayer(game_data.level.unwrap()));
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
struct GamePoints {
    reward: i32,
}

enum GameLevelObjectType {
    Object,
    Bounds,
    Spawn,
    Goal(Vec2),
    Point(GamePoints),
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
                .and_then(|(_, d)| d.get(..2))
                .and_then(|d| match d {
                    "X+" => Some(Vec2::X),
                    "X-" => Some(-Vec2::X),
                    "Y+" => Some(Vec2::Y),
                    "Y-" => Some(-Vec2::Y),
                    _ => None,
                })
                .map(Self::Goal)
                .ok_or(()),

            s if s.starts_with("Point") => s
                .split_once('_')
                .map(|(_, d)| d.chars().take_while(|c| c != &'.').collect::<String>())
                .and_then(|d| d.parse().ok())
                .map(|reward| GamePoints { reward })
                .map(Self::Point)
                .ok_or(()),

            _ => Err(()),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn initialize_game_scene(
    mut commands: Commands,
    window: Query<&Window>,
    entities: Query<(Entity, &Name, Option<&Children>), Added<Name>>,
    meshes: Res<Assets<Mesh>>,
    mesh_entities: Query<&Handle<Mesh>>,
    transforms: Query<&Transform>,
    animation_players: Query<&AnimationPlayer>,
    game_assets: Res<GameAssets>,
    mut rng: NonSendMut<Random>,
) {
    let Some(window) = window.iter().next() else {
        return;
    };
    for (entity, name, children) in entities.iter() {
        let Ok(object_type) = GameLevelObjectType::try_from(name) else {
            continue;
        };

        match object_type {
            GameLevelObjectType::Object => {
                if let Some(children) = children {
                    insert_collider_into_entities(
                        &mut commands,
                        children,
                        &meshes,
                        &mesh_entities,
                        (),
                    );
                }
            }
            GameLevelObjectType::Bounds => {
                if let Some(children) = children {
                    insert_collider_into_entities(
                        &mut commands,
                        children,
                        &meshes,
                        &mesh_entities,
                        (GameBounds, Visibility::Hidden),
                    );
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
                spawn_circle(&mut commands, window);
            }
            GameLevelObjectType::Goal(dir) => {
                if let Some(children) = children {
                    insert_collider_into_entities(
                        &mut commands,
                        children,
                        &meshes,
                        &mesh_entities,
                        (Sensor, GameGoal(dir)),
                    );
                }
            }
            GameLevelObjectType::Point(pt) => {
                if let Some(children) = children {
                    insert_collider_into_entities(
                        &mut commands,
                        children,
                        &meshes,
                        &mesh_entities,
                        (pt, Sensor),
                    );
                }

                let entity_commands = &mut commands.entity(entity);
                entity_commands.try_insert((
                    Animator::<RelativeScale>::new(
                        Tween::new(
                            EaseFunction::SineInOut,
                            Duration::from_secs_f32(rng.gen_range(0.3..1.0)),
                            RelativeScaleLens {
                                start: Vec3::splat(0.95),
                                end: Vec3::splat(1.08),
                            },
                        )
                        .with_repeat_strategy(RepeatStrategy::MirroredRepeat)
                        .with_repeat_count(RepeatCount::Infinite),
                    ),
                    RelativeScale::default(),
                    Rotation::new(Vec3::Z),
                    GameTween,
                ));

                if animation_players.get(entity).is_err() {
                    entity_commands.try_insert((
                        Animator::<Rotation>::new(
                            Tween::new(
                                EaseMethod::Linear,
                                Duration::from_secs_f32(rng.gen_range(0.7..2.0)),
                                RotationLens,
                            )
                            .with_repeat_strategy(RepeatStrategy::Repeat)
                            .with_repeat_count(RepeatCount::Infinite),
                        ),
                        GameTween,
                    ));
                }
            }
        }
    }
}

fn insert_collider_into_entities<'a>(
    commands: &mut Commands,
    entities: impl IntoIterator<Item = &'a Entity>,
    meshes: &Assets<Mesh>,
    mesh_entities: &Query<&Handle<Mesh>>,
    bundle: impl Bundle + Clone,
) {
    for entity in entities {
        let Some(collider) = get_collider_from_mesh_entity(*entity, meshes, mesh_entities) else {
            continue;
        };
        commands
            .entity(*entity)
            .try_insert(collider)
            .try_insert(bundle.clone());
    }
}

fn get_collider_from_mesh_entity(
    entity: Entity,
    meshes: &Assets<Mesh>,
    mesh_entities: &Query<&Handle<Mesh>>,
) -> Option<Collider> {
    Collider::from_bevy_mesh(
        meshes.get(mesh_entities.get(entity).ok()?)?,
        &ComputedColliderShape::TriMesh,
    )
}

fn reward_points_on_collision(
    mut commands: Commands,
    game_points: Query<(Entity, &Parent, &GamePoints), With<GamePoints>>,
    player: Query<Entity, With<Player>>,
    rapier_context: ResMut<RapierContext>,
    mut game_data: ResMut<GameData>,
) {
    let Some(player_entity) = player.iter().next() else {
        return;
    };

    for (points_entity, parent_entity, game_points) in game_points.iter() {
        if rapier_context
            .intersection_pair(player_entity, points_entity)
            .unwrap_or_default()
        {
            commands.entity(**parent_entity).despawn_recursive();
            game_data.points += game_points.reward;
        }
    }
}

#[derive(Component, Clone, Copy)]
struct GameBounds;

fn reload_on_bounds_collision(
    mut set_game_level: EventWriter<SetGameLevel>,
    mut ground_collisions: EventReader<CollisionEvent>,
    game_data: Res<GameData>,
    player: Query<Entity, With<Player>>,
    bounds: Query<Entity, With<GameBounds>>,
) {
    let Some(player_entity) = player.iter().next() else {
        return;
    };
    let Some(bounds_entity) = bounds.iter().next() else {
        return;
    };
    let Some(game_level) = game_data.level else {
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

#[derive(Component, Clone, Copy)]
struct GameGoal(Vec2);

fn reload_on_pass_through_goal(
    mut set_game_level: EventWriter<SetGameLevel>,
    game_data: Res<GameData>,
    player: Query<(Entity, &Transform), With<Player>>,
    goals: Query<(Entity, &GlobalTransform, &GameGoal)>,
    rapier_context: Res<RapierContext>,
    mut started: Local<Option<Entity>>,
) {
    let Some((player_entity, player_transform)) = player.iter().next() else {
        return;
    };
    let Some(game_level) = game_data.level else {
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

fn pause_animation_players(
    mut game_scene_animations: Query<&mut AnimationPlayer, With<GameSceneAnimationPlayer>>,
) {
    for mut player in game_scene_animations.iter_mut() {
        player.set_speed(0.0);
    }
}

fn resume_animation_players(
    mut game_scene_animations: Query<&mut AnimationPlayer, With<GameSceneAnimationPlayer>>,
) {
    for mut player in game_scene_animations.iter_mut() {
        player.set_speed(1.0);
    }
}
