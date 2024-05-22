use super::plugins::loading_view_plugin::LoadingViewPlugin;
use crate::common::plugins::ui_plugin::components::{
    UiBase, UiBuilder, UiButton, UiComponent, UiContainer,
};
use crate::common::plugins::ui_plugin::{UiCommandContext, UiOnClick, UiPointerEventData};
use crate::game::game_plugin::GameState;
use crate::game::plugins::game_scene_plugin::GameData;
use crate::resources::game_assets::{GameAssets, GameColor, GameLevel};
use crate::resources::loadable::Loadable;
use crate::resources::text_styles::{FontSize, FontType};
use crate::{AppState, TextStyles};
use bevy::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MenuNode>()
            .register_type::<PlayNode>()
            .register_type::<SettingsNode>()
            .add_plugins(LoadingViewPlugin)
            .add_systems(OnEnter(AppState::MainMenu), init_main_menu)
            .add_systems(Update, update_menu)
            .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu);
    }
}

#[derive(Component, Default, Reflect)]
enum MenuNode {
    #[default]
    Root,
    Play(PlayNode),
    Settings(SettingsNode),
}

impl MenuNode {
    fn go_back(&mut self) {
        *self = match self {
            MenuNode::Root
            | MenuNode::Play(PlayNode::Root)
            | MenuNode::Settings(SettingsNode::Root) => MenuNode::Root,
            MenuNode::Play(_) => MenuNode::Play(PlayNode::Root),
            MenuNode::Settings(_) => MenuNode::Settings(SettingsNode::Root),
        }
    }
}

#[derive(Resource, Reflect)]
enum PlayNode {
    Root,
    LevelSelect,
    Customize,
    // Shop
    Achievements,
}

#[derive(Resource, Reflect)]
enum SettingsNode {
    Root,
    Graphics,
    Audio,
    Controls,
}

#[derive(Component)]
struct MenuContainer;

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

    base.spawn(&mut commands)
        .insert(MenuNode::default())
        .with_children(|parent| {
            parent.spawn(title);
            menu.spawn(parent).insert(MenuContainer);
        });
}

fn update_menu(
    mut commands: Commands,
    node: Query<&MenuNode, Changed<MenuNode>>,
    container: Query<Entity, With<MenuContainer>>,
    mut ui_builder: UiBuilder,
) {
    let Some((node, container)) = node
        .iter()
        .next()
        .and_then(|n| container.iter().next().map(|c| (n, c)))
    else {
        return;
    };

    commands.entity(container).despawn_descendants();
    match node {
        MenuNode::Root => {
            spawn_root(&mut commands, &mut ui_builder, container);
            return;
        }
        MenuNode::Play(play_node) => match play_node {
            PlayNode::Root => spawn_play_root(&mut commands, &mut ui_builder, container),
            PlayNode::LevelSelect => spawn_level_select(&mut commands, &mut ui_builder, container),
            PlayNode::Customize => {}
            PlayNode::Achievements => {}
        },
        MenuNode::Settings(settings_node) => match settings_node {
            SettingsNode::Root => {}
            SettingsNode::Graphics => {}
            SettingsNode::Audio => {}
            SettingsNode::Controls => {}
        },
    }

    let back_button = ui_builder
        .create::<UiButton>(Val::Auto, Val::Auto)
        .with_text("Go back")
        .with_on_click(UiOnClick::new(|w, _| {
            w.query::<&mut MenuNode>().single_mut(w).go_back();
        }));

    commands.entity(container).with_children(|parent| {
        back_button.spawn(parent);
    });
}

fn spawn_level_select(commands: &mut Commands, ui_builder: &mut UiBuilder, container: Entity) {
    let title = TextBundle {
        text: Text::from_section(
            "Level select",
            ui_builder.text_styles.get(
                FontType::Regular,
                FontSize::Large,
                ui_builder
                    .game_assets
                    .colors
                    .get_content(GameColor::Primary),
            ),
        )
        .with_justify(JustifyText::Center),
        ..Default::default()
    };
    let buttons = vec![ui_builder
        .create::<UiButton>(Val::Auto, Val::Auto)
        .with_text("Demo")
        .with_on_click(UiOnClick::new(|w, ctx| {
            w.resource_mut::<GameData>().level = Some(GameLevel::Demo);
            set_in_game(w, ctx);
        }))];

    commands.entity(container).with_children(|parent| {
        parent.spawn(title);
        for button in buttons.into_iter() {
            button.spawn(parent);
        }
    });
}

fn spawn_play_root(commands: &mut Commands, ui_builder: &mut UiBuilder, container: Entity) {
    let title = TextBundle {
        text: Text::from_section(
            "Play",
            ui_builder.text_styles.get(
                FontType::Regular,
                FontSize::Large,
                ui_builder
                    .game_assets
                    .colors
                    .get_content(GameColor::Primary),
            ),
        )
        .with_justify(JustifyText::Center),
        ..Default::default()
    };
    let buttons = vec![
        ui_builder
            .create::<UiButton>(Val::Auto, Val::Auto)
            .with_text("Level select")
            .with_on_click(UiOnClick::new(move |w, _| {
                *w.query::<&mut MenuNode>().single_mut(w) = MenuNode::Play(PlayNode::LevelSelect);
            })),
        ui_builder
            .create::<UiButton>(Val::Auto, Val::Auto)
            .with_text("Customize")
            .with_on_click(UiOnClick::new(move |w, _| {
                *w.query::<&mut MenuNode>().single_mut(w) = MenuNode::Play(PlayNode::Customize);
            })),
        ui_builder
            .create::<UiButton>(Val::Auto, Val::Auto)
            .with_text("Achievements")
            .with_on_click(UiOnClick::new(move |w, _| {
                *w.query::<&mut MenuNode>().single_mut(w) = MenuNode::Play(PlayNode::Achievements);
            })),
    ];

    commands.entity(container).with_children(|parent| {
        parent.spawn(title);
        for button in buttons.into_iter() {
            button.spawn(parent);
        }
    });
}

fn spawn_root(commands: &mut Commands, ui_builder: &mut UiBuilder, container: Entity) {
    let title = TextBundle {
        text: Text::from_section(
            "Main menu",
            ui_builder.text_styles.get(
                FontType::Regular,
                FontSize::Large,
                ui_builder
                    .game_assets
                    .colors
                    .get_content(GameColor::Primary),
            ),
        )
        .with_justify(JustifyText::Center),
        ..Default::default()
    };
    let buttons = vec![
        ui_builder
            .create::<UiButton>(Val::Auto, Val::Auto)
            .with_text("Play")
            .with_on_click(UiOnClick::new(move |w, _| {
                *w.query::<&mut MenuNode>().single_mut(w) = MenuNode::Play(PlayNode::Root);
            })),
        ui_builder
            .create::<UiButton>(Val::Auto, Val::Auto)
            .with_text("Settings")
            .with_on_click(UiOnClick::new(|w, _| {
                *w.query::<&mut MenuNode>().single_mut(w) = MenuNode::Settings(SettingsNode::Root);
            })),
    ];

    commands.entity(container).with_children(|parent| {
        parent.spawn(title);
        for button in buttons.into_iter() {
            button.spawn(parent);
        }
    });
}

fn set_in_game(world: &mut World, _context: &UiCommandContext<UiPointerEventData>) {
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
