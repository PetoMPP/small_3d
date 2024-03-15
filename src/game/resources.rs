use bevy::prelude::*;

#[derive(Resource, Default, Clone, Deref, DerefMut)]
pub struct GameScene(pub Option<GameSceneData>);

#[derive(Resource, Clone, Copy, Deref, DerefMut)]
pub struct GameSceneData(pub &'static dyn Scene);

pub trait Scene: Send + Sync {
    fn start_pos(&self) -> Vec3;

    fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        asset_server: &AssetServer,
    );
}
