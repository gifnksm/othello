use super::{Size, Point};
use std::ops::BitAnd;

pub type BitBoard = u64;

#[derive(Copy, Clone, Debug)]
pub struct Offset {
    r: usize,
    d: usize,
    dl: usize,
    dr: usize,
    all_mask: BitBoard,
    r_mask: BitBoard,
    d_mask: BitBoard,
    dl_mask: BitBoard,
    dr_mask: BitBoard,
    l_mask: BitBoard,
    u_mask: BitBoard,
    ur_mask: BitBoard,
    ul_mask: BitBoard,
}

impl Offset {
    pub fn from_size(size: Size) -> Self {
        let num_cell = size.0 * size.1;

        let all_mask = if num_cell == 64 {
            !0
        } else {
            (1 << num_cell) - 1
        };

        let mut r_mask = all_mask;
        let mut d_mask = all_mask;
        let mut l_mask = all_mask;
        let mut u_mask = all_mask;
        for y in 0..size.1 {
            r_mask ^= pt2mask((size.0 - 1, y), size);
            l_mask ^= pt2mask((0, y), size);
        }
        for x in 0..size.0 {
            d_mask ^= pt2mask((x, size.1 - 1), size);
            u_mask ^= pt2mask((x, 0), size);
        }

        Offset {
            r: pt2off((1, 0), size) - pt2off((0, 0), size),
            d: pt2off((0, 1), size) - pt2off((0, 0), size),
            dl: pt2off((0, 1), size) - pt2off((1, 0), size),
            dr: pt2off((1, 1), size) - pt2off((0, 0), size),
            all_mask: all_mask,
            r_mask: r_mask,
            d_mask: d_mask,
            dl_mask: d_mask & l_mask,
            dr_mask: d_mask & r_mask,
            l_mask: l_mask,
            u_mask: u_mask,
            ur_mask: u_mask & r_mask,
            ul_mask: u_mask & l_mask,
        }
    }
}

pub const MASK_ELEM_COUNT: usize = 8;

#[derive(Copy, Clone, Debug)]
pub struct Mask {
    r: BitBoard,
    d: BitBoard,
    dl: BitBoard,
    dr: BitBoard,
    l: BitBoard,
    u: BitBoard,
    ur: BitBoard,
    ul: BitBoard,
}

impl Mask {
    pub fn new(mask: BitBoard) -> Self {
        Mask {
            r: mask,
            d: mask,
            dl: mask,
            dr: mask,
            l: mask,
            u: mask,
            ur: mask,
            ul: mask,
        }
    }

    pub fn shift(self, offset: Offset) -> Mask {
        Mask {
            r: (self.r & offset.r_mask) << offset.r,
            d: (self.d & offset.d_mask) << offset.d,
            dl: (self.dl & offset.dl_mask) << offset.dl,
            dr: (self.dr & offset.dr_mask) << offset.dr,
            l: (self.l & offset.l_mask) >> offset.r,
            u: (self.u & offset.u_mask) >> offset.d,
            ur: (self.ur & offset.ur_mask) >> offset.dl,
            ul: (self.ul & offset.ul_mask) >> offset.dr,
        }
    }

    pub fn or_all(self) -> BitBoard {
        self.r | self.d | self.dl | self.dr | self.l | self.u | self.ur | self.ul
    }

    pub fn elems_mut(&mut self) -> [&mut BitBoard; MASK_ELEM_COUNT] {
        [&mut self.r,
         &mut self.d,
         &mut self.dl,
         &mut self.dr,
         &mut self.l,
         &mut self.u,
         &mut self.ur,
         &mut self.ul]
    }
}

impl BitAnd for Mask {
    type Output = Mask;

    fn bitand(self, rhs: Mask) -> Mask {
        Mask {
            r: self.r & rhs.r,
            d: self.d & rhs.d,
            dl: self.dl & rhs.dl,
            dr: self.dr & rhs.dr,
            l: self.l & rhs.l,
            u: self.u & rhs.u,
            ur: self.ur & rhs.ur,
            ul: self.ul & rhs.ul,
        }
    }
}

pub fn pt2mask(pt: Point, size: Size) -> BitBoard {
    1 << pt2off(pt, size)
}

fn pt2off((x, y): Point, (_, sy): Size) -> usize {
    (x + sy * y) as usize
}
