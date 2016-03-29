#[cfg(test)]
mod tests;

mod game;
mod ai;
mod human_ai;
mod constants;
mod helpers;
use game::{VictoryStats, GameStructure};
use constants::{LINES, PARALLELOGRAMS, PLUSSES};
use std::rc::Rc;

#[allow(dead_code)]
fn run_all_matches(endurance : Vec<i32>, precision : i32) -> Vec<Vec<(i32, i32, f32)>> {
    let structure = Rc::new(GameStructure::new(&PARALLELOGRAMS));
    let mut result = Vec::new();
    for i in 0..endurance.len() {
        let mut result_row = Vec::new();
        for j in 0..endurance.len() {
            let mut result_cell = VictoryStats::new();
            println!("Comparing {:?} to {:?}", endurance[i], endurance[j]);
            let mut p1 = ai::MonteCarloAI::new(structure.clone(), endurance[i]);
            let mut p2 = ai::MonteCarloAI::new(structure.clone(), endurance[j]);
            for i in 0..precision {
                println!("    Game {} of {}", i, precision);
                let state = ai::run_match(&structure, &mut p1, &mut p2);
                match state.victory_state {
                    game::VictoryState::Win(game::PlayerColor::White) => result_cell.white += 1,
                    game::VictoryState::Win(game::PlayerColor::Black) => result_cell.black += 1,
                    game::VictoryState::Draw      => result_cell.draws  += 1,
                    game::VictoryState::Undecided => (),
                }
            }
            result_row.push(calculate_rank_difference(result_cell));
        }
        result.push(result_row);
    }
    println!("{:?}", result);
    return result;
}

#[allow(dead_code)]
fn calculate_rank_difference(stats : VictoryStats) -> (i32, i32, f32) {
    let white_loss_frequency = (stats.black as f32) / ((stats.black as f32) + (stats.white as f32));
    return (stats.white, stats.black, (1.0/white_loss_frequency - 1.0).ln());
}



fn main() {
    let structure = Rc::new(GameStructure::new(&PLUSSES));

    /*let mut tree = ai::MCTreeAI::new(structure.clone(), 1000);
    for _ in 0..10000 {
        tree.simulate_playout();
    }
    tree.root.print_some_info();*/

    //run_all_matches(vec![500, 1000, 1500], 10);

    //let mut p1 = ai::MCTreeAI::new(structure.clone(), 10000);
    let mut p1 = ai::MonteCarloAI::new(structure.clone(), 10000);
    //let p2 = ai::TreeJudgementAI::new(structure.clone(), 3);
    //let p1 = ai::RandomSogoAI::new();
    //let p1 = ai::TreeJudgementAI::new(4);
    let mut p2 = human_ai::HumanPlayer::Active;
    let mut statics = VictoryStats { white : 0, black : 0, draws : 0};
    for i in 0..20 {
        println!("Game {}.", i);
        let state = ai::run_match(&structure, &mut p1, &mut p2);
        match state.victory_state {
            game::VictoryState::Win(game::PlayerColor::White) => statics.white += 1,
            game::VictoryState::Win(game::PlayerColor::Black) => statics.black += 1,
            game::VictoryState::Draw      => statics.draws  += 1,
            game::VictoryState::Undecided => (),

        }
        //println!("The game took {:?} turns and ended with {:?}.", state.age, state.victory_state);
        //human_ai::print_gamestate(&state);
    }
    println!("There were {} white and {} black wins as well as {} draws.", statics.white, statics.black, statics.draws);
    //*/
}
