use bevy::{
    color::palettes::css::{GREEN, RED, YELLOW},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn)
        .add_systems(Update, (render, iterate));
}

#[derive(Component)]
struct Node(Vec2);

// impl Node {
//     fn move_to_anchor(&mut self, anchor: Vec2) {
//         self.translation_one = anchor;

//         let relative = self.translation_two - self.translation_one;
//         let direction = relative.normalize();
//         self.translation_two = direction * 10. + self.translation_one;
//     }

//     fn move_to_anchor_reversed(&mut self, anchor: Vec2) {
//         self.translation_two = anchor;

//         let relative = self.translation_one - self.translation_two;
//         let direction = relative.normalize();
//         self.translation_one = direction * 10. + self.translation_two;
//     }
// }

#[derive(Component)]
struct Next(Entity);

#[derive(Component)]
struct Previous(Entity);

fn spawn(mut commands: Commands) {
    let mut next = commands.spawn_empty().id();

    let mut previous = commands.spawn((Node(Vec2::ZERO), Next(next))).id();

    for _ in 0..10 {
        let current = next;
        next = commands.spawn_empty().id();

        previous = commands
            .entity(current)
            .insert((Node(Vec2::ZERO), Next(next), Previous(previous)))
            .id();
    }

    commands
        .entity(next)
        .insert((Node(Vec2::ZERO), Previous(previous)));
}

fn render(
    mut gizmos: Gizmos,
    tails: Query<(&Node, &Next), Without<Previous>>,
    lines: Query<(&Node, Option<&Next>)>,
) {
    for (tail_node, tail_next) in tails {
        let mut maybe_next = Some(tail_next.0);
        let mut previous_translation = tail_node.0;
        while let Some(next) = maybe_next {
            let (node, next) = lines.get(next).unwrap();

            gizmos.line_2d(previous_translation, node.0, RED);
            gizmos.circle_2d(node.0, 5., GREEN);

            maybe_next = next.map(|next| next.0);
            previous_translation = node.0;
        }
    }
}

fn one_to_n_minus_one(
    tail: Entity,
    nodes: &mut Query<(&mut Node, Option<&Next>, Option<&Previous>)>,
    mut f: impl FnMut(Vec2, &mut Node),
) {
    // let Ok((tail_node, Some(tail_next), None)) = nodes.get(tail) else {
    //     panic!()
    // };

    // let mut previous_node = tail_node.0;
    // let mut maybe_entity = Some(tail_next.0);

    // while let Some(entity) = maybe_entity {
    //     let (mut node, next, _) = nodes.get_mut(entity).unwrap();

    //     f(previous_node, &mut node);

    //     maybe_entity = next.map(|next| next.0);
    //     previous_node = node.0;
    // }

    let mut one = tail;

    loop {
        let (_, one_next, _) = nodes.get(one).unwrap();
        let (two_node, two_next, _) = nodes.get(one_next.unwrap().0).unwrap();
        let two_node = two_node.0;
        let two_next = two_next.map(|next| next.0);
    }
}

// Copied from https://www.andreasaristidou.com/publications/papers/FABRIK.pdf
fn iterate(
    //mut lines: Query<(&mut Node, &Next, &Previous)>,
    //mut head: Query<(&mut Node, &Previous), Without<Next>>,
    //mut tail: Query<(&mut Node, &Next), Without<Previous>>,
    tail: Query<Entity, (With<Node>, Without<Previous>)>,
    mut nodes: Query<(&mut Node, Option<&Next>, Option<&Previous>)>,
) {
    return;
    for tail in tail {
        let mut i = 1;

        one_to_n_minus_one(tail, &mut nodes, |_, _| {
            println!("{i}");
            i += 1;
        });

        // let root = Vec2::ZERO;
        // let t = Vec2::new(50., 80.);

        // let dist = root.distance(t);

        // if dist > 10. * 10. {
        //     let mut maybe_next = Some(tail);
        //     while let Some(next) = maybe_next {
        //         maybe_next = next.map(|next| next.0);

        //         let r_i = (t - node.0).abs();
        //         let ki = d / r_i;
        //     }
        // }
    }
}

// fn iterate(
//     mut lines: Query<(&mut Node, &Next, &Previous)>,
//     mut head: Query<(&mut Node, &Previous), Without<Next>>,
//     mut tail: Query<(&mut Node, &Next), Without<Previous>>,
// ) {
//     for _ in 0..1 {
//         for (mut head_line, head_previous) in head.iter_mut() {
//             head_line.move_to_anchor(Vec2::new(50., 80.));

//             let mut entity = head_previous.0;
//             let mut anchor = head_line.translation_two;
//             while let Ok((mut line, _, previous)) = lines.get_mut(entity) {
//                 entity = previous.0;

//                 line.move_to_anchor(anchor);

//                 anchor = line.translation_two;
//             }

//             tail.get_mut(entity).unwrap().0.move_to_anchor(anchor);
//         }

//         for (mut tail_line, tail_next) in tail.iter_mut() {
//             tail_line.move_to_anchor_reversed(Vec2::new(0., 0.));

//             let mut entity = tail_next.0;
//             let mut anchor = tail_line.translation_one;
//             while let Ok((mut line, next, _)) = lines.get_mut(entity) {
//                 entity = next.0;

//                 line.move_to_anchor_reversed(anchor);

//                 anchor = line.translation_one;
//             }

//             head.get_mut(entity)
//                 .unwrap()
//                 .0
//                 .move_to_anchor_reversed(anchor);
//         }
//     }
// }
