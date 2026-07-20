use bevy::{
    camera::{RenderTarget, ScalingMode},
    color::palettes::css::{BLACK, DARK_BLUE},
    prelude::*,
    render::render_resource::TextureFormat,
};
use rand::RngExt;

mod battle;
mod battle_specifics;
mod fabrik;
mod text;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            fabrik::plugin,
            battle::plugin,
            battle_specifics::plugin,
        ))
        .add_systems(
            Startup,
            (
                spawn_ship,
                spawn_ocean,
                spawn_kawkaw,
                spawn_camera,
                spawn_gaster,
            ),
        )
        .run()
}

fn spawn_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = Image::new_target_texture(
        400,
        300,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    );
    let image = images.add(image);

    // Camera 2d
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(BLACK.into()),
            ..default()
        },
        Msaa::Off,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 300.,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    // Camera 3d
    let camera_transform = Transform::looking_at(
        Transform::from_translation(Vec3::new(-5., 5., 0.)),
        Vec3::ZERO,
        Vec3::Y,
    );
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(DARK_BLUE.into()),
            ..default()
        },
        camera_transform,
        RenderTarget::Image(image.clone().into()),
        Msaa::Off,
    ));
    commands.spawn(Sprite { image, ..default() });
}

fn spawn_gaster(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let image = asset_server.load("mystery.png");
    let texture_atlas_layout =
        TextureAtlasLayout::from_grid(UVec2::new(21, 50), 1, 1, None, Some(UVec2::new(3, 19)));
    let layout = texture_atlas_layouts.add(texture_atlas_layout);

    commands.spawn((
        Transform::from_translation(Vec3::new(-140., 20., 2.)),
        Sprite::from_atlas_image(image, TextureAtlas { layout, index: 0 }),
    ));
}

fn spawn_kawkaw(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let head_image = asset_server.load("kawkaw_all.png");
    let head_layout = TextureAtlasLayout::from_grid(
        UVec2::new(40, 28),
        4,
        1,
        Some(UVec2::new(1, 0)),
        Some(UVec2::new(8, 72)),
    );
    let head_layout = texture_atlas_layouts.add(head_layout);

    let body_image = asset_server.load("kawkaw_body.png");
    let body_layout = TextureAtlasLayout::from_grid(
        UVec2::new(40, 28),
        4,
        1,
        Some(UVec2::new(1, 0)),
        Some(UVec2::new(8, 72)),
    );
    let body_layout = texture_atlas_layouts.add(body_layout);

    // commands.spawn((
    //     Transform::from_translation(Vec3::Z * 2.),
    //     Sprite::from_atlas_image(
    //         head_image,
    //         TextureAtlas {
    //             layout: head_layout,
    //             index: 0,
    //         },
    //     ),
    // ));

    // for i in 0..5 {
    //     commands.spawn((
    //         Transform::from_translation(Vec3::new(i as f32 * 9., i as f32 * -7., i as f32 * 3.)),
    //         Sprite::from_atlas_image(
    //             body_image.clone(),
    //             TextureAtlas {
    //                 layout: body_layout.clone(),
    //                 index: 0,
    //             },
    //         ),
    //     ));
    // }

    let nodes = commands.spawn(fabrik::Nodes::new(10)).id();

    commands.spawn((
        fabrik::Node(nodes, 9, |transform, translation, translation_next| {
            transform.translation.x = translation.x;
            transform.translation.y = translation.y;
        }),
        Transform::from_translation(Vec3::new(0., 0., 0.)),
        Sprite::from_atlas_image(
            head_image,
            TextureAtlas {
                layout: head_layout,
                index: 0,
            },
        ),
    ));

    for i in 0..9 {
        commands.spawn((
            fabrik::Node(nodes, i, |transform, translation, translation_next| {
                transform.translation.x = translation.x;
                transform.translation.y = translation.y;
            }),
            Transform::from_translation(Vec3::new(0., 0., i as f32 * 3.)),
            Sprite::from_atlas_image(
                body_image.clone(),
                TextureAtlas {
                    layout: body_layout.clone(),
                    index: 0,
                },
            ),
        ));
    }
}

fn spawn_ocean(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let meshes = [1., 2., 5.].map(|size| meshes.add(Cuboid::new(size, 0.01, size)));

    let vertical_ranges = [-0.5..0.0, -1.3..-0.8, -2.0..-1.3];

    let lighten_ranges = [0.5..0.7, 0.25..0.3, 0.0..0.5];

    commands.spawn_scene(bsn! {
        PointLight
        Transform::from_xyz(4.0, 8.0, 4.0)
    });

    let mut rng = rand::rng();

    let horizontal_range = -10.0..10.0;

    for ((mesh, vertical_range), lighten_range) in
        meshes.into_iter().zip(vertical_ranges).zip(lighten_ranges)
    {
        for _ in 0..100 {
            let material = materials
                .add(Color::srgb(0., 0., 1.).lighter(rng.random_range(lighten_range.clone())));

            let translation = Vec3::new(
                rng.random_range(horizontal_range.clone()),
                rng.random_range(vertical_range.clone()),
                rng.random_range(horizontal_range.clone()),
            );
            commands.spawn_scene(bsn! {
                Transform {translation,}
                Mesh3d({mesh.clone()})
                MeshMaterial3d::<StandardMaterial>(material)
            });
        }
    }
}

fn spawn_ship(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Transform::from_translation(Vec3::new(0., 0.5, -3.)),
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("ship.glb"))),
    ));
}
