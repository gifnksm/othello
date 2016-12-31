use conrod::widget::drop_down_list::Idx;
use model::PlayerKind;
pub use self::config::{GameConfig, ViewConfig};
pub use self::state::{PlayState, StartState, State, StateKind};
use view::DdlString;

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
    fn from_ddl_index(i: Idx) -> Option<BoardSize> {
        if i < 2 || i > 10 {
            None
        } else {
            Some(BoardSize(i as i32))
        }
    }

    fn to_ddl_string(&self) -> String {
        self.0.to_string()
    }

    fn create_strings() -> Vec<String> {
        (2..11).map(|n| n.to_string()).collect::<Vec<_>>()
    }
}

impl DdlString for PlayerKind {
    fn from_ddl_index(i: Idx) -> Option<PlayerKind> {
        match i {
            0 => Some(PlayerKind::Human),
            1 => Some(PlayerKind::AiRandom),
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
