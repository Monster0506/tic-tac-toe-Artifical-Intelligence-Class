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
    if board.playable().next().is_none() {
        return Outcome::Draw;
    }
    if board.playable().any(|col| board.wins(col)) {
        return Outcome::Win;
    }
    for step in 0..G::COLUMNS {
        let col = centre_first::<G>(step);
        if !board.can_play(col) {
            continue;
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

const fn centre_first<G: Geometry>(step: u32) -> u32 {
    let centre = G::COLUMNS / 2;
    let distance = step.div_ceil(2);
    if step.is_multiple_of(2) {
        centre + distance
    } else {
        centre - distance
    }
}
