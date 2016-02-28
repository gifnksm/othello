use std::str::FromStr;

use model::PlayerKind;
use view::DdlString;

pub use self::config::{GameConfig, ViewConfig};
pub use self::state::{PlayState, StartState, State, StateKind};

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
            state: State::Start(StartState::default()),
            game_config: GameConfig::default(),
            view_config: ViewConfig::default(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BoardSize(pub i32);

impl Default for BoardSize {
    fn default() -> BoardSize {
        BoardSize(8)
    }
}

impl DdlString for BoardSize {
    fn from_ddl_str(s: &str) -> Option<BoardSize> {
        i32::from_str(s).ok().and_then(|size| {
            if size < 2 || size > 10 {
                None
            } else {
                Some(BoardSize(size))
            }
        })
    }

    fn to_ddl_string(&self) -> String {
        self.0.to_string()
    }

    fn create_strings() -> Vec<String> {
        (2..11).map(|n| n.to_string()).collect::<Vec<_>>()
    }
}

impl DdlString for PlayerKind {
    fn from_ddl_str(s: &str) -> Option<PlayerKind> {
        match s {
            "Human" => Some(PlayerKind::Human),
            "AI Random" => Some(PlayerKind::AiRandom),
            _ => None,
        }
    }

    fn to_ddl_string(&self) -> String {
        match *self {
            PlayerKind::Human => "Human".to_owned(),
            PlayerKind::AiRandom => "AI Random".to_owned(),
        }
    }

    fn create_strings() -> Vec<String> {
        vec![PlayerKind::Human.to_ddl_string(), PlayerKind::AiRandom.to_ddl_string()]
    }
}
