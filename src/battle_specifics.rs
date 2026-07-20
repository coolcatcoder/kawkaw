use crate::battle::{EnemyTurnEnd, EnemyTurnStart, StartBattle};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, start_battle)
        .add_systems(Update, RedSoul::system);
}

fn start_battle(mut commands: Commands, mut battle: MessageWriter<StartBattle>) {
    commands.spawn(RedSoul);
    battle.write(StartBattle {});
}

#[derive(Component)]
struct RedSoul;
impl RedSoul {
    fn system(
        mut enemy_turn_start: MessageReader<EnemyTurnStart>,
        mut enemy_turn_end: MessageReader<EnemyTurnEnd>,
        mut is_enemy_turn: Local<bool>,
        mut commands: Commands,
    ) {
        for _ in enemy_turn_start.read() {
            info!("Start!");
            *is_enemy_turn = true;
        }

        if *is_enemy_turn {
            info!("Turn!");
        }

        for _ in enemy_turn_end.read() {
            info!("End!");
            *is_enemy_turn = false;
        }
    }
}
