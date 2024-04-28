use crate::{game::game_plugin::GameRunningState, AppState};
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32, Frame, Margin, Vec2},
    EguiContexts,
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_system.run_if(in_state(AppState::InGame)));
    }
}

fn ui_system(mut commands: Commands, mut egui_context: EguiContexts) {
    egui::CentralPanel::default()
        .frame(Frame {
            inner_margin: Margin::same(10.0),
            fill: Color32::from_rgba_premultiplied(0, 0, 0, 0), // transparent
            ..Default::default()
        })
        .show(&egui_context.ctx_mut(), |ui| {
            ui.with_layout(
                egui::Layout::default().with_cross_align(egui::Align::Max),
                |ui| {
                    let pause_button = ui.add(
                        egui::Button::new("")
                            .rounding(10.0)
                            .fill(egui::Rgba::from_rgba_premultiplied(0.2, 0.8, 0.2, 1.0))
                            .min_size(Vec2::splat(80.0)),
                    );
                    if pause_button.clicked() {
                        commands.add(|world: &mut World| {
                            let game_state = *world.resource::<State<GameRunningState>>().get();
                            world
                                .resource_mut::<NextState<GameRunningState>>()
                                .set(GameRunningState(!*game_state));
                        })
                    }
                },
            )
        });
}
