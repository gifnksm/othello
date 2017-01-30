use super::{Evaluate, Score, StrongEvaluator};
use model::{Board, Side, Size};

#[derive(Clone, Debug)]
pub struct Evaluator {
    strong: StrongEvaluator,
}

impl Evaluate for Evaluator {
    fn evaluate(&self, board: &Board, myside: Side) -> Score {
        self.strong.evaluate(board, myside.flip())
    }
}

impl Evaluator {
    pub fn new(size: Size) -> Self {
        Evaluator { strong: StrongEvaluator::new(size) }
    }
}
