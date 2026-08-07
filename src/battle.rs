//! Terms:
//! Each character is in a slot.

use std::ops::{Deref, DerefMut};

use crate::{
    battle::{
        audio::AudioMessage,
        behaviour::BehaviourMessage,
        character::{Characters, SlotMessage},
        draw::Draw,
    },
    input::{Input, UiMove},
};

use avian2d::prelude::*;
use bevy::{
    ecs::{
        change_detection::Tick,
        query::FilteredAccessSet,
        system::{
            ReadOnlySystemParam, StaticSystemParam, SystemMeta, SystemParam,
            SystemParamValidationError,
        },
        world::unsafe_world_cell::UnsafeWorldCell,
    },
    prelude::*,
};

// water plants
// also make three heads from off screen do circles

mod audio;
mod behaviour;
pub mod character;
mod draw;
pub mod soul;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        character::plugin,
        audio::plugin,
        soul::plugin,
        behaviour::plugin,
    ))
    .add_message::<StartBattle>()
    .add_message::<BattleMessage>()
    .add_message::<EnemyTurnMessage>()
    .add_systems(Startup, insert_resources)
    .add_systems(Update, (update_ui, despawn));
}

const DEEP_PURPLE: Srgba = Srgba::rgb(0.2, 0.125, 0.2);

#[derive(Resource)]
enum Ui {
    Empty,
    Character { character: u8, menu_hovered: u8 },
    EnemyTurn,
}

fn insert_resources(mut commands: Commands) {
    commands.insert_resource(Ui::Empty);
}

mod main_box {
    pub const MAX_Y: f32 = main_box::MAX_Y - outline::HEIGHT;

    pub mod outline {
        pub const HEIGHT: f32 = 0.005;
    }

    pub mod main_box {
        pub const MAX_Y: f32 = 0.75;
    }
}

const CHARACTER_HALF: f32 = 0.08;

fn update_ui(
    mut battle_requests: MessageReader<StartBattle>,
    mut battles: Local<Vec<StartBattle>>,
    mut ui: ResMut<Ui>,
    input: Res<Input>,
    mut message: MessageMutator<BattleMessage>,
    mut enemy_turn: EnemyTurn,
    mut audio_message: MessageWriter<AudioMessage>,
    mut character_ui_message: MessageWriter<SlotMessage>,
    mut behaviour_message: MessageWriter<BehaviourMessage>,
    characters: Res<Characters>,
    mut draw: Draw,
) {
    battles.extend(battle_requests.read().cloned());
    let Some(_) = battles.first() else {
        return;
    };

    *ui = match core::mem::replace(&mut *ui, Ui::Empty) {
        Ui::Empty => {
            // Main box.
            draw.depth(5.);
            draw.rectangle((0., main_box::main_box::MAX_Y), (1., 1.));

            // Main box outline.
            draw.fill(DEEP_PURPLE);
            draw.rectangle(
                (0., main_box::main_box::MAX_Y - main_box::outline::HEIGHT),
                (1., main_box::main_box::MAX_Y),
            );

            draw.text((0.05, main_box::MAX_Y + 0.05), "* Floradinn florads in!\n* Floradinn florads in!\n* Floradinn florads in!\n* Is that a cut on your face, or part of your eye?\n* The gash weaves down as if you cry.");

            Ui::Character {
                character: 0,
                menu_hovered: 0,
            }
        }
        Ui::Character {
            mut character,
            mut menu_hovered,
        } => {
            if input.pressed::<UiMove>() {
                audio_message.write(AudioMessage::Sound("snd_menumove_stereo.wav", default()));
                info!("Held = {:?}", input.held::<UiMove>());
                behaviour_message.write(BehaviourMessage::Lowlight(menu_hovered));

                menu_hovered = (menu_hovered as i8 + input.held::<UiMove>() as i8)
                    .rem_euclid(5)
                    .strict_cast();
                info!("{menu_hovered}");

                behaviour_message.write(BehaviourMessage::Highlight(menu_hovered));
            }

            if input.pressed_old(KeyCode::Enter) {
                audio_message.write(AudioMessage::Sound("snd_select.wav", default()));
                character += 1;
                menu_hovered = 0;

                character_ui_message.write(SlotMessage::Lower(character - 1));

                if character == characters.quantity() {
                    message.write(BattleMessage::EnemyTurnStart);
                    enemy_turn.write(EnemyTurnMessage::Start);
                    Some(Ui::EnemyTurn)
                } else {
                    character_ui_message.write(SlotMessage::Raise(character));
                    None
                }
            } else {
                None
            }
            .unwrap_or(Ui::Character {
                character,
                menu_hovered,
            })
        }
        Ui::EnemyTurn => {
            // message.write(BattleMessage::EnemyTurnEnd);
            // let menus = commands.menus(0);
            // commands.character_menu_raise(0);
            // Ui::Character {
            //     character: 0,
            //     menu_hovered: 0,
            //     menus,
            // }
            Ui::EnemyTurn
        }
    };
}

#[derive(Component)]
#[require(RigidBody::Kinematic, Sensor, Despawn(10.))]
pub struct Danger {
    pub despawn_on_collision: bool,
}

#[derive(Component)]
pub struct Despawn(f32);
fn despawn(despawn: Query<(Entity, &mut Despawn)>, mut commands: Commands, time: Res<Time>) {
    let time_delta = time.delta_secs();

    for (entity, mut despawn) in despawn {
        despawn.0 -= time_delta;

        if despawn.0 <= 0. {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Message, Clone)]
pub struct StartBattle {}

#[derive(Message)]
pub enum BattleMessage {
    EnemyTurnStart,
    EnemyTurnEnd,
}

#[derive(Message)]
pub enum EnemyTurnMessage {
    Start,
    End,
}

#[derive(SystemParam)]
pub struct EnemyTurn<'w, 's> {
    inner: CustomLogic<'w, 's, IsEnemyTurn>,
}
impl EnemyTurn<'_, '_> {
    pub fn read(&self, message: EnemyTurnMessage) -> bool {
        match message {
            EnemyTurnMessage::Start => self.inner.0.1.0,
            EnemyTurnMessage::End => self.inner.0.2.0,
        }
    }

    pub fn write(&mut self, message: EnemyTurnMessage) {
        self.inner.0.0.write(message);
    }
}

#[derive(Default)]
struct IsEnemyTurn(bool);
impl SystemParameterLogicExtension for IsEnemyTurn {
    type Parameter<'w, 's> = (
        MessageMutator<'w, 's, EnemyTurnMessage>,
        Temporary<bool>,
        Temporary<bool>,
    );

    fn logic<'w, 's>(
        &mut self,
        (message, enemy_turn_start, enemy_turn_end): &mut <Self::Parameter<'static, 'static> as SystemParam>::Item<'w, 's>,
    ) {
        for message in message.read() {
            match message {
                EnemyTurnMessage::Start => {
                    enemy_turn_start.0 = true;
                    self.0 = true;
                }
                EnemyTurnMessage::End => {
                    enemy_turn_end.0 = true;
                    self.0 = false;
                }
            }
        }
    }
}

#[derive(SystemParam)]
pub struct BattleOld<'w, 's> {
    messages: CustomLogic<'w, 's, BattleMessageLogic>,
}

#[derive(Default)]
struct BattleMessageLogic {
    enemy_turn_start: bool,
    enemy_turn: bool,
    enemy_turn_end: bool,
}
impl SystemParameterLogicExtension for BattleMessageLogic {
    type Parameter<'w, 's> = MessageMutator<'w, 's, BattleMessage>;

    fn logic<'w, 's>(
        &mut self,
        message: &mut <Self::Parameter<'static, 'static> as SystemParam>::Item<'w, 's>,
    ) {
        self.enemy_turn_start = false;
        self.enemy_turn_end = false;

        for message in message.read() {
            match message {
                BattleMessage::EnemyTurnStart => {
                    self.enemy_turn_start = true;
                    self.enemy_turn = true;
                }
                BattleMessage::EnemyTurnEnd => {
                    self.enemy_turn_end = true;
                    self.enemy_turn = false;
                }
            }
        }
    }
}

impl BattleOld<'_, '_> {
    pub fn enemy_turn_start(&mut self) -> bool {
        self.messages.1.enemy_turn_start
    }

    pub fn enemy_turn(&mut self) -> bool {
        self.messages.1.enemy_turn
    }

    pub fn enemy_turn_end(&mut self) -> bool {
        self.messages.1.enemy_turn_end
    }
}

/// Local, but it isn't saved between runs.
pub struct Temporary<T: Default>(pub(crate) T);

// SAFETY: nothing is accessed
unsafe impl<T: Default> ReadOnlySystemParam for Temporary<T> {}

impl<T: Default> Deref for Temporary<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Default> DerefMut for Temporary<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// SAFETY: nothing is accessed
unsafe impl<T: Default> SystemParam for Temporary<T> {
    type State = ();
    type Item<'w, 's> = Temporary<T>;

    fn init_state(_world: &mut World) -> Self::State {}

    fn init_access(
        _state: &Self::State,
        _system_meta: &mut SystemMeta,
        _component_access_set: &mut FilteredAccessSet,
        _world: &mut World,
    ) {
    }

    #[inline]
    unsafe fn get_param<'w, 's>(
        _state: &'s mut Self::State,
        _system_meta: &SystemMeta,
        _world: UnsafeWorldCell<'w>,
        _change_tick: Tick,
    ) -> Result<Self::Item<'w, 's>, SystemParamValidationError> {
        Ok(Temporary(T::default()))
    }
}

trait SystemParameterLogicExtension: FromWorld + Send + 'static {
    type Parameter<'w, 's>: SystemParam;

    fn logic<'w, 's>(
        &mut self,
        parameter: &mut <Self::Parameter<'static, 'static> as SystemParam>::Item<'w, 's>,
    );
}

//#[derive(SystemParam)]
struct CustomLogic<'w, 's, Logic: SystemParameterLogicExtension>(
    StaticSystemParam<
        'w,
        's,
        <Logic as SystemParameterLogicExtension>::Parameter<'static, 'static>,
    >,
    Local<'s, Logic>,
);

// Recursive expansion of SystemParam macro
// Has slight modification.
// =========================================

const _: () = {
    type __StructFieldsAlias<'w, 's, Logic> = (
        StaticSystemParam<
            'w,
            's,
            <Logic as SystemParameterLogicExtension>::Parameter<'static, 'static>,
        >,
        Local<'s, Logic>,
    );
    #[doc(hidden)]
    struct FetchState<Logic: SystemParameterLogicExtension, >{
        state: <__StructFieldsAlias:: <'static,'static,Logic>as ::bevy::ecs::system::SystemParam> ::State,
    }

    #[deny(clippy::missing_trait_methods)]
    unsafe impl<Logic: SystemParameterLogicExtension> ::bevy::ecs::system::SystemParam
        for CustomLogic<'_, '_, Logic>
    {
        type State = FetchState<Logic>;
        type Item<'w, 's> = CustomLogic<'w, 's, Logic>;
        fn init_state(world: &mut ::bevy::ecs::world::World) -> Self::State {
            FetchState {
                state: <__StructFieldsAlias:: <'_,'_,Logic>as ::bevy::ecs::system::SystemParam> ::init_state(world),
            }
        }
        fn init_access(
            state: &Self::State,
            system_meta: &mut ::bevy::ecs::system::SystemMeta,
            component_access_set: &mut ::bevy::ecs::query::FilteredAccessSet,
            world: &mut ::bevy::ecs::world::World,
        ) {
            <__StructFieldsAlias<'_, '_, Logic> as ::bevy::ecs::system::SystemParam>::init_access(
                &state.state,
                system_meta,
                component_access_set,
                world,
            );
        }
        fn apply(
            state: &mut Self::State,
            system_meta: &::bevy::ecs::system::SystemMeta,
            world: &mut ::bevy::ecs::world::World,
        ) {
            <__StructFieldsAlias<'_, '_, Logic> as ::bevy::ecs::system::SystemParam>::apply(
                &mut state.state,
                system_meta,
                world,
            );
        }
        fn queue(
            state: &mut Self::State,
            system_meta: &::bevy::ecs::system::SystemMeta,
            world: ::bevy::ecs::world::DeferredWorld,
        ) {
            <__StructFieldsAlias<'_, '_, Logic> as ::bevy::ecs::system::SystemParam>::queue(
                &mut state.state,
                system_meta,
                world,
            );
        }
        #[inline]
        unsafe fn get_param<'w, 's>(
            state: &'s mut Self::State,
            system_meta: &::bevy::ecs::system::SystemMeta,
            world: ::bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'w>,
            change_tick: ::bevy::ecs::change_detection::Tick,
        ) -> ::core::result::Result<
            Self::Item<'w, 's>,
            ::bevy::ecs::system::SystemParamValidationError,
        > {
            let (field0, field1) = &mut state.state;
            let mut field0 = unsafe {
                <StaticSystemParam<
                    'w,
                    's,
                    <Logic as SystemParameterLogicExtension>::Parameter<'static, 'static>,
                > as ::bevy::ecs::system::SystemParam>::get_param(
                    field0,
                    system_meta,
                    world,
                    change_tick,
                )
            }
            .map_err(|err| {
                ::bevy::ecs::system::SystemParamValidationError::new::<Self>(
                    err.skipped,
                    err.message,
                    "::0",
                )
            })?;
            let mut field1 = unsafe {
                <Local<'s, Logic> as ::bevy::ecs::system::SystemParam>::get_param(
                    field1,
                    system_meta,
                    world,
                    change_tick,
                )
            }
            .map_err(|err| {
                ::bevy::ecs::system::SystemParamValidationError::new::<Self>(
                    err.skipped,
                    err.message,
                    "::1",
                )
            })?;

            field1.logic(&mut *field0);
            Ok(CustomLogic(field0, field1))
        }
    }
    unsafe impl<'w, 's, Logic: SystemParameterLogicExtension> ReadOnlySystemParam
        for CustomLogic<'w, 's, Logic>
    where
        StaticSystemParam<
            'w,
            's,
            <Logic as SystemParameterLogicExtension>::Parameter<'static, 'static>,
        >: ReadOnlySystemParam,
        Local<'s, Logic>: ::bevy::ecs::system::ReadOnlySystemParam,
    {
    }
};
