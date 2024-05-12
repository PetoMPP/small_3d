use crate::common::plugins::ui_plugin::{styles, UiOnClick, UiOnClickBundle};
use crate::game::game_plugin::GameState;
use crate::resources::game_assets::GameAssets;
use crate::resources::loadable::Loadable;
use crate::resources::text_styles::{FontSize, FontType};
use crate::{AppState, TextStyles};
use bevy::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), init_main_menu)
            .add_systems(Update, (update_text).run_if(in_state(AppState::MainMenu)))
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
                style: styles::container(Val::Percent(100.0), Val::Percent(100.0)),
                background_color: Color::rgb_from_array([0.2, 0.2, 0.2]).into(),
                ..default()
            },
            MenuNode,
            UiOnClickBundle {
                ui_on_click: UiOnClick(set_in_game),
                ..Default::default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                TextBundle {
                    text: Text::from_section(
                        LOADING_TEXT,
                        text_styles.get(FontType::Bold, FontSize::Large, Color::WHITE),
                    ),
                    ..Default::default()
                },
                MenuText,
            ));
        });
}

fn set_in_game(world: &mut World) {
    let asset_server = world.get_resource::<AssetServer>().unwrap();
    let game_assets = world.get_resource::<GameAssets>().unwrap();
    let text_styles = world.get_resource::<TextStyles>().unwrap();

    if !game_assets.loaded(&asset_server) || !text_styles.loaded(&asset_server) {
        return;
    }
    world
        .resource_mut::<NextState<AppState>>()
        .set(AppState::InGame);
    world
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
}

fn cleanup_main_menu(
    mut commands: Commands,
    node: Query<Entity, With<MenuNode>>,
    camera: Query<Entity, With<MenuCamera>>,
) {
    commands.entity(node.single()).despawn_recursive();
    commands.entity(camera.single()).despawn_recursive();
}
