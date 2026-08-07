use bevy::{color::palettes::css::BLACK, ecs::system::SystemParam, prelude::*};

use crate::{battle::Temporary, text::Text};

#[derive(SystemParam)]
pub struct Draw<'w, 's> {
    commands: Commands<'w, 's>,
    style: Temporary<Style>,
}

struct Style {
    depth: f32,
    fill: Color,
    outline: Option<(Color, f32)>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            depth: 2.,
            fill: BLACK.into(),
            outline: None,
        }
    }
}

impl<'w, 's> Draw<'w, 's> {
    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        &mut self.commands
    }

    pub fn outline(&mut self, outline: Option<(impl Into<Color>, f32)>) {
        self.style.outline = outline.map(|(colour, thickness)| (colour.into(), thickness));
    }

    pub fn fill(&mut self, fill: impl Into<Color>) {
        self.style.fill = fill.into();
    }

    pub fn depth(&mut self, depth: f32) {
        self.style.depth = depth;
    }

    pub fn translation_to_world(translation: Vec2) -> Vec2 {
        translation * Vec2::new(400., -300.) + Vec2::new(-200., 150.)
    }

    pub fn text(&mut self, translation: (f32, f32), message: &str) -> Entity {
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

    pub fn sprite(&mut self, translation: (f32, f32), sprite: Sprite) -> Entity {
        let translation = Vec2::from(translation);
        let translation = Self::translation_to_world(translation).extend(self.style.depth);

        self.commands
            .spawn((sprite, Transform::from_translation(translation)))
            .id()
    }

    /// Origin is top-left.
    pub fn rectangle(&mut self, from: (f32, f32), to: (f32, f32)) -> Entity {
        let from = Vec2::new(from.0, from.1);
        let to = Vec2::new(to.0, to.1);

        let scale = (from - to).abs() * Vec2::new(400., 300.);

        let translation = from.min(to) * Vec2::new(400., -300.)
            + Vec2::new(-200., 150.)
            + Vec2::new(scale.x * 0.5, scale.y * -0.5);

        let transform = Transform {
            translation: translation.extend(self.style.depth),
            ..default()
        };

        if let Some((outline, thickness)) = self.style.outline.as_ref() {
            self.commands
                .spawn((transform, Sprite::from_color(*outline, scale)))
                .with_child((
                    Transform {
                        translation: Vec3::Z * 1.5,
                        ..default()
                    },
                    Sprite::from_color(self.style.fill, scale - *thickness),
                ))
                .id()
        } else {
            self.commands
                .spawn((transform, Sprite::from_color(self.style.fill, scale)))
                .id()
        }
    }
}
