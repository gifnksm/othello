use Side;
use model::{Board, Point};
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
                        Ok(Message::Place(_, pt)) => {
                            board.place(pt);
                            continue;
                        }
                        Ok(Message::Exit) => break,
                        Err(e) => panic!("error: {}", e),
                    }
                } else {
                    let pt = {
                        let size = board.size();
                        let pts = (0..size.0)
                            .flat_map(|x| (0..size.1).map(move |y| (x, y)))
                            .filter(|&pt| board.can_place(pt));
                        rand::sample(&mut rng, pts, 1)[0]
                    };
                    if !board.place(pt) {
                        panic!("cannot place: {:?}", pt);
                    }
                    tx.send(pt).unwrap();
                }
            }
        }
    }
}
