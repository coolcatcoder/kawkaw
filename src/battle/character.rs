use crate::battle::{CHARACTER_HALF, UiCommands, main_box};
use bevy::{
    color::palettes::css::{BLACK, BLUE},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_message::<CharacterUiMessage>()
        .add_systems(Startup, characters)
        .add_systems(Update, CharacterUiMessage::system);
}

#[derive(Resource)]
pub struct Characters {
    slots: Vec<Character>,
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
                    transform.get_mut(character.parent).unwrap().translation.y =
                        CHARACTER_HALF * 300.;
                }
                Self::Lower(index) => {
                    let character = &characters.slots[index.strict_cast::<usize>()];
                    transform.get_mut(character.parent).unwrap().translation.y -=
                        CHARACTER_HALF * 300.;
                }
            }
        }
    }
}

struct Character {
    health: u32,

    parent: Entity,
    text: Entity,
}

fn characters(mut commands: Commands) {
    commands.insert_resource(Characters { slots: vec![] });
}

impl UiCommands<'_, '_> {
    pub fn characters_setup(&mut self) {
        let previous_style = self.style.clone();

        let slots = (0..3)
            .map(|index| {
                let offset = if index == 0 { 0. } else { CHARACTER_HALF };

                self.depth(2.);
                self.outline(Some((BLUE, 2.)));
                self.fill(BLACK);
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

                let parent = self
                    .commands
                    .spawn((Visibility::Visible, Transform::default()))
                    .add_children(&[top, bottom])
                    .id();

                Character {
                    health: 100,
                    parent,
                    text: Entity::PLACEHOLDER,
                }
            })
            .collect();
        self.characters.slots = slots;

        for i in 0..3 {
            self.health_text(i);
        }

        self.style = previous_style;
    }

    fn health_text(&mut self, character_index: u8) {
        let character = &self.characters.slots[character_index as usize];
        self.commands.entity(character.text).try_despawn();
        let parent = character.parent;
        let health = character.health;

        let text = self.text(
            (
                1. / 3. * character_index as f32 + 0.04,
                main_box::MAX_Y - CHARACTER_HALF * 0.5,
            ),
            &format!("GASTER (HP {health}/1)"),
        );

        self.commands.entity(parent).add_child(text);
        self.characters.slots[character_index as usize].text = text;
    }

    pub fn health(&mut self, character_index: u8, mut f: impl FnMut(&mut u32)) {
        f(&mut self.characters.slots[character_index as usize].health);
        self.health_text(character_index);
    }
}
