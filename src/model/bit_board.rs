use super::{Point, Size};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign,
               Shr, ShrAssign};

#[derive(Copy, Clone, Debug)]
pub struct BitBoard {
    bits: u64,
}

const BIT_BOARD_BITS: u32 = 64;

impl BitBoard {
    pub fn empty() -> Self {
        BitBoard { bits: 0 }
    }

    pub fn all_filled(size: Size) -> Self {
        let num_cell = size.0 * size.1;
        if num_cell == BIT_BOARD_BITS {
            BitBoard { bits: !0 }
        } else {
            BitBoard { bits: (1 << num_cell) - 1 }
        }
    }

    pub fn from_point(pt: Point, size: Size) -> Self {
        Self::from_offset(pt.offset(size))
    }

    fn from_offset(offset: u32) -> Self {
        BitBoard { bits: 1 << offset }
    }

    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    pub fn num_bits(&self) -> u32 {
        self.bits.count_ones()
    }

    pub fn points(&self, size: Size) -> Points {
        Points {
            size: size,
            mask: *self,
        }
    }

    pub fn contains(&self, pt: Point, size: Size) -> bool {
        !(*self & Self::from_point(pt, size)).is_empty()
    }
}

macro_rules! impl_binops {
    ($trt:ty, $op:ident; $trt_assign:ty, $op_assign:ident) => {
        impl $trt for BitBoard {
            type Output = Self;
            fn $op(self, rhs: Self) -> Self {
                BitBoard { bits: self.bits.$op(rhs.bits) }
            }
        }
        impl $trt_assign for BitBoard {
            fn $op_assign(&mut self, rhs: Self) {
                self.bits.$op_assign(rhs.bits)
            }
        }
    }
}
impl_binops!(BitAnd, bitand; BitAndAssign, bitand_assign);
impl_binops!(BitOr, bitor; BitOrAssign, bitor_assign);
impl_binops!(BitXor, bitxor; BitXorAssign, bitxor_assign);

macro_rules! impl_shops {
    ($trt:ty, $op:ident; $trt_assign:ty, $op_assign:ident) => {
        impl $trt for BitBoard {
            type Output = Self;
            fn $op(self, rhs: u32) -> BitBoard {
                BitBoard { bits: self.bits.$op(rhs) }
            }
        }
        impl $trt_assign for BitBoard {
            fn $op_assign(&mut self, rhs: u32) {
                self.bits.$op_assign(rhs)
            }
        }
    }
}
impl_shops!(Shl<u32>, shl; ShlAssign<u32>, shl_assign);
impl_shops!(Shr<u32>, shr; ShrAssign<u32>, shr_assign);

impl Not for BitBoard {
    type Output = BitBoard;
    fn not(self) -> BitBoard {
        BitBoard { bits: !self.bits }
    }
}

pub struct Points {
    size: Size,
    mask: BitBoard,
}

impl Iterator for Points {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.mask.is_empty() {
            return None;
        }
        let off = self.mask.bits.trailing_zeros();
        self.mask ^= BitBoard::from_offset(off);
        Some(Point::from_offset(off, self.size))
    }
}

impl DoubleEndedIterator for Points {
    fn next_back(&mut self) -> Option<Point> {
        if self.mask.is_empty() {
            return None;
        }
        let off = self.mask.bits.leading_zeros();
        self.mask ^= BitBoard::from_offset(off);
        Some(Point::from_offset(off, self.size))
    }
}

impl ExactSizeIterator for Points {
    fn len(&self) -> usize {
        self.mask.bits.count_ones() as usize
    }
}
