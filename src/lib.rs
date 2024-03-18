use crate::main_menu::main_menu_plugin::MainMenuPlugin;
use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowMode};
use game::game_plugin::GamePlugin;
use resources::Fonts;

pub mod game;
mod main_menu;
mod resources;

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
    let mut app = app();
    #[cfg(target_arch = "wasm32")]
    app.add_systems(Update, update_resolution);
    app.run();
}

fn app() -> App {
    let mut app = App::new();
    app.insert_resource(AssetMetaCheck::Never)
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
        .add_plugins(GamePlugin);

    app
}

#[cfg(target_arch = "wasm32")]
fn update_resolution(
    mut windows: Query<&mut Window, Changed<Window>>,
    mut applied: Local<bevy::window::WindowResolution>,
) {
    if let Some(mut window) = windows.iter_mut().next() {
        if *applied == window.resolution {
            return;
        }
        *applied = window.resolution.clone();
        let size = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .get_bounding_client_rect();
        let scale_factor = window.scale_factor();
        window.resolution.set_physical_resolution(
            (size.width() as f32 * scale_factor) as u32,
            (size.height() as f32 * scale_factor) as u32,
        );
    }
}

fn get_window_mode() -> WindowMode {
    #[cfg(target_arch = "wasm32")]
    return WindowMode::Fullscreen;
    #[cfg(not(target_arch = "wasm32"))]
    return WindowMode::default();
}
