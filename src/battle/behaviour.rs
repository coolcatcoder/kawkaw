use std::ops::Index;

use bevy::prelude::*;

use crate::battle::{
    CHARACTER_HALF,
    character::{Characters, SlotMessage},
    draw::Draw,
    main_box,
};

pub fn plugin(app: &mut App) {
    app.add_message::<BehaviourMessage>()
        .add_systems(Startup, insert_behaviour)
        .add_systems(Update, (slot_message, behaviour_message));
}

#[derive(Resource)]
struct Behaviour {
    parent: Entity,
    list: [Entity; 5],
}
impl Index<u8> for Behaviour {
    type Output = Entity;

    fn index(&self, index: u8) -> &Self::Output {
        &self.list[usize::from(index)]
    }
}

fn insert_behaviour(
    mut draw: Draw,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
) {
    let battle = asset_server.load("battle.png");
    let battle_layout = TextureAtlasLayout::from_grid(
        UVec2::new(31, 32),
        6,
        3,
        Some(UVec2::splat(1)),
        Some(UVec2::new(1182, 260)),
    );
    let battle_layout = texture_atlas_layout.add(battle_layout);

    let character_index = 0;

    draw.depth(6.);
    let menus: [_; 5] = core::array::from_fn(|i| {
        let sprite_index = if i == 0 { 0 } else { i + 12 };

        draw.sprite(
            (
                1. / 3. * character_index as f32 + (1. / 3. / 6. * (i + 1) as f32),
                main_box::MAX_Y - CHARACTER_HALF * 0.5,
            ),
            Sprite {
                image: battle.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: battle_layout.clone(),
                    index: sprite_index,
                }),
                custom_size: Some(Vec2::new(31., 32.) * 0.65),
                ..default()
            },
        )
    });

    let parent = draw
        .commands()
        .spawn((Transform::default(), Visibility::Hidden))
        .add_children(&menus)
        .id();

    draw.commands().insert_resource(Behaviour {
        parent,
        list: menus,
    });
}

fn slot_message(
    mut slot_message: MessageReader<SlotMessage>,
    behaviour: Res<Behaviour>,
    mut transform_and_visibility: Query<(&mut Transform, &mut Visibility)>,
) {
    for slot_message in slot_message.read() {
        match slot_message {
            SlotMessage::Raise(index) => {
                let (mut transform, mut visibility) =
                    transform_and_visibility.get_mut(behaviour.parent).unwrap();
                transform.translation.x = 1. / 3. * *index as f32 * 400.;
                *visibility = Visibility::Visible;
            }
            SlotMessage::Lower(_) => {
                let (_, mut visibility) =
                    transform_and_visibility.get_mut(behaviour.parent).unwrap();
                *visibility = Visibility::Hidden;
            }
        }
    }
}

#[derive(Message)]
pub enum BehaviourMessage {
    Highlight(u8),
    Lowlight(u8),
}

fn behaviour_message(
    mut behaviour_message: MessageReader<BehaviourMessage>,
    behaviour: Res<Behaviour>,
    mut sprite: Query<&mut Sprite>,
) {
    for behaviour_message in behaviour_message.read() {
        match behaviour_message {
            BehaviourMessage::Highlight(index) => {
                sprite
                    .get_mut(behaviour[*index])
                    .unwrap()
                    .texture_atlas
                    .as_mut()
                    .unwrap()
                    .index -= 12;
            }
            BehaviourMessage::Lowlight(index) => {
                sprite
                    .get_mut(behaviour[*index])
                    .unwrap()
                    .texture_atlas
                    .as_mut()
                    .unwrap()
                    .index += 12;
            }
        }
    }
}

// fn menus(&mut self, character_index: u8) -> Vec<Entity> {
//         let previous_style = self.style.clone();

//         self.depth(6.);
//         let menus = (0..5)
//             .map(|i| {
//                 let sprite_index = if i == 0 { 0 } else { i + 12 };

//                 self.sprite(
//                     (
//                         1. / 3. * character_index as f32 + (1. / 3. / 6. * (i + 1) as f32),
//                         main_box::MAX_Y - CHARACTER_HALF * 0.5,
//                     ),
//                     Sprite {
//                         image: self.handles.battle.clone(),
//                         texture_atlas: Some(TextureAtlas {
//                             layout: self.handles.battle_layout.clone(),
//                             index: sprite_index,
//                         }),
//                         custom_size: Some(Vec2::new(31., 32.) * 0.65),
//                         ..default()
//                     },
//                 )
//             })
//             .collect();

//         self.style = previous_style;
//         menus
//     }
