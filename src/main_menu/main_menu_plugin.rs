use super::plugins::loading_view_plugin::LoadingViewPlugin;
use crate::common::plugins::ui_plugin::components::{
    UiBase, UiBuilder, UiButton, UiComponent, UiContainer, UiText,
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
        app.init_resource::<MenuState>()
            .register_type::<MenuState>()
            .add_plugins(LoadingViewPlugin)
            .add_systems(OnEnter(AppState::MainMenu), init_main_menu)
            .add_systems(Update, update_menu.run_if(resource_changed::<MenuState>))
            .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu);
    }
}

#[derive(Resource, Default, Clone, Copy, Reflect)]
pub enum MenuState {
    #[default]
    Root,
    Play(PlayMenuState),
    Settings(SettingsMenuState),
}

impl MenuState {
    pub fn go_back(&mut self) {
        *self = match self {
            MenuState::Root
            | MenuState::Play(PlayMenuState::Root)
            | MenuState::Settings(SettingsMenuState::Root) => MenuState::Root,
            MenuState::Play(_) => MenuState::Play(PlayMenuState::Root),
            MenuState::Settings(_) => MenuState::Settings(SettingsMenuState::Root),
        }
    }
}

#[derive(Resource, Reflect, Clone, Copy)]
pub enum PlayMenuState {
    Root,
    LevelSelect,
    Customize,
    // Shop
    Achievements,
}

#[derive(Resource, Reflect, Clone, Copy)]
pub enum SettingsMenuState {
    Root,
    Graphics,
    Audio,
    Controls,
}

#[derive(Component)]
struct MenuNode;

#[derive(Component)]
struct MenuContainer;

fn init_main_menu(mut commands: Commands, mut ui_builder: UiBuilder, mut state: ResMut<MenuState>) {
    // TODO: Background scene'
    let base = UiBase::new(ui_builder.game_assets.colors.get(GameColor::Base));
    let title = ui_builder
        .create_auto::<UiText>()
        .with_text("Small 3D")
        .with_text_style(ui_builder.text_styles.get(
            FontType::Bold,
            FontSize::XLarge,
            ui_builder.game_assets.colors.get_content(GameColor::Base),
        ));
    let menu: UiContainer = ui_builder.create(Val::Auto, Val::Auto);

    base.spawn(&mut commands)
        .insert(MenuNode)
        .with_children(|parent| {
            title.spawn(parent);
            menu.spawn(parent).insert(MenuContainer);
        });

    state.set_changed();
}

fn update_menu(
    mut commands: Commands,
    state: Res<MenuState>,
    container: Query<Entity, With<MenuContainer>>,
    mut ui_builder: UiBuilder,
) {
    let Some(container) = container.iter().next() else {
        return;
    };

    commands.entity(container).despawn_descendants();
    match *state {
        MenuState::Root => {
            spawn_root(&mut commands, &mut ui_builder, container);
            return;
        }
        MenuState::Play(play_node) => match play_node {
            PlayMenuState::Root => spawn_play_root(&mut commands, &mut ui_builder, container),
            PlayMenuState::LevelSelect => {
                spawn_level_select(&mut commands, &mut ui_builder, container)
            }
            PlayMenuState::Customize => {}
            PlayMenuState::Achievements => {}
        },
        MenuState::Settings(settings_node) => match settings_node {
            SettingsMenuState::Root => {}
            SettingsMenuState::Graphics => {}
            SettingsMenuState::Audio => {}
            SettingsMenuState::Controls => {}
        },
    }

    let back_button = ui_builder
        .create::<UiButton>(Val::Auto, Val::Auto)
        .with_text("Go back")
        .with_on_click(UiOnClick::new(|w, _| {
            w.resource_mut::<MenuState>().go_back();
        }))
        .with_game_color(GameColor::Neutral, &ui_builder);

    commands.entity(container).with_children(|parent| {
        back_button.spawn(parent);
    });
}

fn spawn_level_select(commands: &mut Commands, ui_builder: &mut UiBuilder, container: Entity) {
    let title = ui_builder.create_auto::<UiText>().with_text("Level select").with_text_style(
        ui_builder.text_styles.get(
            FontType::Regular,
            FontSize::Large,
            ui_builder.game_assets.colors.get_content(GameColor::Primary),
        ),
    );
    let buttons = vec![ui_builder
        .create::<UiButton>(Val::Auto, Val::Auto)
        .with_text("Demo")
        .with_on_click(UiOnClick::new(|w, ctx| {
            w.resource_mut::<GameData>().level = Some(GameLevel::Demo);
            set_in_game(w, ctx);
        }))];

    commands.entity(container).with_children(|parent| {
        title.spawn(parent);
        for button in buttons.into_iter() {
            button.spawn(parent);
        }
    });
}

fn spawn_play_root(commands: &mut Commands, ui_builder: &mut UiBuilder, container: Entity) {
    let title = ui_builder.create_auto::<UiText>().with_text("Play").with_text_style(
        ui_builder.text_styles.get(
            FontType::Regular,
            FontSize::Large,
            ui_builder.game_assets.colors.get_content(GameColor::Primary),
        ),
    );
    let buttons = vec![
        ui_builder
            .create::<UiButton>(Val::Auto, Val::Auto)
            .with_text("Level select")
            .with_on_click(UiOnClick::new(move |w, _| {
                *w.resource_mut::<MenuState>() = MenuState::Play(PlayMenuState::LevelSelect);
            })),
        ui_builder
            .create::<UiButton>(Val::Auto, Val::Auto)
            .with_text("Customize")
            .with_on_click(UiOnClick::new(move |w, _| {
                *w.resource_mut::<MenuState>() = MenuState::Play(PlayMenuState::Customize);
            })),
        ui_builder
            .create::<UiButton>(Val::Auto, Val::Auto)
            .with_text("Achievements")
            .with_on_click(UiOnClick::new(move |w, _| {
                *w.resource_mut::<MenuState>() = MenuState::Play(PlayMenuState::Achievements);
            })),
    ];

    commands.entity(container).with_children(|parent| {
        title.spawn(parent);
        for button in buttons.into_iter() {
            button.spawn(parent);
        }
    });
}

fn spawn_root(commands: &mut Commands, ui_builder: &mut UiBuilder, container: Entity) {
    let title = ui_builder.create_auto::<UiText>().with_text("Main menu").with_text_style(
        ui_builder.text_styles.get(
            FontType::Regular,
            FontSize::Large,
            ui_builder.game_assets.colors.get_content(GameColor::Primary),
        ),
    );
    let buttons = vec![
        ui_builder
            .create::<UiButton>(Val::Auto, Val::Auto)
            .with_text("Play")
            .with_on_click(UiOnClick::new(move |w, _| {
                *w.resource_mut::<MenuState>() = MenuState::Play(PlayMenuState::Root);
            })),
        ui_builder
            .create::<UiButton>(Val::Auto, Val::Auto)
            .with_text("Settings")
            .with_on_click(UiOnClick::new(|w, _| {
                *w.resource_mut::<MenuState>() = MenuState::Settings(SettingsMenuState::Root);
            })),
    ];

    commands.entity(container).with_children(|parent| {
        title.spawn(parent);
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
