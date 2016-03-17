#[cfg(test)]
use game::{GameStructure, VictoryState, PlayerColor};
#[cfg(test)]
use ai::{MonteCarloAI, TreeJudgementAI, run_match};

#[test]
fn match_mc() {
    let structure = GameStructure::new();
    let white_player = MonteCarloAI::new(1000);
    let black_player = MonteCarloAI::new(1000);
    run_match(&structure, &white_player, &black_player);
}

#[test]
fn match_mc_tree() {
    let structure = GameStructure::new();
    let white_player = MonteCarloAI::new(1000);
    let black_player = TreeJudgementAI::new(2);
    run_match(&structure, &white_player, &black_player);
}

#[test]
fn match_tree() {
    let structure = GameStructure::new();
    let white_player = TreeJudgementAI::new(2);
    let black_player = TreeJudgementAI::new(2);
    let result = run_match(&structure, &white_player, &black_player);
    // As the TreeJudgementAI is deterministic, the same player wins all the time.
    assert_eq!(result.victory_state, VictoryState::Win(PlayerColor::Black));
}

#[test]
fn game_structure_size() {
    let structure = GameStructure::new();
    assert_eq!(structure.points.len(), 4*4*4);
    let mut i = 0;
    for p in &structure.points {
        assert_eq!(p.flat_coordinate, i);
        i += 1;
    }
}
