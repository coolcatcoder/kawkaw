use crate::battle::{Battle, StartBattle};
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, (start_battle, souls))
        .add_systems(Update, RedSoul::system);
}

fn start_battle(mut commands: Commands, mut battle: MessageWriter<StartBattle>) {
    commands.spawn(RedSoul);
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
        soul_handles: Res<SoulHandles>,
        mut commands: Commands,
    ) {
        if battle.enemy_turn_start() {
            for red_soul in red_soul {
                commands.entity(red_soul).with_child((
                    Sprite::from_atlas_image(
                        soul_handles.image.clone(),
                        TextureAtlas {
                            layout: soul_handles.layout.clone(),
                            index: 0,
                        },
                    ),
                    RigidBody::Kinematic,
                    Collider::circle(10.),
                ));
            }
        }

        if battle.enemy_turn() {}

        if battle.enemy_turn_end() {
            for red_soul in red_soul {
                commands.entity(red_soul).despawn_children();
            }
        }
    }
}
