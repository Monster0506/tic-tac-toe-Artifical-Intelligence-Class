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
