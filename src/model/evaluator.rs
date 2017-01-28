use super::{BitBoard, Board, Point, Side, Size};
use std::{f64, i32};
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug)]
pub enum Score {
    NegInfinity,
    Infinity,
    Running(f64),
    Ended(i32),
}

pub const MIN_SCORE: Score = Score::NegInfinity;
pub const MAX_SCORE: Score = Score::Infinity;

impl PartialEq for Score {
    fn eq(&self, other: &Score) -> bool {
        match (*self, *other) {
            (Score::Running(s), Score::Running(o)) => s == o || (s.is_nan() && o.is_nan()),
            (Score::Ended(s), Score::Ended(o)) => s == o,
            _ => false,
        }
    }
}

impl Eq for Score {}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Score) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Score) -> Ordering {
        match (*self, *other) {
            (Score::NegInfinity, Score::NegInfinity) |
            (Score::Infinity, Score::Infinity) => Ordering::Equal,

            (Score::NegInfinity, _) |
            (_, Score::Infinity) => Ordering::Less,

            (_, Score::NegInfinity) |
            (Score::Infinity, _) => Ordering::Greater,

            (Score::Running(s), Score::Running(o)) => s.partial_cmp(&o).unwrap(),
            (Score::Ended(s), Score::Ended(o)) => s.cmp(&o),

            (Score::Running(s), Score::Ended(o)) => {
                match o.cmp(&0) {
                    // o must loose
                    Ordering::Less => Ordering::Greater,
                    // o must win
                    Ordering::Greater => Ordering::Less,

                    Ordering::Equal => s.partial_cmp(&0.0).unwrap(),
                }
            }
            (Score::Ended(s), Score::Running(o)) => {
                match s.cmp(&0) {
                    // s must loose
                    Ordering::Less => Ordering::Less,
                    // s must win
                    Ordering::Greater => Ordering::Greater,

                    Ordering::Equal => o.partial_cmp(&0.0).unwrap().reverse(),
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Evaluator {
    weights: Vec<(i32, BitBoard)>,
}

impl Evaluator {
    pub fn new(size: Size) -> Self {
        // c  : corner
        // e? : edge
        // i? : iedge
        // b? : box
        //    0  1  2  3  .. |
        // 0: c  eC eA eB    |  30 -12  0 -1
        // 1: eC eX iC iA    | -12 -15 -3 -3
        // 2: eA iC iX bC    |   0 - 3  0 -1
        // 3: eB iA bC bX    | - 1 - 3 -1 -1
        // ..
        let corner_mask = weight_mask(BitBoard::from_point(Point(0, 0), size), size);
        let edge_c_mask = weight_mask(BitBoard::from_point(Point(0, 1), size) |
                                      BitBoard::from_point(Point(1, 0), size),
                                      size);
        let edge_b_mask = weight_mask(BitBoard::from_point(Point(0, 3), size) |
                                      BitBoard::from_point(Point(3, 0), size),
                                      size);
        // let edge_a_mask = weight_mask(BitBoard::from_point(Point(0, 2), size) | BitBoard::from_point(Point(2, 0), size), size);
        let edge_x_mask = weight_mask(BitBoard::from_point(Point(1, 1), size), size);
        let iedge_c_mask = weight_mask(BitBoard::from_point(Point(2, 1), size) |
                                       BitBoard::from_point(Point(1, 2), size),
                                       size);
        let iedge_a_mask = weight_mask(BitBoard::from_point(Point(3, 1), size) |
                                       BitBoard::from_point(Point(1, 3), size),
                                       size);
        // let iedge_x_mask = weight_mask(BitBoard::from_point(Point(2, 2), size), size);
        let box_c_mask = weight_mask(BitBoard::from_point(Point(3, 2), size) |
                                     BitBoard::from_point(Point(2, 3), size),
                                     size);
        let box_x_mask = weight_mask(BitBoard::from_point(Point(3, 3), size), size);

        // http://uguisu.skr.jp/othello/5-1.html
        let weights = vec![(30, corner_mask),
                           // (0, edge_a_mask | iedge_x_mask),
                           (-1, edge_b_mask | box_c_mask | box_x_mask),
                           (-3, iedge_c_mask | iedge_a_mask),
                           (-12, edge_c_mask),
                           (-15, edge_x_mask)];

        Evaluator { weights: weights }
    }

    pub fn eval_board(&self, board: &Board, myside: Side) -> Score {
        match board.turn() {
            Some(_) => {
                let num_disk = (board.black_cells() | board.white_cells()).num_bits() as f64;
                let disk_score = self.eval_disk_place(board) as f64;
                let cand_score = self.eval_move_candidates(board) as f64;
                // TODO: set appropriate score weights
                let black_score = disk_score / num_disk + 0.1 * cand_score;
                let score = match myside {
                    Side::Black => black_score,
                    Side::White => -black_score,
                };
                Score::Running(score)
            }
            None => {
                let black = board.black_cells().num_bits() as i32;
                let white = board.white_cells().num_bits() as i32;
                let black_score = black - white;
                let score = match myside {
                    Side::Black => black_score,
                    Side::White => -black_score,
                };
                Score::Ended(score)
            }
        }
    }

    fn eval_disk_place(&self, board: &Board) -> i32 {
        let black_cells = board.black_cells();
        let white_cells = board.white_cells();
        let mut black = 0;
        let mut white = 0;
        for &(val, mask) in &self.weights {
            black += val * (mask & black_cells).num_bits() as i32;
            white += val * (mask & white_cells).num_bits() as i32;
        }
        black - white
    }

    fn eval_move_candidates(&self, board: &Board) -> i32 {
        let num_cand = board.move_candidates().num_bits() as i32;
        match board.turn() {
            Some(Side::Black) => num_cand,
            Some(Side::White) => -num_cand,
            None => 0,
        }
    }
}

fn weight_mask(mask: BitBoard, size: Size) -> BitBoard {
    let mut out_mask = BitBoard::empty();

    let ul_size = (size.0 / 2, size.1 / 2);
    let dr_size = (size.0 - ul_size.0, size.1 - ul_size.1);

    for x in 0..ul_size.0 {
        let rx = size.0 - x - 1;
        for y in 0..ul_size.1 {
            let ry = size.1 - y - 1;
            if mask.contains(Point(x, y), size) {
                out_mask |= BitBoard::from_point(Point(x, y), size) |
                            BitBoard::from_point(Point(rx, y), size) |
                            BitBoard::from_point(Point(x, ry), size) |
                            BitBoard::from_point(Point(rx, ry), size);
            }
        }
        for y in ul_size.1..dr_size.1 {
            let ry = size.1 - y - 1;
            if mask.contains(Point(x, y), size) {
                out_mask |= BitBoard::from_point(Point(x, ry), size) |
                            BitBoard::from_point(Point(rx, ry), size);
            }
        }
    }
    for x in ul_size.0..dr_size.0 {
        let rx = size.0 - x - 1;
        for y in 0..ul_size.1 {
            let ry = size.1 - y - 1;
            if mask.contains(Point(x, y), size) {
                out_mask |= BitBoard::from_point(Point(rx, y), size) |
                            BitBoard::from_point(Point(rx, ry), size);
            }
        }
        for y in ul_size.1..dr_size.1 {
            let ry = size.1 - y - 1;
            if mask.contains(Point(x, y), size) {
                out_mask |= BitBoard::from_point(Point(rx, ry), size);
            }
        }
    }

    out_mask
}
