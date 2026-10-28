use crate::bits::Bits;
use crate::board::{Board, Geometry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
    Loss,
    Draw,
    Win,
}

impl Outcome {
    #[must_use]
    pub const fn flip(self) -> Self {
        match self {
            Self::Loss => Self::Win,
            Self::Draw => Self::Draw,
            Self::Win => Self::Loss,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub result: Outcome,
    pub columns: Vec<Option<Outcome>>,
}

const TABLE_BITS: u32 = 16;
const MASK: usize = (1 << TABLE_BITS) - 1;
const _: () = assert!(0 < TABLE_BITS && TABLE_BITS < usize::BITS);

#[derive(Clone, Copy)]
struct Entry<B> {
    key: B,
    lower: Outcome,
    upper: Outcome,
}

impl<B> Entry<B> {
    fn new(key: B, value: Outcome, low: Outcome, high: Outcome) -> Self {
        let lower = if value > low { value } else { Outcome::Loss };
        let upper = if value < high { value } else { Outcome::Win };
        Self { key, lower, upper }
    }
}

pub struct Solver<G: Geometry> {
    slots: Vec<Option<Entry<G::Bits>>>,
}

impl<G: Geometry> Default for Solver<G> {
    fn default() -> Self {
        Self {
            slots: vec![None; 1 << TABLE_BITS],
        }
    }
}

impl<G: Geometry> Solver<G> {
    #[must_use]
    pub fn solve(&mut self, board: Board<G>) -> Outcome {
        self.negamax(board, Outcome::Loss, Outcome::Win)
    }

    #[must_use]
    pub fn label(&mut self, board: Board<G>) -> Label {
        let columns: Vec<_> = (0..G::COLUMNS)
            .map(|col| board.can_play(col).then(|| self.reply(board, col)))
            .collect();
        let result = columns
            .iter()
            .flatten()
            .max()
            .copied()
            .unwrap_or(Outcome::Draw);
        Label { result, columns }
    }

    fn reply(&mut self, mut board: Board<G>, col: u32) -> Outcome {
        if board.wins(col) {
            return Outcome::Win;
        }
        board.play(col);
        self.solve(board).flip()
    }

    fn negamax(&mut self, board: Board<G>, low: Outcome, high: Outcome) -> Outcome {
        if board.playable().next().is_none() {
            return Outcome::Draw;
        }
        if board.playable().any(|col| board.wins(col)) {
            return Outcome::Win;
        }
        let key = board.key();
        let index = key.mix() & MASK;
        let cached = self.slots[index].filter(|entry| entry.key == key);
        let alpha = cached.map_or(low, |entry| low.max(entry.lower));
        let beta = cached.map_or(high, |entry| high.min(entry.upper));
        if alpha >= beta {
            return alpha;
        }
        let alpha = search(self, board, alpha, beta);
        self.slots[index] = Some(Entry::new(key, alpha, low, high));
        alpha
    }
}

fn search<G: Geometry>(
    solver: &mut Solver<G>,
    board: Board<G>,
    mut alpha: Outcome,
    beta: Outcome,
) -> Outcome {
    for step in 0..G::COLUMNS {
        let col = centre_first::<G>(step);
        if !board.can_play(col) {
            continue;
        }
        let mut child = board;
        child.play(col);
        alpha = alpha.max(solver.negamax(child, beta.flip(), alpha.flip()).flip());
        if alpha >= beta {
            break;
        }
    }
    alpha
}

const fn centre_first<G: Geometry>(step: u32) -> u32 {
    let centre = G::COLUMNS / 2;
    let distance = step.div_ceil(2);
    if step.is_multiple_of(2) {
        centre + distance
    } else {
        centre - distance
    }
}
