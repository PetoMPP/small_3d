use bevy::prelude::*;
use std::f32::{consts::PI, EPSILON};

#[derive(Component, Clone, Copy)]
pub struct GameEntity;

#[derive(Component)]
pub struct GameCamera {
    distance: f32,
    offset: Vec2,
}

impl GameCamera {
    pub fn get_distance(&self) -> f32 {
        self.distance
    }

    pub fn distance(&mut self, distance: f32) {
        self.distance = distance.clamp(1.0, 25.0);
    }

    pub fn get_offset(&self) -> Vec2 {
        self.offset
    }

    pub fn offset(&mut self, offset: Vec2) {
        self.offset = Vec2::new(
            self.offset.x + offset.x,
            (self.offset.y + offset.y).clamp(0.0 + EPSILON, PI - EPSILON),
        );
    }
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
