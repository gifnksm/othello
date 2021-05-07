use crate::model::{AiPlayer, Board, PlayerKind, Point, Side, Size};
use std::mem;
use std::sync::mpsc::TryRecvError;

pub enum State {
    Start,
    Play(Box<PlayState>),
}

pub struct PlayState {
    board: Board,
    black_kind: PlayerKind,
    white_kind: PlayerKind,
    black_ai_player: Option<AiPlayer>,
    white_ai_player: Option<AiPlayer>,
}

impl PlayState {
    pub fn new(size: Size, black_kind: PlayerKind, white_kind: PlayerKind) -> PlayState {
        let board = Board::new(size);
        PlayState {
            board,
            black_kind,
            white_kind,
            black_ai_player: AiPlayer::try_new(black_kind, &board, Side::Black),
            white_ai_player: AiPlayer::try_new(white_kind, &board, Side::White),
        }
    }

    fn finish(&mut self) {
        if let Some(p) = mem::replace(&mut self.black_ai_player, None) {
            p.finish();
        }
        if let Some(p) = mem::replace(&mut self.white_ai_player, None) {
            p.finish();
        }
    }

    pub fn is_waiting_user_input(&self) -> bool {
        self.board
            .turn()
            .map(|side| self.ai_player(side).is_none())
            .unwrap_or(false)
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn player_kind(&self, side: Side) -> PlayerKind {
        match side {
            Side::Black => self.black_kind,
            Side::White => self.white_kind,
        }
    }

    fn ai_player(&self, side: Side) -> &Option<AiPlayer> {
        match side {
            Side::Black => &self.black_ai_player,
            Side::White => &self.white_ai_player,
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

        let pt = if let Some(ref player) = *self.ai_player(turn) {
            match player.listen() {
                Ok(pt) => pt,
                Err(TryRecvError::Empty) => return,
                Err(e) => panic!("error: {}", e),
            }
        } else {
            return;
        };

        if !self.make_move(pt) {
            panic!("cannot make_move: {:?}", pt);
        }
    }

    pub fn make_move(&mut self, pt: Point) -> bool {
        let turn = match self.board.turn() {
            Some(turn) => turn,
            None => return false,
        };

        self.board = match self.board.make_move(pt) {
            None => return false,
            Some(board) => board,
        };

        if let Some(ref player) = *self.ai_player(turn.flip()) {
            player.make_move(turn, pt).unwrap();
        }

        true
    }
}

impl Drop for PlayState {
    fn drop(&mut self) {
        self.finish();
    }
}
