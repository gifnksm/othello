use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use conrod::{self, Button, Canvas, Circle, Frameable, Labelable, LineStyle, Rectangle, Sizeable,
             Text, Widget, WidgetMatrix};
use conrod::color::Colorable;
use conrod::Positionable;
use geom::{Point, Size};
use piston_window::Glyphs;

use Side;
use self::othello_disk::OthelloDisk;
use model::PlayerKind;
use view_model::{App, BoardSize, PlayState, StartState, State};

pub use self::ddl_builder::{DdlBuilder, DdlString};

pub mod ddl_builder;
mod othello_disk;

pub type Ui = conrod::Ui<Glyphs>;

widget_ids! {
    CANVAS,

    START_BUTTON,
    TIMES_LABEL,
    ROWS_DDL,
    COLS_DDL,
    BLACK_PLAYER_DDL,
    WHITE_PLAYER_DDL,

    PLAY_CANVAS,
    BOARD,
    DOT with 4,
    INDICATOR_LABEL_ICON with 2,
    INDICATOR_LABEL_TEXT with 2,
}

pub fn set_widgets(ui: &mut Ui, app_ref: Rc<RefCell<App>>) {
    let func: fn(ui: &mut Ui, Rc<RefCell<App>>) = {
        let app = app_ref.deref().borrow();
        match app.state {
            State::Start(_) => set_widgets_start,
            State::Play(_) => set_widgets_play,
        }
    };
    func(ui, app_ref.clone())
}

fn set_widgets_start(ui: &mut Ui, app_ref: Rc<RefCell<App>>) {
    let (gc, vc) = {
        let app = app_ref.deref().borrow();
        (app.game_config, app.view_config)
    };

    Canvas::new().color(vc.board_color).scroll_kids().set(CANVAS, ui);
    Text::new(&"x")
        .w_h(30.0, 50.0)
        .font_size(40)
        .align_text_middle()
        .mid_top_with_margin_on(CANVAS, 40.0)
        .set(TIMES_LABEL, ui);

    {
        let mut app = app_ref.deref().borrow_mut();
        let mut rows = app.game_config.rows;
        let mut cols = app.game_config.cols;
        {
            let start: &mut StartState = app.state.as_mut();
            start.build_ddl_rows()
                 .w_h(50.0, 50.0)
                 .left_from(TIMES_LABEL, 30.0)
                 .label("Rows")
                 .react(|selected_idx: &mut Option<usize>, new_idx, string: &str| {
                     *selected_idx = Some(new_idx);
                     rows = BoardSize::from_ddl_str(string).unwrap().0;
                 })
                 .set(ROWS_DDL, ui);
            start.build_ddl_cols()
                 .w_h(50.0, 50.0)
                 .right_from(TIMES_LABEL, 30.0)
                 .label("Cols")
                 .react(|selected_idx: &mut Option<usize>, new_idx, string: &str| {
                     *selected_idx = Some(new_idx);
                     cols = BoardSize::from_ddl_str(string).unwrap().0;
                 })
                 .set(COLS_DDL, ui);
        }
        app.game_config.rows = rows;
        app.game_config.cols = cols;
    }

    {
        let mut app = app_ref.deref().borrow_mut();
        let mut black_player = app.game_config.black_player;
        let mut white_player = app.game_config.white_player;
        {
            let start: &mut StartState = app.state.as_mut();
            start.build_ddl_black_player()
                 .w_h(150.0, 50.0)
                 .down_from(TIMES_LABEL, 40.0)
                 .left_from(TIMES_LABEL, 30.0)
                 .label("Black Player")
                 .react(|selected_idx: &mut Option<usize>, new_idx, string: &str| {
                     *selected_idx = Some(new_idx);
                     black_player = PlayerKind::from_ddl_str(string).unwrap();
                 })
                 .set(BLACK_PLAYER_DDL, ui);
            start.build_ddl_white_player()
                 .w_h(150.0, 50.0)
                 .down_from(TIMES_LABEL, 40.0)
                 .right_from(TIMES_LABEL, 30.0)
                 .label("White Player")
                 .react(|selected_idx: &mut Option<usize>, new_idx, string: &str| {
                     *selected_idx = Some(new_idx);
                     white_player = PlayerKind::from_ddl_str(string).unwrap();
                 })
                 .set(WHITE_PLAYER_DDL, ui);
        }
        app.game_config.black_player = black_player;
        app.game_config.white_player = white_player;
    }

    Button::new()
        .w_h(200.0, 50.0)
        .down_from(TIMES_LABEL, 130.0)
        .align_middle_x_of(TIMES_LABEL)
        .label("start")
        .react(|| {
            let mut app = app_ref.deref().borrow_mut();
            app.state = State::Play(PlayState::new(Size(gc.rows, gc.cols),
                                                   gc.black_player,
                                                   gc.white_player));
        })
        .set(START_BUTTON, ui);
}

fn set_widgets_play(ui: &mut Ui, app_ref: Rc<RefCell<App>>) {
    let (gc, vc) = {
        let app = app_ref.deref().borrow();
        (app.game_config, app.view_config)
    };

    {
        let mut app = app_ref.deref().borrow_mut();
        let play: &mut PlayState = app.state.as_mut();
        play.listen_player();
    }

    Canvas::new().color(vc.board_color).scroll_kids().set(CANVAS, ui);

    let board_width = vc.cell_size * (gc.cols as f64);
    let indicator_width = vc.cell_size + vc.indicator_text_width;
    let width = board_width + vc.board_margin * 2.0 + indicator_width + vc.board_margin;

    let board_height = vc.cell_size * (gc.rows as f64);
    let indicator_height = vc.cell_size * 2.0;
    let height = vc.board_margin * 2.0 + f64::max(board_height, indicator_height);

    let style = LineStyle::new().thickness(0.0);
    let rect = Rectangle::outline_styled([width, height], style);

    // FIXME (PistonDevelopers/conrod#659): cropped when window is smaller than canvas.
    match (ui.win_w < board_width, ui.win_h < board_height) {
        (true, true) => rect.top_left_of(CANVAS),
        (false, true) => rect.mid_top_of(CANVAS),
        (true, false) => rect.mid_left_of(CANVAS),
        (false, false) => rect.middle_of(CANVAS),
    }
    .set(PLAY_CANVAS, ui);

    WidgetMatrix::new(gc.cols as usize, gc.rows as usize)
        .top_left_with_margins_on(PLAY_CANVAS, vc.board_margin, vc.board_margin)
        .w_h(vc.cell_size * (gc.cols as f64),
             vc.cell_size * (gc.rows as f64))
        .each_widget(|_n, col, row| {
            let pt = Point(row as i32, col as i32);

            let app_ref = app_ref.clone();
            {
                let app = app_ref.deref().borrow();
                let play: &PlayState = app.state.as_ref();

                match play.turn() {
                    Some(turn) if play.can_locate(pt) && !play.has_player(turn) => {
                        OthelloDisk::new().flow_disk(Some(turn))
                    }
                    _ => OthelloDisk::new(),
                }
                .disk(play.get_disk_at(pt))
                .background_color(vc.board_color)
                .frame(vc.frame_width)
                .frame_color(vc.frame_color)
                .white_color(vc.white_color)
                .black_color(vc.black_color)
                .radius(vc.disk_radius)
            }
            .react(move || {
                let mut app = app_ref.deref().borrow_mut();
                let play: &mut PlayState = app.state.as_mut();
                if let Some(turn) = play.turn() {
                    if !play.has_player(turn) {
                        play.locate(pt);
                    }
                }
            })
        })
        .set(BOARD, ui);

    let x = vc.cell_size * ((gc.cols / 4) as f64);
    let y = vc.cell_size * ((gc.rows / 4) as f64);
    let signs = &[(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)];
    for (i, &(sx, sy)) in signs.iter().enumerate() {
        Circle::fill(vc.dot_radius)
            .x_y_relative_to(BOARD, sx * x, sy * y)
            .color(vc.frame_color)
            .set(DOT + i, ui);
    }

    for (i, &side) in [Side::Black, Side::White].iter().enumerate() {
        let app = app_ref.deref().borrow();
        let play: &PlayState = app.state.as_ref();

        if i == 0 {
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

        Text::new(&play.num_disk(side).to_string())
            .w(vc.indicator_text_width)
            .right_from(INDICATOR_LABEL_ICON + i, 0.0)
            .font_size(60)
            .align_text_right()
            .set(INDICATOR_LABEL_TEXT + i, ui);
    }
}
