use super::{game_assets::GameAssets, text_styles::TextStyles};
use bevy::prelude::*;

pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextStyles>()
            .init_resource::<GameAssets>();
    }
}
