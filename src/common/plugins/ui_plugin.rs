use crate::resources::{
    loadable::Loadable,
    text_styles::{FontData, TextStyles, TextStylesLoader},
};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin).add_systems(
            Update,
            setup_egui.run_if(|ts: Res<TextStyles>, ass: Res<AssetServer>| ts.loaded(&ass)),
        );
    }
}

fn setup_egui(
    mut egui_contexts: EguiContexts,
    text_styles: Res<TextStyles>,
    mut fonts: ResMut<Assets<FontData>>,
    mut ran: Local<bool>,
) {
    if *ran {
        return;
    }

    *ran = true;
    egui_contexts
        .ctx_mut()
        .load_text_styles(&text_styles, &mut *fonts);
}
