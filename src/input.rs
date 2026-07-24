use std::any::TypeId;

use bevy::{
    input::{
        ButtonState, InputSystems,
        gamepad::{RawGamepadAxisChangedEvent, RawGamepadButtonChangedEvent},
        keyboard::KeyboardInput,
    },
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.insert_resource(Input {
        pressed: default(),
        held: default(),
        bindings: default(),

        default_dead_zone: 0.5,
        dead_zones: default(),
    })
    .add_systems(Startup, bindings)
    .add_systems(PreUpdate, input.in_set(InputSystems));
}

// TODO: Axis is just two 0.0..1.0s. Everything can be pressed. Pressed is triggered when the value leaves the dead zone, but not when it changes within the live zone.
#[derive(Resource)]
pub struct Input {
    pressed: foldhash::HashSet<InputSource>,
    held: foldhash::HashSet<InputSource>,
    bindings: foldhash::HashMap<(TypeId, u8), Vec<InputSource>>,

    default_dead_zone: f32,
    dead_zones: foldhash::HashMap<InputSource, f32>,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum InputSource {
    GamepadButton(GamepadButton),
    GamepadAxisPositive(GamepadAxis),
    GamepadAxisNegative(GamepadAxis),
    KeyCode(KeyCode),
}
impl From<KeyCode> for InputSource {
    fn from(value: KeyCode) -> Self {
        Self::KeyCode(value)
    }
}
impl From<GamepadButton> for InputSource {
    fn from(value: GamepadButton) -> Self {
        Self::GamepadButton(value)
    }
}

impl Input {
    pub fn event(&mut self, input_source: InputSource, value: f32) {
        let dead_zone = match self.dead_zones.get(&input_source) {
            Some(dead_zone) => *dead_zone,
            None => self.default_dead_zone,
        };

        if value >= dead_zone {
            if self.held.insert(input_source) {
                self.pressed.insert(input_source);
            }
        } else {
            self.held.remove(&input_source);
        }
    }

    #[deprecated]
    pub fn pressed_old(&self, key_code: KeyCode) -> bool {
        self.pressed.contains(&InputSource::KeyCode(key_code))
    }

    pub fn bind<T: ActionTemplate>(&mut self, index: u8, input_source: impl Into<InputSource>) {
        self.bindings
            .entry((TypeId::of::<T>(), index))
            .or_default()
            .push(input_source.into());
    }

    pub fn pressed<T: ActionTemplate>(&self) -> bool {
        T::Template::pressed(|i| {
            self.bindings
                .get(&(TypeId::of::<T>(), i))
                .and_then(|bindings| {
                    bindings
                        .iter()
                        .find_map(|binding| self.pressed.get(binding))
                })
                .is_some()
        })
    }
    pub fn held<T: ActionTemplate>(&self) -> <T::Template as Action>::Output {
        T::Template::held(|i| {
            self.bindings
                .get(&(TypeId::of::<T>(), i))
                .and_then(|bindings| bindings.iter().find_map(|binding| self.held.get(binding)))
                .is_some()
        })
    }
}

fn input(
    mut input: ResMut<Input>,
    mut keyboard_input: MessageReader<KeyboardInput>,
    mut gamepad_button_input: MessageReader<RawGamepadButtonChangedEvent>,
    mut gamepad_axis_input: MessageReader<RawGamepadAxisChangedEvent>,
) {
    input.bypass_change_detection().pressed.clear();

    for keyboard_input in keyboard_input.read() {
        match keyboard_input.state {
            ButtonState::Pressed => {
                input.event(InputSource::KeyCode(keyboard_input.key_code), 1.);
            }
            ButtonState::Released => {
                input.event(InputSource::KeyCode(keyboard_input.key_code), 0.);
            }
        }
    }

    for gamepad_input in gamepad_button_input.read() {
        input.event(
            InputSource::GamepadButton(gamepad_input.button),
            gamepad_input.value,
        );
    }

    for gamepad_input in gamepad_axis_input.read() {
        //info!("{gamepad_input:?}");
    }
}

pub trait ActionTemplate: 'static {
    type Template: Action;
}

pub trait Action: Sync + Send + 'static {
    type Output;
    fn pressed(mut check: impl FnMut(u8) -> bool) -> bool;
    fn held(mut check: impl FnMut(u8) -> bool) -> Self::Output;
}
impl<T: Action> ActionTemplate for T {
    type Template = Self;
}

fn bindings(mut input: ResMut<Input>) {
    input.bind::<UiMove>(0, KeyCode::KeyA);
    input.bind::<UiMove>(1, KeyCode::KeyD);

    input.bind::<UiMove>(0, GamepadButton::DPadLeft);
    input.bind::<UiMove>(1, GamepadButton::DPadRight);
}

#[derive(Debug)]
pub enum UiMove {
    Backwards = -1,
    None = 0,
    Forwards = 1,
}
impl Action for UiMove {
    type Output = Self;

    fn pressed(mut check: impl FnMut(u8) -> bool) -> bool {
        check(0) ^ check(1)
    }
    fn held(mut check: impl FnMut(u8) -> bool) -> Self::Output {
        match (check(0), check(1)) {
            (true, false) => Self::Backwards,
            (false, true) => Self::Forwards,
            _ => Self::None,
        }
    }
}
