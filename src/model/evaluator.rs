use super::bit_board::BitBoard;
use model::{Board, Side, Size};
use std::{f64, i32};
use std::cmp::Ordering;
use std::ops::Mul;

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

impl Mul<i32> for Score {
    type Output = Score;

    fn mul(self, coef: i32) -> Score {
        match self {
            Score::NegInfinity => Score::NegInfinity,
            Score::Infinity => Score::Infinity,
            Score::Running(v) => Score::Running((coef as f64) * v),
            Score::Ended(v) => Score::Ended(coef * v),

        }
    }
}

#[derive(Clone, Debug)]
pub struct Evaluator {
    weights: Vec<(i32, BitBoard)>,
}

impl Evaluator {
    pub fn new(size: Size) -> Self {
        use super::bit_board::pt2mask as m;
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
        let corner_mask = weight_mask(m((0, 0), size), size);
        let edge_c_mask = weight_mask(m((0, 1), size) | m((1, 0), size), size);
        let edge_b_mask = weight_mask(m((0, 3), size) | m((3, 0), size), size);
        // let edge_a_mask = weight_mask(m((0, 2), size) | m((2, 0), size), size);
        let edge_x_mask = weight_mask(m((1, 1), size), size);
        let iedge_c_mask = weight_mask(m((2, 1), size) | m((1, 2), size), size);
        let iedge_a_mask = weight_mask(m((3, 1), size) | m((1, 3), size), size);
        // let iedge_x_mask = weight_mask(m((2, 2), size), size);
        let box_c_mask = weight_mask(m((3, 2), size) | m((2, 3), size), size);
        let box_x_mask = weight_mask(m((3, 3), size), size);

        // http://uguisu.skr.jp/othello/5-1.html
        let weights = vec![(30, corner_mask),
                           // (0, edge_a_mask | iedge_x_mask),
                           (-1, edge_b_mask | box_c_mask | box_x_mask),
                           (-3, iedge_c_mask | iedge_a_mask),
                           (-12, edge_c_mask),
                           (-15, edge_x_mask)];

        Evaluator { weights: weights }
    }

    pub fn eval_board(&self, board: &Board) -> Score {
        match board.turn() {
            Some(_) => {
                let num_disk = (board.black_cells() | board.white_cells()).count_ones() as f64;
                let disk_score = self.eval_disk_place(board) as f64;
                let cand_score = self.eval_place_candidates(board) as f64;
                // TODO: set appropriate score weights
                Score::Running(disk_score / num_disk + 0.1 * cand_score)
            }
            None => {
                let black = board.black_cells().count_ones() as i32;
                let white = board.white_cells().count_ones() as i32;
                Score::Ended(black - white)
            }
        }
    }

    fn eval_disk_place(&self, board: &Board) -> i32 {
        let black_cells = board.black_cells();
        let white_cells = board.white_cells();
        let mut black = 0;
        let mut white = 0;
        for &(val, mask) in &self.weights {
            black += val * (mask & black_cells).count_ones() as i32;
            white += val * (mask & white_cells).count_ones() as i32;
        }
        black - white
    }

    fn eval_place_candidates(&self, board: &Board) -> i32 {
        let num_cand = board.place_candidates().count_ones() as i32;
        match board.turn() {
            Some(Side::Black) => num_cand,
            Some(Side::White) => -num_cand,
            None => 0,
        }
    }
}

fn weight_mask(mask: BitBoard, size: Size) -> BitBoard {
    use super::bit_board::pt2mask as m;

    let mut out_mask = 0;

    let ul_size = (size.0 / 2, size.1 / 2);
    let dr_size = (size.0 - ul_size.0, size.1 - ul_size.1);

    for x in 0..ul_size.0 {
        let rx = size.0 - x - 1;
        for y in 0..ul_size.1 {
            let ry = size.1 - y - 1;
            if (mask & m((x, y), size)) != 0 {
                out_mask |= m((x, y), size) | m((rx, y), size) | m((x, ry), size) |
                            m((rx, ry), size);
            }
        }
        for y in ul_size.1..dr_size.1 {
            let ry = size.1 - y - 1;
            if (mask & m((x, y), size)) != 0 {
                out_mask |= m((x, ry), size) | m((rx, ry), size);
            }
        }
    }
    for x in ul_size.0..dr_size.0 {
        let rx = size.0 - x - 1;
        for y in 0..ul_size.1 {
            let ry = size.1 - y - 1;
            if (mask & m((x, y), size)) != 0 {
                out_mask |= m((rx, y), size) | m((rx, ry), size);
            }
        }
        for y in ul_size.1..dr_size.1 {
            let ry = size.1 - y - 1;
            if (mask & m((x, y), size)) != 0 {
                out_mask |= m((rx, ry), size);
            }
        }
    }

    out_mask
}
