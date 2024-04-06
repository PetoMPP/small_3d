use crate::{
    common::plugins::user_input_plugin::{Pressed, UserInput, UserInputPosition},
    game::components::{GameCamera, GameEntity, Player},
    log,
    resources::{
        game_assets::{GameAssets, GameMaterial, GameScene},
        inputs::Inputs,
    },
    AppState,
};
use bevy::{prelude::*, scene::SceneInstance};
use bevy_mod_picking::prelude::*;
use bevy_picking_rapier::bevy_rapier3d::prelude::*;
use bevy_tweening::{
    component_animator_system, Animator, EaseFunction, Lens, RepeatCount, RepeatStrategy, Tween,
};
use std::time::Duration;

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
                    component_animator_system::<ArrowScene>,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(Update, adjust_arrow.after(update_arrow));
    }
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
            Animator::<ArrowScene>::new(
                Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_secs_f32(0.5),
                    ArrowSizeLens,
                )
                .with_repeat_strategy(RepeatStrategy::MirroredRepeat)
                .with_repeat_count(RepeatCount::Infinite),
            ),
        ));
}

#[derive(Component)]
pub struct ArrowAnimationPlayer;

#[derive(Component, Default)]
pub struct ArrowScene {
    power: f32,
    angle: f32,
    size_tween: f32,
}

pub struct ArrowSizeLens;

impl Lens<ArrowScene> for ArrowSizeLens {
    fn lerp(&mut self, target: &mut ArrowScene, ratio: f32) {
        target.size_tween = 0.98.lerp(1.03, ratio);
    }
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
    player: Query<(Entity, &Transform), With<Player>>,
    mut arrow: Query<(&Transform, &mut ArrowScene)>,
    drag_info: Res<DragInfo>,
    rapier_context: Res<RapierContext>,
) {
    let Some(window) = window.iter().next() else {
        return;
    };

    let Some((camera, camera_transform)) = camera.iter().next() else {
        return;
    };

    let Some((player_entity, player_transform)) = player.iter().next() else {
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

    let normal = get_contact_normal(player_entity, &rapier_context).unwrap_or(Vec3::Z);

    let Some(ray) = camera.viewport_to_world(camera_transform, drag_info.point) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(arrow_point, Plane3d::new(normal)) else {
        return;
    };

    let drag_point = ray.get_point(distance);
    let drag_point = drag_point.lerp(arrow_point, 2.0);
    let angle = (drag_point.y - arrow_point.y).atan2(drag_point.x - arrow_point.x)
        - std::f32::consts::FRAC_PI_2;

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
        .with_scale(Vec3::splat(
            0.65.lerp(1.10, arrow_scene.power) * arrow_scene.size_tween,
        ));

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
    player: Query<(Entity, &Player)>,
    user_input_position: Res<UserInputPosition>,
    mut presses: EventReader<Pointer<Pressed>>,
    mut drag_info: ResMut<DragInfo>,
) {
    let Some((player_entity, player)) = player.iter().next() else {
        return;
    };

    for press in presses.read() {
        log!("press");
        if press.target == player_entity && player.shots > 0 {
            log!("press target");
            let Some(position) = user_input_position.get(*press.user_input) else {
                log!("no input position");
                break;
            };
            **drag_info = Some(DragInfoData {
                point: position,
                user_input: press.user_input,
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
    mut player: Query<(Entity, &Transform, &mut Player, &mut ExternalImpulse)>,
    arrow: Query<&ArrowScene>,
    camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut drag_info: ResMut<DragInfo>,
    rapier_context: Res<RapierContext>,
) {
    let (camera, camera_transform) = camera.single();

    let Some((entity, transform, mut player, mut impulse)) = player.iter_mut().next() else {
        return;
    };

    let Some(drag_info_data) = &mut **drag_info else {
        return;
    };

    if drag_info_data.confirmed {
        if let Some(new_impulse) = calculate_impulse(
            entity,
            transform,
            &mut player,
            drag_info_data,
            &arrow,
            camera,
            camera_transform,
            &rapier_context,
        ) {
            *impulse = new_impulse;
        }
        **drag_info = None;
    }
}

#[allow(clippy::too_many_arguments)]
fn calculate_impulse(
    entity: Entity,
    transform: &Transform,
    player: &mut Player,
    drag_info_data: &DragInfoData,
    arrow: &Query<'_, '_, &ArrowScene>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    rapier_context: &Res<RapierContext>,
) -> Option<ExternalImpulse> {
    let arrow = arrow.iter().next()?;
    if arrow.power <= 0.0 {
        return None;
    }
    let normal = get_contact_normal(entity, rapier_context).unwrap_or(Vec3::Z);
    let ray = camera.viewport_to_world(camera_transform, drag_info_data.point)?;
    let distance = ray.intersect_plane(transform.translation, Plane3d::new(normal))?;
    let point = ray.get_point(distance);
    player.shots -= 1;
    let push = (transform.translation - point).normalize() * arrow.power * 250.0;

    Some(ExternalImpulse::at_point(
        push,
        transform.translation,
        transform.translation,
    ))
}

fn get_contact_normal(player_entity: Entity, rapier_context: &Res<RapierContext>) -> Option<Vec3> {
    let Some(contact_pair) = rapier_context.contact_pairs_with(player_entity).next() else {
        log!("no contact pair");
        return None;
    };

    contact_pair
        .manifold(0)
        .map(|m| m.normal())
        .map(|n| match n == Vec3::ZERO {
            true => None,
            false => Some(n),
        })
        .flatten()
}
