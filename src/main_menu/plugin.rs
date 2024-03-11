use bevy::input::keyboard::KeyboardInput;
use crate::{AppState, Fonts};
use bevy::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::MainMenu), init_main_menu)
            .add_systems(Update, set_in_game.run_if(in_state(AppState::MainMenu)))
            .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu);
    }
}
fn set_in_game(mut next_state: ResMut<NextState<AppState>>, mut key_event: EventReader<KeyboardInput>) {
    let Some(_) = key_event.read().next() else {
        return;
    };
    next_state.set(AppState::InGame);
}

#[derive(Component)]
struct MenuCamera;

#[derive(Component)]
struct MenuNode;

fn init_main_menu(mut commands: Commands, fonts: Res<Fonts>) {
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
            root.spawn(TextBundle {
                text: Text::from_section(
                    "Press any key to start..",
                    TextStyle {
                        font: fonts.regular.clone(),
                        font_size: 60.0,
                        color: Color::WHITE,
                    },
                ),
                ..Default::default()
            });
        });
}

fn cleanup_main_menu(mut commands: Commands, node: Query<Entity, With<MenuNode>>, camera: Query<Entity, With<MenuCamera>>) {
    commands.entity(node.single()).despawn_recursive();
    commands.entity(camera.single()).despawn_recursive();
}
