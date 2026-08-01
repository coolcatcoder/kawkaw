use crate::battle::{CHARACTER_HALF, UiCommands, draw::Draw, main_box};
use bevy::{
    color::palettes::css::{BLACK, BLUE},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_message::<CharacterUiMessage>()
        .add_systems(Startup, characters)
        .add_systems(Update, (CharacterUiMessage::system, handle_party));
}

#[derive(Resource)]
pub struct Characters {
    slots: Vec<CharacterParasite>,
}

#[derive(Message)]
pub enum CharacterUiMessage {
    Raise(u8),
    Lower(u8),
}
impl CharacterUiMessage {
    fn system(
        mut message: MessageReader<Self>,
        characters: Res<Characters>,
        mut transform: Query<&mut Transform>,
    ) {
        for message in message.read() {
            match message {
                Self::Raise(index) => {
                    let character = &characters.slots[index.strict_cast::<usize>()];
                    transform
                        .get_mut(character.ui_parent)
                        .unwrap()
                        .translation
                        .y = CHARACTER_HALF * 300.;
                }
                Self::Lower(index) => {
                    let character = &characters.slots[index.strict_cast::<usize>()];
                    transform
                        .get_mut(character.ui_parent)
                        .unwrap()
                        .translation
                        .y -= CHARACTER_HALF * 300.;
                }
            }
        }
    }
}

struct CharacterParasite {
    host: Entity,
    // TODO: Remove.
    health: u32,

    ui_parent: Entity,
    text: Entity,
}

fn characters(mut commands: Commands) {
    commands.insert_resource(Characters { slots: vec![] });
}

impl UiCommands<'_, '_> {
    // pub fn characters_setup(&mut self) {
    //     let previous_style = self.style.clone();

    //     let slots = (0..3)
    //         .map(|index| {
    //             let offset = if index == 0 { 0. } else { CHARACTER_HALF };

    //             self.depth(2.);
    //             self.outline(Some((BLUE, 2.)));
    //             self.fill(BLACK);
    //             let bottom = self.rectangle(
    //                 (
    //                     1. / 3. * index as f32,
    //                     main_box::MAX_Y - CHARACTER_HALF + offset,
    //                 ),
    //                 (1. / 3. * (index + 1) as f32, main_box::MAX_Y + offset),
    //             );
    //             let top = self.rectangle(
    //                 (
    //                     1. / 3. * index as f32,
    //                     main_box::MAX_Y - CHARACTER_HALF * 2. + offset,
    //                 ),
    //                 (
    //                     1. / 3. * (index + 1) as f32,
    //                     main_box::MAX_Y - CHARACTER_HALF + offset,
    //                 ),
    //             );

    //             let parent = self
    //                 .commands
    //                 .spawn((Visibility::Visible, Transform::default()))
    //                 .add_children(&[top, bottom])
    //                 .id();

    //             CharacterParasite {
    //                 host: Entity::PLACEHOLDER,
    //                 health: 100,
    //                 ui_parent: parent,
    //                 text: Entity::PLACEHOLDER,
    //             }
    //         })
    //         .collect();
    //     self.characters.slots = slots;

    //     for i in 0..3 {
    //         self.health_text(i);
    //     }

    //     self.style = previous_style;
    // }

    // fn health_text_suspicious(&mut self, character_index: u8) {
    //     let character = &self.characters.slots[character_index as usize];
    //     self.commands.entity(character.text).try_despawn();
    //     let parent = character.ui_parent;
    //     let health = character.health;

    //     let text = self.text(
    //         (
    //             1. / 3. * character_index as f32 + 0.04,
    //             main_box::MAX_Y - CHARACTER_HALF * 0.5,
    //         ),
    //         &format!("GASTER (HP {health}/1)"),
    //     );

    //     self.commands.entity(parent).add_child(text);
    //     self.characters.slots[character_index as usize].text = text;
    // }

    // pub fn health_suspicious(&mut self, character_index: u8, mut f: impl FnMut(&mut u32)) {
    //     f(&mut self.characters.slots[character_index as usize].health);
    //     self.health_text_suspicious(character_index);
    // }
}

#[derive(Component)]
pub struct Character;

fn handle_party(
    character: Query<Entity, With<Character>>,
    mut party: ResMut<Characters>,
    mut draw: Draw,
    mut commands: Commands,
) {
    // See if any characters have been removed.
    for parasite in &party.slots {
        if character.contains(parasite.host) {
            continue;
        }

        todo!("Remove character!");
    }

    for character in character {
        if party
            .slots
            .iter()
            .any(|parasite| parasite.host == character)
        {
            continue;
        }

        if party.slots.len() == 3 {
            todo!("Add support for more than three slots.");
        }

        let index = party.slots.len();
        let offset = if index == 0 { 0. } else { CHARACTER_HALF };

        draw.depth(2.);
        draw.outline(Some((BLUE, 2.)));
        draw.fill(BLACK);
        let bottom = draw.rectangle(
            (
                1. / 3. * index as f32,
                main_box::MAX_Y - CHARACTER_HALF + offset,
            ),
            (1. / 3. * (index + 1) as f32, main_box::MAX_Y + offset),
        );
        let top = draw.rectangle(
            (
                1. / 3. * index as f32,
                main_box::MAX_Y - CHARACTER_HALF * 2. + offset,
            ),
            (
                1. / 3. * (index + 1) as f32,
                main_box::MAX_Y - CHARACTER_HALF + offset,
            ),
        );

        let parent = commands
            .spawn((Visibility::Visible, Transform::default()))
            .add_children(&[top, bottom])
            .id();

        party.slots.push(CharacterParasite {
            host: character,
            health: 100,

            ui_parent: parent,
            text: Entity::PLACEHOLDER,
        });
    }
}
