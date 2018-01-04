use self::alpha_beta::Player as AlphaBetaPlayer;
pub use self::evaluator::{Evaluate, EvenEvaluator, Score, StrongEvaluator, WeakEvaluator,
                          MAX_SCORE, MIN_SCORE};
use self::random::Player as RandomPlayer;
use model::{Board, Point, Side};
use std::sync::mpsc::{self, Receiver, SendError, Sender, TryRecvError};
use std::thread::{self, JoinHandle};

mod alpha_beta;
mod evaluator;
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
    AlphaBetaStrong(AiPower),
    AlphaBetaEven(AiPower),
    AlphaBetaWeak(AiPower),
}

#[derive(Copy, Clone, Debug)]
pub enum AiPower {
    Small,
    Medium,
    Large,
}

impl AiPower {
    fn to_alpha_beta_power(&self) -> u32 {
        use self::AiPower::*;
        match *self {
            Small => 1_000_000,
            Medium => 10_000_000,
            Large => 100_000_000,
        }
    }
}

impl Default for PlayerKind {
    fn default() -> PlayerKind {
        PlayerKind::Human
    }
}

impl AsRef<str> for PlayerKind {
    fn as_ref(&self) -> &str {
        use self::AiKind::*;
        use self::AiPower::*;
        use self::PlayerKind::*;
        match *self {
            Human => "Human",
            Ai(Random) => "AI: random",
            Ai(AlphaBetaStrong(Small)) => "AI: alpha-beta strong S",
            Ai(AlphaBetaStrong(Medium)) => "AI: alpha-beta strong M",
            Ai(AlphaBetaStrong(Large)) => "AI: alpha-beta strong L",
            Ai(AlphaBetaEven(Small)) => "AI: alpha-beta even S",
            Ai(AlphaBetaEven(Medium)) => "AI: alpha-beta even M",
            Ai(AlphaBetaEven(Large)) => "AI: alpha-beta even L",
            Ai(AlphaBetaWeak(Small)) => "AI: alpha-beta weak S",
            Ai(AlphaBetaWeak(Medium)) => "AI: alpha-beta weak M",
            Ai(AlphaBetaWeak(Large)) => "AI: alpha-beta weak L",
        }
    }
}

impl PlayerKind {
    pub fn all_values() -> [Self; 11] {
        use self::AiKind::*;
        use self::AiPower::*;
        use self::PlayerKind::*;
        [
            Human,
            Ai(Random),
            Ai(AlphaBetaStrong(Small)),
            Ai(AlphaBetaStrong(Medium)),
            Ai(AlphaBetaStrong(Large)),
            Ai(AlphaBetaEven(Small)),
            Ai(AlphaBetaEven(Medium)),
            Ai(AlphaBetaEven(Large)),
            Ai(AlphaBetaWeak(Small)),
            Ai(AlphaBetaWeak(Medium)),
            Ai(AlphaBetaWeak(Large)),
        ]
    }

    pub fn to_index(&self) -> usize {
        use self::AiKind::*;
        use self::AiPower::*;
        use self::PlayerKind::*;
        match *self {
            Human => 0,
            Ai(Random) => 1,
            Ai(AlphaBetaStrong(Small)) => 2,
            Ai(AlphaBetaStrong(Medium)) => 3,
            Ai(AlphaBetaStrong(Large)) => 4,
            Ai(AlphaBetaEven(Small)) => 5,
            Ai(AlphaBetaEven(Medium)) => 6,
            Ai(AlphaBetaEven(Large)) => 7,
            Ai(AlphaBetaWeak(Small)) => 8,
            Ai(AlphaBetaWeak(Medium)) => 9,
            Ai(AlphaBetaWeak(Large)) => 10,
        }
    }
}

pub struct AiPlayer {
    handle: JoinHandle<()>,
    receiver: Receiver<Point>,
    sender: Sender<Message>,
}

impl AiPlayer {
    pub fn new(kind: PlayerKind, board: &Board, side: Side) -> Option<AiPlayer> {
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
                AiKind::AlphaBetaStrong(power) => {
                    let evaluator = StrongEvaluator::new(board.size());
                    Box::new(AlphaBetaPlayer::new(
                        side,
                        power.to_alpha_beta_power(),
                        evaluator,
                    ))
                }
                AiKind::AlphaBetaEven(power) => {
                    let evaluator = EvenEvaluator::new(board.size());
                    Box::new(AlphaBetaPlayer::new(
                        side,
                        power.to_alpha_beta_power(),
                        evaluator,
                    ))
                }
                AiKind::AlphaBetaWeak(power) => {
                    let evaluator = WeakEvaluator::new(board.size());
                    Box::new(AlphaBetaPlayer::new(
                        side,
                        power.to_alpha_beta_power(),
                        evaluator,
                    ))
                }
            };
            ai_main(side, &player_tx, &player_rx, board, &mut *player);
        });

        Some(AiPlayer {
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

pub fn ai_main(
    side: Side,
    tx: &Sender<Point>,
    rx: &Receiver<Message>,
    mut board: Board,
    mut player: &mut FindMove,
) {
    loop {
        match board.turn() {
            None => match rx.recv() {
                Ok(Message::Exit) => break,
                Ok(msg) => panic!("{:?}", msg),
                Err(e) => panic!("error: {}", e),
            },
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
