use super::loadable::Loadable;
use crate::game::plugins::aiming_plugin::ArrowAnimationPlayer;
use bevy::{asset::UntypedAssetId, prelude::*, utils::HashMap};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameScene {
    Player,
    AimArrow,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameAnimation {
    AimArrow,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameMaterial {
    AimArrowBody,
    AimArrowBorder,
}

#[derive(Resource, Clone)]
pub struct GameAssets {
    scenes: HashMap<GameScene, Handle<Scene>>,
    animations: HashMap<GameAnimation, Vec<Handle<AnimationClip>>>,
    materials: HashMap<GameMaterial, Handle<StandardMaterial>>,
}

impl GameAssets {
    pub fn get_scene(&self, asset: GameScene) -> Handle<Scene> {
        self.scenes[&asset].clone_weak()
    }

    pub fn get_animations(&self, asset: GameAnimation) -> Vec<Handle<AnimationClip>> {
        self.animations[&asset].clone()
    }

    pub fn get_material(&self, asset: GameMaterial) -> Handle<StandardMaterial> {
        self.materials[&asset].clone_weak()
    }

    pub fn get_id_by_animation_handle(
        &self,
        handle: &Handle<AnimationClip>,
    ) -> Option<GameAnimation> {
        self.animations
            .iter()
            .find_map(|(k, v)| if v.contains(handle) { Some(*k) } else { None })
    }

    pub fn init_assets_system(
        mut commands: Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
        new_base_materials: Query<&Handle<StandardMaterial>, Added<Handle<StandardMaterial>>>,
        mut new_animations: Query<
            (Entity, &Handle<AnimationClip>, &mut AnimationPlayer),
            Added<AnimationPlayer>,
        >,
        game_assets: Res<GameAssets>,
    ) {
        for handle in new_base_materials.iter() {
            let id: UntypedAssetId = handle.into();
            if game_assets.into_iter().any(|h| h == id) {
                continue;
            }
            let Some(material) = materials.get_mut(handle) else {
                continue;
            };
            material.reflectance = 0.0;
        }

        for (entity, handle, mut player) in new_animations.iter_mut() {
            let Some(game_animation) = game_assets.get_id_by_animation_handle(handle) else {
                continue;
            };
            player.play(handle.clone_weak());
            player.pause();
            if let GameAnimation::AimArrow = game_animation {
                commands.entity(entity).insert(ArrowAnimationPlayer);
            }
        }
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
                    .flat_map(|h| h.iter().map(|h| Into::<UntypedAssetId>::into(h))),
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
        let mut arrow_animations = Vec::new();
        arrow_animations.push(asset_server.load("models/arrow.glb#Animation1"));
        arrow_animations.push(asset_server.load("models/arrow.glb#Animation0"));
        arrow_animations.push(asset_server.load("models/arrow.glb#Animation2"));
        arrow_animations.push(asset_server.load("models/arrow.glb#Animation3"));
        arrow_animations.push(asset_server.load("models/arrow.glb#Animation4"));
        animations.insert(GameAnimation::AimArrow, arrow_animations);
        let mut materials = HashMap::default();
        materials.insert(
            GameMaterial::AimArrowBody,
            asset_server.load("models/arrow.glb#Material1"),
        );
        materials.insert(
            GameMaterial::AimArrowBorder,
            asset_server.load("models/arrow.glb#Material0"),
        );
        Self {
            scenes,
            animations,
            materials,
        }
    }
}
