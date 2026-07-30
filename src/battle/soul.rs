use avian2d::prelude::*;
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::battle::{Danger, audio::AudioMessage};

pub fn plugin(app: &mut App) {
    app.add_message::<Hurt>()
        .init_resource::<Invincibility>()
        .add_systems(Update, decrease_invincibility);
}

#[derive(Resource, Default)]
struct Invincibility(f32);
fn decrease_invincibility(
    time: Res<Time>,
    mut invincibility: ResMut<Invincibility>,
    sprite: Query<&mut Sprite, With<Soul>>,
) {
    if invincibility.0 <= 0. {
        for mut sprite in sprite {
            sprite.color = default();
        }

        return;
    }

    let flash_colour = if (invincibility.0 * 3.).fract() < 0.5 {
        Color::default().darker(0.7)
    } else {
        default()
    };
    for mut sprite in sprite {
        sprite.color = flash_colour;
    }

    invincibility.0 -= time.delta_secs();
}

#[derive(Message)]
struct Hurt;

#[derive(Component, Default)]
#[component(on_add = Self::on_add)]
pub struct Soul;
impl Soul {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world.commands().entity(context.entity).observe(
            |on: On<CollisionStart>,
             mut commands: Commands,
             danger: Query<&Danger>,
             mut invincibility: ResMut<Invincibility>,
             mut audio_message: MessageWriter<AudioMessage>| {
                let Ok(danger) = danger.get(on.collider2) else {
                    return;
                };

                if invincibility.0 <= 0. {
                    invincibility.0 = 0.7;
                    audio_message.write(AudioMessage::Sound("snd_hurt1.wav", default()));
                }

                if danger.despawn_on_collision {
                    commands.entity(on.collider2).despawn();
                }
            },
        );
    }
}
