#[cfg(test)]
use game;
use game::{GameStructure, VictoryState, PlayerColor};
use ai;
use ai::run_match;
use constants::LINES;
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
fn subset_coherence() {
    // This is a property based test, see QuickCheck for more information.
    use rand::{thread_rng, Rng};

    let mut rng = thread_rng();

    for _ in 0..10000 {
        // Ensure that all positions returned by the Subset iterator are
        // contained in the Subset.
        let subset = game::Subset(rng.next_u64());
        for position in subset.iter() {
            println!("{:?}", position);
            assert!(subset.contains(position));
        }
    }
}

#[test]
fn easy_judgement_values() {
    let structure = game::GameStructure::new(&LINES);

    let mut state = game::State::new();
    assert_eq!(0, ai::tree::easy_judgement(&structure, &state, game::PlayerColor::White));

    state.insert(&structure, game::Position2::new(0, 0));
    assert_eq!(7, ai::tree::easy_judgement(&structure, &state, game::PlayerColor::White));

    state.insert(&structure, game::Position2::new(0, 3));
    assert_eq!(0, ai::tree::easy_judgement(&structure, &state, game::PlayerColor::White));
}
