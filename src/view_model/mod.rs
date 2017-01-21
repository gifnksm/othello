pub use self::config::{GameConfig, ViewConfig};
pub use self::state::{PlayState, State};

mod config;
mod state;

pub struct App {
    pub state: State,
    pub game_config: GameConfig,
    pub view_config: ViewConfig,
}

impl Default for App {
    fn default() -> App {
        App {
            state: State::Start,
            game_config: GameConfig::default(),
            view_config: ViewConfig::default(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BoardSize {
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
}

impl AsRef<str> for BoardSize {
    fn as_ref(&self) -> &str {
        use self::BoardSize::*;
        match *self {
            N2 => "2",
            N3 => "3",
            N4 => "4",
            N5 => "5",
            N6 => "6",
            N7 => "7",
            N8 => "8",
        }
    }
}

impl BoardSize {
    pub fn all_values() -> [Self; 7] {
        use self::BoardSize::*;
        [N2, N3, N4, N5, N6, N7, N8]
    }

    pub fn to_index(&self) -> usize {
        *self as usize
    }

    pub fn to_value(&self) -> u32 {
        use self::BoardSize::*;
        match *self {
            N2 => 2,
            N3 => 3,
            N4 => 4,
            N5 => 5,
            N6 => 6,
            N7 => 7,
            N8 => 8,
        }
    }
}
