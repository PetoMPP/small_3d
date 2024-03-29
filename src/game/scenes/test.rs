use crate::{
    game::{
        components::{GameEntity, Ground},
        plugins::{game_scene_plugin::Scene, player_plugin::spawn_player},
    },
    resources::game_assets::GameAssets,
};
use bevy::prelude::*;
use bevy_picking_rapier::bevy_rapier3d::prelude::*;

pub struct TestLevel;

pub static TEST_LEVEL: TestLevel = TestLevel;

impl Scene for TestLevel {
    fn start_pos(&self) -> Vec3 {
        Vec3::Z
    }

    fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        game_assets: &Res<GameAssets>,
    ) {
        // Platform
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 1.0, 0.1)),
                material: materials.add(Color::BLUE),
                ..Default::default()
            })
            .insert(GameEntity)
            .insert((
                Collider::cuboid(0.5, 0.5, 0.05),
                Friction::coefficient(1.0),
                ColliderMassProperties::Mass(1000.0),
            ));

        // Ground
        // Base
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Circle::new(500.0)),
                material: materials.add(Color::WHITE),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
                ..Default::default()
            })
            .insert((GameEntity, Ground::default()))
            .insert((
                Collider::cuboid(500.0, 500.0, 0.0),
                Friction::coefficient(1.0),
                Restitution::new(0.8),
                ColliderMassProperties::Mass(1000.0),
                ActiveEvents::COLLISION_EVENTS,
            ));

        // North axis
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.05, 500.0, 0.0)),
                material: materials.add(Color::CRIMSON),
                transform: Transform::from_translation(Vec3::new(0.0, 250.0, 0.002 - 10.0)),
                ..Default::default()
            },
            GameEntity,
        ));
        // Center
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Circle::new(1.0)),
                material: materials.add(Color::BLACK),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.001 - 10.0)),
                ..Default::default()
            },
            GameEntity,
        ));

        spawn_player(commands, game_assets, self.start_pos());
    }
}
