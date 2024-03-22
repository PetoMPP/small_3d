use crate::resources::Inputs;
use bevy::{input::mouse::mouse_button_input_system, prelude::*, utils::HashMap};
use bevy_mod_picking::{
    backend::HitData,
    events::{Down, Pointer},
    input::touch::touch_pick_events,
    pointer::PointerId,
};

pub struct UserInputPlugin;

#[derive(Clone, PartialEq, Debug, Reflect)]
pub struct Pressed {
    pub user_input: UserInput,
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
                user_input: value.pointer_id.into(),
                hit: value.hit.clone(),
            },
        }
    }
}

#[derive(Debug, Deref, DerefMut, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct UserInput(pub u64);

impl From<PointerId> for UserInput {
    fn from(value: PointerId) -> Self {
        match value {
            PointerId::Mouse => UserInput(0),
            PointerId::Touch(id) => UserInput(id),
            PointerId::Custom(uuid) => UserInput(uuid.as_u64_pair().0),
        }
    }
}

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
    if input.just_released(MouseButton::Left) {
        user_input.release(UserInput(0));
        user_input_position.set(0, None);
        return;
    }

    if input.just_pressed(MouseButton::Left) {
        user_input.press(UserInput(0));
    }
    if input.pressed(MouseButton::Left) {
        user_input_position.set(
            0,
            windows
                .iter()
                .next()
                .and_then(|window| window.cursor_position()),
        );
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
