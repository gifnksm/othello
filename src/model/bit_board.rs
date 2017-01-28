use super::{Point, Size};
use std::ops::BitAnd;
use std::slice;

pub type BitBoard = u64;
const BIT_BOARD_BITS: u32 = 64;

#[derive(Copy, Clone, Debug)]
pub struct Offset {
    offs: [u32; 4],
    masks: [BitBoard; 8],
}

impl Offset {
    pub fn from_size(size: Size) -> Self {
        Offset {
            offs: Self::offs(size),
            masks: Self::masks(size),
        }
    }

    fn offs(size: Size) -> [u32; 4] {
        let r = pt2off((1, 0), size);
        let d = pt2off((0, 1), size);
        let r_off = r;
        let d_off = d;
        let dl_off = d - r;
        let dr_off = d + r;

        [r_off, d_off, dl_off, dr_off]
    }

    fn masks(size: Size) -> [BitBoard; 8] {
        let all_mask = all_mask(size);
        let mut r_mask = all_mask;
        let mut l_mask = all_mask;
        for y in 0..size.1 {
            r_mask ^= pt2mask((size.0 - 1, y), size);
            l_mask ^= pt2mask((0, y), size);
        }

        let mut d_mask = all_mask;
        let mut u_mask = all_mask;
        for x in 0..size.0 {
            d_mask ^= pt2mask((x, size.1 - 1), size);
            u_mask ^= pt2mask((x, 0), size);
        }

        let dl_mask = d_mask & l_mask;
        let dr_mask = d_mask & r_mask;
        let ur_mask = u_mask & r_mask;
        let ul_mask = u_mask & l_mask;

        [r_mask, d_mask, dl_mask, dr_mask, l_mask, u_mask, ur_mask, ul_mask]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Mask {
    masks: [BitBoard; 8],
}

impl Mask {
    pub fn new(mask: BitBoard) -> Self {
        Mask { masks: [mask; 8] }
    }

    pub fn shift(self, offset: Offset) -> Mask {
        Mask {
            masks: [(self.masks[0] & offset.masks[0]) << offset.offs[0],
                    (self.masks[1] & offset.masks[1]) << offset.offs[1],
                    (self.masks[2] & offset.masks[2]) << offset.offs[2],
                    (self.masks[3] & offset.masks[3]) << offset.offs[3],
                    (self.masks[4] & offset.masks[4]) >> offset.offs[0],
                    (self.masks[5] & offset.masks[5]) >> offset.offs[1],
                    (self.masks[6] & offset.masks[6]) >> offset.offs[2],
                    (self.masks[7] & offset.masks[7]) >> offset.offs[3]],
        }
    }

    pub fn or_all(self) -> BitBoard {
        self.masks[0] | self.masks[1] | self.masks[2] | self.masks[3] | self.masks[4] |
        self.masks[5] | self.masks[6] | self.masks[7]
    }

    pub fn iter_mut(&mut self) -> slice::IterMut<BitBoard> {
        self.masks.iter_mut()
    }
}

impl BitAnd for Mask {
    type Output = Mask;

    fn bitand(self, rhs: Mask) -> Mask {
        Mask {
            masks: [self.masks[0] & rhs.masks[0],
                    self.masks[1] & rhs.masks[1],
                    self.masks[2] & rhs.masks[2],
                    self.masks[3] & rhs.masks[3],
                    self.masks[4] & rhs.masks[4],
                    self.masks[5] & rhs.masks[5],
                    self.masks[6] & rhs.masks[6],
                    self.masks[7] & rhs.masks[7]],
        }
    }
}

pub fn all_mask(size: Size) -> BitBoard {
    let num_cell = size.0 * size.1;
    if num_cell == BIT_BOARD_BITS {
        !0
    } else {
        (1 << num_cell) - 1
    }
}

pub fn pt2mask(pt: Point, size: Size) -> BitBoard {
    1 << pt2off(pt, size)
}

fn pt2off((x, y): Point, (_, sy): Size) -> u32 {
    (x + sy * y)
}

fn off2ptr(off: u32, size: Size) -> Point {
    (off % size.1, off / size.1)
}

pub fn points(mask: BitBoard, size: Size) -> Points {
    Points {
        size: size,
        mask: mask,
    }
}

pub struct Points {
    size: Size,
    mask: BitBoard,
}

impl Iterator for Points {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.mask == 0 {
            return None;
        }
        let off = self.mask.trailing_zeros();
        self.mask ^= 1 << off;
        Some(off2ptr(off, self.size))
    }
}

impl DoubleEndedIterator for Points {
    fn next_back(&mut self) -> Option<Point> {
        if self.mask == 0 {
            return None;
        }
        let off = self.mask.leading_zeros();
        self.mask ^= 1 << off;
        Some(off2ptr(off, self.size))
    }
}

impl ExactSizeIterator for Points {
    fn len(&self) -> usize {
        self.mask.count_ones() as usize
    }
}
