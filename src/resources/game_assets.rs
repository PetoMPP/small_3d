use bevy::{asset::UntypedAssetId, prelude::*, utils::HashMap};

use super::loadable::Loadable;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameScene {
    Player,
    AimArrow,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameAnimation {
    AimArrow,
}

#[derive(Resource, Clone)]
pub struct GameAssets {
    scenes: HashMap<GameScene, Handle<Scene>>,
    animations: HashMap<GameAnimation, Handle<AnimationClip>>,
}

impl GameAssets {
    pub fn get_scene(&self, asset: GameScene) -> Handle<Scene> {
        self.scenes[&asset].clone_weak()
    }

    pub fn get_animation(&self, asset: GameAnimation) -> Handle<AnimationClip> {
        self.animations[&asset].clone_weak()
    }
}

impl Loadable for GameAssets {
    fn loaded(&self, asset_server: &AssetServer) -> bool {
        self.into_iter()
            .all(|id| asset_server.is_loaded_with_dependencies(id))
    }
}

impl IntoIterator for &GameAssets {
    type Item = UntypedAssetId;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.scenes
            .values()
            .map(|h| Into::<UntypedAssetId>::into(h))
            .chain(
                self.animations
                    .values()
                    .map(|h| Into::<UntypedAssetId>::into(h)),
            )
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let mut scenes = HashMap::default();
        scenes.insert(
            GameScene::Player,
            asset_server.load("models/player.glb#Scene0"),
        );
        scenes.insert(
            GameScene::AimArrow,
            asset_server.load("models/arrow.glb#Scene0"),
        );
        let mut animations = HashMap::default();
        animations.insert(
            GameAnimation::AimArrow,
            asset_server.load("models/arrow.glb#Animation0"),
        );
        Self { scenes, animations }
    }
}
