pub use self::even::Evaluator as EvenEvaluator;
pub use self::strong::Evaluator as StrongEvaluator;
pub use self::weak::Evaluator as WeakEvaluator;
use model::{Board, Side};
use std::{f64, i32};
use std::cmp::Ordering;

mod even;
mod strong;
mod weak;

pub trait Evaluate {
    fn evaluate(&self, board: &Board, myside: Side) -> Score;
}

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
