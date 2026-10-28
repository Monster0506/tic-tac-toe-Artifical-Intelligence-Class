use std::collections::{HashMap, HashSet};

use c4::board::{B87, B98, Board, Geometry};

// Linear congruential generator, Numerical Recipes (Press et al.).
const LCG_MULTIPLIER: u32 = 1_664_525;
const LCG_INCREMENT: u32 = 1_013_904_223;
const OUTPUT_SHIFT: u32 = 16;
const GAMES: u32 = 3_000;
const MOVES: u32 = 200;

fn grid_wins(own: &HashSet<(i64, i64)>, (column, row): (i64, i64)) -> bool {
    let run = |(dcol, drow): (i64, i64)| {
        (1..4)
            .take_while(|&step| own.contains(&(column + step * dcol, row + step * drow)))
            .count()
    };
    [(1, 0), (0, 1), (1, 1), (1, -1)]
        .into_iter()
        .any(|(dcol, drow)| 1 + run((dcol, drow)) + run((-dcol, -drow)) >= 4)
}

fn play_game<G: Geometry>(seed: u32) {
    let mut board = Board::<G>::default();
    let mut sequence = String::new();
    let mut stones = [HashSet::new(), HashSet::new()];
    let mut heights = HashMap::new();
    let mut state = seed;
    for step in 0..MOVES {
        state = state
            .wrapping_mul(LCG_MULTIPLIER)
            .wrapping_add(LCG_INCREMENT);
        let column = (state >> OUTPUT_SHIFT) % G::COLUMNS;
        let next = format!("{sequence}{}", column + 1);
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
                Board::<G>::from_moves(&sequence).map(|replayed| replayed.key()),
                Some(board.key())
            );
            assert!(Board::<G>::from_moves(&next).is_none());
            return;
        }
        stones[mover].insert(cell);
        *height += 1;
        board.play(column);
        sequence = next;
    }
}

#[test]
fn wins_match_grid_oracle() {
    for seed in 0..GAMES {
        play_game::<B87>(seed);
        play_game::<B98>(seed);
    }
}
