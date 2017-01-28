use self::ai::Player as AiPlayer;
use self::random::Player as RandomPlayer;
use model::{Board, Point, Side};
use std::sync::mpsc::{self, Receiver, SendError, Sender, TryRecvError};
use std::thread::{self, JoinHandle};

mod ai;
mod random;

#[derive(Clone, Debug)]
pub enum Message {
    MakeMove(Side, Point),
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
    Medium,
    Strong,
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
            Ai(Medium) => "AI (medium)",
            Ai(Strong) => "AI (strong)",
        }
    }
}

impl PlayerKind {
    pub fn all_values() -> [Self; 5] {
        use self::PlayerKind::*;
        use self::AiKind::*;
        [Human, Ai(Random), Ai(Weak), Ai(Medium), Ai(Strong)]
    }

    pub fn to_index(&self) -> usize {
        use self::PlayerKind::*;
        use self::AiKind::*;
        match *self {
            Human => 0,
            Ai(Random) => 1,
            Ai(Weak) => 2,
            Ai(Medium) => 3,
            Ai(Strong) => 4,
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
            let mut player: Box<FindMove> = match ai_kind {
                AiKind::Random => Box::new(RandomPlayer::new()),
                AiKind::Weak => Box::new(AiPlayer::new_weak(side, board.size())),
                AiKind::Medium => Box::new(AiPlayer::new_medium(side, board.size())),
                AiKind::Strong => Box::new(AiPlayer::new_strong(side, board.size())),
            };
            ai_main(side, player_tx, player_rx, board, &mut *player);
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

    pub fn make_move(&self, turn: Side, pt: Point) -> Result<(), SendError<Message>> {
        self.sender.send(Message::MakeMove(turn, pt))
    }
}

pub trait FindMove {
    fn find_move(&mut self, board: Board) -> Point;
}

pub fn ai_main(side: Side,
               tx: Sender<Point>,
               rx: Receiver<Message>,
               mut board: Board,
               mut player: &mut FindMove) {
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
                        Ok(Message::MakeMove(_, pt)) => {
                            board = board.make_move(pt).expect("cannot make_move");
                            continue;
                        }
                        Ok(Message::Exit) => break,
                        Err(e) => panic!("error: {}", e),
                    }
                }

                let pt = player.find_move(board);
                board = board.make_move(pt).expect("cannot make_move");
                tx.send(pt).unwrap();
            }
        }
    }
}
