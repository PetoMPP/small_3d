use crate::common::plugins::user_input_plugin::UserInput;
use crate::resources::game_assets::GameAssets;
use crate::resources::loadable::Loadable;
use crate::resources::{
    inputs::Inputs,
    text_styles::{FontSize, FontType},
};
use crate::{AppState, TextStyles};
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), init_main_menu)
            .add_systems(
                Update,
                (update_text, set_in_game).run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu);
    }
}

fn update_text(
    mut query: Query<(&mut Text, &MenuText)>,
    asset_server: Res<AssetServer>,
    game_assets: Res<GameAssets>,
    text_styles: Res<TextStyles>,
) {
    let Some((mut text, _)) = query.iter_mut().next() else {
        return;
    };

    if text.sections[0].value.as_str() == READY_TEXT {
        return;
    }

    if !game_assets.loaded(&asset_server) || !text_styles.loaded(&asset_server) {
        return;
    }

    text.sections[0].value = READY_TEXT.to_string();
}

fn set_in_game(
    mut next_state: ResMut<NextState<AppState>>,
    mut key_event: EventReader<KeyboardInput>,
    user_input: Res<Inputs<UserInput>>,
    asset_server: Res<AssetServer>,
    game_assets: Res<GameAssets>,
    text_styles: Res<TextStyles>,
) {
    if key_event.read().next().is_none() && user_input.iter_just_pressed().next().is_none() {
        return;
    };
    if !game_assets.loaded(&asset_server) || !text_styles.loaded(&asset_server) {
        return;
    }
    next_state.set(AppState::InGame);
}

#[derive(Component)]
struct MenuCamera;

#[derive(Component)]
struct MenuNode;

#[derive(Component)]
struct MenuText;

const READY_TEXT: &str = "Press any key to start";
const LOADING_TEXT: &str = "Loading..";

fn init_main_menu(mut commands: Commands, text_styles: Res<TextStyles>) {
    commands.spawn((Camera2dBundle::default(), MenuCamera));
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            MenuNode,
        ))
        .with_children(|root| {
            root.spawn((
                TextBundle {
                    text: Text::from_section(
                        LOADING_TEXT,
                        text_styles.get(FontType::Regular, FontSize::Large, Color::WHITE),
                    ),
                    ..Default::default()
                },
                MenuText,
            ));
        });
}

fn cleanup_main_menu(
    mut commands: Commands,
    node: Query<Entity, With<MenuNode>>,
    camera: Query<Entity, With<MenuCamera>>,
) {
    commands.entity(node.single()).despawn_recursive();
    commands.entity(camera.single()).despawn_recursive();
}
