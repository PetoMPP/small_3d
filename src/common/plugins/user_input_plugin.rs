use crate::resources::inputs::Inputs;
use bevy::{input::mouse::mouse_button_input_system, prelude::*, utils::HashMap};

pub struct UserInputPlugin;

#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Pressed {
    pub user_input: UserInput,
}

#[derive(Debug, Deref, DerefMut, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct UserInput(pub u64);

#[derive(Debug, Resource, Clone, Default, Deref, DerefMut)]
pub struct UserInputPosition(HashMap<u64, Vec2>);

impl UserInputPosition {
    pub fn get(&self, id: u64) -> Option<Vec2> {
        self.0.get(&id).copied()
    }

    pub fn set(&mut self, id: u64, position: Option<Vec2>) {
        let Some(position) = position else {
            self.0.remove(&id);
            return;
        };
        self.0.insert(id, position);
    }
}

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Inputs<UserInput>>()
            .init_resource::<UserInputPosition>()
            .add_systems(
                PreUpdate,
                clear_input_state.before(mouse_button_input_system),
            )
            .add_systems(
                PreUpdate,
                (handle_input_state_touch, handle_input_state_mouse)
                    .after(mouse_button_input_system),
            );
    }
}

fn clear_input_state(mut user_input: ResMut<Inputs<UserInput>>) {
    user_input.clear();
}

fn handle_input_state_touch(
    mut user_input: ResMut<Inputs<UserInput>>,
    mut user_input_position: ResMut<UserInputPosition>,
    touch_input: Res<Touches>,
) {
    for touch in touch_input.iter_just_released() {
        let id = touch.id();
        user_input.release(UserInput(id));
        user_input_position.set(id, None);
    }

    for touch in touch_input.iter_just_pressed() {
        let id = touch.id();
        user_input.press(UserInput(id));
        user_input_position.set(id, Some(touch.position()));
    }

    for touch in user_input
        .iter_pressed()
        .filter(|ui| ***ui != 0)
        .copied()
        .collect::<Vec<_>>()
        .into_iter()
    {
        if let None = touch_input.get_pressed(*touch) {
            user_input.release(touch);
            user_input_position.set(*touch, None);
        }
    }

    for id in user_input_position
        .keys()
        .copied()
        .collect::<Vec<_>>()
        .into_iter()
    {
        if let Some(touch) = touch_input.get_pressed(id) {
            user_input_position.set(id, Some(touch.position()));
        }
    }
}

fn handle_input_state_mouse(
    mut user_input: ResMut<Inputs<UserInput>>,
    mut user_input_position: ResMut<UserInputPosition>,
    input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
) {
    user_input_position.set(
        0,
        windows
            .iter()
            .next()
            .and_then(|window| window.cursor_position()),
    );
    if input.just_released(MouseButton::Left) {
        user_input.release(UserInput(0));
        user_input_position.set(0, None);
        return;
    }

    if input.just_pressed(MouseButton::Left) {
        user_input.press(UserInput(0));
    }
}
