use crate::game::{
    components::{GameEntity, Ground},
    plugins::player_plugin::spawn_player,
    resources::Scene,
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
        asset_server: &AssetServer,
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
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Circle::new(500.0)),
                material: materials.add(Color::CRIMSON),
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

        spawn_player(commands, asset_server, self.start_pos());
    }
}
