use self::ai::Player as AiPlayer;
use self::random::Player as RandomPlayer;
use Side;
use model::{Board, Point};
use std::sync::mpsc::{self, Receiver, SendError, Sender, TryRecvError};
use std::thread::{self, JoinHandle};

mod ai;
mod random;

#[derive(Clone, Debug)]
pub enum Message {
    Place(Side, Point),
    Exit,
}

#[derive(Copy, Clone, Debug)]
pub enum PlayerKind {
    Human,
    Ai(AiKind),
}

#[derive(Copy, Clone, Debug)]
pub enum AiKind {
    Random,
    Weak,
}

impl Default for PlayerKind {
    fn default() -> PlayerKind {
        PlayerKind::Human
    }
}

impl AsRef<str> for PlayerKind {
    fn as_ref(&self) -> &str {
        use self::PlayerKind::*;
        use self::AiKind::*;
        match *self {
            Human => "Human",
            Ai(Random) => "Random",
            Ai(Weak) => "AI (weak)",
        }
    }
}

impl PlayerKind {
    pub fn all_values() -> [Self; 3] {
        use self::PlayerKind::*;
        use self::AiKind::*;
        [Human, Ai(Random), Ai(Weak)]
    }

    pub fn to_index(&self) -> usize {
        use self::PlayerKind::*;
        use self::AiKind::*;
        match *self {
            Human => 0,
            Ai(Random) => 1,
            Ai(Weak) => 2,
        }
    }
}

pub struct Player {
    handle: JoinHandle<()>,
    receiver: Receiver<Point>,
    sender: Sender<Message>,
}

impl Player {
    pub fn new(kind: PlayerKind, board: &Board, side: Side) -> Option<Player> {
        let ai_kind = match kind {
            PlayerKind::Human => return None,
            PlayerKind::Ai(ai_kind) => ai_kind,
        };

        let (host_tx, player_rx) = mpsc::channel();
        let (player_tx, host_rx) = mpsc::channel();
        let board = *board;
        let handle = thread::spawn(move || {
            match ai_kind {
                AiKind::Random => ai_main(side, player_tx, player_rx, board, RandomPlayer::new()),
                AiKind::Weak => {
                    let size = board.size();
                    ai_main(side, player_tx, player_rx, board, AiPlayer::new_weak(size))
                }
            };
        });

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

    pub fn place(&self, turn: Side, pt: Point) -> Result<(), SendError<Message>> {
        self.sender.send(Message::Place(turn, pt))
    }
}

pub trait FindMove {
    fn find_move(&mut self, board: Board) -> Point;
}

pub fn ai_main<T>(side: Side,
                  tx: Sender<Point>,
                  rx: Receiver<Message>,
                  mut board: Board,
                  mut player: T)
    where T: FindMove
{
    loop {
        match board.turn() {
            None => {
                match rx.recv() {
                    Ok(Message::Exit) => break,
                    Ok(msg) => panic!("{:?}", msg),
                    Err(e) => panic!("error: {}", e),
                }
            }
            Some(turn) => {
                if turn != side {
                    match rx.recv() {
                        Ok(Message::Place(_, pt)) => {
                            board.place(pt);
                            continue;
                        }
                        Ok(Message::Exit) => break,
                        Err(e) => panic!("error: {}", e),
                    }
                }

                let pt = player.find_move(board);
                if !board.place(pt) {
                    panic!("cannot place: {:?}", pt);
                }
                tx.send(pt).unwrap();
            }
        }
    }
}
