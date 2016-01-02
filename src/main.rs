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
#[macro_use]
extern crate conrod;
extern crate find_folder;
extern crate piston_window;
extern crate vecmath;

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use geom::{Point, Size};

use conrod::{Canvas, Circle, Frameable, LineStyle, Rectangle, Sizeable, Text, Theme, Widget,
             WidgetMatrix};
use conrod::color::{self, Color, Colorable};
use conrod::Positionable;

use piston_window::{Glyphs, PistonWindow, UpdateEvent, WindowSettings};

use board::Board;
use othello_disk::OthelloDisk;

type Ui = conrod::Ui<Glyphs>;

mod board;
mod othello_disk;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Side {
    Black,
    White,
}

impl Side {
    fn flip(self) -> Side {
        match self {
            Side::Black => Side::White,
            Side::White => Side::Black,
        }
    }
}

#[derive(Clone, Debug)]
struct App {
    board: Board,
    view_config: ViewConfig,
}

#[derive(Copy, Clone, Debug)]
struct ViewConfig {
    frame_width: f64,
    cell_size: f64,
    disk_radius: f64,
    dot_radius: f64,
    board_margin: f64,
    indicator_text_width: f64,

    frame_color: Color,
    board_color: Color,
    white_color: Color,
    black_color: Color,
}

impl Default for ViewConfig {
    fn default() -> ViewConfig {
        ViewConfig {
            frame_width: 1.0,
            cell_size: 80.0,
            disk_radius: 32.0,
            dot_radius: 6.0,
            board_margin: 40.0,
            indicator_text_width: 90.0,

            frame_color: color::black(),
            board_color: color::rgba(0.0, 0.5, 0.0, 1.0),
            white_color: color::white(),
            black_color: color::black(),
        }
    }
}

impl Default for App {
    fn default() -> App {
        App {
            board: Board::new(Size(8, 8)),
            view_config: ViewConfig::default(),
        }
    }
}

fn main() {
    let app_ref = Rc::new(RefCell::new(App::default()));

    let window: PistonWindow = {
        WindowSettings::new("Othello", (1024, 768))
            .exit_on_esc(true)
            .vsync(true)
            .build()
            .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e))
    };

    let mut ui = {
        let assets = find_folder::Search::KidsThenParents(3, 5)
                         .for_folder("assets")
                         .unwrap();
        let ref font_path = assets.join("FiraSans-Regular.ttf");
        let factory = window.factory.borrow().clone();
        let glyph_cache = Glyphs::new(font_path, factory).unwrap();
        let theme = Theme::default();
        Ui::new(glyph_cache, theme)
    };

    for event in window {
        ui.handle_event(&event);

        let _ = event.update(|_| ui.set_widgets(|ui| set_widgets(ui, app_ref.clone())));
        event.draw_2d(|c, g| ui.draw_if_changed(c, g));
    }
}

widget_ids! {
    CANVAS,
    OTHELLO_CANVAS,
    BOARD,
    DOT with 4,
    INDICATOR_LABEL_ICON with 2,
    INDICATOR_LABEL_TEXT with 2,
}

fn set_widgets(ui: &mut Ui, app_ref: Rc<RefCell<App>>) {
    let (Size(rows, cols), vc) = {
        let app = app_ref.deref().borrow();
        let size = app.board.size();
        (size, app.view_config)
    };

    Canvas::new().color(vc.board_color).scrolling(true).set(CANVAS, ui);

    let board_width = vc.cell_size * (cols as f64);
    let indicator_width = vc.cell_size + vc.indicator_text_width;
    let width = board_width + vc.board_margin * 2.0 + indicator_width + vc.board_margin;

    let board_height = vc.cell_size * (rows as f64);
    let indicator_height = vc.cell_size * 2.0;
    let height = vc.board_margin * 2.0 + f64::max(board_height, indicator_height);

    let style = LineStyle::new().thickness(0.0);
    let rect = Rectangle::outline_styled([width, height], style);

    match (ui.win_w < board_width, ui.win_h < board_height) {
        (true, true) => rect.top_left_of(CANVAS),
        (false, true) => rect.mid_top_of(CANVAS),
        (true, false) => rect.mid_left_of(CANVAS),
        (false, false) => rect.middle_of(CANVAS),
    }
    .set(OTHELLO_CANVAS, ui);

    WidgetMatrix::new(cols as usize, rows as usize)
        .top_left_with_margins_on(OTHELLO_CANVAS, vc.board_margin, vc.board_margin)
        .w_h(vc.cell_size * (cols as f64), vc.cell_size * (rows as f64))
        .each_widget(|_n, col, row| {
            let pt = Point(row as i32, col as i32);
            let disk = {
                let app_ref = app_ref.clone();
                let app = app_ref.deref().borrow();
                let mut disk = OthelloDisk::new()
                                   .background_color(vc.board_color)
                                   .frame(vc.frame_width)
                                   .frame_color(vc.frame_color)
                                   .white_color(vc.white_color)
                                   .black_color(vc.black_color)
                                   .radius(vc.disk_radius)
                                   .disk(app.board[pt]);
                if let Some(turn) = app.board.turn() {
                    if app.board.can_locate(pt) {
                        disk = disk.flow_disk(Some(turn));
                    }
                }
                disk
            };

            let app_ref = app_ref.clone();
            disk.react(move || {
                app_ref.borrow_mut().board.locate(pt);
            })
        })
        .set(BOARD, ui);

    let x = vc.cell_size * ((cols / 4) as f64);
    let y = vc.cell_size * ((rows / 4) as f64);
    let signs = &[(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)];
    for (i, &(sx, sy)) in signs.iter().enumerate() {
        Circle::fill(vc.dot_radius)
            .x_y_relative_to(BOARD, sx * x, sy * y)
            .color(vc.frame_color)
            .set(DOT + i, ui);
    }

    for (i, &side) in [Side::Black, Side::White].iter().enumerate() {
        let app = app_ref.deref().borrow();
        if side == Side::Black {
            OthelloDisk::new().right_from(BOARD, vc.board_margin)
        } else {
            OthelloDisk::new().down_from(INDICATOR_LABEL_ICON + (i - 1), 0.0)
        }
        .w_h(vc.cell_size, vc.cell_size)
        .background_color(vc.board_color)
        .frame(0.0)
        .white_color(vc.white_color)
        .black_color(vc.black_color)
        .radius(vc.disk_radius)
        .disk(Some(side))
        .react(|| {})
        .set(INDICATOR_LABEL_ICON + i, ui);

        let text = match side {
            Side::Black => app.board.num_black().to_string(),
            Side::White => app.board.num_white().to_string(),
        };
        Text::new(&text)
            .w(vc.indicator_text_width)
            .right_from(INDICATOR_LABEL_ICON + i, 0.0)
            .font_size(60)
            .align_text_right()
            .set(INDICATOR_LABEL_TEXT + i, ui);
    }
}
