#[cfg(test)]
use game;
use ai;
use ai::run_match;
use constants::LINES;
use std::sync::Arc;

#[test]
fn match_mc() {
    let structure = Arc::new(game::Structure::new(&LINES));
    let mut white_player = ai::mc::MonteCarloAI::new(1000);
    let mut black_player = ai::mc::MonteCarloAI::new(1000);
    run_match(structure, &mut white_player, &mut black_player);
}

#[test]
fn match_mc_tree() {
    let structure = Arc::new(game::Structure::new(&LINES));
    let mut white_player = ai::mc::MonteCarloAI::new(1000);
    let mut black_player =
        ai::tree::TreeJudgementAI::new(structure.clone(), 2, ai::value::Simple::Subsets);
    run_match(structure, &mut white_player, &mut black_player);
}

#[test]
fn match_tree() {
    let structure = Arc::new(game::Structure::new(&LINES));
    let mut white_player =
        ai::tree::TreeJudgementAI::new(structure.clone(), 2, ai::value::Simple::Subsets);
    let mut black_player =
        ai::tree::TreeJudgementAI::new(structure.clone(), 2, ai::value::Simple::Subsets);
    run_match(structure, &mut white_player, &mut black_player);
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
