#[cfg(test)]
use game::{GameStructure, VictoryState, PlayerColor};
use ai;
use ai::{run_match};
use constants::{LINES};
use std::rc::Rc;

#[test]
fn match_mc() {
    let structure = Rc::new(GameStructure::new(&LINES));
    let mut white_player = ai::mc::MonteCarloAI::new(structure.clone(), 1000);
    let mut black_player = ai::mc::MonteCarloAI::new(structure.clone(), 1000);
    run_match(&structure, &mut white_player, &mut black_player);
}

#[test]
fn match_mc_tree() {
    let structure = Rc::new(GameStructure::new(&LINES));
    let mut white_player = ai::mc::MonteCarloAI::new(structure.clone(), 1000);
    let mut black_player = ai::tree::TreeJudgementAI::new(structure.clone(), 2);
    run_match(&structure, &mut white_player, &mut black_player);
}

#[test]
fn match_tree() {
    let structure = Rc::new(GameStructure::new(&LINES));
    let mut white_player = ai::tree::TreeJudgementAI::new(structure.clone(), 2);
    let mut black_player = ai::tree::TreeJudgementAI::new(structure.clone(), 2);
    let result = run_match(&structure, &mut white_player, &mut black_player);
    // As the TreeJudgementAI is deterministic, the same player wins all the time.
    assert_eq!(result.victory_state, VictoryState::Win(PlayerColor::Black));
}

#[test]
fn game_structure_size() {
    let structure = Rc::new(GameStructure::new(&LINES));
    assert_eq!(structure.points.len(), 4*4*4);
    let mut i = 0;
    for p in &structure.points {
        assert_eq!(p.flat_coordinate, i);
        i += 1;
    }
}
