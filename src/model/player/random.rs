use super::FindMove;
use crate::model::{Board, Point};
use rand::{self, rngs::ThreadRng, seq::IteratorRandom as _};

pub struct Player {
    rng: ThreadRng,
}

impl Player {
    pub fn new() -> Self {
        Player {
            rng: rand::thread_rng(),
        }
    }
}

impl FindMove for Player {
    fn find_move(&mut self, board: Board) -> Point {
        let size = board.size();
        let pts = board.move_candidates().points(size);
        pts.choose(&mut self.rng).unwrap()
    }
}
