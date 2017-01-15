use Side;

use geom::Point;
use model::Board;
use std::sync::mpsc::{self, Receiver, SendError, Sender, TryRecvError};
use std::thread::{self, JoinHandle};

mod ai;

#[derive(Clone, Debug)]
pub enum Message {
    Locate(Side, Point),
    Exit,
}

#[derive(Copy, Clone, Debug)]
pub enum PlayerKind {
    Human,
    AiRandom,
}

impl Default for PlayerKind {
    fn default() -> PlayerKind {
        PlayerKind::Human
    }
}

impl AsRef<str> for PlayerKind {
    fn as_ref(&self) -> &str {
        use self::PlayerKind::*;
        match *self {
            Human => "Human",
            AiRandom => "AI Random",
        }
    }
}

impl PlayerKind {
    pub fn all_values() -> [Self; 2] {
        use self::PlayerKind::*;
        [Human, AiRandom]
    }

    pub fn to_index(&self) -> usize {
        *self as usize
    }
}

pub struct Player {
    handle: JoinHandle<()>,
    receiver: Receiver<Point>,
    sender: Sender<Message>,
}

impl Player {
    pub fn new(kind: PlayerKind, board: &Board, side: Side) -> Option<Player> {
        let ai_routine = match kind {
            PlayerKind::Human => return None,
            PlayerKind::AiRandom => ai::random_player::main,
        };

        let (host_tx, player_rx) = mpsc::channel();
        let (player_tx, host_rx) = mpsc::channel();
        let board = board.clone();
        let handle = thread::spawn(move || ai_routine(side, player_tx, player_rx, board));

        Some(Player {
            handle: handle,
            receiver: host_rx,
            sender: host_tx,
        })
    }

    pub fn finish(self) {
        let _ = self.sender.send(Message::Exit);
        let _ = self.handle.join();
    }

    pub fn listen(&self) -> Result<Point, TryRecvError> {
        self.receiver.try_recv()
    }

    pub fn locate(&self, turn: Side, pt: Point) -> Result<(), SendError<Message>> {
        self.sender.send(Message::Locate(turn, pt))
    }
}
