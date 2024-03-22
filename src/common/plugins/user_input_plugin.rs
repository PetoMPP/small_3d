use bevy::{
    input::{mouse::mouse_button_input_system, touch::Touch},
    prelude::*,
};
use bevy_mod_picking::{
    backend::HitData,
    events::{Down, Pointer},
    input::touch::touch_pick_events,
    pointer::PointerButton,
};

pub struct UserInputPlugin;

#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Pressed {
    /// Pointer button pressed to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub hit: HitData,
}

trait ToFromPointer<T: std::fmt::Debug + Clone + Reflect> {
    fn to_from_pointer(value: Pointer<T>) -> Pointer<Self>
    where
        Self: std::fmt::Debug + Clone + Reflect;
}

impl ToFromPointer<Down> for Pressed {
    fn to_from_pointer(value: Pointer<Down>) -> Pointer<Self> {
        Pointer::<Pressed> {
            target: value.target,
            pointer_id: value.pointer_id,
            pointer_location: value.pointer_location.clone(),
            event: Pressed {
                button: value.button,
                hit: value.hit.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserInput;

#[derive(Debug, Resource, Clone, Copy, Default, Deref, DerefMut)]
pub struct UserInputPosition(pub Option<Vec2>);

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonInput<UserInput>>()
            .init_resource::<UserInputPosition>()
            .add_event::<Pointer<Pressed>>()
            .add_systems(
                PreUpdate,
                clear_input_state.before(mouse_button_input_system),
            )
            .add_systems(
                PreUpdate,
                (
                    handle_pressed,
                    handle_input_state_touch,
                    handle_input_state_mouse,
                )
                    .after(mouse_button_input_system)
                    .after(touch_pick_events),
            );
    }
}

fn clear_input_state(mut user_input: ResMut<ButtonInput<UserInput>>) {
    user_input.clear();
}

fn handle_input_state_touch(
    mut user_input: ResMut<ButtonInput<UserInput>>,
    mut user_input_position: ResMut<UserInputPosition>,
    touch_input: Res<Touches>,
    mut curr_touch: Local<Option<Touch>>,
) {
    match *curr_touch {
        Some(touch) => {
            if touch_input.just_released(touch.id()) {
                *curr_touch = None;
                **user_input_position = None;
                user_input.release(UserInput);
                return;
            }
            if let Some(new_touch) = touch_input.get_pressed(touch.id()) {
                *curr_touch = Some(*new_touch);
                **user_input_position = Some(new_touch.position());
            }
        }
        None => {
            if let Some(touch) = touch_input.iter_just_pressed().next() {
                *curr_touch = Some(*touch);
                **user_input_position = Some(touch.position());
                user_input.press(UserInput);
            }
        }
    }
}

fn handle_input_state_mouse(
    mut user_input: ResMut<ButtonInput<UserInput>>,
    mut user_input_position: ResMut<UserInputPosition>,
    input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
) {
    if input.just_released(MouseButton::Left) {
        user_input.release(UserInput);
        **user_input_position = None;
        return;
    }

    if input.just_pressed(MouseButton::Left) {
        user_input.press(UserInput);
    }
    if input.pressed(MouseButton::Left) {
        **user_input_position = windows
            .iter()
            .next()
            .and_then(|window| window.cursor_position());
    }
}

fn handle_pressed(
    mut down_reader: EventReader<Pointer<Down>>,
    mut pressed_writer: EventWriter<Pointer<Pressed>>,
) {
    for down in down_reader.read() {
        pressed_writer.send(Pressed::to_from_pointer(down.clone()));
    }
}
