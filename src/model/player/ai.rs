use super::FindMove;
use super::super::bit_board;
use super::super::evaluator::{Evaluator, MAX_SCORE, MIN_SCORE, Score};
use model::{Board, Point, Side, Size};
use std::cmp;

const WEAK_NUM_EVAL: u32 = 10000;
const MEDIUM_NUM_EVAL: u32 = 100000;
const STRONG_NUM_EVAL: u32 = 1000000;

#[derive(Clone, Debug)]
pub struct Player {
    num_eval: u32,
    evaluator: Evaluator,
}

impl Player {
    pub fn new(size: Size, num_eval: u32) -> Self {
        Player {
            num_eval: num_eval,
            evaluator: Evaluator::new(size),
        }
    }

    pub fn new_weak(size: Size) -> Self {
        Self::new(size, WEAK_NUM_EVAL)
    }

    pub fn new_medium(size: Size) -> Self {
        Self::new(size, MEDIUM_NUM_EVAL)
    }

    pub fn new_strong(size: Size) -> Self {
        Self::new(size, STRONG_NUM_EVAL)
    }
}

impl FindMove for Player {
    fn find_move(&mut self, board: Board) -> Point {
        let side = board.turn().unwrap();

        let cands = board.place_candidates();
        assert!(cands > 0);

        let size = board.size();
        let num_cands = cands.count_ones();
        let child_num_eval = (self.num_eval as f64) / (num_cands as f64);

        let mut max = None;
        let it = bit_board::points(cands, size)
            .map(move |pt| {
                let mut board = board;
                board.place(pt);
                (pt, board)
            })
            .map(|(pt, board)| (pt, self.get_score(&board, child_num_eval, side)));

        for (pt, score) in it {
            if let Some((_, max_score)) = max {
                if score > max_score {
                    max = Some((pt, score))
                }
            } else {
                max = Some((pt, score))
            }
        }

        max.unwrap().0
    }
}

impl Player {
    fn get_score(&self, board: &Board, num_eval: f64, side: Side) -> Score {
        let side_coef = if side == Side::Black { 1 } else { -1 };
        self.alphabeta(board, num_eval, side_coef, false, MIN_SCORE, MAX_SCORE)
    }

    fn alphabeta(&self,
                 board: &Board,
                 num_eval: f64,
                 side_coef: i32,
                 get_max: bool,
                 mut alpha: Score,
                 mut beta: Score)
                 -> Score {
        if num_eval <= 1.0 {
            return self.evaluator.eval_board(board) * side_coef;
        }

        let cands = board.place_candidates();
        if cands == 0 {
            return self.evaluator.eval_board(board) * side_coef;
        }

        let size = board.size();
        let num_cands = cands.count_ones();
        let child_num_eval = num_eval / (num_cands as f64);

        let it = bit_board::points(cands, size).map(|pt| {
            let mut board = *board;
            board.place(pt);
            board
        });

        if get_max {
            for board in it {
                let score =
                    self.alphabeta(&board, child_num_eval, side_coef, !get_max, alpha, beta);
                alpha = cmp::max(alpha, score);
                if alpha >= beta {
                    return beta;
                }
            }
            alpha
        } else {
            for board in it {
                let score =
                    self.alphabeta(&board, child_num_eval, side_coef, !get_max, alpha, beta);
                beta = cmp::min(beta, score);
                if alpha >= beta {
                    return alpha;
                }
            }
            beta
        }
    }
}
