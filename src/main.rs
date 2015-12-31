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
extern crate find_folder;
extern crate piston_window;

use geom::{Geom, Move, Point, Size, Table};

use piston_window::{Button, Graphics, Glyphs, PistonWindow, PressEvent, ReleaseEvent, Transformed,
                    WindowSettings};
use piston_window::character::CharacterCache;
use piston_window::context::Context;
use piston_window::text::Text;
use piston_window::types::Color;
use piston_window::mouse::{MouseButton, MouseCursorEvent};

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

const TEXT_WIDTH: f64 = 200.0;

const WINDOW_WIDTH: u32 = (BOARD_H_MARGIN * 2.0 + BOARD_WIDTH + TEXT_WIDTH + 0.5) as u32;
const WINDOW_HEIGHT: u32 = (BOARD_V_MARGIN * 2.0 + BOARD_HEIGHT + 0.5) as u32;

const BLACK: Color = [0.0, 0.0, 0.0, 1.0];
const WHITE: Color = [1.0, 1.0, 1.0, 1.0];
const GREEN: Color = [0.0, 0.5, 0.0, 1.0];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Side {
    Black,
    White,
}

impl Into<Color> for Side {
    fn into(self) -> Color {
        match self {
            Side::Black => BLACK,
            Side::White => WHITE,
        }
    }
}

impl Side {
    fn flip(self) -> Side {
        match self {
            Side::Black => Side::White,
            Side::White => Side::Black,
        }
    }
}

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
struct Board {
    cells: Table<Option<Side>>,
    locates: Table<Locate>,
    turn: Option<Side>,
    num_black: usize,
    num_white: usize,
    num_locates: usize,
}

impl Board {
    fn new() -> Board {
        let size = Size(8, 8);
        let mut board = Board {
            cells: Table::new_empty(size, None, None),
            locates: Table::new_empty(size, Locate::default(), Locate::default()),
            turn: Some(Side::Black),
            num_black: 2,
            num_white: 2,
            num_locates: 0,
        };
        board.cells[Point(3, 3)] = Some(Side::White);
        board.cells[Point(4, 4)] = Some(Side::White);
        board.cells[Point(3, 4)] = Some(Side::Black);
        board.cells[Point(4, 3)] = Some(Side::Black);
        board.update_locates();
        board
    }

    fn can_locate(&self, pt: Point) -> bool {
        self.locates[pt].num_end > 0
    }

    fn locate(&mut self, pt: Point) {
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
        if self.num_locates > 0 {
            return;
        }

        self.turn = Some(turn);
        self.update_locates();
        if self.num_locates > 0 {
            return;
        }

        self.turn = None;
    }

    fn num_black(&self) -> usize {
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

    fn num_white(&self) -> usize {
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
        self.num_locates = 0;

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
                        self.num_locates += 1;
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

fn draw_2d<G, C>(c: Context,
                 g: &mut G,
                 board: &mut Board,
                 mouse_pos: Option<Point>,
                 glyphs: &mut C)
    where G: Graphics<Texture = C::Texture>,
          C: CharacterCache
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
                    piston_window::ellipse(cell.into(),
                                           [fx, fy, DISK_DIAMETER, DISK_DIAMETER],
                                           board_trans,
                                           g);
                }
                None => {
                    if let Some(turn) = board.turn {
                        if board.can_locate(pt) {
                            let mut color: Color = turn.into();
                            color[3] = if mouse_pos == Some(pt) {
                                0.7
                            } else {
                                0.3
                            };
                            piston_window::ellipse(color,
                                                   [fx, fy, DISK_DIAMETER, DISK_DIAMETER],
                                                   board_trans,
                                                   g);
                        }
                    }
                }
            }
        }
    }

    // draw texts
    let text_trans = c.transform.trans(BOARD_H_MARGIN * 2.0 + BOARD_WIDTH, BOARD_V_MARGIN);

    piston_window::ellipse(Side::Black.into(),
                           [0.0, 0.0, DISK_DIAMETER, DISK_DIAMETER],
                           text_trans,
                           g);
    let black_text = format!("{:2}", board.num_black());
    Text::new_color(BLACK, 60).draw(&black_text,
                                    glyphs,
                                    &c.draw_state,
                                    text_trans.trans(DISK_DIAMETER + 30.0, 50.0),
                                    g);

    piston_window::ellipse(Side::White.into(),
                           [0.0, 80.0, DISK_DIAMETER, DISK_DIAMETER],
                           text_trans,
                           g);
    let black_text = format!("{:2}", board.num_white());
    Text::new_color(BLACK, 60).draw(&black_text,
                                    glyphs,
                                    &c.draw_state,
                                    text_trans.trans(DISK_DIAMETER + 30.0, 80.0 + 50.0),
                                    g);
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
                                   .unwrap_or_else(|e| {
                                       panic!("Failed to build PistonWindow: {}", e)
                                   });
    let assets = find_folder::Search::ParentsThenKids(3, 3)
                     .for_folder("assets")
                     .unwrap();
    let ref font = assets.join("FiraSans-Regular.ttf");
    let factory = window.factory.borrow().clone();
    let mut glyphs = Glyphs::new(font, factory).unwrap();

    let mut board = Board::new();
    let mut mouse_pos = None;
    let mut mouse_press_pos = None;

    for e in window {
        e.draw_2d(|c, g| draw_2d(c, g, &mut board, mouse_pos, &mut glyphs));

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
