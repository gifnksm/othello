use super::FindMove;
use model::{Board, Point};
use rand::{self, ThreadRng};

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
        rand::seq::sample_iter(&mut self.rng, pts, 1).unwrap()[0]
    }
}
