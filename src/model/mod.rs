pub use self::board::Board;
pub use self::player::{Player, PlayerKind};

mod bit_board;
mod board;
mod evaluator;
mod player;

pub type Point = (u32, u32);
pub type Size = (u32, u32);

pub const MIN_SIZE: u32 = 2;
pub const MAX_SIZE: u32 = 8;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Side {
    Black,
    White,
}

impl Side {
    pub fn flip(self) -> Side {
        match self {
            Side::Black => Side::White,
            Side::White => Side::Black,
        }
    }
}
