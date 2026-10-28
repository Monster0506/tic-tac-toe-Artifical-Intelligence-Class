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

#[must_use]
pub fn solve<G: Geometry>(board: Board<G>) -> Outcome {
    negamax(board, Outcome::Loss, Outcome::Win)
}

fn negamax<G: Geometry>(board: Board<G>, mut alpha: Outcome, beta: Outcome) -> Outcome {
    if !(0..G::COLUMNS).any(|col| board.can_play(col)) {
        return Outcome::Draw;
    }
    for col in (0..G::COLUMNS).filter(|&col| board.can_play(col)) {
        if board.wins(col) {
            return Outcome::Win;
        }
        let mut child = board;
        child.play(col);
        alpha = alpha.max(negamax(child, beta.flip(), alpha.flip()).flip());
        if alpha >= beta {
            break;
        }
    }
    alpha
}
