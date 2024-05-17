use super::loadable::Loadable;
use crate::game::plugins::{
    aiming_plugin::ArrowAnimationPlayer,
    game_scene_plugin::{GameData, GameSceneAnimationPlayer},
};
use bevy::{
    asset::UntypedAssetId, prelude::*, render::render_resource::Face, utils::hashbrown::HashMap,
};
use bevy_rapier3d::{
    dynamics::{Ccd, RigidBody},
    geometry::ColliderMassProperties,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameScene {
    Player,
    AimArrow,
    Level(GameLevel),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameMaterial {
    AimArrowBody,
    AimArrowBorder,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
pub enum GameLevel {
    Demo,
}

pub struct GameLevelMeta {
    pub shots: u32,
    pub star_point_thresholds: [u32; 3],
}

impl Into<GameData> for GameLevel {
    fn into(self) -> GameData {
        let meta = self.get_meta();
        GameData {
            level: Some(self),
            shots: meta.shots,
            points: 0,
        }
    }
}

impl GameLevel {
    pub fn get_filename(&self) -> &str {
        match self {
            Self::Demo => "models/levels/demo.glb",
        }
    }

    pub fn get_meta(&self) -> GameLevelMeta {
        match self {
            Self::Demo => GameLevelMeta {
                shots: 3,
                star_point_thresholds: [25, 50, 75],
            },
        }
    }
}

pub trait GameAnimationSource {
    fn get_animation_filename(&self) -> &str;
}

#[derive(Resource, Clone)]
pub struct GameAssets {
    scenes: HashMap<GameScene, Handle<Scene>>,
    animations: HashMap<String, HashMap<Name, Handle<AnimationClip>>>,
    materials: HashMap<GameMaterial, Handle<StandardMaterial>>,
}

impl GameAssets {
    pub fn get_scene(&self, asset: GameScene) -> Handle<Scene> {
        self.scenes[&asset].clone_weak()
    }

    pub fn get_next_animation(
        &mut self,
        asset_name: &Name,
        asset_source: &impl GameAnimationSource,
        asset_server: &AssetServer,
    ) -> Handle<AnimationClip> {
        let filename = asset_source.get_animation_filename();
        let Some(animations) = self.animations.get_mut(filename) else {
            let mut animations = HashMap::default();
            animations.insert(
                asset_name.clone(),
                asset_server.load(format!("{}#Animation0", filename)),
            );
            self.animations.insert(filename.to_string(), animations);
            return self.animations[filename][asset_name].clone_weak();
        };
        if let Some(animation) = animations.get(asset_name) {
            return animation.clone_weak();
        }
        let next_index = animations.len();
        animations.insert(
            asset_name.clone(),
            asset_server.load(format!("{}#Animation{}", filename, next_index)),
        );
        animations[asset_name].clone_weak()
    }

    pub fn get_material(&self, asset: GameMaterial) -> Handle<StandardMaterial> {
        self.materials[&asset].clone_weak()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn init_assets_system(
        mut commands: Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
        new_base_materials: Query<&Handle<StandardMaterial>, Added<Handle<StandardMaterial>>>,
        mut new_arrow_animations: Query<
            (&Name, &mut AnimationPlayer),
            (
                Added<ArrowAnimationPlayer>,
                Without<GameSceneAnimationPlayer>,
            ),
        >,
        mut new_game_scene_animations: Query<
            (Entity, &Name, &mut AnimationPlayer),
            Added<GameSceneAnimationPlayer>,
        >,
        mut game_assets: ResMut<GameAssets>,
        asset_server: Res<AssetServer>,
        game_data: Res<GameData>,
        mut material_queue: Local<HashMap<Handle<StandardMaterial>, u64>>,
    ) {
        for handle in new_base_materials.iter() {
            material_queue.insert(handle.clone_weak(), 12);
        }

        let mut to_remove = Vec::new();
        for (handle, remaining) in material_queue.iter_mut() {
            *remaining -= 1;
            if *remaining > 0 {
                continue;
            }

            to_remove.push(handle.clone_weak());
            let Some(material) = materials.get_mut(handle) else {
                continue;
            };

            material.reflectance = 0.0;
            material.double_sided = *handle
                == game_assets.get_material(GameMaterial::AimArrowBorder)
                || *handle == game_assets.get_material(GameMaterial::AimArrowBody);
            if material.base_color.a() < 1.0 {
                material.cull_mode = Some(Face::Front);
                material.alpha_mode = AlphaMode::Blend;
            }
        }

        for handle in to_remove {
            material_queue.remove(&handle);
        }

        for (name, mut player) in new_arrow_animations.iter_mut() {
            player.play(game_assets.get_next_animation(name, &ArrowAnimationPlayer, &asset_server));
            player.pause();
        }
        for (entity, name, mut player) in new_game_scene_animations.iter_mut() {
            player.play(game_assets.get_next_animation(
                name,
                &GameSceneAnimationPlayer(game_data.level.unwrap()),
                &asset_server,
            ));
            player.repeat();
            commands.entity(entity).insert((
                Ccd::enabled(),
                RigidBody::KinematicPositionBased,
                ColliderMassProperties::Density(100.0),
            ));
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
            .map(Into::<UntypedAssetId>::into)
            .chain(
                self.animations
                    .values()
                    .flat_map(|v| v.values().map(Into::into)),
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
        scenes.insert(
            GameScene::Level(GameLevel::Demo),
            asset_server.load(format!("{}#Scene0", GameLevel::Demo.get_filename())),
        );
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
            animations: Default::default(),
            materials,
        }
    }
}
