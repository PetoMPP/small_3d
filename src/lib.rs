use crate::main_menu::main_menu_plugin::MainMenuPlugin;
use bevy::prelude::*;
use game::game_plugin::GamePlugin;
use resources::Fonts;

pub mod game;
mod main_menu;
mod resources;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Fonts>()
        .init_state::<AppState>()
        .add_plugins(MainMenuPlugin)
        .add_plugins(GamePlugin)
        .run();
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}
