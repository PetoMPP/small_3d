use super::{
    aiming_plugin::{spawn_arrow, DragInfo},
    custom_tweening_plugin::GameTween,
    game_camera_plugin::GameCamera,
};
use crate::{
    game::{
        game_plugin::GameState,
        plugins::custom_tweening_plugin::{
            RelativeScale, RelativeScaleLens, Rotation, RotationLens,
        },
    },
    log,
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

#[derive(Component, Clone, Copy)]
pub struct GameEntity;

#[derive(Resource, Default, Clone, Reflect)]
#[reflect(Resource)]
pub struct GameData {
    pub level: Option<GameLevel>,
    pub shots: u32,
    pub points: i32,
    pub result: Option<bool>,
}

#[derive(Event)]
struct LevelChanged;

#[derive(Event, Deref, DerefMut)]
pub struct SetGameLevel(pub Option<GameLevel>);

pub struct GameScenePlugin;

impl Plugin for GameScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameData>()
            .register_type::<GameData>()
            .add_event::<SetGameLevel>()
            .add_event::<LevelChanged>()
            .add_systems(
                Update,
                (
                    initialize_game_scene,
                    initialize_game_scene_components,
                    reward_points_on_collision,
                    reload_scene,
                    lose_on_pass_through_bounds,
                    win_on_pass_through_goal,
                )
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, (reset_state, spawn_game_scene, set_game_scene))
            .add_systems(OnEnter(GameState::Paused), pause_animation_players)
            .add_systems(OnEnter(GameState::Playing), resume_animation_players);
    }
}

fn set_game_scene(
    mut game_data: ResMut<GameData>,
    mut set_game_level: EventReader<SetGameLevel>,
    mut level_changed: EventWriter<LevelChanged>,
) {
    for set_game_level in set_game_level.read() {
        *game_data = set_game_level.0.map(Into::into).unwrap_or_default();
        if set_game_level.0.is_none() {
            log!("Game level is not set");
        }
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
        set_game_scene.send(SetGameLevel(game_data.level));
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
    entities: Query<(Entity, &Name, Option<&Children>), Added<Name>>,
    meshes: Res<Assets<Mesh>>,
    mesh_entities: Query<&Handle<Mesh>>,
    transforms: Query<&Transform>,
    animation_players: Query<&AnimationPlayer>,
    game_assets: Res<GameAssets>,
    mut rng: NonSendMut<Random>,
) {
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
                        (GameBounds, Sensor, Visibility::Hidden),
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

#[derive(Component)]
pub struct Player;

pub const PLAYER_RADIUS: f32 = 0.2;

pub fn spawn_player(commands: &mut Commands, game_assets: &Res<GameAssets>, pos: Vec3) {
    commands
        .spawn(SceneBundle {
            scene: game_assets.get_scene(GameScene::Player),
            transform: Transform::from_translation(pos),
            ..Default::default()
        })
        .try_insert((
            Player,
            Sleeping::default(),
            ExternalImpulse::default(),
            RigidBody::Dynamic,
            Collider::ball(PLAYER_RADIUS),
            Friction::coefficient(0.6),
            Restitution::new(0.3),
            Damping {
                linear_damping: 0.5,
                angular_damping: 0.5,
            },
            ColliderMassProperties::Mass(10.0),
            ActiveEvents::COLLISION_EVENTS,
            Ccd::enabled(),
            GameEntity,
        ));
    spawn_arrow(commands, game_assets, pos);
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
    game_points: Query<(Entity, &Parent, &GamePoints)>,
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

fn lose_on_pass_through_bounds(
    rapier_context: ResMut<RapierContext>,
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<GameState>>,
    player: Query<Entity, With<Player>>,
    bounds: Query<Entity, With<GameBounds>>,
    mut started: Local<bool>,
) {
    let Some(player_entity) = player.iter().next() else {
        return;
    };
    let Some(bounds_entity) = bounds.iter().next() else {
        return;
    };

    match *started {
        true => {
            if !rapier_context
                .intersection_pair(player_entity, bounds_entity)
                .unwrap_or_default()
            {
                *started = false;
                game_data.result = Some(false);
                next_state.set(GameState::Finished);
            }
        }
        false => {
            if rapier_context
                .intersection_pair(player_entity, bounds_entity)
                .unwrap_or_default()
            {
                *started = true;
            }
        }
    }
}

#[derive(Component, Clone, Copy)]
struct GameGoal(Vec2);

fn win_on_pass_through_goal(
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<GameState>>,
    player: Query<(Entity, &Transform), With<Player>>,
    goals: Query<(Entity, &GlobalTransform, &GameGoal)>,
    rapier_context: Res<RapierContext>,
    mut started: Local<Option<Entity>>,
) {
    let Some((player_entity, player_transform)) = player.iter().next() else {
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
        game_data.result = Some(true);
        next_state.set(GameState::Finished);
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