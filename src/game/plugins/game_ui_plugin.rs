use crate::{
    common::plugins::ui_plugin::{styles, UiNode, UiOnClick, UiOnClickBundle, UiStyle},
    game::game_plugin::GameState,
    resources::text_styles::{FontSize, FontType, TextStyles},
    AppState,
};
use bevy::prelude::*;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), switch_ui)
            .add_systems(
                Update,
                switch_ui
                    .run_if(state_changed::<GameState>)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

fn switch_ui(
    mut commands: Commands,
    window: Query<&Window>,
    text_styles: Res<TextStyles>,
    game_state: Res<State<GameState>>,
    playing: Query<Entity, With<PlayingElement>>,
    paused: Query<Entity, With<PausedElement>>,
) {
    playing.iter().chain(paused.iter()).for_each(|entity| {
        commands.entity(entity).despawn_recursive();
    });

    let window = window.single();
    match **game_state {
        GameState::Paused => spawn_pause_menu(commands, &text_styles),
        GameState::Playing => spawn_pause_button(commands, window),
    }
}

fn spawn_pause_menu(mut commands: Commands, text_styles: &TextStyles) {
    let buttons = vec![
        (
            "Resume",
            UiOnClick(|w: &mut World| {
                w.resource_mut::<NextState<GameState>>()
                    .set(GameState::Playing)
            }),
        ),
        (
            "Back to main menu",
            UiOnClick(|w: &mut World| {
                w.resource_mut::<NextState<AppState>>()
                    .set(AppState::MainMenu)
            }),
        ),
    ]
    .into_iter()
    .map(|(text, ui_on_click)| {
        let mut button_node = styles::container_node(Val::Auto, Val::Auto);
        button_node.style.padding = UiRect::all(Val::Px(12.0));
        let button = (
            button_node.clone(),
            UiNode::container(UiStyle {
                color: Color::BLUE,
                border_color: Some(Color::MIDNIGHT_BLUE),
                ..Default::default()
            }),
            UiOnClickBundle {
                ui_on_click,
                ..Default::default()
            },
        );
        let text = TextBundle::from_section(
            text,
            text_styles.get(FontType::Regular, FontSize::Medium, Color::WHITE),
        );
        (button, text)
    });

    let menu = {
        let mut menu_node = styles::container_node(Val::Auto, Val::Auto);
        menu_node.style.padding = UiRect::all(Val::Px(36.0));
        menu_node.style.flex_direction = FlexDirection::Column;
        menu_node.style.row_gap = Val::Px(12.0);
        (
            menu_node,
            UiNode::container(UiStyle {
                color: Color::WHITE,
                ..Default::default()
            }),
        )
    };

    let container = {
        let mut container_node = styles::container_node(Val::Percent(100.0), Val::Percent(100.0));
        container_node.background_color = Color::rgba(0.0, 0.0, 0.0, 0.5).into();
        (container_node, PausedElement)
    };

    commands.spawn(container).with_children(|parent| {
        parent.spawn(menu).with_children(|parent| {
            for (button, text) in buttons {
                parent.spawn(button).with_children(|parent| {
                    parent.spawn(text);
                });
            }
        });
    });
}

#[derive(Component)]
struct PausedElement;

#[derive(Component)]
struct PlayingElement;

fn spawn_pause_button(mut commands: Commands, window: &Window) {
    // Spawn pause button
    let a = window.height().min(window.width()) / 5.0;
    let offset = a / 10.0;
    let pause_button = (
        styles::container_node(Val::Px(a), Val::Px(a)),
        UiNode::container(UiStyle {
            color: Color::YELLOW_GREEN,
            border_color: Some(Color::DARK_GREEN),
            border_width: a / 20.0,
            ..Default::default()
        }),
        UiOnClickBundle {
            ui_on_click: UiOnClick(|world| {
                world
                    .resource_mut::<NextState<GameState>>()
                    .set(GameState::Paused);
            }),
            ..Default::default()
        },
    );

    let mut container = styles::container_node(Val::Percent(100.0), Val::Auto);
    container.style.justify_content = JustifyContent::End;
    container.style.padding = UiRect::all(Val::Px(offset));

    commands
        .spawn((container, PlayingElement))
        .with_children(|parent| {
            parent.spawn(pause_button);
        });
}
