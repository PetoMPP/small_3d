use super::custom_tweening_plugin::{update_scale, RelativeScale, RelativeScaleLens};
use crate::{
    common::plugins::user_input_plugin::{UserInput, UserInputPosition},
    game::components::{GameCamera, GameEntity, GameUiCamera, Player},
    log,
    resources::{
        game_assets::{GameAnimationSource, GameAssets, GameMaterial, GameScene},
        inputs::Inputs,
    },
    AppState,
};
use bevy::{prelude::*, scene::SceneInstance};
use bevy_rapier3d::prelude::*;
use bevy_tweening::{Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween};
use bevy_vector_shapes::{
    painter::{BuildShapeChildren, ShapeConfig},
    shapes::{DiscSpawner, ThicknessType},
};
use std::{f32::consts::PI, time::Duration};

pub struct AimingPlugin;

impl Plugin for AimingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragInfo>()
            .add_systems(
                Update,
                (
                    start_player_aim,
                    cancel_player_aim,
                    aim_player,
                    fire_player,
                    initialize_arrow_components,
                    update_arrow,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(Update, adjust_arrow.after(update_arrow).after(update_scale));
    }
}

#[derive(Component)]
pub struct AimCircle(f32);

pub fn spawn_circle(commands: &mut Commands, window: &Window) {
    let mut shapes_config = ShapeConfig::default_2d();
    shapes_config.color = Color::WHITE.with_a(0.9);
    shapes_config.hollow = true;
    shapes_config.thickness = 0.8;
    shapes_config.thickness_type = ThicknessType::Screen;
    let radius = window.height().min(window.width()) / 3.5 / 2.0;

    commands
        .spawn((AimCircle(radius), Visibility::Visible, GameEntity))
        .with_shape_children(&shapes_config, |builder| {
            const STEPS: usize = 16;
            for i in 0..=STEPS {
                const STEP: f32 = 1.0 / STEPS as f32 * 2.0 * PI;
                const OFFSET: f32 = 0.3 * STEP;
                let angle = i as f32 * STEP + OFFSET;
                builder.arc(radius, angle, angle + STEP / 2.7);
            }
        });
}

pub fn spawn_arrow(commands: &mut Commands, game_assets: &Res<GameAssets>, pos: Vec3) {
    commands
        .spawn(SceneBundle {
            scene: game_assets.get_scene(GameScene::AimArrow),
            transform: Transform::from_translation(pos),
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .insert((
            GameEntity,
            ArrowScene::default(),
            RelativeScale::default(),
            Animator::<RelativeScale>::new(
                Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_secs_f32(0.66),
                    RelativeScaleLens {
                        start: Vec3::splat(0.98),
                        end: Vec3::splat(1.03),
                    },
                )
                .with_repeat_strategy(RepeatStrategy::MirroredRepeat)
                .with_repeat_count(RepeatCount::Infinite),
            ),
        ));
}

#[derive(Component)]
pub struct ArrowAnimationPlayer;

impl GameAnimationSource for ArrowAnimationPlayer {
    fn get_animation_filename(&self) -> &str {
        "models/arrow.glb"
    }
}

#[derive(Component, Default)]
pub struct ArrowScene {
    power: f32,
    angle: f32,
}

#[derive(Component)]
pub struct ArrowEntity;

fn initialize_arrow_components(
    mut commands: Commands,
    spawned_arrow_scene: Query<&Children, (Added<SceneInstance>, With<ArrowScene>)>,
    mut new_animations: Query<(Entity, &Parent), Added<AnimationPlayer>>,
) {
    let mut arrow_entities = Vec::new();
    for children in spawned_arrow_scene.iter() {
        for child in children.iter() {
            commands.entity(*child).insert(ArrowEntity);
            arrow_entities.push(*child);
        }
    }

    for (entity, parent) in new_animations.iter_mut() {
        if arrow_entities.contains(&**parent) {
            commands.entity(entity).insert(ArrowAnimationPlayer);
        }
    }
}

fn update_arrow(
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    player: Query<&Transform, With<Player>>,
    mut arrow: Query<(&Transform, &mut ArrowScene)>,
    drag_info: Res<DragInfo>,
) {
    let Some(window) = window.iter().next() else {
        return;
    };

    let Some((camera, camera_transform)) = camera.iter().next() else {
        return;
    };

    let Some(player_transform) = player.iter().next() else {
        log!("no player");
        return;
    };

    let Some(player_pos) = camera.world_to_viewport(camera_transform, player_transform.translation)
    else {
        log!("no player pos");
        return;
    };

    let Some((arrow_transform, mut arrow_scene)) = arrow.iter_mut().next() else {
        return;
    };

    let arrow_point = arrow_transform.translation;

    let Some(drag_info) = **drag_info else {
        *arrow_scene = ArrowScene::default();
        return;
    };

    // 1.0 is half the screen height
    const MIN: f32 = 0.2;
    const MAX: f32 = 0.7;

    let max = window.height() / 2.0;
    let power = player_pos.distance(drag_info.point).min(max) / max;
    let power = match power {
        x if x < MIN => 0.0,
        x if x > MAX => 1.0,
        x => (x - MIN) / (MAX - MIN),
    };

    let Some(ray) = camera.viewport_to_world(camera_transform, drag_info.point) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(arrow_point, Plane3d::new(Vec3::Z)) else {
        return;
    };

    let drag_point = ray.get_point(distance);
    let drag_point = drag_point.lerp(arrow_point, 2.0);
    let angle = (drag_point.y - arrow_point.y).atan2(drag_point.x - arrow_point.x)
        - std::f32::consts::FRAC_PI_2;

    log!("power: {}, angle: {}", power, angle.to_degrees());
    arrow_scene.power = power;
    arrow_scene.angle = angle;
}

fn adjust_arrow(
    mut animation_players: Query<&mut AnimationPlayer, With<ArrowAnimationPlayer>>,
    mut arrow: Query<
        (&mut Transform, &mut Visibility, &ArrowScene),
        (Changed<ArrowScene>, Without<Player>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player: Query<&Transform, With<Player>>,
    game_assets: Res<GameAssets>,
) {
    let Some((mut arrow_transform, mut arrow_visibility, arrow_scene)) = arrow.iter_mut().next()
    else {
        return;
    };

    let visibility = match arrow_scene.power {
        x if x <= 0.0 => Visibility::Hidden,
        _ => Visibility::Visible,
    };

    let Some(color_material) =
        materials.get_mut(game_assets.get_material(GameMaterial::AimArrowBody))
    else {
        return;
    };

    let color = get_power_color(arrow_scene.power);

    let Some(player_transform) = player.iter().next() else {
        return;
    };

    let transform = player_transform
        .with_rotation(Quat::from_rotation_z(arrow_scene.angle))
        .with_scale(0.65.lerp(1.10, arrow_scene.power) * arrow_transform.scale);

    if *arrow_visibility != visibility {
        *arrow_visibility = visibility;
    }
    if color_material.base_color != color {
        color_material.base_color = color;
    }
    if *arrow_transform != transform {
        *arrow_transform = transform;
    }

    // Clip is played in 24fps at 1.0 speed
    // Duration is 60 frames = 2.5 seconds
    const DURATION: f32 = 2.5;
    let power = arrow_scene.power * DURATION;

    for mut animation_player in animation_players.iter_mut() {
        animation_player.seek_to(power);
    }
}

fn get_power_color(power: f32) -> Color {
    const BRIGHTNESS: f32 = 0.7;
    // #3CDD3C
    const FROM: Vec3 = Vec3::new(0.0, 1.0, 0.0);
    // #EDC25E
    const THROUGH: Vec3 = Vec3::new(1.0, 1.0, 0.0);
    // #E23636
    const TO: Vec3 = Vec3::new(1.0, 0.0, 0.0);
    let rgb = match power {
        x if x < 0.5 => FROM.lerp(THROUGH, x * 2.0),
        x => THROUGH.lerp(TO, (x - 0.5) * 2.0),
    };
    // Color::rgba_linear_from_array(Vec3::ZERO.lerp(rgb, BRIGHTNESS).extend(0.4))
    Color::rgb_linear_from_array(Vec3::ZERO.lerp(rgb, BRIGHTNESS))
}

#[derive(Resource, Default, Deref, DerefMut, Clone, Copy)]
pub struct DragInfo(Option<DragInfoData>);

#[derive(Debug, Clone, Copy)]
pub struct DragInfoData {
    point: Vec2,
    user_input: UserInput,
    confirmed: bool,
}

fn start_player_aim(
    aim_circle: Query<&AimCircle>,
    user_input: Res<Inputs<UserInput>>,
    user_input_position: Res<UserInputPosition>,
    ui_camera: Query<(&Camera, &GlobalTransform), With<GameUiCamera>>,
    mut drag_info: ResMut<DragInfo>,
) {
    let Some(circle) = aim_circle.iter().next() else {
        return;
    };
    let Some((camera, camera_transform)) = ui_camera.iter().next() else {
        return;
    };

    for user_input in user_input.iter_just_pressed() {
        let Some(position) = user_input_position.get(**user_input) else {
            log!("no input position");
            break;
        };
        let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, position) else {
            log!("no world position");
            break;
        };
        if world_pos.distance_squared(Vec2::ZERO) <= circle.0.powi(2) {
            **drag_info = Some(DragInfoData {
                point: position,
                user_input: *user_input,
                confirmed: false,
            });
        }
    }
}

fn cancel_player_aim(mut drag_info: ResMut<DragInfo>, user_input: Res<Inputs<UserInput>>) {
    if user_input.iter_pressed().count() > 1 {
        **drag_info = None;
    }
}

fn aim_player(
    user_input_position: Res<UserInputPosition>,
    user_input: Res<Inputs<UserInput>>,
    mut drag_info: ResMut<DragInfo>,
) {
    let Some(drag_info_data) = &mut **drag_info else {
        return;
    };

    if let Some(cursor_position) = user_input_position.get(*drag_info_data.user_input) {
        drag_info_data.point = cursor_position;
    } else {
        log!("no cursor position");
    }

    if user_input.just_released(drag_info_data.user_input) {
        drag_info_data.confirmed = true;
    }
}

fn fire_player(
    mut player: Query<(&Transform, &mut Player, &mut ExternalImpulse)>,
    arrow: Query<&ArrowScene>,
    camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut drag_info: ResMut<DragInfo>,
) {
    let (camera, camera_transform) = camera.single();

    let Some((transform, mut player, mut impulse)) = player.iter_mut().next() else {
        return;
    };

    let Some(drag_info_data) = &mut **drag_info else {
        return;
    };

    if drag_info_data.confirmed {
        if let Some(new_impulse) = calculate_impulse(
            transform,
            &mut player,
            drag_info_data,
            &arrow,
            camera,
            camera_transform,
        ) {
            *impulse = new_impulse;
        }
        **drag_info = None;
    }
}

fn calculate_impulse(
    transform: &Transform,
    player: &mut Player,
    drag_info_data: &DragInfoData,
    arrow: &Query<'_, '_, &ArrowScene>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<ExternalImpulse> {
    let arrow = arrow.iter().next()?;
    if arrow.power <= 0.0 {
        return None;
    }
    let ray = camera.viewport_to_world(camera_transform, drag_info_data.point)?;
    let distance = ray.intersect_plane(transform.translation, Plane3d::new(Vec3::Z))?;
    let point = ray.get_point(distance);
    player.shots -= 1;
    let push = (transform.translation - point).normalize() * arrow.power * 250.0;

    Some(ExternalImpulse::at_point(
        push,
        transform.translation,
        transform.translation,
    ))
}
