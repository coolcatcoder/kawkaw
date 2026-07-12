use bevy::prelude::*;

#[derive(Component)]
pub struct Text;

impl FromTemplate for Text {
    type Template = TextTemplate;
}

#[derive(Default)]
pub struct TextTemplate(pub String);

impl Template for TextTemplate {
    type Output = Text;

    fn clone_template(&self) -> Self {
        Self(self.0.clone())
    }

    fn build_template(
        &self,
        context: &mut bevy::ecs::template::TemplateContext,
    ) -> Result<Self::Output> {
        let mut x = 0.;
        let mut y = 0.;

        for char in self.0.chars() {
            if char == '\n' {
                x = 0.;
                y -= 13.;
                continue;
            }

            let char = char as u32;

            let image = context.resource::<AssetServer>().load(format!(
                "Deltarune/Fonts/English/main (8bitoperator JVE)/{char:0>5}.png"
            ));

            context.entity.with_child((
                Transform::from_translation(Vec3::new(x, y, 5.)),
                Sprite::from_image(image),
            ));

            x += 8.
        }
        Ok(Text)
    }
}
