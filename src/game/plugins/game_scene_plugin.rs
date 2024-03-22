use crate::{
    game::{
        components::{GameCamera, GameEntity, Ground, Player},
        resources::{GameScene, GameSceneData},
    },
    resources::{FontSize, FontType},
    AppState, TextStyles,
};
use bevy::{input::keyboard::KeyboardInput, prelude::*};
use bevy_picking_rapier::bevy_rapier3d::prelude::*;

pub struct GameScenePlugin;

#[derive(Event, Deref, DerefMut)]
pub struct SetGameScene(pub GameSceneData);

impl Plugin for GameScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameScene>()
            .init_resource::<ReloadTimerInitialized>()
            .add_event::<SetGameScene>()
            .add_systems(
                Update,
                (
                    set_game_scene,
                    reload_scene,
                    reload_on_ground_collision,
                    update_distance,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (reset_state, spawn_distance_text, spawn_game_scene)
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
        **game_scene = Some(**set_game_scene);
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
        if let Some(game_scene) = **game_scene {
            set_game_scene.send(SetGameScene(game_scene));
        }
    }
}

fn reset_state(
    mut commands: Commands,
    mut camera: Query<&mut GameCamera>,
    mut timer_initialized: ResMut<ReloadTimerInitialized>,
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

    // Reset reload timer
    **timer_initialized = false;
}

#[derive(Component)]
struct DistanceText;

fn spawn_distance_text(mut commands: Commands, text_styles: Res<TextStyles>) {
    // Text to describe the controls.
    const GAME_MANUAL_TEXT: &str = "\
    Drag the ball to launch it\n\
    Swipe to rotate the camera\n\
    Middle mouse to zoom camera\n\
    Press R to reset the level";

    println!("Spawning distance text");
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            GameEntity,
        ))
        .with_children(|root| {
            root.spawn((
                TextBundle {
                    text: Text::from_section(
                        GAME_MANUAL_TEXT,
                        text_styles.get(FontType::Regular, FontSize::Medium, Color::WHITE),
                    ),
                    ..Default::default()
                },
                DistanceText,
            ));
        });
}

fn spawn_game_scene(
    mut commands: Commands,
    game_scene: Res<GameScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    println!("Spawning game scene");
    if let Some(game_scene) = &game_scene.0 {
        game_scene.spawn(&mut commands, &mut meshes, &mut materials, &asset_server);
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct ReloadTimerInitialized(bool);

fn reload_on_ground_collision(
    mut set_game_scene: EventWriter<SetGameScene>,
    mut ground_collisions: EventReader<CollisionEvent>,
    game_scene: Res<GameScene>,
    player: Query<(Entity, Ref<Sleeping>), With<Player>>,
    mut ground: Query<(Entity, &mut Ground), Without<Player>>,
    time: Res<Time>,
    mut initialized: ResMut<ReloadTimerInitialized>,
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
        if let Some(game_scene) = **game_scene {
            set_game_scene.send(SetGameScene(game_scene));
        }
        return;
    }

    // After first ground collision, wait for player to sleep before resetting
    if **initialized && player_sleep.is_changed() && player_sleep.sleeping {
        **initialized = false;
        ground.unpause();
        ground.reset();
    }

    let entities = [player_entity, ground_entity];
    for collision in ground_collisions.read() {
        match collision {
            // Set initialized to true if player and ground collide
            CollisionEvent::Started(e1, e2, _) => {
                if entities.contains(e1) && entities.contains(e2) {
                    **initialized = true;
                }
            }
            // Pause the ground timer if player and ground stop colliding
            CollisionEvent::Stopped(e1, e2, _) => {
                if entities.contains(e1) && entities.contains(e2) {
                    ground.pause();
                }
            }
        }
    }
}

fn update_distance(
    mut distance_text: Query<&mut Text, With<DistanceText>>,
    player: Query<&Transform, With<Player>>,
    scene: Res<GameScene>,
) {
    if let (Some(transform), Some(scene)) = (player.iter().next(), scene.0.as_ref()) {
        if let Some(mut distance_text) = distance_text.iter_mut().next() {
            let distance = (transform.translation.xy() - scene.start_pos().xy()).length();
            if distance > 0.0 {
                distance_text.sections[0].value = format!("Distance: {:.2} m", distance);
            }
        }
    }
}
