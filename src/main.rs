#[cfg(test)]
mod tests;

mod game;
mod ai;
mod human_ai;
use game::{VictoryStats};

fn main() {
    let structure = game::GameStructure::new();
    let p2 = ai::MonteCarloAI::new(3000);
    //let p2 = ai::MonteCarloAI::new(1000);
    //let p2 = ai::TreeJudgementAI::new(2);
    //let p1 = ai::RandomSogoAI::new();
    //let p1 = ai::TreeJudgementAI::new(4);
    let p1 = human_ai::HumanPlayer::Active;
    let mut statics = VictoryStats { white : 0, black : 0, draws : 0};
    for i in 0..1 {
        println!("Game {}.", i);
        let state = ai::run_match(&structure, &p1, &p2);
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
}
