use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Component, Clone, Copy)]
pub struct GameEntity;

#[derive(Component)]
pub struct GameCamera {
    distance: f32,
    offset: Vec2,
}

#[derive(Component)]
pub struct GameUiCamera;

impl GameCamera {
    pub fn get_distance(&self) -> f32 {
        self.distance
    }

    pub fn distance(&mut self, distance: f32) {
        self.distance = (self.distance + distance).clamp(4.0, 8.0);
    }

    pub fn get_offset(&self) -> Vec2 {
        self.offset
    }

    pub fn offset(&mut self, offset: Vec2) {
        self.offset = Vec2::new(
            self.offset.x + offset.x,
            (self.offset.y + offset.y).clamp(0.0 + 0.001, PI - 0.001),
        );
    }
}

impl Default for GameCamera {
    fn default() -> Self {
        Self {
            distance: 6.0,
            offset: Vec2::new(0.0, PI / 3.0),
        }
    }
}

#[derive(Component)]
pub struct GameLight;

#[derive(Component)]
pub struct Player {
    pub shots: u32,
    pub points: i32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            shots: 10,
            points: 0,
        }
    }
}
