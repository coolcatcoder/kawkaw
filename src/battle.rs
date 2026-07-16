use super::text::Text;
use bevy::{
    color::palettes::css::{BLACK, BLUE, GREEN, PURPLE},
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, insert_resources)
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
enum Ui {
    Empty,
    Character {
        character: u8,
        characters: Vec<CharacterMenu>,

        menu_hovered: u8,
        menus: Vec<Entity>,
    },
}

struct UiCommands<'w, 's> {
    commands: Commands<'w, 's>,
    handles: &'w Handles,

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
                self.text(
                    (
                        1. / 3. * index as f32 + 0.04,
                        main_box::MAX_Y - CHARACTER_HALF * 1.5 + offset,
                    ),
                    "GASTER (HP 1/1)",
                );

                CharacterMenu { top, bottom }
            })
            .collect();

        self.style = previous_style;
        character_menus
    }
}

#[derive(Clone)]
struct Style {
    depth: f32,
    fill: Colour,
    outline: Option<(Colour, f32)>,
}

struct CharacterMenu {
    top: Entity,
    bottom: Entity,
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

    let [black, blue, purple, green, deep_purple] =
        [BLACK, BLUE, PURPLE, GREEN, Srgba::rgb(0.2, 0.125, 0.2)]
            .map(|colour| materials.add(Color::Srgba(colour)));

    commands.insert_resource(Handles {
        battle,
        battle_layout,

        square: meshes.add(Rectangle::new(1., 1.)),

        colour_handles: GeneratedColourHandles::new(&mut materials),
    });
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

const MAIN_BOX_HEIGHT: f32 = 0.75;
const MAIN_BOX_OUTLINE_HEIGHT: f32 = 0.005;
const MAIN_BOX_FULL_HEIGHT: f32 = MAIN_BOX_HEIGHT - MAIN_BOX_OUTLINE_HEIGHT;
const CHARACTER_HALF: f32 = 0.08;

fn update_ui(
    handles: Res<Handles>,
    mut ui: ResMut<Ui>,
    commands: Commands,
    mut keyboard_input: MessageReader<KeyboardInput>,
    mut sprites: Query<&mut Sprite>,
    mut transforms: Query<&mut Transform>,
) {
    let mut commands = UiCommands {
        commands,
        handles: &handles,
        style: Style {
            depth: 2.,
            fill: Colour::Black,
            outline: None,
        },
    };

    match &mut *ui {
        Ui::Empty => {
            // Main box.
            commands.depth(5.);
            commands.rectangle((0., MAIN_BOX_HEIGHT), (1., 1.));

            // Main box outline.
            commands.fill(Colour::DeepPurple);
            commands.rectangle(
                (0., MAIN_BOX_HEIGHT - MAIN_BOX_OUTLINE_HEIGHT),
                (1., MAIN_BOX_HEIGHT),
            );

            let menus = commands.menus(0);

            commands.text((0.05, main_box::MAX_Y + 0.05), "* Floradinn florads in!\n* Floradinn florads in!\n* Floradinn florads in!\n* Is that a cut on your face, or part of your eye?\n* The gash weaves down as if you cry.");

            let characters = commands.character_menus();

            *ui = Ui::Character {
                character: 0,
                characters,
                menu_hovered: 0,
                menus,
            };
        }
        Ui::Character {
            character,
            characters,
            menu_hovered,
            menus,
        } => {
            for keyboard_input in keyboard_input.read() {
                if matches!(keyboard_input.state, ButtonState::Released) {
                    continue;
                }

                match keyboard_input.key_code {
                    KeyCode::KeyA => {
                        sprites
                            .get_mut(menus[*menu_hovered as usize])
                            .unwrap()
                            .texture_atlas
                            .as_mut()
                            .unwrap()
                            .index += 12;

                        if *menu_hovered == 0 {
                            *menu_hovered = 4
                        } else {
                            *menu_hovered -= 1;
                        }

                        sprites
                            .get_mut(menus[*menu_hovered as usize])
                            .unwrap()
                            .texture_atlas
                            .as_mut()
                            .unwrap()
                            .index -= 12;
                    }
                    KeyCode::KeyD => {
                        sprites
                            .get_mut(menus[*menu_hovered as usize])
                            .unwrap()
                            .texture_atlas
                            .as_mut()
                            .unwrap()
                            .index += 12;

                        if *menu_hovered == 4 {
                            *menu_hovered = 0
                        } else {
                            *menu_hovered += 1;
                        }

                        sprites
                            .get_mut(menus[*menu_hovered as usize])
                            .unwrap()
                            .texture_atlas
                            .as_mut()
                            .unwrap()
                            .index -= 12;
                    }
                    _ => (),
                }
            }
        }
        _ => todo!(),
    }

    keyboard_input.read().for_each(|_| {});
}
