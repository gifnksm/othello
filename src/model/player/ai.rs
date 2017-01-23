use super::FindMove;
use super::super::bit_board;
use super::super::evaluator::{Evaluator, Score};
use Side;
use model::{Board, Point, Size};
use std::f64;

#[derive(Clone, Debug)]
pub struct Player {
    evaluator: Evaluator,
}

impl Player {
    pub fn new_weak(size: Size) -> Self {
        Player { evaluator: Evaluator::new(size) }
    }
}

impl FindMove for Player {
    fn find_move(&mut self, board: Board) -> Point {
        let side = board.turn().unwrap();
        let side_coef = if side == Side::Black { 1 } else { -1 };

        let cands = board.place_candidates();
        assert!(cands > 0);

        let size = board.size();
        let num_cands = cands.count_ones();
        let child_num_eval = 10000.0 / (num_cands as f64);

        let mut max = None;
        let it = bit_board::points(cands, size)
            .map(move |pt| {
                let mut board = board;
                board.place(pt);
                (pt, board)
            })
            .map(|(pt, board)| (pt, self.get_score(&board, child_num_eval, side_coef, false)));

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
    fn get_score(&self, board: &Board, num_eval: f64, side_coef: i32, get_max: bool) -> Score {
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

        let it = bit_board::points(cands, size)
            .map(|pt| {
                let mut board = *board;
                board.place(pt);
                board
            })
            .map(|board| self.get_score(&board, child_num_eval, side_coef, !get_max));
        if get_max { it.max() } else { it.min() }.unwrap()
    }
}
