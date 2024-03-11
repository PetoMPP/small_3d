use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use game::plugin::GamePlugin;
use crate::main_menu::plugin::MainMenuPlugin;

pub mod game;
mod main_menu;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Fonts>()
        .init_state::<AppState>()
        .add_plugins(MainMenuPlugin)
        .add_plugins(GamePlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .run();
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}

#[derive(Resource, Clone)]
pub struct Fonts {
    pub regular: Handle<Font>,
    pub bold: Handle<Font>,
    pub italic: Handle<Font>,
    pub italic_bold: Handle<Font>,
}

impl FromWorld for Fonts {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            regular: asset_server.load("fonts/OpenSans/OpenSans-Regular.ttf"),
            bold: asset_server.load("fonts/OpenSans/OpenSans-Bold.ttf"),
            italic: asset_server.load("fonts/OpenSans/OpenSans-Italic.ttf"),
            italic_bold: asset_server.load("fonts/OpenSans/OpenSans-BoldItalic.ttf"),
        }
    }
}
