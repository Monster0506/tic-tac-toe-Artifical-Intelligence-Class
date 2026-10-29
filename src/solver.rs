use crate::bits::Bits;
use crate::board::{Board, COLUMN_LIMIT, Geometry};

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

#[derive(Clone, Copy)]
struct Entry<B> {
    key: B,
    lower: Outcome,
    upper: Outcome,
    effort: u32,
}

impl<B> Entry<B> {
    fn new(key: B, value: Outcome, low: Outcome, high: Outcome) -> Self {
        let lower = if value > low { value } else { Outcome::Loss };
        let upper = if value < high { value } else { Outcome::Win };
        Self {
            key,
            lower,
            upper,
            effort: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct Bucket<B> {
    deep: Option<Entry<B>>,
    always: Option<Entry<B>>,
}

impl<B: Bits> Bucket<B> {
    fn probe(&self, key: B) -> Option<Entry<B>> {
        let matches = |entry: &Entry<B>| entry.key == key;
        self.deep
            .filter(matches)
            .or_else(|| self.always.filter(matches))
    }

    fn store(&mut self, entry: Entry<B>) {
        match self.deep {
            Some(deep) if deep.key == entry.key => {
                self.deep = Some(Entry {
                    lower: deep.lower.max(entry.lower),
                    upper: deep.upper.min(entry.upper),
                    effort: deep.effort.max(entry.effort),
                    ..deep
                });
            }
            Some(deep) if deep.effort > entry.effort => self.always = Some(entry),
            _ => self.deep = Some(entry),
        }
    }
}

pub struct Solver<G: Geometry> {
    buckets: Vec<Bucket<G::Bits>>,
    nodes: u64,
}

impl<G: Geometry> Solver<G> {
    #[must_use]
    pub fn new(table_bits: u32) -> Self {
        debug_assert!((1..usize::BITS).contains(&table_bits));
        Self {
            buckets: vec![
                Bucket {
                    deep: None,
                    always: None,
                };
                1 << table_bits
            ],
            nodes: 0,
        }
    }

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
        if board.can_win() {
            return Outcome::Win;
        }
        let key = board.key();
        let index = key.mix() & (self.buckets.len() - 1);
        let cached = self.buckets[index].probe(key);
        let alpha = cached.map_or(low, |entry| low.max(entry.lower));
        let beta = cached.map_or(high, |entry| high.min(entry.upper));
        if alpha >= beta {
            return alpha;
        }
        self.nodes += 1;
        let before = self.nodes;
        let alpha = search(self, board, alpha, beta);
        let effort = u32::try_from(self.nodes - before).unwrap_or(u32::MAX);
        let entry = Entry {
            effort,
            ..Entry::new(key, alpha, low, high)
        };
        self.buckets[index].store(entry);
        alpha
    }
}

fn search<G: Geometry>(
    solver: &mut Solver<G>,
    board: Board<G>,
    mut alpha: Outcome,
    beta: Outcome,
) -> Outcome {
    let mut order = [(0, 0); COLUMN_LIMIT];
    let mut count = 0;
    for step in 0..G::COLUMNS {
        let col = centre_first::<G>(step);
        if !board.can_play(col) {
            continue;
        }
        let score = board.threat_count_after(col);
        let mut at = count;
        while at > 0 && order[at - 1].0 < score {
            order[at] = order[at - 1];
            at -= 1;
        }
        order[at] = (score, col);
        count += 1;
    }
    for &(_, col) in &order[..count] {
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
