use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, BitAnd, BitOr, BitXor, Not, Shl, Shr, Sub};

pub trait Bits:
    Copy
    + Eq
    + Hash
    + Debug
    + Add<Output = Self>
    + Sub<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Not<Output = Self>
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
{
    const ZERO: Self;
    const ONE: Self;
    const WIDTH: u32;
}

macro_rules! bits {
    ($($t:ty),*) => {
        $(
            impl Bits for $t {
                const ZERO: Self = 0;
                const ONE: Self = 1;
                const WIDTH: u32 = <$t>::BITS;
            }
        )*
    };
}

bits!(u64, u128);
