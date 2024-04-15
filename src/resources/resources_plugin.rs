use super::{game_assets::GameAssets, random::Random, text_styles::TextStyles};
use bevy::prelude::*;

pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextStyles>()
            .init_resource::<GameAssets>()
            .init_non_send_resource::<Random>()
            .add_systems(Update, GameAssets::init_assets_system);
    }
}
