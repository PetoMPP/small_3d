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
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_picking_rapier::bevy_rapier3d::prelude::*;

pub struct AimingPlugin;

impl Plugin for AimingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragInfo>().add_systems(
            Update,
            (
                start_player_aim,
                cancel_player_aim,
                aim_player,
                fire_player,
                adjust_arrow,
            )
                .run_if(in_state(AppState::InGame)),
        );
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
        .insert((GameEntity, ArrowScene::default()));
}

#[derive(Component)]
pub struct ArrowAnimationPlayer;

#[derive(Component, Default)]
struct ArrowScene {
    power: f32,
}

fn adjust_arrow(
    mut animation_players: Query<&mut AnimationPlayer, With<ArrowAnimationPlayer>>,
    player: Query<(Entity, Ref<Transform>), (With<Player>, Without<ArrowScene>)>,
    mut arrow: Query<(&mut Transform, &mut Visibility, &mut ArrowScene)>,
    camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    drag_info: Res<DragInfo>,
    window: Query<&Window>,
    rapier_context: Res<RapierContext>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
) {
    let Some(window) = window.iter().next() else {
        return;
    };

    let Some((camera, camera_transform)) = camera.iter().next() else {
        return;
    };

    let Some((player_entity, player_transform)) = player.iter().next() else {
        return;
    };

    let Some((mut arrow_transform, mut arrow_visibility, mut arrow_power)) =
        arrow.iter_mut().next()
    else {
        return;
    };

    let Some(drag_info) = **drag_info else {
        if let Visibility::Visible = *arrow_visibility {
            *arrow_visibility = Visibility::Hidden;
        }
        return;
    };

    if let Visibility::Hidden = *arrow_visibility {
        *arrow_visibility = Visibility::Visible;
    }

    let Some(player_pos) = camera.world_to_viewport(camera_transform, player_transform.translation)
    else {
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

    arrow_power.power = power;

    if power <= 0.0 {
        if let Visibility::Visible = *arrow_visibility {
            *arrow_visibility = Visibility::Hidden;
        }
        return;
    }

    let Some(color_material) =
        materials.get_mut(game_assets.get_material(GameMaterial::AimArrowBody))
    else {
        return;
    };

    color_material.base_color = get_power_color(power);

    let Some((arrow_point, normal)) = get_contact_position(player_entity, &rapier_context) else {
        return;
    };

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
    *arrow_transform = player_transform
        .with_rotation(Quat::from_rotation_z(angle))
        .with_scale(Vec3::splat(0.55.lerp(1.15, power)));

    // Clip is played in 24fps at 1.0 speed
    // Duration is 60 frames = 2.5 seconds
    const DURATION: f32 = 2.5;
    let power = power * DURATION;

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
        if press.target == player_entity && player.shots > 0 {
            let Some(position) = user_input_position.get(*press.user_input) else {
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
        return;
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
    };

    if user_input.just_released(drag_info_data.user_input) {
        drag_info_data.confirmed = true;
        return;
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
            &drag_info_data,
            &arrow,
            camera,
            camera_transform,
            &rapier_context,
        ) {
            log!("Player shot with power: {:?}", new_impulse);
            *impulse = new_impulse;
        }
        **drag_info = None;
    }
}

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
    let (player_point, normal) = get_contact_position(entity, rapier_context)?;
    let ray = camera.viewport_to_world(camera_transform, drag_info_data.point)?;
    let distance = ray.intersect_plane(player_point, Plane3d::new(normal))?;
    let point = ray.get_point(distance);
    player.shots -= 1;
    let push = (player_point - point).normalize() * arrow.power * 250.0;

    Some(ExternalImpulse::at_point(
        push,
        transform.translation,
        transform.translation,
    ))
}

fn get_contact_position(
    player_entity: Entity,
    rapier_context: &Res<RapierContext>,
) -> Option<(Vec3, Vec3)> {
    let Some(contact_pair) = rapier_context.contact_pairs_with(player_entity).next() else {
        return None;
    };

    let other = if contact_pair.collider1() == player_entity {
        1
    } else {
        2
    };

    let Some(manifold) = contact_pair.manifold(0) else {
        return None;
    };
    let Some(point) = manifold.point(0) else {
        return None;
    };
    let point = if other == 1 {
        point.local_p2()
    } else {
        point.local_p1()
    };

    Some((point + Vec3::Z * 0.001, manifold.normal()))
}
