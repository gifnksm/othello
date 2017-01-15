use Side;

use conrod::{Borderable, Labelable, Sizeable, Widget, UiCell};
use conrod::Positionable;
use conrod::color::Colorable;
use conrod::widget::{Button, Canvas, Circle, Rectangle, Text, Matrix};
use conrod::widget::line::Style as LineStyle;
use geom::{Point, Size};
use model::PlayerKind;

pub use self::ddl_builder::{DdlBuilder, DdlString};
use self::othello_disk::OthelloDisk;
use view_model::{App, BoardSize, PlayState, StartState, State, StateKind};

pub mod ddl_builder;
mod othello_disk;

widget_ids! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Ids {
        canvas,

        start_button,
        times_label,
        rows_ddl,
        cols_ddl,
        black_player_ddl,
        white_player_ddl,

        play_canvas,
        board,
        dots[],
        indicator_label_icons[],
        indicator_label_texts[]
    }
}

pub fn set_widgets(ui: &mut UiCell, ids: &mut Ids, app: &mut App) {
    let state = {
        app.state.kind()
    };
    match state {
        StateKind::Start => set_widgets_start(ui, ids, app),
        StateKind::Play => set_widgets_play(ui, ids, app),
    }
}

fn set_widgets_start(ui: &mut UiCell, ids: &mut Ids, app: &mut App) {
    let (gc, vc) = {
        (app.game_config, app.view_config)
    };

    Canvas::new().color(vc.board_color).scroll_kids().set(ids.canvas, ui);
    Text::new("x")
        .w_h(30.0, 50.0)
        .font_size(40)
        .align_text_middle()
        .mid_top_with_margin_on(ids.canvas, 40.0)
        .set(ids.times_label, ui);

    {
        let mut rows = app.game_config.rows;
        let mut cols = app.game_config.cols;
        {
            let start: &mut StartState = app.state.as_mut();
            let maybe_rows = start.build_ddl_rows()
                .w_h(50.0, 50.0)
                .left_from(ids.times_label, 30.0)
                .label("Rows")
                .set(ids.rows_ddl, ui)
                .and_then(BoardSize::from_ddl_index);
            if let Some(BoardSize(size)) = maybe_rows {
                rows = size;
            }

            let maybe_cols = start.build_ddl_cols()
                .w_h(50.0, 50.0)
                .right_from(ids.times_label, 30.0)
                .label("Cols")
                .set(ids.cols_ddl, ui)
                .and_then(BoardSize::from_ddl_index);
            if let Some(BoardSize(size)) = maybe_cols {
                cols = size;
            }
        }
        app.game_config.rows = rows;
        app.game_config.cols = cols;
    }

    {
        let mut black_player = app.game_config.black_player;
        let mut white_player = app.game_config.white_player;
        {
            let start: &mut StartState = app.state.as_mut();
            let _ = start.build_ddl_black_player()
                .w_h(150.0, 50.0)
                .down_from(ids.times_label, 40.0)
                .left_from(ids.times_label, 30.0)
                .label("Black Player")
                .set(ids.black_player_ddl, ui)
                .and_then(PlayerKind::from_ddl_index)
                .map(|kind| black_player = kind);
            let _ = start.build_ddl_white_player()
                .w_h(150.0, 50.0)
                .down_from(ids.times_label, 40.0)
                .right_from(ids.times_label, 30.0)
                .label("White Player")
                .set(ids.white_player_ddl, ui)
                .and_then(PlayerKind::from_ddl_index)
                .map(|kind| white_player = kind);
        }
        app.game_config.black_player = black_player;
        app.game_config.white_player = white_player;
    }

    let clicked = Button::new()
        .w_h(200.0, 50.0)
        .down_from(ids.times_label, 130.0)
        .align_middle_x_of(ids.times_label)
        .label("start")
        .set(ids.start_button, ui)
        .was_clicked();
    if clicked {
        app.state =
            State::Play(PlayState::new(Size(gc.rows, gc.cols), gc.black_player, gc.white_player));
    }
}

fn set_widgets_play(ui: &mut UiCell, ids: &mut Ids, app: &mut App) {
    let (gc, vc) = {
        (app.game_config, app.view_config)
    };

    {
        let play: &mut PlayState = app.state.as_mut();
        play.listen_player();
    }

    Canvas::new().color(vc.board_color).scroll_kids().set(ids.canvas, ui);

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
            (true, true) => rect.top_left_of(ids.canvas),
            (false, true) => rect.mid_top_of(ids.canvas),
            (true, false) => rect.mid_left_of(ids.canvas),
            (false, false) => rect.middle_of(ids.canvas),
        }
        .set(ids.play_canvas, ui);

    let mut elements = Matrix::new(gc.cols as usize, gc.rows as usize)
        .top_left_with_margins_on(ids.play_canvas, vc.board_margin, vc.board_margin)
        .w_h(vc.cell_size * (gc.cols as f64),
             vc.cell_size * (gc.rows as f64))
        .set(ids.board, ui);

    while let Some(element) = elements.next(ui) {
        let pt = Point(element.row as i32, element.col as i32);

        let disk = {
            let play: &PlayState = app.state.as_ref();

            let mut disk = OthelloDisk::new();
            if let Some(turn) = play.turn() {
                if play.can_locate(pt) && !play.has_player(turn) {
                    disk = disk.flow_disk(turn);
                }
            }
            if let Some(side) = play.get_disk_at(pt) {
                disk = disk.disk(side);
            }
            disk.background_color(vc.board_color)
                .border(vc.border_width)
                .border_color(vc.border_color)
                .white_color(vc.white_color)
                .black_color(vc.black_color)
                .radius_ratio(vc.disk_radius_ratio)
        };

        let clicked = element.set(disk, ui);
        if clicked {
            let play: &mut PlayState = app.state.as_mut();
            if let Some(turn) = play.turn() {
                if !play.has_player(turn) {
                    play.locate(pt);
                }
            }
        }
    }


    let x = vc.cell_size * ((gc.cols / 4) as f64);
    let y = vc.cell_size * ((gc.rows / 4) as f64);
    let signs = &[(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)];
    ids.dots.resize(signs.len(), &mut ui.widget_id_generator());
    for (&id, &(sx, sy)) in ids.dots.iter().zip(signs) {
        Circle::fill(vc.dot_radius)
            .x_y_relative_to(ids.board, sx * x, sy * y)
            .color(vc.border_color)
            .set(id, ui);
    }

    let sides = &[Side::Black, Side::White];
    ids.indicator_label_icons.resize(sides.len(), &mut ui.widget_id_generator());
    ids.indicator_label_texts.resize(sides.len(), &mut ui.widget_id_generator());
    let iter = ids.indicator_label_icons.iter().zip(ids.indicator_label_texts.iter()).zip(sides);
    for ((&icon_id, &text_id), &side) in iter {
        let play: &PlayState = app.state.as_ref();

        if icon_id == ids.indicator_label_icons[0] {
                OthelloDisk::new().right_from(ids.board, vc.board_margin)
            } else {
                OthelloDisk::new().down_from(ids.indicator_label_icons[0], 0.0)
            }
            .w_h(vc.cell_size, vc.cell_size)
            .background_color(vc.board_color)
            .border(0.0)
            .white_color(vc.white_color)
            .black_color(vc.black_color)
            .radius_ratio(vc.disk_radius_ratio)
            .disk(side)
            .set(icon_id, ui);

        Text::new(&play.num_disk(side).to_string())
            .w(vc.indicator_text_width)
            .right_from(icon_id, 0.0)
            .font_size(60)
            .align_text_right()
            .set(text_id, ui);
    }
}
