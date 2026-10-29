use std::time::Instant;

use crate::board::{Board, Geometry};
use crate::solver::Solver;

pub trait Agent<G: Geometry> {
    fn choose(&mut self, board: Board<G>, deadline: Instant) -> u32;
}

pub struct Perfect<G: Geometry> {
    solver: Solver<G>,
}

impl<G: Geometry> Perfect<G> {
    #[must_use]
    pub fn new(table_bits: u32) -> Self {
        Self {
            solver: Solver::new(table_bits),
        }
    }
}

impl<G: Geometry> Agent<G> for Perfect<G> {
    fn choose(&mut self, board: Board<G>, _deadline: Instant) -> u32 {
        debug_assert!(board.playable().next().is_some());
        let label = self.solver.label(board);
        (0..G::COLUMNS)
            .zip(label.columns)
            .filter_map(|(col, outcome)| Some((outcome?, col)))
            .max_by_key(|&(outcome, col)| (outcome, std::cmp::Reverse(col)))
            .map_or(0, |(_, col)| col)
    }
}
