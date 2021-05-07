use super::{Evaluate, FindMove, Score, MAX_SCORE, MIN_SCORE};
use crate::model::{Board, Point, Side};
use std::{cmp, u32};

#[derive(Clone, Debug)]
pub struct Player<E> {
    side: Side,
    num_eval: u32,
    evaluator: E,
}

impl<E> Player<E> {
    pub fn new(side: Side, num_eval: u32, evaluator: E) -> Self {
        Player {
            side,
            num_eval,
            evaluator,
        }
    }
}

impl<E> FindMove for Player<E>
where
    E: Evaluate,
{
    fn find_move(&mut self, board: Board) -> Point {
        assert_eq!(board.turn(), Some(self.side));

        let cands = board.move_candidates();
        let size = board.size();
        let num_cands = cands.num_bits();
        let child_num_eval = f64::from(self.num_eval) / f64::from(num_cands);

        cands
            .points(size)
            .map(move |pt| (pt, board.make_move(pt).unwrap()))
            .map(|(pt, board)| (pt, self.get_score(&board, child_num_eval)))
            .max_by_key(|e| e.1)
            .unwrap()
            .0
    }
}

impl<E> Player<E>
where
    E: Evaluate,
{
    fn get_score(&self, board: &Board, num_eval: f64) -> Score {
        self.alphabeta(board, num_eval, MIN_SCORE, MAX_SCORE)
    }

    fn alphabeta(&self, board: &Board, num_eval: f64, alpha: Score, beta: Score) -> Score {
        if num_eval <= 1.0 || board.turn().is_none() {
            return self.evaluator.evaluate(board, self.side);
        }

        let cands = board.move_candidates();
        let size = board.size();
        let num_cands = cands.num_bits();
        let child_num_eval = num_eval / f64::from(num_cands);

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
