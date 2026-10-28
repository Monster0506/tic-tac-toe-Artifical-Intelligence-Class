use std::{cmp::Ordering, collections::HashMap, error::Error};

use c4::board::{B76, Board, Geometry, Grid};

// Pascal Pons, end-game test set Test_L3_R1: http://blog.gamesolver.org/data/Test_L3_R1
const PONS: &str = include_str!("data/Test_L3_R1");
const PONS_POSITIONS: usize = 1000;

fn replay<G: Geometry>(moves: &str) -> Option<Board<G>> {
    let mut board = Board::<G>::default();
    for ch in moves.chars() {
        let col = ch.to_digit(10)?.checked_sub(1)?;
        if col >= G::COLUMNS || !board.can_play(col) || board.wins(col) {
            return None;
        }
        board.play(col);
    }
    Some(board)
}

fn solve<G: Geometry>(board: Board<G>, memo: &mut HashMap<G::Bits, Ordering>) -> Ordering {
    let key = board.key();
    if let Some(&known) = memo.get(&key) {
        return known;
    }
    let outcome = (0..G::COLUMNS)
        .filter(|&col| board.can_play(col))
        .map(|col| {
            if board.wins(col) {
                return Ordering::Greater;
            }
            let mut child = board;
            child.play(col);
            solve(child, memo).reverse()
        })
        .max()
        .unwrap_or(Ordering::Equal);
    memo.insert(key, outcome);
    outcome
}

fn pons_matches_oracle<G: Geometry>() -> Result<(), Box<dyn Error>> {
    assert_eq!(PONS.lines().count(), PONS_POSITIONS);
    let mut memo = HashMap::new();
    for line in PONS.lines() {
        let (moves, score) = line.split_once(' ').ok_or(line)?;
        let board = replay::<G>(moves).ok_or(line)?;
        let expected = score.parse::<i32>()?.cmp(&0);
        assert_eq!(solve(board, &mut memo), expected, "{line}");
    }
    Ok(())
}

#[test]
fn pons_positions_match_oracle() -> Result<(), Box<dyn Error>> {
    pons_matches_oracle::<B76>()?;
    pons_matches_oracle::<Grid<7, 6, u128>>()
}
