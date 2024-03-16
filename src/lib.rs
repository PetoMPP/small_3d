use crate::main_menu::main_menu_plugin::MainMenuPlugin;
use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowMode};
use game::game_plugin::GamePlugin;
use resources::Fonts;

pub mod game;
mod main_menu;
mod resources;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}

pub fn run() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Game".to_string(),
                mode: get_window_mode(),
                canvas: Some("#canvas".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .init_resource::<Fonts>()
        .init_state::<AppState>()
        .add_plugins(MainMenuPlugin)
        .add_plugins(GamePlugin)
        .run();
}

fn get_window_mode() -> WindowMode {
    #[cfg(target_arch = "wasm32")]
    return WindowMode::Fullscreen;

    WindowMode::default()
}
