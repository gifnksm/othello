use super::{MAX_SIZE, MIN_SIZE, Point, Size};
use super::bit_board::{self, BitBoard, Mask, Offset};
use Side;
use std::cmp;

#[derive(Copy, Clone, Debug)]
pub struct Board {
    size: Size,
    turn: Option<Side>,
    offset: Offset,
    black_cells: BitBoard,
    white_cells: BitBoard,
    place_cand: BitBoard,
}

impl Board {
    pub fn new(size: Size) -> Self {
        assert!(MIN_SIZE <= size.0 && size.0 <= MAX_SIZE);
        assert!(MIN_SIZE <= size.1 && size.1 <= MAX_SIZE);

        let (x, y) = (size.0 / 2 - 1, size.1 / 2 - 1);
        let mut board = Board {
            size: size,
            turn: Some(Side::Black),
            offset: Offset::from_size(size),
            black_cells: bit_board::pt2mask((x, y), size) |
                         bit_board::pt2mask((x + 1, y + 1), size),
            white_cells: bit_board::pt2mask((x + 1, y), size) |
                         bit_board::pt2mask((x, y + 1), size),
            place_cand: 0,
        };
        board.update_place_cand();
        board
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn turn(&self) -> Option<Side> {
        self.turn
    }

    pub fn num_disk(&self, side: Side) -> u32 {
        match side {
            Side::Black => self.black_cells.count_ones(),
            Side::White => self.white_cells.count_ones(),
        }
    }

    pub fn get(&self, pt: Point) -> Option<Side> {
        let mask = self.pt2mask(pt);
        if (self.black_cells & mask) != 0 {
            Some(Side::Black)
        } else if (self.white_cells & mask) != 0 {
            Some(Side::White)
        } else {
            None
        }
    }

    pub fn can_place(&self, pt: Point) -> bool {
        let mask = self.pt2mask(pt);
        (self.place_cand & mask) != 0
    }

    pub fn flip_disks(&self, pt: Point) -> Option<(Side, BitBoard)> {
        let turn = if let Some(turn) = self.turn {
            turn
        } else {
            return None;
        };

        if !self.can_place(pt) {
            return None;
        }

        let (me, you) = match turn {
            Side::Black => (self.black_cells, self.white_cells),
            Side::White => (self.white_cells, self.black_cells),
        };

        let mut flip = self.pt2mask(pt);

        let mut mask = Mask::new(flip);
        let mut flip_cand = Mask::new(0);
        let cnt = cmp::max(cmp::max(pt.0, MAX_SIZE - pt.0 - 1),
                           cmp::max(pt.1, MAX_SIZE - pt.1 - 1));
        for _ in 0..cnt {
            mask = mask.shift(self.offset);
            for (mask, cand) in mask.iter_mut().zip(flip_cand.iter_mut()) {
                if (*mask & you) != 0 {
                    *cand |= *mask;
                } else {
                    if (*mask & me) != 0 {
                        flip |= *cand;
                    }
                    *mask = 0;
                }
            }
        }

        Some((turn, flip))
    }

    pub fn place(&mut self, pt: Point) -> bool {
        let (turn, flip) = if let Some(tp) = self.flip_disks(pt) {
            tp
        } else {
            return false;
        };

        match turn {
            Side::Black => {
                self.black_cells |= flip;
                self.white_cells &= !flip;
                self.turn = Some(Side::White);
            }
            Side::White => {
                self.white_cells |= flip;
                self.black_cells &= !flip;
                self.turn = Some(Side::Black);
            }
        }
        self.update_place_cand();
        if self.place_cand > 0 {
            return true;
        }

        self.turn = Some(turn);
        self.update_place_cand();
        if self.place_cand > 0 {
            return true;
        }

        self.turn = None;
        self.place_cand = 0;
        true
    }

    fn pt2mask(&self, pt: Point) -> BitBoard {
        bit_board::pt2mask(pt, self.size)
    }

    fn update_place_cand(&mut self) {
        let (me, you) = match self.turn {
            Some(Side::Black) => (self.black_cells, self.white_cells),
            Some(Side::White) => (self.white_cells, self.black_cells),
            None => return,
        };

        let mut cand = 0;

        // Search: E Y Y Y M
        let empty = Mask::new(!me & !you);

        let mut you_mask = Mask::new(you);
        let mut me_mask = Mask::new(me).shift(self.offset);
        let mut you_cont_mask = empty;

        for _ in 0..(MAX_SIZE - 2) {
            you_mask = you_mask.shift(self.offset);
            me_mask = me_mask.shift(self.offset);
            you_cont_mask = you_cont_mask & you_mask;

            cand |= (you_cont_mask & me_mask).or_all();
        }

        self.place_cand = cand;
    }
}
