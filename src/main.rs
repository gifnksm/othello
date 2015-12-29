#![warn(bad_style)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
// #![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

extern crate board_game_geom as geom;
extern crate piston_window;

use geom::{Geom, Move, Point, Size, Table};

use piston_window::{Button, Context, Graphics, MouseButton, MouseCursorEvent, PistonWindow, PressEvent, ReleaseEvent, Transformed, WindowSettings};
use piston_window::types::Color;

const BOARD_ROWS: usize = 8;
const BOARD_COLS: usize = 8;

const CELL_WIDTH: f64 = 80.0;
const CELL_HEIGHT: f64 = 80.0;

const BOARD_WIDTH: f64 = CELL_WIDTH * (BOARD_COLS as f64);
const BOARD_HEIGHT: f64 = CELL_HEIGHT * (BOARD_ROWS as f64);
const BOARD_H_MARGIN: f64 = 40.0;
const BOARD_V_MARGIN: f64 = 40.0;

const DOT_RADIUS: f64 = 6.0;
const DOT_DIAMETER: f64 = DOT_RADIUS * 2.0;

const DISK_RADIUS: f64 = 32.0;
const DISK_DIAMETER: f64 = DISK_RADIUS * 2.0;

const WINDOW_WIDTH: u32 = (BOARD_H_MARGIN * 2.0 + BOARD_WIDTH + 0.5) as u32;
const WINDOW_HEIGHT: u32 = (BOARD_V_MARGIN * 2.0 + BOARD_HEIGHT + 0.5) as u32;

const BLACK: Color = [0.0, 0.0, 0.0, 1.0];
const WHITE: Color = [1.0, 1.0, 1.0, 1.0];
const GREEN: Color = [0.0, 0.5, 0.0, 1.0];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Cell {
    Black, White
}

impl Into<Color> for Cell {
    fn into(self) -> Color {
        match self {
            Cell::Black => BLACK,
            Cell::White => WHITE,
        }
    }
}

impl Cell {
    fn flip(self) -> Cell {
        match self {
            Cell::Black => Cell::White,
            Cell::White => Cell::Black,
        }
    }
}

#[derive(Clone, Debug)]
struct Board {
    cells: Table<Option<Cell>>,
    turn: Option<Cell>,
}

impl Board {
    fn new() -> Board {
        let size = Size(8, 8);
        let mut board = Board {
            cells: Table::new_empty(size, None, None),
            turn: Some(Cell::Black)
        };
        board.cells[Point(3, 3)] = Some(Cell::White);
        board.cells[Point(4, 4)] = Some(Cell::White);
        board.cells[Point(3, 4)] = Some(Cell::Black);
        board.cells[Point(4, 3)] = Some(Cell::Black);
        board
    }

    fn can_locate(&self, pt: Point) -> bool {
        if self.turn.is_none() {
            return false;
        }

        if !self.cells.contains(pt) || self.cells[pt].is_some() {
            return false
        }

        for &mv in &Move::ALL_ADJACENTS {
            if self.can_locate_mv(pt, mv).is_some() {
                return true
            }
        }

        false
    }

    fn can_locate_mv(&self, pt: Point, mv: Move) -> Option<Point> {
        let turn = if let Some(turn) = self.turn {
            turn
        } else {
            return None
        };

        let flip = turn.flip();

        let mut pt = pt + mv;
        if !self.cells.contains(pt) || self.cells[pt] != Some(flip) {
            return None
        }

        while self.cells.contains(pt) {
            if let Some(x) = self.cells[pt] {
                pt = pt + mv;
                if x == flip {
                    continue
                }
                return Some(pt)
            }
            return None
        }
        None
    }

    fn locate(&mut self, pt: Point) {
        let turn = if let Some(turn) = self.turn {
            turn
        } else {
            return
        };
        let flip = turn.flip();

        if !self.can_locate(pt) {
            return
        }

        for &mv in &Move::ALL_ADJACENTS {
            if let Some(end) = self.can_locate_mv(pt, mv) {
                let mut pt = pt;
                while pt != end {
                    self.cells[pt] = Some(turn);
                    pt = pt + mv;
                }
            }
        }

        self.turn = Some(flip);
        for pt in self.cells.points() {
            if self.can_locate(pt) {
                return
            }
        }

        self.turn = Some(turn);
        for pt in self.cells.points() {
            if self.can_locate(pt) {
                return
            }
        }

        self.turn = None;
    }
}

fn draw_2d<G>(c: Context, g: &mut G, board: &mut Board, mouse_pos: Option<Point>)
    where G: Graphics
{
    piston_window::clear(GREEN, g);
    let board_trans = c.transform.trans(BOARD_H_MARGIN, BOARD_V_MARGIN);

    // draw lines
    for y in 0..(BOARD_ROWS + 1) {
        let fy = (y as f64) * CELL_HEIGHT;
        piston_window::line(BLACK, 1.0, [0.0, fy, BOARD_WIDTH, fy], board_trans, g);
    }
    for x in 0..(BOARD_COLS + 1) {
        let fx = (x as f64) * CELL_WIDTH;
        piston_window::line(BLACK, 1.0, [fx, 0.0, fx, BOARD_HEIGHT], board_trans, g);
    }

    // draw dots
    for x in 0..2 {
        let fx = ((x * 4 + 2) as f64) * CELL_WIDTH - DOT_RADIUS;
        for y in 0..2 {
            let fy = ((y * 4 + 2) as f64) * CELL_HEIGHT - DOT_RADIUS;
            piston_window::ellipse(BLACK, [fx, fy, DOT_DIAMETER, DOT_DIAMETER], board_trans, g)
        }
    }

    // draw disks
    for x in 0..BOARD_COLS {
        let fx = (x as f64 + 0.5) * CELL_WIDTH - DISK_RADIUS;
        for y in 0..BOARD_ROWS {
            let fy = (y as f64 + 0.5) * CELL_HEIGHT - DISK_RADIUS;
            let pt = Point(x as i32, y as i32);
            match board.cells[pt] {
                Some(cell) => {
                    piston_window::ellipse(cell.into(), [fx, fy, DISK_DIAMETER, DISK_DIAMETER], board_trans, g);
                }
                None => {
                    if let Some(turn) = board.turn {
                        if board.can_locate(pt) {
                            let mut color: Color = turn.into();
                            color[3] = if mouse_pos == Some(pt) { 0.7 } else { 0.3 };
                            piston_window::ellipse(color, [fx, fy, DISK_DIAMETER, DISK_DIAMETER], board_trans, g);
                        }
                    }
                }
            }
        }
    }
}

fn update_mouse_pos(e: &PistonWindow, pt: &mut Option<Point>) {
    if let Some(pos) = e.mouse_cursor_args() {
        let fx = (pos[0] - BOARD_H_MARGIN) / CELL_WIDTH;
        let fy = (pos[1] - BOARD_V_MARGIN) / CELL_HEIGHT;
        if fx < 0.0 || fx > (BOARD_COLS as f64) || fy < 0.0 || fy > (BOARD_ROWS as f64) {
            *pt = None;
        } else {
            *pt = Some(Point(fx as i32, fy as i32));
        }
    }
}

fn main() {
    let window: PistonWindow = WindowSettings::new("Othello", (WINDOW_WIDTH, WINDOW_HEIGHT))
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));
    let mut board = Board::new();
    let mut mouse_pos = None;
    let mut mouse_press_pos = None;

    for e in window {
        e.draw_2d(|c, g| draw_2d(c, g, &mut board, mouse_pos));

        update_mouse_pos(&e, &mut mouse_pos);

        if let Some(button) = e.press_args() {
            if button == Button::Mouse(MouseButton::Left) {
                mouse_press_pos = mouse_pos;
            }
        }

        if let Some(button) = e.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                if mouse_pos == mouse_press_pos {
                    if let Some(pt) = mouse_pos {
                        board.locate(pt);
                    }
                }
                mouse_press_pos = None;
            }
        }
    }
}
