use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Receiver, SendError, Sender, TryRecvError};

use geom::{Point, Size};

use Side;
use model::Board;

mod ai;

#[derive(Clone, Debug)]
pub enum Message {
    Board(Size, Vec<(Side, Point)>),
    Locate(Side, Point),
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
        let handle = thread::spawn(move || ai_routine(side, player_tx, player_rx));

        let disks = board.create_disks();
        let message = Message::Board(board.size(), disks);

        host_tx.send(message).unwrap();

        Some(Player {
            handle: handle,
            receiver: host_rx,
            sender: host_tx,
        })
    }

    pub fn finish(self) {
        let _ = self.handle.join();
    }

    pub fn listen(&self) -> Result<Point, TryRecvError> {
        self.receiver.try_recv()
    }

    pub fn locate(&self, turn: Side, pt: Point) -> Result<(), SendError<Message>> {
        self.sender.send(Message::Locate(turn, pt))
    }
}
