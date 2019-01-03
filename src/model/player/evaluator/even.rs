use super::{Evaluate, Score, StrongEvaluator};
use crate::model::{Board, Side, Size};

#[derive(Clone, Debug)]
pub struct Evaluator {
    strong: StrongEvaluator,
}

impl Evaluate for Evaluator {
    fn evaluate(&self, board: &Board, myside: Side) -> Score {
        match self.strong.evaluate(board, myside) {
            Score::Infinity => Score::Infinity,
            Score::NegInfinity => Score::NegInfinity,
            Score::Running(v) => Score::Running(-v.abs()),
            Score::Ended(v) => Score::Ended(-v.abs()),
        }
    }
}

impl Evaluator {
    pub fn new(size: Size) -> Self {
        Evaluator {
            strong: StrongEvaluator::new(size),
        }
    }
}
