use Side;
use conrod::widget::DropDownList;
use geom::{Point, Size};
use model::{Board, Player, PlayerKind};
use std::mem;
use std::sync::mpsc::TryRecvError;
use super::BoardSize;
use view::DdlBuilder;

pub enum StateKind {
    Start,
    Play,
}

pub enum State {
    Start(StartState),
    Play(PlayState),
}

impl State {
    pub fn kind(&self) -> StateKind {
        match *self {
            State::Start(_) => StateKind::Start,
            State::Play(_) => StateKind::Play,
        }
    }
}

macro_rules! impl_state {
    ($state_ty:ty, $state_name:ident) => {
        impl AsRef<$state_ty> for State {
            fn as_ref(&self) -> &$state_ty {
                match *self {
                    State::$state_name(ref s) => s,
                    _ => panic!(),
                }
            }
        }
        impl AsMut<$state_ty> for State {
            fn as_mut(&mut self) -> &mut $state_ty {
                match *self {
                    State::$state_name(ref mut s) => s,
                    _ => panic!(),
                }
            }
        }
    }
}

impl_state!(StartState, Start);
impl_state!(PlayState, Play);

#[derive(Clone, Debug)]
pub struct StartState {
    ddl_rows: DdlBuilder<BoardSize>,
    ddl_cols: DdlBuilder<BoardSize>,
    ddl_black_player: DdlBuilder<PlayerKind>,
    ddl_white_player: DdlBuilder<PlayerKind>,
}

impl Default for StartState {
    fn default() -> StartState {
        StartState {
            ddl_rows: DdlBuilder::new(),
            ddl_cols: DdlBuilder::new(),
            ddl_black_player: DdlBuilder::new(),
            ddl_white_player: DdlBuilder::new(),
        }
    }
}

impl StartState {
    pub fn build_ddl_rows(&self) -> DropDownList<String> {
        self.ddl_rows.build_drop_down_list()
    }

    pub fn build_ddl_cols(&self) -> DropDownList<String> {
        self.ddl_cols.build_drop_down_list()
    }

    pub fn build_ddl_black_player(&self) -> DropDownList<String> {
        self.ddl_black_player.build_drop_down_list()
    }

    pub fn build_ddl_white_player(&self) -> DropDownList<String> {
        self.ddl_white_player.build_drop_down_list()
    }
}

pub struct PlayState {
    black_player: Option<Player>,
    white_player: Option<Player>,
    board: Board,
}

impl PlayState {
    pub fn new(size: Size, black_kind: PlayerKind, white_kind: PlayerKind) -> PlayState {
        let board = Board::new(size);
        let black_player = Player::new(black_kind, &board, Side::Black);
        let white_player = Player::new(white_kind, &board, Side::White);
        PlayState {
            board: board,
            black_player: black_player,
            white_player: white_player,
        }
    }

    pub fn finish(&mut self) {
        if let Some(p) = mem::replace(&mut self.black_player, None) {
            p.finish();
        }
        if let Some(p) = mem::replace(&mut self.white_player, None) {
            p.finish();
        }
    }

    pub fn has_player(&self, side: Side) -> bool {
        self.get_player(side).is_some()
    }

    pub fn get_player(&self, side: Side) -> &Option<Player> {
        match side {
            Side::Black => &self.black_player,
            Side::White => &self.white_player,
        }
    }

    pub fn listen_player(&mut self) {
        let turn = match self.board.turn() {
            Some(turn) => turn,
            None => {
                self.finish();
                return;
            }
        };

        let loc = if let Some(ref player) = *self.get_player(turn) {
            match player.listen() {
                Ok(loc) => loc,
                Err(TryRecvError::Empty) => return,
                Err(e) => panic!("error: {}", e),
            }
        } else {
            return;
        };

        if !self.locate(loc) {
            panic!("cannot locate: {:?}", loc);
        }
    }

    pub fn turn(&self) -> Option<Side> {
        self.board.turn()
    }

    pub fn can_locate(&self, pt: Point) -> bool {
        self.board.can_locate(pt)
    }

    pub fn get_disk_at(&self, pt: Point) -> Option<Side> {
        self.board[pt]
    }

    pub fn num_disk(&self, side: Side) -> usize {
        self.board.num_disk(side)
    }


    pub fn locate(&mut self, pt: Point) -> bool {
        let turn = match self.board.turn() {
            Some(turn) => turn,
            None => return false,
        };

        if !self.board.locate(pt) {
            return false;
        }

        if let Some(ref player) = *self.get_player(turn.flip()) {
            player.locate(turn, pt).unwrap();
        }

        true
    }
}
