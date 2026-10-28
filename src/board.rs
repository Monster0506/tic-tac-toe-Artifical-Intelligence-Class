use std::marker::PhantomData;

use crate::bits::Bits;

pub trait Geometry: Copy {
    const COLUMNS: u32;
    const ROWS: u32;
    type Bits: Bits;
}

#[derive(Clone, Copy)]
pub struct Grid<const W: u32, const H: u32, B>(PhantomData<B>);

impl<const W: u32, const H: u32, B: Bits> Geometry for Grid<W, H, B> {
    const COLUMNS: u32 = W;
    const ROWS: u32 = H;
    type Bits = B;
}

pub type B76 = Grid<7, 6, u64>;
pub type B87 = Grid<8, 7, u64>;
pub type B98 = Grid<9, 8, u128>;

#[derive(Clone, Copy)]
pub struct Board<G: Geometry> {
    current: G::Bits,
    mask: G::Bits,
}

impl<G: Geometry> Default for Board<G> {
    fn default() -> Self {
        let () = Self::FITS;
        Self {
            current: G::Bits::ZERO,
            mask: G::Bits::ZERO,
        }
    }
}

impl<G: Geometry> Board<G> {
    const STRIDE: u32 = G::ROWS + 1;
    const FITS: () = {
        assert!(
            G::COLUMNS * (G::ROWS + 1) <= G::Bits::BITS,
            "board does not fit its bit type"
        );
    };

    pub fn can_play(&self, col: u32) -> bool {
        self.mask & Self::top(col) == G::Bits::ZERO
    }

    pub fn playable(self) -> impl Iterator<Item = u32> {
        (0..G::COLUMNS).filter(move |&col| self.can_play(col))
    }

    pub fn wins(&self, col: u32) -> bool {
        debug_assert!(self.can_play(col));
        Self::aligned(self.current | self.spot(col))
    }

    pub fn play(&mut self, col: u32) {
        debug_assert!(self.can_play(col));
        let spot = self.spot(col);
        self.current = self.current ^ self.mask;
        self.mask = self.mask | spot;
    }

    pub fn key(&self) -> G::Bits {
        self.current + self.mask
    }

    #[must_use]
    pub fn from_moves(moves: &str) -> Option<Self> {
        moves.chars().try_fold(Self::default(), Self::push)
    }

    fn push(mut self, ch: char) -> Option<Self> {
        let col = ch.to_digit(10)?.checked_sub(1)?;
        if col >= G::COLUMNS || !self.can_play(col) || self.wins(col) {
            return None;
        }
        self.play(col);
        Some(self)
    }

    fn spot(&self, col: u32) -> G::Bits {
        (self.mask + Self::bottom(col)) & Self::column(col)
    }

    fn bottom(col: u32) -> G::Bits {
        G::Bits::ONE << (col * Self::STRIDE)
    }

    fn top(col: u32) -> G::Bits {
        Self::bottom(col) << (G::ROWS - 1)
    }

    fn column(col: u32) -> G::Bits {
        ((G::Bits::ONE << G::ROWS) - G::Bits::ONE) << (col * Self::STRIDE)
    }

    fn aligned(stones: G::Bits) -> bool {
        [1, Self::STRIDE - 1, Self::STRIDE, Self::STRIDE + 1]
            .into_iter()
            .any(|shift| {
                let pairs = stones & (stones >> shift);
                pairs & (pairs >> (2 * shift)) != G::Bits::ZERO
            })
    }
}
