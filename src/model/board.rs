use super::{
    multi_direction::{MdMask, MdOffset},
    BitBoard, Point, Side, Size, MAX_SIZE, MIN_SIZE,
};
use std::cmp;

#[derive(Copy, Clone, Debug)]
pub struct Board {
    size: Size,
    turn: Option<Side>,
    offset: MdOffset,
    black_cells: BitBoard,
    white_cells: BitBoard,
    move_cand: BitBoard,
}

impl Board {
    pub fn new(size: Size) -> Self {
        assert!(MIN_SIZE <= size.0 && size.0 <= MAX_SIZE);
        assert!(MIN_SIZE <= size.1 && size.1 <= MAX_SIZE);

        let (x, y) = (size.0 / 2 - 1, size.1 / 2 - 1);
        let mut board = Board {
            size,
            turn: Some(Side::Black),
            offset: MdOffset::from_size(size),
            black_cells: BitBoard::from_point(Point(x + 1, y), size)
                | BitBoard::from_point(Point(x, y + 1), size),
            white_cells: BitBoard::from_point(Point(x, y), size)
                | BitBoard::from_point(Point(x + 1, y + 1), size),
            move_cand: BitBoard::empty(),
        };
        board.move_cand = board.compute_move_cand();
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

    pub fn move_candidates(&self) -> BitBoard {
        self.move_cand
    }

    pub fn num_disk(&self, side: Side) -> u32 {
        match side {
            Side::Black => self.black_cells.num_bits(),
            Side::White => self.white_cells.num_bits(),
        }
    }

    pub fn get(&self, pt: Point) -> Option<Side> {
        assert!(pt.0 < self.size.0 && pt.1 < self.size.1);

        if self.black_cells.contains(pt, self.size) {
            Some(Side::Black)
        } else if self.white_cells.contains(pt, self.size) {
            Some(Side::White)
        } else {
            None
        }
    }

    pub fn make_move(&self, pt: Point) -> Option<Board> {
        assert!(pt.0 < self.size.0 && pt.1 < self.size.1);

        let (turn, flip) = self.flip_disks(pt)?;

        let mut board = *self;
        match turn {
            Side::Black => {
                board.black_cells |= flip;
                board.white_cells &= !flip;
            }
            Side::White => {
                board.white_cells |= flip;
                board.black_cells &= !flip;
            }
        }

        for &t in &[Some(turn.flip()), Some(turn), None] {
            board.turn = t;
            board.move_cand = board.compute_move_cand();
            if !board.move_cand.is_empty() {
                break;
            }
        }

        Some(board)
    }

    fn flip_disks(&self, pt: Point) -> Option<(Side, BitBoard)> {
        let turn = self.turn?;

        if !self.move_cand.contains(pt, self.size) {
            return None;
        }

        let (me, you) = match turn {
            Side::Black => (self.black_cells, self.white_cells),
            Side::White => (self.white_cells, self.black_cells),
        };

        let mut flip = BitBoard::from_point(pt, self.size);

        let mut mask = MdMask::new(flip);
        let mut flip_cand = MdMask::new(BitBoard::empty());
        let cnt = cmp::max(
            cmp::max(pt.0, MAX_SIZE - pt.0 - 1),
            cmp::max(pt.1, MAX_SIZE - pt.1 - 1),
        );
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

    fn compute_move_cand(&self) -> BitBoard {
        let (me, you) = match self.turn {
            Some(Side::Black) => (self.black_cells, self.white_cells),
            Some(Side::White) => (self.white_cells, self.black_cells),
            None => return BitBoard::empty(),
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

        cand
    }
}
