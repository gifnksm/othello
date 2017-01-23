use super::FindMove;
use model::{Board, Point};
use rand::{self, ThreadRng};

pub struct Player {
    rng: ThreadRng,
}

impl Player {
    pub fn new() -> Self {
        Player { rng: rand::thread_rng() }
    }
}

impl FindMove for Player {
    fn find_move(&mut self, board: Board) -> Point {
        let size = board.size();
        let pts = (0..size.0)
            .flat_map(|x| (0..size.1).map(move |y| (x, y)))
            .filter(|&pt| board.can_place(pt));
        rand::sample(&mut self.rng, pts, 1)[0]
    }
}
