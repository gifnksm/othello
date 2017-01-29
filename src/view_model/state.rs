use model::{Board, Player, PlayerKind, Point, Side, Size};
use std::mem;
use std::sync::mpsc::TryRecvError;

pub enum State {
    Start,
    Play(PlayState),
}

pub struct PlayState {
    board: Board,
    black_kind: PlayerKind,
    white_kind: PlayerKind,
    black_player: Option<Player>,
    white_player: Option<Player>,
}

impl PlayState {
    pub fn new(size: Size, black_kind: PlayerKind, white_kind: PlayerKind) -> PlayState {
        let board = Board::new(size);
        let black_player = Player::new(black_kind, &board, Side::Black);
        let white_player = Player::new(white_kind, &board, Side::White);
        PlayState {
            board: board,
            black_kind: black_kind,
            white_kind: white_kind,
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
        self.player(side).is_some()
    }

    fn player(&self, side: Side) -> &Option<Player> {
        match side {
            Side::Black => &self.black_player,
            Side::White => &self.white_player,
        }
    }

    pub fn player_kind(&self, side: Side) -> PlayerKind {
        match side {
            Side::Black => self.black_kind,
            Side::White => self.white_kind,
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

        let pt = if let Some(ref player) = *self.player(turn) {
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

    pub fn turn(&self) -> Option<Side> {
        self.board.turn()
    }

    pub fn can_move(&self, pt: Point) -> bool {
        self.board.move_candidates().contains(pt, self.board.size())
    }

    pub fn get_disk_at(&self, pt: Point) -> Option<Side> {
        self.board.get(pt)
    }

    pub fn num_disk(&self, side: Side) -> u32 {
        self.board.num_disk(side)
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

        if let Some(ref player) = *self.player(turn.flip()) {
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
