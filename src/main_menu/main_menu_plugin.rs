use super::plugins::loading_view_plugin::LoadingViewPlugin;
use crate::common::plugins::ui_plugin::components::{
    UiBase, UiBuilder, UiButton, UiComponent, UiContainer,
};
use crate::common::plugins::ui_plugin::UiOnClick;
use crate::common::plugins::user_input_plugin::UserInput;
use crate::game::game_plugin::GameState;
use crate::resources::game_assets::{GameAssets, GameColor};
use crate::resources::loadable::Loadable;
use crate::resources::text_styles::{FontSize, FontType};
use crate::{AppState, TextStyles};
use bevy::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LoadingViewPlugin)
            .add_systems(OnEnter(AppState::MainMenu), init_main_menu)
            .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu);
    }
}

#[derive(Component)]
struct MenuNode;

fn init_main_menu(mut commands: Commands, mut ui_builder: UiBuilder) {
    // TODO: Background scene'
    let base = UiBase::new(ui_builder.game_assets.colors.get(GameColor::Base));
    let title = TextBundle {
        text: Text::from_section(
            "Small 3D",
            ui_builder.text_styles.get(
                FontType::Bold,
                FontSize::XLarge,
                ui_builder.game_assets.colors.get_content(GameColor::Base),
            ),
        )
        .with_justify(JustifyText::Center),
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Vh(10.0),
            ..Default::default()
        },
        ..Default::default()
    };
    let menu: UiContainer = ui_builder.create(Val::Auto, Val::Auto);
    let buttons = vec![ui_builder
        .create::<UiButton>(Val::Auto, Val::Auto)
        .with_text("Play")
        .with_on_click(UiOnClick::new(set_in_game))];

    base.spawn(&mut commands)
        .insert(MenuNode)
        .with_children(|parent| {
            parent.spawn(title);
            menu.spawn(parent).with_children(|parent| {
                for button in buttons {
                    button.spawn(parent);
                }
            });
        });
}

fn set_in_game(world: &mut World, _user_input: Option<(&UserInput, Vec2)>) {
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

fn cleanup_main_menu(mut commands: Commands, node: Query<Entity, With<MenuNode>>) {
    commands.entity(node.single()).despawn_recursive();
}
