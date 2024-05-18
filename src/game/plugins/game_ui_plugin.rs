use super::game_scene_plugin::{GameData, SetGameLevel};
use crate::{
    common::plugins::ui_plugin::{styles, UiNode, UiOnClick, UiOnClickBundle, UiState, UiStyle},
    game::game_plugin::GameState,
    resources::text_styles::{FontSize, FontType, TextStyles},
    AppState,
};
use bevy::prelude::*;
use bevy_vector_shapes::shapes::RectPainter;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), switch_ui)
            .add_systems(
                Update,
                (
                    switch_ui
                        .run_if(state_changed::<GameState>)
                        .run_if(in_state(AppState::InGame)),
                    update_score_tracker
                        .run_if(resource_changed::<GameData>)
                        .run_if(in_state(AppState::InGame)),
                ),
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
        GameState::Paused => spawn_pause_menu(&mut commands, window, &text_styles),
        GameState::Playing => spawn_game_menu(&mut commands, window, &text_styles),
    }
}

trait GameUiOnClick {
    fn pause_game() -> Self;
    fn resume_game() -> Self;
    fn restart_game() -> Self;
    fn back_to_main_menu() -> Self;
}

impl GameUiOnClick for UiOnClick {
    fn pause_game() -> Self {
        Self::new(|world| {
            world
                .resource_mut::<NextState<GameState>>()
                .set(GameState::Paused);
        })
    }

    fn resume_game() -> Self {
        Self::new(|w: &mut World| {
            w.resource_mut::<NextState<GameState>>()
                .set(GameState::Playing)
        })
    }

    fn restart_game() -> Self {
        Self::new(|w: &mut World| {
            w.resource_mut::<NextState<GameState>>()
                .set(GameState::Playing);
            w.send_event(SetGameLevel(w.resource::<GameData>().level));
        })
    }

    fn back_to_main_menu() -> Self {
        Self::new(|w: &mut World| {
            w.resource_mut::<NextState<AppState>>()
                .set(AppState::MainMenu)
        })
    }
}

fn spawn_pause_menu(commands: &mut Commands, window: &Window, text_styles: &TextStyles) {
    let buttons = vec![
        ("Resume", UiOnClick::resume_game()),
        ("Restart", UiOnClick::restart_game()),
        ("Back to main menu", UiOnClick::back_to_main_menu()),
    ]
    .into_iter()
    .map(|(text, ui_on_click)| {
        let mut button_node = styles::container_node(Val::Auto, Val::Auto);
        button_node.style.padding = UiRect::all(Val::Px(12.0 * window.scale_factor()));
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
        menu_node.style.padding = UiRect::all(Val::Px(36.0 * window.scale_factor()));
        menu_node.style.flex_direction = FlexDirection::Column;
        menu_node.style.row_gap = Val::Px(12.0 * window.scale_factor());
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

fn spawn_game_menu(commands: &mut Commands, window: &Window, text_styles: &TextStyles) {
    spawn_pause_button(commands, window);
    spawn_score_tracker(commands, window, text_styles);
}

#[derive(Component)]
struct ScoreTracker;

#[derive(Component)]
struct ProgressTracker;

#[derive(Component)]
struct ProgressStars;

fn update_score_tracker(
    mut score: Query<&mut Text, With<ScoreTracker>>,
    mut progress: Query<&mut Style, With<ProgressTracker>>,
    mut progress_state: Query<&mut UiState, With<ProgressStars>>,
    game_data: Res<GameData>,
) {
    score.single_mut().sections[0].value = game_data.points.to_string();
    let Some(level) = game_data.level else {
        return;
    };
    let mut progress_state = progress_state.single_mut();
    let meta = level.get_meta();
    let thresholds = vec![
        (meta.star_point_thresholds[0] as i32, 0.35),
        (meta.star_point_thresholds[1] as i32, 0.65),
        (meta.star_point_thresholds[2] as i32, 1.0),
    ];
    match game_data.points {
        points if points < thresholds[0].0 => {
            let add = points as f32 / thresholds[0].0 as f32 * thresholds[0].1;
            progress.single_mut().width = Val::Percent(add * 100.0);
            progress_state.0 = 0;
        }
        points if points < thresholds[1].0 => {
            let add = (points - thresholds[0].0) as f32
                / (thresholds[1].0 - thresholds[0].0) as f32
                * (thresholds[1].1 - thresholds[0].1);
            progress.single_mut().width = Val::Percent((thresholds[0].1 + add) * 100.0);
            progress_state.0 = 1;
        }
        points if points < thresholds[2].0 => {
            let add = (points - thresholds[1].0) as f32
                / (thresholds[2].0 - thresholds[1].0) as f32
                * (thresholds[2].1 - thresholds[1].1);
            progress.single_mut().width = Val::Percent((thresholds[1].1 + add) * 100.0);
            progress_state.0 = 2;
        }
        _ => {
            progress.single_mut().width = Val::Percent(100.0);
            progress_state.0 = 3;
        }
    }
}

fn spawn_score_tracker(commands: &mut Commands, window: &Window, text_styles: &TextStyles) {
    let a = window.height().min(window.width()) / 6.0;
    let offset = a / 10.0;
    let mut score_text = (
        TextBundle::from_section(
            "0",
            TextStyle {
                font_size: a / 2.0,
                ..text_styles.get(FontType::ItalicBold, FontSize::Large, Color::BLACK)
            },
        ),
        ScoreTracker,
    );
    score_text.0.style.padding = UiRect::all(Val::Px(offset));
    score_text.0.style.margin = UiRect::horizontal(Val::Px(offset));

    let mut progress_bar_container = (
        styles::container_node(Val::Px(a * 1.75), Val::Auto),
        UiNode::container(UiStyle {
            color: Color::rgba(0.0, 0.0, 0.0, 0.0),
            border_width: 0.0,
            ..Default::default()
        }),
    );
    progress_bar_container.0.style.padding = UiRect::all(Val::Px(offset));
    progress_bar_container.0.style.flex_direction = FlexDirection::Column;
    let mut progress_stars = (
        styles::container_node(Val::Percent(80.0), Val::Px(a / 2.0)),
        UiState(0),
        ProgressStars,
    );
    progress_stars.0.style.justify_content = JustifyContent::SpaceBetween;
    progress_stars.0.style.align_self = AlignSelf::End;
    let progress_star_single = (
        styles::container_node(Val::Px(a / 3.0), Val::Px(a / 3.0)),
        UiNode {
            paint: Box::new(|painter, size, _, state| {
                let color = match state > 0 {
                    true => Color::YELLOW,
                    false => Color::GRAY,
                };
                painter.color = Color::BLACK;
                painter.rect(size * 0.7);
                painter.color = color;
                painter.rect(size * 0.65);
            }),
            ..Default::default()
        },
    );
    let progress_star_double = (
        styles::container_node(Val::Px(a / 3.0), Val::Px(a / 3.0)),
        UiNode {
            paint: Box::new(|painter, size, _, state| {
                let color = match state > 1 {
                    true => Color::YELLOW,
                    false => Color::GRAY,
                };
                painter.translate(Vec3::new(size.x * -0.15, size.y * -0.05, 0.0));
                painter.color = Color::BLACK;
                painter.rect(size * 0.7);
                painter.color = color;
                painter.rect(size * 0.65);
                painter.translate(Vec3::new(size.x * 0.15, size.y * 0.1, 0.0));
                painter.color = Color::BLACK;
                painter.rect(size * 0.7);
                painter.color = color;
                painter.rect(size * 0.65);
            }),
            ..Default::default()
        },
    );
    let progress_star_triple = (
        styles::container_node(Val::Px(a / 3.0), Val::Px(a / 3.0)),
        UiNode {
            paint: Box::new(|painter, size, _, state| {
                let color = match state > 2 {
                    true => Color::YELLOW,
                    false => Color::GRAY,
                };
                painter.translate(Vec3::new(size.x * -0.45 / 2.0, size.y * -0.1, 0.0));
                painter.color = Color::BLACK;
                painter.rect(size * 0.7);
                painter.color = color;
                painter.rect(size * 0.65);
                painter.translate(Vec3::new(size.x * 0.45 / 2.0, size.y * 0.1, 0.0));
                painter.color = Color::BLACK;
                painter.rect(size * 0.7);
                painter.color = color;
                painter.rect(size * 0.65);
                painter.translate(Vec3::new(size.x * 0.45 / 2.0, size.y * 0.1, 0.0));
                painter.color = Color::BLACK;
                painter.rect(size * 0.7);
                painter.color = color;
                painter.rect(size * 0.65);
            }),
            ..Default::default()
        },
    );

    let mut progress_bar_base = (
        styles::container_node(Val::Percent(90.0), Val::Px(a / 5.0)),
        UiNode::container(UiStyle {
            color: Color::DARK_GRAY,
            border_color: Some(Color::BLACK),
            border_radius: 0.0,
            border_width: 2.0,
        }),
    );
    progress_bar_base.0.style.justify_content = JustifyContent::Start;
    progress_bar_base.0.style.align_self = AlignSelf::Start;
    progress_bar_base.0.style.padding = UiRect::all(Val::Px(2.0));
    let progress_bar = (
        styles::container_node(Val::Percent(0.0), Val::Percent(100.0)),
        UiNode {
            paint: Box::new(move |painter, size, _, _| {
                painter.color = Color::YELLOW;
                painter.rect(Vec2::new(size.x, size.y));
            }),
            ..Default::default()
        },
        ProgressTracker,
    );
    let mut score = (
        styles::container_node(Val::Auto, Val::Px(a)),
        UiNode::container(UiStyle {
            color: Color::WHITE,
            border_color: Some(Color::BLACK),
            ..Default::default()
        }),
    );
    score.0.style.justify_content = JustifyContent::End;
    score.0.style.padding = UiRect::all(Val::Px(offset));

    let mut container = styles::container_node(Val::Percent(100.0), Val::Percent(100.0));
    container.style.justify_content = JustifyContent::Center;
    container.style.align_items = AlignItems::Start;
    container.style.padding = UiRect::all(Val::Px(offset));

    commands
        .spawn((container, PlayingElement))
        .with_children(|parent| {
            parent.spawn(score).with_children(|parent| {
                parent
                    .spawn(progress_bar_container)
                    .with_children(|parent| {
                        parent.spawn(progress_stars).with_children(|parent| {
                            parent.spawn(progress_star_single);
                            parent.spawn(progress_star_double);
                            parent.spawn(progress_star_triple);
                        });
                        parent.spawn(progress_bar_base).with_children(|parent| {
                            parent.spawn(progress_bar);
                        });
                    });
                parent.spawn(score_text);
            });
        });
}

fn spawn_pause_button(commands: &mut Commands, window: &Window) {
    let a = window.height().min(window.width()) / 6.0;
    let offset = a / 10.0;
    let pause_inner = (
        styles::container_node(Val::Px(a), Val::Px(a)),
        UiNode {
            paint: Box::new(|painter, size, int, _| {
                painter.color = Color::WHITE
                    * match int {
                        Interaction::Hovered => 1.35,
                        Interaction::Pressed => 0.7,
                        _ => 1.0,
                    };
                painter.translate(Vec3::new(size.x * -0.15, 0.0, 0.0));
                painter.rect(Vec2::new(size.x * 0.1, size.y * 0.65));
                painter.translate(Vec3::new(size.x * 0.3, 0.0, 0.0));
                painter.rect(Vec2::new(size.x * 0.1, size.y * 0.65));
            }),
            ..Default::default()
        },
    );

    let pause_button = (
        styles::container_node(Val::Px(a), Val::Px(a)),
        UiNode::container(UiStyle {
            color: Color::YELLOW_GREEN,
            border_color: Some(Color::DARK_GREEN),
            ..Default::default()
        }),
        UiOnClickBundle {
            ui_on_click: UiOnClick::pause_game(),
            ..Default::default()
        },
    );

    let mut container = styles::container_node(Val::Percent(100.0), Val::Auto);
    container.style.position_type = PositionType::Absolute;
    container.style.justify_content = JustifyContent::End;
    container.style.padding = UiRect::all(Val::Px(offset));

    commands
        .spawn((container, PlayingElement))
        .with_children(|parent| {
            parent.spawn(pause_button).with_children(|parent| {
                parent.spawn(pause_inner);
            });
        });
}
