use crate::{
    common::plugins::ui_plugin::{
        components::{UiBuilder, UiComponent, UiText},
        styles, UiOnClick, UiOnClickBundle,
    },
    resources::{
        game_assets::{GameAssets, GameImage},
        loadable::Loadable,
        text_styles::{FontSize, FontType, TextStyles},
    },
    AppState,
};
use bevy::prelude::*;
use bevy_tweening::{
    lens::TransformScaleLens, Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween,
};
use std::time::Duration;

pub struct LoadingViewPlugin;

impl Plugin for LoadingViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), init_loading_view)
            .add_systems(Update, (update_text).run_if(in_state(AppState::Loading)))
            .add_systems(OnExit(AppState::Loading), cleanup_loading_view);
    }
}

#[derive(Component)]
struct LoadingNode;

#[derive(Component)]
struct LoadingText;

trait LoadingCommands {
    fn loaded() -> Self;
}

impl LoadingCommands for UiOnClick {
    fn loaded() -> Self {
        Self::new(|w, _| {
            if !w
                .resource::<GameAssets>()
                .loaded(w.resource::<AssetServer>())
            {
                return;
            }
            w.resource_mut::<NextState<AppState>>()
                .set(AppState::MainMenu);
        })
    }
}

fn init_loading_view(mut commands: Commands, mut ui_builder: UiBuilder) {
    let container = (
        styles::container_node(Val::Percent(100.0), Val::Percent(100.0)),
        LoadingNode,
        UiOnClickBundle {
            ui_on_click: UiOnClick::loaded(),
            ..Default::default()
        },
    );
    let splash = ImageBundle {
        image: UiImage::new(ui_builder.game_assets.get_image(GameImage::Splash)),
        style: Style {
            align_items: AlignItems::End,
            padding: UiRect::vertical(Val::Percent(15.0)),
            ..styles::container(Val::Percent(100.0), Val::Percent(100.0))
        },
        ..Default::default()
    };
    let text = ui_builder
        .create_auto::<UiText>()
        .with_text("Loading..")
        .with_text_style(ui_builder.text_styles.get(
            FontType::Bold,
            FontSize::XLarge,
            Color::WHITE,
        ));
    let text_components = (
        LoadingText,
        Animator::<Transform>::new(
            Tween::new(
                EaseFunction::BackInOut,
                Duration::from_secs_f32(2.5),
                TransformScaleLens {
                    start: Vec3::splat(1.0),
                    end: Vec3::splat(1.1),
                },
            )
            .with_repeat_count(RepeatCount::Infinite)
            .with_repeat_strategy(RepeatStrategy::MirroredRepeat),
        ),
    );

    commands.spawn(container).with_children(|parent| {
        parent.spawn(splash).with_children(|root| {
            text.spawn(root).insert(text_components);
        });
    });
}

fn update_text(
    mut query: Query<(&mut Text, &LoadingText)>,
    game_assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    text_styles: Res<TextStyles>,
) {
    const DONE_TEXT: &str = "Tap to start!";
    let Some((mut text, _)) = query.iter_mut().next() else {
        return;
    };

    if text.sections[0].value.as_str() == DONE_TEXT {
        return;
    }

    if !game_assets.loaded(&asset_server) || !text_styles.loaded(&asset_server) {
        return;
    }

    text.sections[0].value = DONE_TEXT.to_string();
}

fn cleanup_loading_view(mut commands: Commands, query: Query<Entity, With<LoadingNode>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
