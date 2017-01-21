pub use self::board::Board;
pub use self::player::{Player, PlayerKind};

pub type Point = (u32, u32);
pub type Size = (u32, u32);

pub const MIN_SIZE: u32 = 2;
pub const MAX_SIZE: u32 = 8;

mod bit_board;
mod board;
mod player;
