use crate::common::plugins::user_input_plugin::UserInputPlugin;
use crate::main_menu::main_menu_plugin::MainMenuPlugin;
use bevy::{asset::AssetMetaCheck, prelude::*};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_vector_shapes::ShapePlugin;
use common::plugins::ui_plugin::UiPlugin;
use game::game_plugin::GamePlugin;
use resources::{resources_plugin::ResourcesPlugin, text_styles::TextStyles};

mod common;
mod game;
mod main_menu;
mod resources;
mod utils;

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let res = std::fmt::format(format_args!($($arg)*));
        #[cfg(target_arch = "wasm32")]
        gloo::console::log!(res);
        #[cfg(not(target_arch = "wasm32"))]
        println!("{}", res);
    }}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}

pub fn run() {
    app().run();
}

fn app() -> App {
    let mut app = App::new();
    app.insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(get_window()),
            ..Default::default()
        }))
        .init_state::<AppState>()
        // .add_plugins(WorldInspectorPlugin::default())
        .add_plugins(UiPlugin)
        .add_plugins(ShapePlugin::default())
        .add_plugins(ResourcesPlugin)
        .add_plugins(MainMenuPlugin)
        .add_plugins(GamePlugin)
        .add_plugins(UserInputPlugin);

    app
}

fn get_window() -> Window {
    #[cfg(target_arch = "wasm32")]
    return Window {
        title: "Small 3D".to_string(),
        canvas: Some("#canvas".to_string()),
        resolution: {
            let root = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .body()
                .unwrap();
            let width = root.client_width() as f32;
            let height = root.client_height() as f32;
            bevy::window::WindowResolution::new(width, height)
        },
        ..Default::default()
    };
    #[cfg(not(target_arch = "wasm32"))]
    return Window {
        title: "Small 3D".to_string(),
        ..Default::default()
    };
}
