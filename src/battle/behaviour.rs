use bevy::prelude::*;

use crate::battle::character::Characters;

pub fn plugin(app: &mut App) {
    app.init_resource::<Behaviour>()
        .add_message::<BehaviourMessage>()
        .add_systems(Update, late_start);
}

fn late_start(characters: Res<Characters>) {
    if characters.is_changed() {
        info!("Changed.");
    }
}

#[derive(Resource, Default)]
struct Behaviour {
    parent: Option<Entity>,
}

// TODO: Instead react to generic NextCharacter message, so this happens on the same frame as character.rs.
#[derive(Message)]
pub enum BehaviourMessage {
    Hide,
    Show(u8),
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
