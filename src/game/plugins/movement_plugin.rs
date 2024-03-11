use crate::AppState;
use bevy::prelude::*;

pub const MOVE_TICK: f32 = 1.0 / 60.0;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_entities.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component, Default)]
pub struct Velocity {
    pub linear: f32,
    pub angular: Quat,
}

fn move_entities(mut entities: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in entities.iter_mut() {
        if velocity.angular != Quat::IDENTITY {
            transform.rotate(velocity.angular);
        }
        if velocity.linear == 0.0 {
            continue;
        }

        let rotation = transform.rotation;
        transform.translation += rotation * Vec3::X * velocity.linear * MOVE_TICK;
    }
}
