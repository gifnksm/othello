use Side;
use geom::Point;
use model::Board;
use model::player::Message;

use rand;
use std::sync::mpsc::{Receiver, Sender};

pub fn main(side: Side, tx: Sender<Point>, rx: Receiver<Message>, mut board: Board) {
    let mut rng = rand::thread_rng();
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
                        Ok(Message::Locate(_, pt)) => {
                            board.locate(pt);
                            continue;
                        }
                        Ok(Message::Exit) => break,
                        Err(e) => panic!("error: {}", e),
                    }
                } else {
                    let pt = {
                        let pts = board.points().filter(|&pt| board.can_locate(pt));
                        rand::sample(&mut rng, pts, 1)[0]
                    };
                    if !board.locate(pt) {
                        panic!("cannot locate: {:?}", pt);
                    }
                    tx.send(pt).unwrap();
                }
            }
        }
    }
}
