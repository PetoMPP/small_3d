use crate::log;
use bevy::{
    input::{mouse::mouse_button_input_system, touch::Touch},
    prelude::*,
};
use bevy_mod_picking::{
    backend::HitData,
    events::{Down, Pointer},
    focus::HoverMap,
    input::touch::touch_pick_events,
    pointer::{InputPress, PointerButton, PointerId, PointerLocation, PointerMap, PressDirection},
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
pub struct UserInput(pub PointerButton);

#[derive(Debug, Resource, Clone, Copy, Default, Deref, DerefMut)]
pub struct UserInputPosition(pub Option<Vec2>);

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct TouchOffset(pub Vec2);

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonInput<UserInput>>()
            .init_resource::<UserInputPosition>()
            .init_resource::<TouchOffset>()
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
                user_input.release(UserInput(PointerButton::Primary));
            }
            let new_touch = touch_input.get_pressed(touch.id());
            *curr_touch = new_touch.copied();
            **user_input_position = new_touch.map(|t| t.position());
        }
        None => {
            if let Some(touch) = touch_input.iter_just_pressed().next() {
                *curr_touch = Some(*touch);
                **user_input_position = Some(touch.position());
                user_input.press(UserInput(PointerButton::Primary));
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
        user_input.release(UserInput(PointerButton::Primary));
        **user_input_position = None;
        return;
    }

    if input.just_pressed(MouseButton::Left) {
        user_input.press(UserInput(PointerButton::Primary));
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
    mut input_reader: EventReader<InputPress>,
    mut pressed_writer: EventWriter<Pointer<Pressed>>,
    pointers: Query<&PointerLocation>,
    pointer_map: Res<PointerMap>,
    hover_map: Res<HoverMap>,
) {
    let pointer_location = |pointer_id: PointerId| {
        pointer_map
            .get_entity(pointer_id)
            .and_then(|entity| pointers.get(entity).ok())
            .and_then(|pointer| pointer.location.clone())
    };

    for down in down_reader.read() {
        pressed_writer.send(Pressed::to_from_pointer(down.clone()));
    }

    for input in input_reader.read() {
        for (entity, hit) in hover_map
            .get(&input.pointer_id)
            .iter()
            .flat_map(|hm| hm.iter())
        {
            log!("hit {:?}", hit);
            if let PressDirection::Down = input.direction {
                let Some(location) = pointer_location(input.pointer_id) else {
                    log!("no location!");
                    continue;
                };
                pressed_writer.send(Pointer::new(
                    input.pointer_id,
                    location,
                    *entity,
                    Pressed {
                        button: input.button,
                        hit: hit.clone(),
                    },
                ));
            }
        }
    }
}
