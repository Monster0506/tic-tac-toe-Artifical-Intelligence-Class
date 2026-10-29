use std::collections::{HashMap, HashSet};

use c4::board::{B87, B98, Board, Geometry, format_moves};

#[path = "common/oracle.rs"]
pub mod oracle;
use oracle::grid_wins;

// Linear congruential generator, Numerical Recipes (Press et al.).
const LCG_MULTIPLIER: u32 = 1_664_525;
const LCG_INCREMENT: u32 = 1_013_904_223;
const OUTPUT_SHIFT: u32 = 16;
const GAMES: u32 = 3_000;
const MOVES: u32 = 200;

fn play_game<G: Geometry>(seed: u32) {
    let mut board = Board::<G>::default();
    let mut twin = Board::<G>::default();
    let mut played = Vec::new();
    let mut stones = [HashSet::new(), HashSet::new()];
    let mut heights = HashMap::new();
    let mut state = seed;
    for step in 0..MOVES {
        state = state
            .wrapping_mul(LCG_MULTIPLIER)
            .wrapping_add(LCG_INCREMENT);
        let column = (state >> OUTPUT_SHIFT) % G::COLUMNS;
        let next = format_moves(&[played.as_slice(), &[column]].concat());
        let height = heights.entry(column).or_insert(0_i64);
        let full = *height == i64::from(G::ROWS);
        assert_eq!(board.can_play(column), !full, "seed {seed} step {step}");
        if full {
            assert!(Board::<G>::from_moves(&next).is_none());
            continue;
        }
        let cell = (i64::from(column), *height);
        let mover = (stones[0].len() + stones[1].len()) % 2;
        let win = grid_wins(&stones[mover], cell);
        assert_eq!(board.wins(column), win, "seed {seed} step {step}");
        if win {
            assert_eq!(
                Board::<G>::from_moves(&format_moves(&played)).map(|replayed| replayed.key()),
                Some(board.key())
            );
            assert!(Board::<G>::from_moves(&next).is_none());
            return;
        }
        stones[mover].insert(cell);
        *height += 1;
        board.play(column);
        twin.play(G::COLUMNS - 1 - column);
        let mirrored = board.key().min(twin.key());
        assert_eq!(board.canonical(), mirrored, "seed {seed} step {step}");
        played.push(column);
    }
}

#[test]
fn wins_match_grid_oracle() {
    for seed in 0..GAMES {
        play_game::<B87>(seed);
        play_game::<B98>(seed);
    }
}
