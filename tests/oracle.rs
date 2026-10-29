use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use c4::board::{B76, Board, Geometry, Grid};
use c4::solver::{Outcome, Solver};

// Pascal Pons, end-game test set Test_L3_R1: http://blog.gamesolver.org/data/Test_L3_R1
const PONS: &str = include_str!("data/Test_L3_R1");
const PONS_POSITIONS: usize = 1000;
const TABLE_BITS: u32 = 21;
const TINY_TABLE_BITS: u32 = 4;

fn solve<G: Geometry>(board: Board<G>, memo: &mut HashMap<G::Bits, Outcome>) -> Outcome {
    let key = board.key();
    if let Some(&known) = memo.get(&key) {
        return known;
    }
    let outcome = board
        .playable()
        .map(|col| column(board, col, memo))
        .max()
        .unwrap_or(Outcome::Draw);
    memo.insert(key, outcome);
    outcome
}

fn column<G: Geometry>(board: Board<G>, col: u32, memo: &mut HashMap<G::Bits, Outcome>) -> Outcome {
    if board.wins(col) {
        return Outcome::Win;
    }
    let mut child = board;
    child.play(col);
    solve(child, memo).flip()
}

fn pons_matches_oracle<G: Geometry>() -> Result<(), Box<dyn Error>> {
    assert_eq!(PONS.lines().count(), PONS_POSITIONS);
    let mut memo = HashMap::new();
    let mut solver = Solver::<G>::new(TABLE_BITS);
    for line in PONS.lines() {
        let (moves, score) = line.split_once(' ').ok_or(line)?;
        let board = Board::<G>::from_moves(moves).ok_or(line)?;
        let expected = match score.parse::<i32>()? {
            ..0 => Outcome::Loss,
            0 => Outcome::Draw,
            1.. => Outcome::Win,
        };
        assert_eq!(solve(board, &mut memo), expected, "{line}");
        assert_eq!(solver.solve(board), expected, "{line}");
    }
    Ok(())
}

#[test]
fn pons_positions_match_oracle() -> Result<(), Box<dyn Error>> {
    pons_matches_oracle::<B76>()?;
    pons_matches_oracle::<Grid<7, 6, u128>>()
}

fn walk<G: Geometry>(
    board: Board<G>,
    seen: &mut HashSet<G::Bits>,
    memo: &mut HashMap<G::Bits, Outcome>,
    solver: &mut Solver<G>,
) {
    if seen.insert(board.key()) {
        let label = solver.label(board);
        assert_eq!(solver.solve(board), solve(board, memo));
        assert_eq!(label.result, solve(board, memo));
        let truth: Vec<_> = (0..G::COLUMNS)
            .map(|col| board.can_play(col).then(|| column(board, col, memo)))
            .collect();
        assert_eq!(label.columns, truth);
        for col in board.playable().filter(|&col| !board.wins(col)) {
            let mut child = board;
            child.play(col);
            walk(child, seen, memo, solver);
        }
    }
}

#[test]
fn even_width_matches_oracle() {
    walk(
        Board::<Grid<4, 4, u64>>::default(),
        &mut HashSet::new(),
        &mut HashMap::new(),
        &mut Solver::new(TABLE_BITS),
    );
    walk(
        Board::<Grid<4, 4, u128>>::default(),
        &mut HashSet::new(),
        &mut HashMap::new(),
        &mut Solver::new(TABLE_BITS),
    );
}

#[test]
fn tiny_table_matches_oracle() {
    walk(
        Board::<Grid<4, 4, u64>>::default(),
        &mut HashSet::new(),
        &mut HashMap::new(),
        &mut Solver::new(TINY_TABLE_BITS),
    );
    walk(
        Board::<Grid<4, 4, u128>>::default(),
        &mut HashSet::new(),
        &mut HashMap::new(),
        &mut Solver::new(TINY_TABLE_BITS),
    );
}
