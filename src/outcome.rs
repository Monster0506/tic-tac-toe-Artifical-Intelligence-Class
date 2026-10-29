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

    #[must_use]
    pub const fn symbol(self) -> char {
        match self {
            Self::Win => '+',
            Self::Draw => '=',
            Self::Loss => '-',
        }
    }

    #[must_use]
    pub const fn from_symbol(symbol: char) -> Option<Self> {
        match symbol {
            '+' => Some(Self::Win),
            '=' => Some(Self::Draw),
            '-' => Some(Self::Loss),
            _ => None,
        }
    }
}
