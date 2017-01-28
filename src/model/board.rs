use super::{BitBoard, MAX_SIZE, MIN_SIZE, Point, Side, Size};
use super::multi_direction::{MdMask, MdOffset};
use std::cmp;

#[derive(Copy, Clone, Debug)]
pub struct Board {
    size: Size,
    turn: Option<Side>,
    offset: MdOffset,
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
            offset: MdOffset::from_size(size),
            black_cells: BitBoard::from_point(Point(x, y), size) |
                         BitBoard::from_point(Point(x + 1, y + 1), size),
            white_cells: BitBoard::from_point(Point(x + 1, y), size) |
                         BitBoard::from_point(Point(x, y + 1), size),
            place_cand: BitBoard::empty(),
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

    pub fn black_cells(&self) -> BitBoard {
        self.black_cells
    }

    pub fn white_cells(&self) -> BitBoard {
        self.white_cells
    }

    pub fn place_candidates(&self) -> BitBoard {
        self.place_cand
    }

    pub fn num_disk(&self, side: Side) -> u32 {
        match side {
            Side::Black => self.black_cells.num_bits(),
            Side::White => self.white_cells.num_bits(),
        }
    }

    pub fn get(&self, pt: Point) -> Option<Side> {
        if self.black_cells.contains(pt, self.size) {
            Some(Side::Black)
        } else if self.white_cells.contains(pt, self.size) {
            Some(Side::White)
        } else {
            None
        }
    }

    pub fn flip_disks(&self, pt: Point) -> Option<(Side, BitBoard)> {
        let turn = if let Some(turn) = self.turn {
            turn
        } else {
            return None;
        };

        if !self.place_cand.contains(pt, self.size) {
            return None;
        }

        let (me, you) = match turn {
            Side::Black => (self.black_cells, self.white_cells),
            Side::White => (self.white_cells, self.black_cells),
        };

        let mut flip = BitBoard::from_point(pt, self.size);

        let mut mask = MdMask::new(flip);
        let mut flip_cand = MdMask::new(BitBoard::empty());
        let cnt = cmp::max(cmp::max(pt.0, MAX_SIZE - pt.0 - 1),
                           cmp::max(pt.1, MAX_SIZE - pt.1 - 1));
        for _ in 0..cnt {
            mask = mask.shift(self.offset);
            for (mask, cand) in mask.iter_mut().zip(flip_cand.iter_mut()) {
                if !(*mask & you).is_empty() {
                    *cand |= *mask;
                } else {
                    if !(*mask & me).is_empty() {
                        flip |= *cand;
                    }
                    *mask = BitBoard::empty();
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
        if !self.place_cand.is_empty() {
            return true;
        }

        self.turn = Some(turn);
        self.update_place_cand();
        if !self.place_cand.is_empty() {
            return true;
        }

        self.turn = None;
        self.place_cand = BitBoard::empty();
        true
    }

    fn update_place_cand(&mut self) {
        let (me, you) = match self.turn {
            Some(Side::Black) => (self.black_cells, self.white_cells),
            Some(Side::White) => (self.white_cells, self.black_cells),
            None => return,
        };

        let mut cand = BitBoard::empty();

        // Search: E Y Y Y M
        let empty = MdMask::new(!me & !you);

        let mut you_mask = MdMask::new(you);
        let mut me_mask = MdMask::new(me).shift(self.offset);
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
