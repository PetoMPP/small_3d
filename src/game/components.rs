use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Component, Clone, Copy)]
pub struct GameEntity;

#[derive(Component)]
pub struct GameCamera {
    pub distance: f32,
    pub offset: Vec2,
}

impl Default for GameCamera {
    fn default() -> Self {
        Self {
            distance: 5.0,
            offset: Vec2::new(0.0, PI / 4.0),
        }
    }
}

#[derive(Component)]
pub struct GameLight;

#[derive(Component, Deref, DerefMut)]
pub struct Ground(pub Timer);

impl Default for Ground {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.75, TimerMode::Once);
        timer.pause();
        Self(timer)
    }
}

#[derive(Component)]
pub struct Player {
    pub shots: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self { shots: 1 }
    }
}
