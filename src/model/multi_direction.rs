use super::{BitBoard, Point, Size};
use std::ops::BitAnd;
use std::slice;

#[derive(Copy, Clone, Debug)]
pub struct MdOffset {
    offs: [u32; 4],
    masks: [BitBoard; 8],
}

impl MdOffset {
    pub fn from_size(size: Size) -> Self {
        MdOffset {
            offs: Self::offs(size),
            masks: Self::masks(size),
        }
    }

    fn offs(size: Size) -> [u32; 4] {
        let r = Point(1, 0).offset(size);
        let d = Point(0, 1).offset(size);
        let r_off = r;
        let d_off = d;
        let dl_off = d - r;
        let dr_off = d + r;

        [r_off, d_off, dl_off, dr_off]
    }

    fn masks(size: Size) -> [BitBoard; 8] {
        let all_mask = BitBoard::all_filled(size);
        let mut r_mask = all_mask;
        let mut l_mask = all_mask;
        for y in 0..size.1 {
            r_mask ^= BitBoard::from_point(Point(size.0 - 1, y), size);
            l_mask ^= BitBoard::from_point(Point(0, y), size);
        }

        let mut d_mask = all_mask;
        let mut u_mask = all_mask;
        for x in 0..size.0 {
            d_mask ^= BitBoard::from_point(Point(x, size.1 - 1), size);
            u_mask ^= BitBoard::from_point(Point(x, 0), size);
        }

        let dl_mask = d_mask & l_mask;
        let dr_mask = d_mask & r_mask;
        let ur_mask = u_mask & r_mask;
        let ul_mask = u_mask & l_mask;

        [
            r_mask, d_mask, dl_mask, dr_mask, l_mask, u_mask, ur_mask, ul_mask,
        ]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MdMask {
    masks: [BitBoard; 8],
}

impl MdMask {
    pub fn new(mask: BitBoard) -> Self {
        MdMask { masks: [mask; 8] }
    }

    pub fn shift(self, offset: MdOffset) -> Self {
        MdMask {
            masks: [
                (self.masks[0] & offset.masks[0]) << offset.offs[0],
                (self.masks[1] & offset.masks[1]) << offset.offs[1],
                (self.masks[2] & offset.masks[2]) << offset.offs[2],
                (self.masks[3] & offset.masks[3]) << offset.offs[3],
                (self.masks[4] & offset.masks[4]) >> offset.offs[0],
                (self.masks[5] & offset.masks[5]) >> offset.offs[1],
                (self.masks[6] & offset.masks[6]) >> offset.offs[2],
                (self.masks[7] & offset.masks[7]) >> offset.offs[3],
            ],
        }
    }

    pub fn or_all(self) -> BitBoard {
        self.masks[0]
            | self.masks[1]
            | self.masks[2]
            | self.masks[3]
            | self.masks[4]
            | self.masks[5]
            | self.masks[6]
            | self.masks[7]
    }

    pub fn iter_mut(&mut self) -> slice::IterMut<'_, BitBoard> {
        self.masks.iter_mut()
    }
}

impl BitAnd for MdMask {
    type Output = MdMask;

    fn bitand(self, rhs: Self) -> Self {
        MdMask {
            masks: [
                self.masks[0] & rhs.masks[0],
                self.masks[1] & rhs.masks[1],
                self.masks[2] & rhs.masks[2],
                self.masks[3] & rhs.masks[3],
                self.masks[4] & rhs.masks[4],
                self.masks[5] & rhs.masks[5],
                self.masks[6] & rhs.masks[6],
                self.masks[7] & rhs.masks[7],
            ],
        }
    }
}
