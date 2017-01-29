use super::FindMove;
use super::super::evaluator::{Evaluator, MAX_SCORE, MIN_SCORE, Score};
use model::{Board, Point, Side, Size};
use std::{cmp, u32};

const WEAK_NUM_EVAL: u32 = 1_000_000;
const MEDIUM_NUM_EVAL: u32 = 10_000_000;
const STRONG_NUM_EVAL: u32 = 100_000_000;

#[derive(Clone, Debug)]
pub struct Player {
    side: Side,
    num_eval: u32,
    evaluator: Evaluator,
}

impl Player {
    pub fn new(side: Side, size: Size, num_eval: u32) -> Self {
        Player {
            side: side,
            num_eval: num_eval,
            evaluator: Evaluator::new(size),
        }
    }

    pub fn new_weak(side: Side, size: Size) -> Self {
        Self::new(side, size, WEAK_NUM_EVAL)
    }

    pub fn new_medium(side: Side, size: Size) -> Self {
        Self::new(side, size, MEDIUM_NUM_EVAL)
    }

    pub fn new_strong(side: Side, size: Size) -> Self {
        Self::new(side, size, STRONG_NUM_EVAL)
    }
}

impl FindMove for Player {
    fn find_move(&mut self, board: Board) -> Point {
        assert!(board.turn() == Some(self.side));

        let cands = board.move_candidates();
        let size = board.size();
        let num_cands = cands.num_bits();
        let child_num_eval = (self.num_eval as f64) / (num_cands as f64);

        cands.points(size)
            .map(move |pt| (pt, board.make_move(pt).unwrap()))
            .map(|(pt, board)| (pt, self.get_score(&board, child_num_eval)))
            .max_by_key(|e| e.1)
            .unwrap()
            .0
    }
}

impl Player {
    fn get_score(&self, board: &Board, num_eval: f64) -> Score {
        self.alphabeta(board, num_eval, MIN_SCORE, MAX_SCORE)
    }

    fn alphabeta(&self, board: &Board, num_eval: f64, alpha: Score, beta: Score) -> Score {
        if num_eval <= 1.0 || board.turn().is_none() {
            return self.evaluator.eval_board(board, self.side);
        }

        let cands = board.move_candidates();
        let size = board.size();
        let num_cands = cands.num_bits();
        let child_num_eval = num_eval / (num_cands as f64);

        let it = cands.points(size).map(|pt| board.make_move(pt).unwrap());

        if board.turn() == Some(self.side) {
            let mut alpha = alpha;
            for board in it {
                let score = self.alphabeta(&board, child_num_eval, alpha, beta);
                alpha = cmp::max(alpha, score);
                if alpha >= beta {
                    return beta;
                }
            }
            alpha
        } else {
            let mut beta = beta;
            for board in it {
                let score = self.alphabeta(&board, child_num_eval, alpha, beta);
                beta = cmp::min(beta, score);
                if alpha >= beta {
                    return alpha;
                }
            }
            beta
        }
    }
}
