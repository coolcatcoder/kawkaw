use crate::battle::{BattleMessage, CHARACTER_HALF, Ui, draw::Draw, main_box};
use bevy::{
    color::palettes::css::{BLACK, BLUE},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_message::<SlotMessage>()
        .add_systems(Startup, characters)
        .add_systems(Update, (SlotMessage::system, handle_party).chain());
}

#[derive(Resource)]
pub struct Characters {
    slots: Vec<CharacterParasite>,
}
impl Characters {
    pub fn quantity(&self) -> u8 {
        self.slots.len().strict_cast()
    }
}

#[derive(Message)]
pub enum SlotMessage {
    Raise(u8),
    Lower(u8),
}
impl SlotMessage {
    fn system(
        mut message: MessageReader<Self>,
        mut battle_message: MessageWriter<BattleMessage>,
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

#[derive(Component)]
pub struct Character;

fn handle_party(
    character: Query<Entity, With<Character>>,
    mut party: ResMut<Characters>,
    mut draw: Draw,
    mut commands: Commands,
    ui: Res<Ui>,
    mut character_ui_message: MessageWriter<SlotMessage>,
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

        draw.depth(2.);
        draw.outline(Some((BLUE, 2.)));
        draw.fill(BLACK);
        let bottom = draw.rectangle(
            (1. / 3. * index as f32, main_box::MAX_Y),
            (
                1. / 3. * (index + 1) as f32,
                main_box::MAX_Y + CHARACTER_HALF,
            ),
        );
        let top = draw.rectangle(
            (1. / 3. * index as f32, main_box::MAX_Y - CHARACTER_HALF),
            (1. / 3. * (index + 1) as f32, main_box::MAX_Y),
        );

        if matches!(*ui, Ui::Empty | Ui::Character { .. }) && index == 0 {
            character_ui_message.write(SlotMessage::Raise(0));
        }

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
