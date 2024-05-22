use super::{
    aiming_plugin::DragInfo,
    game_scene_plugin::{GameData, SetGameLevel},
};
use crate::{
    common::plugins::ui_plugin::{
        components::{UiBase, UiBuilder, UiButton, UiComponent, UiContainer},
        render_ui, styles, UiNode, UiOnClick, UiOnClickBundle, UiState, UiStyle,
    },
    game::game_plugin::GameState,
    resources::{
        game_assets::{GameColor, GameImage},
        text_styles::{FontSize, FontType},
    },
    utils::rotate_point,
    AppState,
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_vector_shapes::shapes::{DiscPainter, RectPainter, ThicknessType};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), switch_ui)
            .add_systems(
                Update,
                (
                    switch_ui.run_if(state_changed::<GameState>),
                    (
                        update_score_tracker.run_if(resource_changed::<GameData>),
                        set_aim_circle_visibility
                            .run_if(in_state(GameState::Playing))
                            .before(render_ui),
                    )
                        .after(switch_ui),
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), cleanup);
    }
}

fn cleanup(
    mut commands: Commands,
    playing: Query<Entity, With<PlayingElement>>,
    paused: Query<Entity, With<PausedElement>>,
) {
    playing.iter().chain(paused.iter()).for_each(|entity| {
        commands.entity(entity).despawn_recursive();
    });
}

fn switch_ui(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    game_state: Res<State<GameState>>,
    playing: Query<Entity, With<PlayingElement>>,
    paused: Query<Entity, With<PausedElement>>,
    mut ui_builder: UiBuilder,
) {
    cleanup(commands.reborrow(), playing, paused);
    match **game_state {
        GameState::Paused => spawn_pause_menu(&mut commands, &mut ui_builder),
        GameState::Playing => spawn_game_menu(&mut commands, &mut ui_builder, &mut game_data),
    }
}

trait GameUiOnClick {
    fn pause_game() -> Self;
    fn resume_game() -> Self;
    fn restart_game() -> Self;
    fn back_to_main_menu() -> Self;
    fn start_aim() -> Self;
}

impl GameUiOnClick for UiOnClick {
    fn pause_game() -> Self {
        Self::new(|w, _| {
            w.resource_mut::<NextState<GameState>>()
                .set(GameState::Paused);
        })
    }

    fn resume_game() -> Self {
        Self::new(|w, _| {
            w.resource_mut::<NextState<GameState>>()
                .set(GameState::Playing)
        })
    }

    fn restart_game() -> Self {
        Self::new(|w, _| {
            w.resource_mut::<NextState<GameState>>()
                .set(GameState::Playing);
            w.send_event(SetGameLevel(w.resource::<GameData>().level));
        })
    }

    fn back_to_main_menu() -> Self {
        Self::new(|w, _| {
            w.resource_mut::<NextState<AppState>>()
                .set(AppState::MainMenu)
        })
    }

    fn start_aim() -> Self {
        let mut res = Self::new(|w, ctx| {
            let Some(pointer_data) = ctx.event_data else {
                return;
            };
            let game_data = w.resource::<GameData>();
            if game_data.shots == 0 {
                return;
            }
            let mut drag_info = w.resource_mut::<DragInfo>();
            drag_info.start(pointer_data.pos, pointer_data.user_input);
        });
        res.eager_handle = true;
        res
    }
}

fn spawn_pause_menu(commands: &mut Commands, ui_builder: &mut UiBuilder) {
    let base = UiBase::new(Color::rgba(0.0, 0.0, 0.0, 0.5));
    let menu: UiContainer = ui_builder.create(Val::Auto, Val::Auto);
    let buttons = vec![
        ui_builder
            .create::<UiButton>(Val::Auto, Val::Auto)
            .with_text("Resume")
            .with_on_click(UiOnClick::resume_game()),
        ui_builder
            .create::<UiButton>(Val::Auto, Val::Auto)
            .with_text("Restart")
            .with_on_click(UiOnClick::restart_game()),
        ui_builder
            .create::<UiButton>(Val::Auto, Val::Auto)
            .with_text("Back to main menu")
            .with_on_click(UiOnClick::back_to_main_menu()),
    ];

    base.spawn(commands)
        .insert(PausedElement)
        .with_children(|c| {
            menu.spawn(c).with_children(|m| {
                for button in buttons {
                    button.spawn(m);
                }
            });
        });
}

#[derive(Component)]
struct PausedElement;

#[derive(Component)]
struct PlayingElement;

fn spawn_game_menu(
    commands: &mut Commands,
    ui_builder: &mut UiBuilder,
    game_data: &mut ResMut<GameData>,
) {
    spawn_aim_circle(commands, ui_builder);
    spawn_pause_button(commands, ui_builder);
    spawn_score_tracker(commands, ui_builder, game_data);
}

#[derive(Component)]
pub struct AimCircle;

#[derive(Component)]
pub struct ShotsTracker;

struct ShotsComponent {
    pub inner_ratio: f32,
    shots: Box<dyn Fn(f32) -> (NodeBundle, AimCircle, UiNode, UiState, ShotsTracker)>,
    circle: Box<dyn Fn(f32, f32) -> (NodeBundle, UiOnClickBundle, UiNode)>,
}

impl UiComponent for ShotsComponent {
    fn spawn<'a>(&'a self, parent: &'a mut ChildBuilder) -> EntityCommands {
        let shots = (self.shots)(self.inner_ratio);
        let circle = (self.circle)(self.inner_ratio, shots.2.z + 0.5);
        let mut parent = parent.spawn(shots);
        parent.with_children(|parent| {
            parent.spawn(circle);
        });
        parent
    }

    fn new<'a>(builder: &'a mut UiBuilder, width: Val, height: Val, z: f32) -> Self {
        let (Val::Px(w), Val::Px(h)) = (width, height) else {
            panic!("Width and height must be in pixels");
        };
        let radius = w.min(h) / 2.0;
        let texture = builder.game_assets.get_image(GameImage::Player);
        Self {
            inner_ratio: 0.6,
            shots: Box::new(move |ratio| {
                let texture = texture.clone();
                let inner_radius = radius * ratio;
                (
                    styles::container_node(width, height),
                    AimCircle,
                    UiNode {
                        paint: Box::new(move |painter, size, _, state| {
                            let radius = (radius - inner_radius) / 2.0;
                            let paint_radius = radius * 0.8;
                            painter.color = Color::WHITE;
                            painter.texture = Some(texture.clone());
                            let start = Vec2::new(0.0, size.y / 2.0 - radius);
                            const STEPS: usize = 12;
                            for i in 0..state as usize {
                                const STEP: f32 = 1.0 / STEPS as f32 * 2.0 * std::f32::consts::PI;
                                const OFFSET: f32 = 1.2 * STEP;
                                let angle = i as f32 * STEP + OFFSET;
                                let point = rotate_point(start, Vec2::ZERO, -angle);
                                painter.transform.translation = point.extend(size.z);
                                painter.circle(paint_radius);
                            }
                        }),
                        z,
                        ..Default::default()
                    },
                    UiState(0),
                    ShotsTracker,
                )
            }),
            circle: Box::new(move |ratio, z| {
                let inner_radius = radius * ratio;
                (
                    styles::container_node(
                        Val::Px(inner_radius * 2.0),
                        Val::Px(inner_radius * 2.0),
                    ),
                    UiOnClickBundle {
                        ui_on_click: UiOnClick::start_aim(),
                        ..Default::default()
                    },
                    UiNode {
                        paint: Box::new(|painter, size, _, _| {
                            let radius = size.x.min(size.y) / 2.0;
                            painter.color = Color::WHITE.with_a(0.9);
                            painter.hollow = true;
                            painter.thickness = 0.8;
                            painter.thickness_type = ThicknessType::Screen;
                            const STEPS: usize = 16;
                            for i in 0..=STEPS {
                                const STEP: f32 = 1.0 / STEPS as f32 * 2.0 * std::f32::consts::PI;
                                const OFFSET: f32 = 0.3 * STEP;
                                let angle = i as f32 * STEP + OFFSET;
                                painter.arc(radius, angle, angle + STEP / 2.7);
                            }
                        }),
                        corner_radius: radius,
                        z,
                    },
                )
            }),
        }
    }
}

pub fn spawn_aim_circle(commands: &mut Commands, ui_builder: &mut UiBuilder) {
    let window = ui_builder.window();
    let radius = window.height().min(window.width()) / 4.0;
    let base = UiBase::new(Color::rgba(0.0, 0.0, 0.0, 0.0));
    let shots: ShotsComponent = ui_builder.create(Val::Px(radius * 2.0), Val::Px(radius * 2.0));

    base.spawn(commands)
        .insert(PlayingElement)
        .with_children(|parent| {
            shots.spawn(parent);
        });
}

fn set_aim_circle_visibility(
    mut circle: Query<&mut Style, With<AimCircle>>,
    game_data: Res<GameData>,
    drag_info: Res<DragInfo>,
    game_state: Res<State<GameState>>,
) {
    if !game_data.is_changed() && !drag_info.is_changed() && !game_state.is_changed() {
        return;
    }

    let visible =
        game_data.shots > 0 && drag_info.is_none() && game_state.get() == &GameState::Playing;
    circle.single_mut().display = match visible {
        true => Display::Flex,
        false => Display::None,
    };
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
    mut shots_state: Query<&mut UiState, (With<ShotsTracker>, Without<ProgressStars>)>,
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

    shots_state.single_mut().0 = game_data.shots as u64;
}

struct ScoreComponent {
    pub state: UiState,
    pub score: i32,
    base: UiContainer,
    score_text: Box<dyn Fn(i32) -> TextBundle>,
    content: UiContainer,
    progress: ProgressComponent,
    star_container: UiContainer,
    stars: Vec<StarComponent>,
}

impl UiComponent for ScoreComponent {
    fn spawn<'a>(&'a self, parent: &'a mut ChildBuilder) -> EntityCommands {
        let mut parent = self.base.spawn(parent);
        parent.with_children(|parent| {
            self.content.spawn(parent).with_children(|parent| {
                self.star_container
                    .spawn(parent)
                    .insert((ProgressStars, self.state.clone()))
                    .with_children(|parent| {
                        for star in &self.stars {
                            star.spawn(parent);
                        }
                    });
                self.progress.spawn(parent);
            });
            parent
                .spawn((self.score_text)(self.score))
                .insert(ScoreTracker);
        });
        parent
    }

    fn new<'a>(builder: &'a mut UiBuilder, width: Val, height: Val, _z: f32) -> Self {
        let mut base: UiContainer = builder.create(width, height);
        base.style.justify_content = JustifyContent::End;
        base.style.flex_direction = FlexDirection::Row;
        base.ui_style.border_color = Some(Color::BLACK);
        let mut content: UiContainer = builder.create(height * 2.25, height / 2.0);
        content.ui_style = UiStyle::empty();
        let progress = builder.create::<ProgressComponent>(Val::Percent(90.0), height / 4.0);
        let mut star_container: UiContainer = builder.create(Val::Percent(80.0), height / 2.0);
        star_container.ui_style = UiStyle::empty();
        star_container.style.flex_direction = FlexDirection::Row;
        star_container.style.justify_content = JustifyContent::SpaceBetween;
        star_container.style.padding = UiRect {
            left: Val::Px(42.0),
            right: Val::Px(-16.0),
            ..star_container.style.padding
        };

        let stars = vec![
            builder
                .create::<StarComponent>(height / 3.0, height / 3.0)
                .with_count(1),
            builder
                .create::<StarComponent>(height / 3.0, height / 3.0)
                .with_count(2),
            builder
                .create::<StarComponent>(height / 3.0, height / 3.0)
                .with_count(3),
        ];
        let text_style =
            builder
                .text_styles
                .get(FontType::ItalicBold, FontSize::XLarge, Color::BLACK);
        Self {
            state: UiState(0),
            score: 0,
            base,
            score_text: Box::new(move |s| TextBundle {
                text: Text::from_section(s.to_string(), text_style.clone()),
                style: Style {
                    padding: UiRect::all(height / 10.0),
                    margin: UiRect::horizontal(height / 10.0),
                    ..Default::default()
                },
                ..Default::default()
            }),
            content,
            progress,
            star_container,
            stars,
        }
    }
}

struct ProgressComponent {
    base: UiContainer,
    progress_bar: UiContainer,
}

impl UiComponent for ProgressComponent {
    fn spawn<'a>(&'a self, parent: &'a mut ChildBuilder) -> EntityCommands {
        let mut parent = self.base.spawn(parent);
        parent.with_children(|parent| {
            self.progress_bar.spawn(parent).insert(ProgressTracker);
        });
        parent
    }

    fn new<'a>(builder: &'a mut UiBuilder, width: Val, height: Val, _z: f32) -> Self {
        let mut base: UiContainer = builder.create(width, Val::Auto);
        base.ui_style.border_radius = 0.0;
        base.ui_style.border_width = 2.0 * builder.window().scale_factor();
        base.style.align_items = AlignItems::Start;
        base.style.padding = UiRect::all(Val::Px(2.0 * builder.window().scale_factor()));

        let mut progress_bar: UiContainer = builder.create(Val::Percent(0.0), height);
        progress_bar.ui_style = UiStyle {
            color: builder.game_assets.colors.get(GameColor::Accent),
            ..UiStyle::empty()
        };
        progress_bar.style.padding = UiRect::all(Val::Px(0.0));
        Self { base, progress_bar }
    }
}

struct StarComponent {
    pub count: u8,
    node: Box<dyn Fn(u8) -> (NodeBundle, UiNode)>,
}

impl StarComponent {
    pub fn with_count(mut self, count: u8) -> Self {
        self.count = count;
        self
    }
}

impl UiComponent for StarComponent {
    fn spawn<'a>(&'a self, parent: &'a mut ChildBuilder) -> EntityCommands {
        parent.spawn((self.node)(self.count))
    }

    fn new<'a>(builder: &'a mut UiBuilder, width: Val, height: Val, z: f32) -> Self {
        let star = builder.game_assets.get_image(GameImage::Star);
        Self {
            count: 0,
            node: Box::new(move |count| {
                let star = star.clone();
                (
                    styles::container_node(width, height),
                    UiNode {
                        paint: Box::new(move |painter, size, _, state| {
                            if count == 0 {
                                return;
                            }
                            painter.color = match state >= count as u64 {
                                true => Color::WHITE,
                                false => Color::GRAY,
                            };
                            painter.texture = Some(star.clone());
                            painter.transform.translation.z = size.z;
                            let step = size.xy() * Vec2::new(0.3, 0.1);
                            let start = step - step * (count + 1) as f32 / 2.0;
                            painter.translate(start.extend(0.0));
                            for _ in 0..count {
                                painter.translate(step.extend(0.0));
                                painter.rect(size.xy());
                            }
                        }),
                        z,
                        ..Default::default()
                    },
                )
            }),
        }
    }
}

fn spawn_score_tracker(
    commands: &mut Commands,
    ui_builder: &mut UiBuilder,
    game_data: &mut ResMut<GameData>,
) {
    game_data.set_changed(); // to trigger update
    let window = ui_builder.window();
    let a = window.height().min(window.width()) / 6.0;
    let offset = a / 10.0;
    let mut base = UiBase::new(Color::rgba(0.0, 0.0, 0.0, 0.0));
    base.style.align_items = AlignItems::Start;
    base.style.padding = UiRect::all(Val::Px(offset));

    let score: ScoreComponent = ui_builder.create(Val::Auto, Val::Px(a));
    base.spawn(commands)
        .insert(PlayingElement)
        .with_children(|parent| {
            score.spawn(parent);
        });
}

fn spawn_pause_button(commands: &mut Commands, ui_builder: &mut UiBuilder) {
    let window = ui_builder.window();
    let a = window.height().min(window.width()) / 6.0;
    let offset = a / 10.0;
    let mut base = UiBase::new(Color::rgba(0.0, 0.0, 0.0, 0.0));
    base.style.padding = UiRect::all(Val::Px(offset));
    base.style.align_items = AlignItems::Start;
    base.style.justify_content = JustifyContent::End;
    let mut pause_button = ui_builder
        .create::<UiButton>(Val::Px(a), Val::Px(a))
        .with_on_click(UiOnClick::pause_game());
    pause_button.ui_style.color = ui_builder.game_assets.colors.get(GameColor::Warning);
    pause_button.ui_style.border_color = Some(
        ui_builder
            .game_assets
            .colors
            .get_content(GameColor::Warning),
    );
    pause_button.style.padding = UiRect::all(Val::Px(0.0));
    pause_button.style.margin = UiRect::all(Val::Px(0.0));
    let pause_inner = (
        styles::container_node(Val::Px(a), Val::Px(a)),
        UiNode {
            paint: Box::new(|painter, size, int, _| {
                painter.transform.translation.z = size.z;
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
            z: ui_builder.get_next_z(),
            ..Default::default()
        },
    );

    base.spawn(commands)
        .insert(PlayingElement)
        .with_children(|parent| {
            pause_button.spawn(parent).with_children(|parent| {
                parent.spawn(pause_inner);
            });
        });
}
