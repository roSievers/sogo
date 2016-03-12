use game::{GameStructure};
use ai::{MonteCarloAI, run_match};

#[test]
fn run_AI_match() {
    let structure = GameStructure::new();
    let black_player = MonteCarloAI::new(1000);
    let white_player = MonteCarloAI::new(1000);
    run_match(&structure, &white_player, &black_player);
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
    assert_eq!(structure.lines.len(), 76);
}
