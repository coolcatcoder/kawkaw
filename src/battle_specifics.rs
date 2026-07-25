use crate::{
    battle::{Battle, StartBattle},
    input::{Input, SoulMove},
};
use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{BLACK, LIME},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, (start_battle, souls))
        .add_systems(Update, (RedSoul::system, arena));
}

fn start_battle(mut commands: Commands, mut battle: MessageWriter<StartBattle>) {
    commands.spawn(RedSoul);
    commands.spawn(Arena);
    battle.write(StartBattle {});
}

#[derive(Resource)]
struct SoulHandles {
    layout: Handle<TextureAtlasLayout>,
    image: Handle<Image>,
}
fn souls(
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let red_soul_layout = TextureAtlasLayout::from_grid(
        UVec2::new(20, 20),
        1,
        1,
        Some(UVec2::ONE),
        Some(UVec2::new(1, 15)),
    );
    let red_soul_layout = texture_atlas_layouts.add(red_soul_layout);

    commands.insert_resource(SoulHandles {
        layout: red_soul_layout,
        image: asset_server.load("soul.png"),
    });
}

#[derive(Component)]
#[require(Transform, Visibility::Visible)]
struct RedSoul;
impl RedSoul {
    fn system(
        mut battle: Battle,
        red_soul: Query<Entity, With<RedSoul>>,
        velocity: Query<&mut LinearVelocity, With<RedSoul>>,
        soul_handles: Res<SoulHandles>,
        mut commands: Commands,
        input: Res<Input>,
    ) {
        if battle.enemy_turn_start() {
            for red_soul in red_soul {
                let mut sprite = Sprite::from_atlas_image(
                    soul_handles.image.clone(),
                    TextureAtlas {
                        layout: soul_handles.layout.clone(),
                        index: 0,
                    },
                );
                sprite.custom_size = Some(Vec2::splat(12.));

                commands.entity(red_soul).insert((
                    Transform::from_translation(Vec3::new(0., 50., 1.)),
                    sprite,
                    RigidBody::Dynamic,
                    Collider::circle(5.),
                    GravityScale(0.),
                    LockedAxes::ROTATION_LOCKED,
                ));
            }
        }

        if battle.enemy_turn() {
            for mut velocity in velocity {
                velocity.0 = input.held::<SoulMove>() * 120.;
            }
        }

        if battle.enemy_turn_end() {
            for red_soul in red_soul {
                commands.entity(red_soul).clear().insert(RedSoul);
            }
        }
    }
}

#[derive(Component)]
struct Arena;
fn arena(mut battle: Battle, entity: Query<Entity, With<Arena>>, mut commands: Commands) {
    if battle.enemy_turn_start() {
        for entity in entity {
            commands
                .entity(entity)
                .insert((
                    Sprite::from_color(BLACK, Vec2::splat(100.)),
                    Transform::from_translation(Vec3::new(0., 50., 0.1)),
                ))
                .with_child((
                    Sprite::from_color(LIME, Vec2::splat(105.)),
                    Transform::from_translation(Vec3::Z * 0.),
                ))
                .with_child((
                    Transform::from_translation(Vec3::X * (50. + 2.5 * 0.5)),
                    RigidBody::Static,
                    Collider::rectangle(2.5, 105.),
                ))
                .with_child((
                    Transform::from_translation(Vec3::X * -(50. + 2.5 * 0.5)),
                    RigidBody::Static,
                    Collider::rectangle(2.5, 105.),
                ))
                .with_child((
                    Transform::from_translation(Vec3::Y * (50. + 2.5 * 0.5)),
                    RigidBody::Static,
                    Collider::rectangle(105., 2.5),
                ))
                .with_child((
                    Transform::from_translation(Vec3::Y * -(50. + 2.5 * 0.5)),
                    RigidBody::Static,
                    Collider::rectangle(105., 2.5),
                ));
        }
    }

    if battle.enemy_turn_end() {
        for entity in entity {
            commands.entity(entity).clear().insert(Arena);
        }
    }
}
