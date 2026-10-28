use std::marker::PhantomData;

use crate::bits::Bits;

pub trait Geometry {
    const W: u32;
    const H: u32;
    type Bits: Bits;
}

pub struct Grid<const W: u32, const H: u32, B>(PhantomData<B>);

impl<const W: u32, const H: u32, B: Bits> Geometry for Grid<W, H, B> {
    const W: u32 = W;
    const H: u32 = H;
    type Bits = B;
}

pub type B76 = Grid<7, 6, u64>;
pub type B87 = Grid<8, 7, u64>;
pub type B98 = Grid<9, 8, u128>;

pub struct Board<G: Geometry> {
    current: G::Bits,
    mask: G::Bits,
    moves: u32,
}

impl<G: Geometry> Clone for Board<G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<G: Geometry> Copy for Board<G> {}

impl<G: Geometry> Default for Board<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: Geometry> Board<G> {
    const STRIDE: u32 = G::H + 1;
    const FITS: () = {
        assert!(
            G::W * (G::H + 1) <= G::Bits::WIDTH,
            "board does not fit its bit type"
        );
    };

    #[must_use] 
    pub fn new() -> Self {
        let () = Self::FITS;
        Self {
            current: G::Bits::ZERO,
            mask: G::Bits::ZERO,
            moves: 0,
        }
    }

    pub fn moves(&self) -> u32 {
        self.moves
    }

    pub fn can_play(&self, col: u32) -> bool {
        self.mask & Self::top(col) == G::Bits::ZERO
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
        self.moves += 1;
    }

    pub fn key(&self) -> G::Bits {
        self.current + self.mask
    }

    fn spot(&self, col: u32) -> G::Bits {
        (self.mask + Self::bottom(col)) & Self::column(col)
    }

    fn bottom(col: u32) -> G::Bits {
        G::Bits::ONE << (col * Self::STRIDE)
    }

    fn top(col: u32) -> G::Bits {
        Self::bottom(col) << (G::H - 1)
    }

    fn column(col: u32) -> G::Bits {
        ((G::Bits::ONE << G::H) - G::Bits::ONE) << (col * Self::STRIDE)
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
