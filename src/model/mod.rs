pub use self::{
    bit_board::BitBoard,
    board::Board,
    player::{AiPlayer, PlayerKind},
};

mod bit_board;
mod board;
mod multi_direction;
mod player;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Point(pub u32, pub u32);

impl Point {
    fn from_offset(off: u32, size: Size) -> Point {
        Point(off % size.0, off / size.0)
    }

    fn offset(self, size: Size) -> u32 {
        self.0 + size.0 * self.1
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Size(pub u32, pub u32);

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
