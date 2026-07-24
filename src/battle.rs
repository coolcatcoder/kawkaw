use std::ops::{Deref, DerefMut};

use crate::input::{Input, UiMove};

use super::text::Text;
use bevy::{
    color::palettes::css::{BLACK, BLUE},
    diagnostic::FrameCount,
    ecs::{
        change_detection::Tick,
        query::FilteredAccessSet,
        system::{
            ReadOnlySystemParam, StaticSystemParam, SystemMeta, SystemParam,
            SystemParamValidationError,
        },
        world::unsafe_world_cell::UnsafeWorldCell,
    },
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_message::<StartBattle>()
        .add_message::<BattleMessage>()
        .add_systems(Startup, insert_resources)
        .add_systems(Update, update_ui);
}

macro_rules! colour {
    ($($variant:ident($colour:expr)),*$(,)?) => {
        #[derive(Clone)]
        enum Colour {
            $(
                $variant
            ),*
        }
        impl Colour {
            fn to_handle(&self, colour_handles: &GeneratedColourHandles) -> Handle<ColorMaterial> {
                match self {
                    $(
                        Colour::$variant => colour_handles.$variant.clone(),
                    )*
                }
            }
        }

        #[expect(nonstandard_style)]
        struct GeneratedColourHandles {
            $(
                $variant: Handle<ColorMaterial>
            ),*
        }
        impl GeneratedColourHandles {
            fn new(colour_materials: &mut Assets<ColorMaterial>) -> Self {
                Self {
                    $(
                        $variant: colour_materials.add(Color::from($colour))
                    ),*
                }
            }
        }
    };
}

colour!(
    Black(BLACK),
    Blue(BLUE),
    DeepPurple(Srgba::rgb(0.2, 0.125, 0.2)),
);

#[derive(Resource)]
struct Handles {
    battle: Handle<Image>,
    battle_layout: Handle<TextureAtlasLayout>,

    square: Handle<Mesh>,

    colour_handles: GeneratedColourHandles,
}

#[derive(Resource)]
struct Entities {
    character_menus: Vec<CharacterMenu>,
}

#[derive(Resource)]
enum Ui {
    Empty,
    Character {
        character: u8,

        menu_hovered: u8,
        menus: Vec<Entity>,
    },
    EnemyTurn,
}

struct UiCommands<'w, 's> {
    commands: Commands<'w, 's>,
    handles: &'w Handles,
    entities: &'w mut Entities,
    sprites: Query<'w, 's, &'static mut Sprite>,
    transforms: Query<'w, 's, &'static mut Transform>,

    // For drawing rectangles.
    style: Style,
}

impl UiCommands<'_, '_> {
    fn outline(&mut self, outline: Option<(Colour, f32)>) {
        self.style.outline = outline;
    }

    fn fill(&mut self, fill: Colour) {
        self.style.fill = fill;
    }

    fn depth(&mut self, depth: f32) {
        self.style.depth = depth;
    }

    fn translation_to_world(translation: Vec2) -> Vec2 {
        translation * Vec2::new(400., -300.) + Vec2::new(-200., 150.)
    }

    fn text(&mut self, translation: (f32, f32), message: &str) -> Entity {
        let translation = Vec2::from(translation);
        let translation = Self::translation_to_world(translation).extend(1.);

        self.commands
            .spawn_scene(bsn! {
                Text(message)
                Transform {
                    translation,
                }
            })
            .id()
    }

    fn sprite(&mut self, translation: (f32, f32), sprite: Sprite) -> Entity {
        let translation = Vec2::from(translation);
        let translation = Self::translation_to_world(translation).extend(self.style.depth);

        self.commands
            .spawn((sprite, Transform::from_translation(translation)))
            .id()
    }

    /// Origin is top-left.
    fn rectangle(&mut self, from: (f32, f32), to: (f32, f32)) -> Entity {
        let from = Vec2::new(from.0, from.1);
        let to = Vec2::new(to.0, to.1);

        let scale = (from - to).abs() * Vec2::new(400., 300.);
        info!("{scale}");

        let translation = from.min(to) * Vec2::new(400., -300.)
            + Vec2::new(-200., 150.)
            + Vec2::new(scale.x * 0.5, scale.y * -0.5);
        info!("{translation}");

        let transform = Transform {
            scale: scale.extend(1.),
            translation: translation.extend(self.style.depth),
            ..default()
        };

        if let Some((outline, thickness)) = self.style.outline.as_ref() {
            self.commands
                .spawn((
                    transform,
                    Mesh2d(self.handles.square.clone()),
                    MeshMaterial2d(outline.to_handle(&self.handles.colour_handles)),
                ))
                .with_child((
                    Transform {
                        scale: ((scale - Vec2::splat(*thickness)) / scale).extend(1.),
                        translation: Vec3::Z * 1.5,
                        ..default()
                    },
                    Mesh2d(self.handles.square.clone()),
                    MeshMaterial2d(self.style.fill.to_handle(&self.handles.colour_handles)),
                ))
                .id()
        } else {
            self.commands
                .spawn((
                    transform,
                    Mesh2d(self.handles.square.clone()),
                    MeshMaterial2d(self.style.fill.to_handle(&self.handles.colour_handles)),
                ))
                .id()
        }
    }

    fn menus(&mut self, character_index: u8) -> Vec<Entity> {
        let previous_style = self.style.clone();

        self.depth(6.);
        let menus = (0..5)
            .map(|i| {
                let sprite_index = if i == 0 { 0 } else { i + 12 };

                self.sprite(
                    (
                        1. / 3. * character_index as f32 + (1. / 3. / 6. * (i + 1) as f32),
                        main_box::MAX_Y - CHARACTER_HALF * 0.5,
                    ),
                    Sprite {
                        image: self.handles.battle.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: self.handles.battle_layout.clone(),
                            index: sprite_index,
                        }),
                        custom_size: Some(Vec2::new(31., 32.) * 0.65),
                        ..default()
                    },
                )
            })
            .collect();

        self.style = previous_style;
        menus
    }

    fn character_menu_raise(&mut self, character_menu: u8) {
        let character_menu = self.entities.character_menus[character_menu as usize];
        for entity in [
            character_menu.bottom,
            character_menu.top,
            character_menu.text,
        ] {
            self.transforms.get_mut(entity).unwrap().translation.y += CHARACTER_HALF * 300.;
        }
    }
    fn character_menu_lower(&mut self, character_menu: u8) {
        let character_menu = self.entities.character_menus[character_menu as usize];
        for entity in [
            character_menu.bottom,
            character_menu.top,
            character_menu.text,
        ] {
            self.transforms.get_mut(entity).unwrap().translation.y -= CHARACTER_HALF * 300.;
        }
    }

    fn character_menus(&mut self) -> Vec<CharacterMenu> {
        let previous_style = self.style.clone();

        let character_menus = (0..3)
            .map(|index| {
                let offset = if index == 0 { 0. } else { CHARACTER_HALF };

                self.depth(2.);
                self.outline(Some((Colour::Blue, 2.)));
                self.fill(Colour::Black);
                let bottom = self.rectangle(
                    (
                        1. / 3. * index as f32,
                        main_box::MAX_Y - CHARACTER_HALF + offset,
                    ),
                    (1. / 3. * (index + 1) as f32, main_box::MAX_Y + offset),
                );
                let top = self.rectangle(
                    (
                        1. / 3. * index as f32,
                        main_box::MAX_Y - CHARACTER_HALF * 2. + offset,
                    ),
                    (
                        1. / 3. * (index + 1) as f32,
                        main_box::MAX_Y - CHARACTER_HALF + offset,
                    ),
                );
                let text = self.text(
                    (
                        1. / 3. * index as f32 + 0.04,
                        main_box::MAX_Y - CHARACTER_HALF * 1.5 + offset,
                    ),
                    "GASTER (HP 1/1)",
                );

                CharacterMenu { top, bottom, text }
            })
            .collect();

        self.style = previous_style;
        character_menus
    }

    fn highlight_off_option_under_name(&mut self, entity: Entity) {
        self.sprites
            .get_mut(entity)
            .unwrap()
            .texture_atlas
            .as_mut()
            .unwrap()
            .index += 12;
    }

    fn highlight_on_option_under_name(&mut self, entity: Entity) {
        self.sprites
            .get_mut(entity)
            .unwrap()
            .texture_atlas
            .as_mut()
            .unwrap()
            .index -= 12;
    }
}

#[derive(Clone)]
struct Style {
    depth: f32,
    fill: Colour,
    outline: Option<(Colour, f32)>,
}

#[derive(Clone, Copy)]
struct CharacterMenu {
    top: Entity,
    bottom: Entity,
    text: Entity,
}

fn insert_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let battle = asset_server.load("battle.png");
    let texture_atlas_layout = TextureAtlasLayout::from_grid(
        UVec2::new(31, 32),
        6,
        3,
        Some(UVec2::splat(1)),
        Some(UVec2::new(1182, 260)),
    );
    let battle_layout = texture_atlas_layouts.add(texture_atlas_layout);

    commands.insert_resource(Handles {
        battle,
        battle_layout,

        square: meshes.add(Rectangle::new(1., 1.)),

        colour_handles: GeneratedColourHandles::new(&mut materials),
    });
    commands.insert_resource(Ui::Empty);
    commands.insert_resource(Entities {
        character_menus: vec![],
    });
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
    handles: Res<Handles>,
    mut entities: ResMut<Entities>,
    mut ui: ResMut<Ui>,
    commands: Commands,
    input: Res<Input>,
    sprites: Query<&'static mut Sprite>,
    transforms: Query<&'static mut Transform>,
    mut message: MessageWriter<BattleMessage>,
) {
    battles.extend(battle_requests.read().cloned());
    let Some(battle) = battles.first() else {
        return;
    };

    let mut commands = UiCommands {
        commands,
        handles: &handles,
        entities: &mut entities,
        sprites,
        transforms,
        style: Style {
            depth: 2.,
            fill: Colour::Black,
            outline: None,
        },
    };

    *ui = match core::mem::replace(&mut *ui, Ui::Empty) {
        Ui::Empty => {
            // Main box.
            commands.depth(5.);
            commands.rectangle((0., main_box::main_box::MAX_Y), (1., 1.));

            // Main box outline.
            commands.fill(Colour::DeepPurple);
            commands.rectangle(
                (0., main_box::main_box::MAX_Y - main_box::outline::HEIGHT),
                (1., main_box::main_box::MAX_Y),
            );

            let menus = commands.menus(0);

            commands.text((0.05, main_box::MAX_Y + 0.05), "* Floradinn florads in!\n* Floradinn florads in!\n* Floradinn florads in!\n* Is that a cut on your face, or part of your eye?\n* The gash weaves down as if you cry.");

            commands.entities.character_menus = commands.character_menus();

            Ui::Character {
                character: 0,
                menu_hovered: 0,
                menus,
            }
        }
        Ui::Character {
            mut character,
            mut menu_hovered,
            mut menus,
        } => {
            if input.pressed::<UiMove>() {
                info!("Held = {:?}", input.held::<UiMove>());
                commands.highlight_off_option_under_name(menus[menu_hovered as usize]);

                menu_hovered = (menu_hovered as i8 + input.held::<UiMove>() as i8)
                    .rem_euclid(5)
                    .strict_cast();
                info!("{menu_hovered}");

                commands.highlight_on_option_under_name(menus[menu_hovered as usize]);
            }

            if input.pressed_old(KeyCode::Enter) {
                character += 1;
                menu_hovered = 0;

                for entity in menus.iter() {
                    commands.commands.entity(*entity).despawn();
                }

                commands.character_menu_lower(character - 1);
                if character == 3 {
                    message.write(BattleMessage::EnemyTurnStart);
                    Some(Ui::EnemyTurn)
                } else {
                    menus = commands.menus(character);
                    commands.character_menu_raise(character);
                    None
                }
            } else {
                None
            }
            .unwrap_or(Ui::Character {
                character,
                menu_hovered,
                menus,
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

#[derive(Message, Clone)]
pub struct StartBattle {}

#[derive(Message)]
pub enum BattleMessage {
    EnemyTurnStart,
    EnemyTurnEnd,
}

#[derive(SystemParam)]
pub struct Battle<'w, 's> {
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

impl Battle<'_, '_> {
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
