use std::ops::Index;
use geom::{Geom, Move, Point, Size, Table};
use Side;

#[derive(Copy, Clone, Debug)]
struct Locate {
    num_end: usize,
    end_points: [(Move, Point); 8],
}

impl Default for Locate {
    fn default() -> Locate {
        Locate {
            num_end: 0,
            end_points: [(Move(0, 0), Point(-1, -1)); 8],
        }
    }
}

impl Locate {
    fn reset(&mut self) {
        self.num_end = 0;
    }

    fn push(&mut self, mv: Move, end: Point) {
        self.end_points[self.num_end] = (mv, end);
        self.num_end += 1;
    }
}

#[derive(Clone, Debug)]
pub struct Board {
    cells: Table<Option<Side>>,
    locates: Table<Locate>,
    turn: Option<Side>,
    num_black: usize,
    num_white: usize,
    num_locate: usize,
}

impl Index<Point> for Board {
    type Output = Option<Side>;

    fn index(&self, p: Point) -> &Option<Side> {
        self.cells.index(p)
    }
}

impl Board {
    pub fn new() -> Board {
        let size = Size(8, 8);
        let mut board = Board {
            cells: Table::new_empty(size, None, None),
            locates: Table::new_empty(size, Locate::default(), Locate::default()),
            turn: Some(Side::Black),
            num_black: 2,
            num_white: 2,
            num_locate: 0,
        };
        board.cells[Point(3, 3)] = Some(Side::White);
        board.cells[Point(4, 4)] = Some(Side::White);
        board.cells[Point(3, 4)] = Some(Side::Black);
        board.cells[Point(4, 3)] = Some(Side::Black);
        board.update_locates();
        board
    }

    pub fn can_locate(&self, pt: Point) -> bool {
        self.locates[pt].num_end > 0
    }

    pub fn locate(&mut self, pt: Point) {
        let turn = if let Some(turn) = self.turn {
            turn
        } else {
            return;
        };
        let flip = turn.flip();

        if !self.can_locate(pt) {
            return;
        }

        self.cells[pt] = Some(turn);

        let mut num_flip = 0;
        let loc = self.locates[pt];
        for &(mv, end) in &loc.end_points[..loc.num_end] {
            let mut pt = pt + mv;
            while pt != end {
                num_flip += 1;
                self.cells[pt] = Some(turn);
                pt = pt + mv;
            }
        }
        debug_assert!(num_flip > 0);

        match turn {
            Side::Black => {
                self.num_black += num_flip + 1;
                self.num_white -= num_flip;
            }
            Side::White => {
                self.num_white += num_flip + 1;
                self.num_black -= num_flip;
            }
        }

        self.turn = Some(flip);
        self.update_locates();
        if self.num_locate > 0 {
            return;
        }

        self.turn = Some(turn);
        self.update_locates();
        if self.num_locate > 0 {
            return;
        }

        self.turn = None;
    }

    pub fn turn(&self) -> Option<Side> {
        self.turn
    }

    pub fn num_black(&self) -> usize {
        if cfg!(debug) {
            let cnt = self.cells
                          .points()
                          .map(|pt| self.cells[pt])
                          .filter(|&cell| cell == Some(Side::Black))
                          .count();
            assert_eq!(cnt, self.num_black);
        }
        self.num_black
    }

    pub fn num_white(&self) -> usize {
        if cfg!(debug) {
            let cnt = self.cells
                          .points()
                          .map(|pt| self.cells[pt])
                          .filter(|&cell| cell == Some(Side::White))
                          .count();
            assert_eq!(cnt, self.num_white);
        }
        self.num_white
    }

    fn update_locates(&mut self) {
        self.num_locate = 0;

        if let Some(turn) = self.turn {
            for pt in self.cells.points() {
                let mut loc = Locate::default();

                if self.cells[pt].is_none() {
                    for &mv in &Move::ALL_ADJACENTS {
                        if let Some(end) = self.can_locate_mv(turn, pt, mv) {
                            loc.push(mv, end);
                        }
                    }

                    if loc.num_end > 0 {
                        self.num_locate += 1;
                    }
                }

                self.locates[pt] = loc;
            }
        } else {
            for pt in self.cells.points() {
                self.locates[pt].reset();
            }
        };
    }

    fn can_locate_mv(&self, turn: Side, pt: Point, mv: Move) -> Option<Point> {
        let flip = turn.flip();

        let mut pt = pt + mv;
        if !self.cells.contains(pt) || self.cells[pt] != Some(flip) {
            return None;
        }

        while self.cells.contains(pt) {
            if let Some(x) = self.cells[pt] {
                if x == flip {
                    pt = pt + mv;
                    continue;
                }
                return Some(pt);
            }
            return None;
        }
        None
    }
}
