use bevy::{color::palettes::css::GREEN, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (debug_render, render, iterate));
}

#[derive(Component)]
pub struct Node(
    pub Entity,
    pub usize,
    pub fn(&mut Transform, Vec2, Option<Vec2>),
);

#[derive(Component)]
pub struct Nodes(Vec<Vec2>);
impl Nodes {
    pub fn new(quantity: usize) -> Self {
        Self(
            (0..quantity)
                .map(|i| Vec2::new(0., i as f32 * LENGTH))
                .collect(),
        )
    }
}

const LENGTH: f32 = 10.;

fn render(node: Query<(&Node, &mut Transform)>, nodes: Query<&Nodes, Without<Node>>) {
    for (node, mut transform) in node {
        let nodes = nodes.get(node.0).unwrap();
        let translation = nodes.0[node.1];
        let translation_next = nodes.0.get(node.1 + 1).copied();
        (node.2)(&mut transform, translation, translation_next);
    }
}

fn debug_render(mut gizmos: Gizmos, nodes: Query<&Nodes>) {
    for nodes in nodes {
        for node in &nodes.0 {
            gizmos.circle_2d(*node, 5., GREEN);
        }
    }
}

// Copied from https://www.andreasaristidou.com/publications/papers/FABRIK.pdf
fn iterate(nodes: Query<&mut Nodes>) {
    let t = Vec2::new(50., 50.);
    let tol = 5.;

    for mut nodes in nodes {
        let p = &mut nodes.0;
        let n = p.len();

        // The distance between the root and the target.
        let dist = (p[0] - t).length();

        // Check whether the target is within reach.
        if dist > LENGTH * n as f32 {
            // The target is unreachable.
            for i in 0..(n - 1) {
                // Find the distance r[i] between the target t and the joint position p[i].
                let r = (t - p[i]).length();
                let k = LENGTH / r;

                // Find the new joint positions p[i].
                p[i + 1] = (1. - k) * p[i] + k * t;
            }
        } else {
            // The target is reachable; thus, set as b the initial position of the joint p[0].
            let b = p[0];
            // Check whether the distance between the end effector p[n-1] and the target t is greater than a tolerance.
            //while (p[n - 1] - t).length() > tol {
            // STAGE 1: FORWARD REACHING
            // Set the end effector p[n-1] as target t.
            p[n - 1] = t;
            for i in (0..(n - 1)).rev() {
                // Find the distance r[i] between the new joint position p[i+1] and the joint p[i].
                let r = (p[i + 1] - p[i]).length();
                let k = LENGTH / r;
                // Find the new joint positions p[i].
                p[i] = (1. - k) * p[i + 1] + k * p[i];
            }

            // STAGE 2: BACKWARD REACHING
            // Set the root p[0] its initial position.
            p[0] = b;
            for i in 0..(n - 1) {
                // Find the distance r[i] between the new joint position p[i] and the joint p[i+1].
                let r = (p[i + 1] - p[i]).length();
                let k = LENGTH / r;
                // Find the new joint positions p[i].
                p[i + 1] = (1. - k) * p[i] + k * p[i + 1];
            }
            //}
        }
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
