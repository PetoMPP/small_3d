use crate::resources::game_assets::GameAssets;
use crate::resources::loadable::Loadable;
use crate::resources::text_styles::{FontSize, FontType};
use crate::{AppState, TextStyles};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_system.run_if(in_state(AppState::MainMenu)));
    }
}

fn ui_system(
    mut commands: Commands,
    mut egui_contexts: EguiContexts,
    asset_server: Res<AssetServer>,
    game_assets: Res<GameAssets>,
    text_styles: Res<TextStyles>,
) {
    let font_name = egui::FontFamily::Name(FontType::Bold.as_ref().into());
    // we need fonts to draw ui
    if !egui_contexts
        .ctx_mut()
        .fonts(|f| f.families().contains(&font_name))
    {
        return;
    }
    egui::CentralPanel::default().show(egui_contexts.ctx_mut(), |ui| {
        ui.with_layout(
            egui::Layout {
                main_align: egui::Align::Center,
                main_justify: true,
                cross_align: egui::Align::Center,
                cross_justify: true,
                ..Default::default()
            },
            |ui| {
                render_main_menu(
                    ui,
                    &mut commands,
                    game_assets.loaded(&asset_server) && text_styles.loaded(&asset_server),
                    font_name,
                )
            },
        );
    });
}

const READY_TEXT: &str = "Press to start";
const LOADING_TEXT: &str = "Loading..";

fn render_main_menu(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    loaded: bool,
    font_name: egui::FontFamily,
) {
    let text = match loaded {
        true => READY_TEXT,
        false => LOADING_TEXT,
    };
    if loaded
        && ui
            .add(
                egui::Label::new(
                    egui::RichText::new(text)
                        .size(*FontSize::Large)
                        .family(font_name)
                        .color(egui::Color32::LIGHT_GRAY),
                )
                .selectable(false)
                .sense(egui::Sense::click()),
            )
            .clicked()
    {
        commands.add(|world: &mut World| {
            world
                .resource_mut::<NextState<AppState>>()
                .set(AppState::InGame);
        })
    }
}
