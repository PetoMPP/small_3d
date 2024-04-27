use crate::{game::game_plugin::GameRunningState, AppState};
use bevy::{prelude::*, utils::HashMap};
use bevy_tweening::{component_animator_system, Animator, Lens};
use std::f32::consts::PI;

pub struct CustomTweeningPlugin;

impl Plugin for CustomTweeningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                component_animator_system::<RelativeScale>.before(update_scale),
                update_scale,
                component_animator_system::<Rotation>.before(update_rotation),
                update_rotation,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            OnEnter(GameRunningState(false)),
            (
                pause_game_tweening::<RelativeScale>,
                pause_game_tweening::<Rotation>,
            ),
        )
        .add_systems(
            OnEnter(GameRunningState(true)),
            (
                resume_game_tweening::<RelativeScale>,
                resume_game_tweening::<Rotation>,
            ),
        );
    }
}

#[derive(Component, Clone, Copy)]
pub struct GameTween;

#[derive(Component, Clone, Copy)]
pub struct RelativeScale(Vec3);

impl Default for RelativeScale {
    fn default() -> Self {
        Self(Vec3::ONE)
    }
}

pub struct RelativeScaleLens {
    pub start: Vec3,
    pub end: Vec3,
}

impl Lens<RelativeScale> for RelativeScaleLens {
    fn lerp(&mut self, target: &mut RelativeScale, ratio: f32) {
        target.0 = self.start.lerp(self.end, ratio);
    }
}

pub fn update_scale(
    mut relative_scales: Query<(Entity, &RelativeScale, &mut Transform)>,
    mut initial_map: Local<HashMap<Entity, Vec3>>,
) {
    for (entity, relative_scale, mut transform) in relative_scales.iter_mut() {
        let initial_scale = *initial_map.entry(entity).or_insert_with(|| transform.scale);
        if initial_scale == Vec3::ZERO {
            unreachable!("Initial transform scale is zero");
        }
        if relative_scale.0 == Vec3::ZERO {
            unreachable!("Size tween is zero");
        }
        transform.scale = initial_scale * relative_scale.0;
    }
}

#[derive(Component, Clone, Copy)]
pub struct Rotation {
    angle: f32,
    axis: Vec3,
}

impl Rotation {
    pub fn new(axis: Vec3) -> Self {
        if !axis.is_normalized() {
            unreachable!("Rotation axis must be normalized");
        }
        Self { axis, angle: 0.001 }
    }
}

pub struct RotationLens;

impl Lens<Rotation> for RotationLens {
    fn lerp(&mut self, target: &mut Rotation, ratio: f32) {
        target.angle = 0.001.lerp(2.0 * PI - 0.001, ratio);
    }
}

pub fn update_rotation(
    mut rotations: Query<(Entity, &Rotation, &mut Transform)>,
    mut initial_map: Local<HashMap<Entity, Quat>>,
) {
    for (entity, rotation, mut transform) in rotations.iter_mut() {
        let initial_rotation = *initial_map
            .entry(entity)
            .or_insert_with(|| transform.rotation);
        transform.rotation = initial_rotation;
        let point = transform.translation;
        transform.rotate_around(point, Quat::from_axis_angle(rotation.axis, rotation.angle));
    }
}

fn pause_game_tweening<T: Component>(mut game_tweens: Query<&mut Animator<T>, With<GameTween>>) {
    for mut animator in game_tweens.iter_mut() {
        animator.set_speed(0.0);
    }
}

fn resume_game_tweening<T: Component>(mut game_tweens: Query<&mut Animator<T>, With<GameTween>>) {
    for mut animator in game_tweens.iter_mut() {
        animator.set_speed(1.0);
    }
}
