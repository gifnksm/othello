use conrod::{Labelable, Sizeable, Widget, UiCell};
use conrod::Positionable;
use conrod::color::Colorable;
use conrod::widget::{Button, Canvas, Text, DropDownList};
use geom::Size;

use model::PlayerKind;
use super::Ids;
use view_model::{PlayState, State, BoardSize, GameConfig, ViewConfig};

pub fn set_widgets(ui: &mut UiCell,
                   ids: &mut Ids,
                   gc: &mut GameConfig,
                   vc: &ViewConfig)
                   -> Option<State> {
    Canvas::new().color(vc.board_color).scroll_kids().set(ids.canvas, ui);

    Text::new("x")
        .w_h(30.0, 50.0)
        .font_size(40)
        .align_text_middle()
        .mid_top_with_margin_on(ids.canvas, 40.0)
        .set(ids.times_label, ui);

    let board_sizes = BoardSize::all_values();
    gc.rows = DropDownList::new(&board_sizes, Some(gc.rows.to_index()))
        .w_h(50.0, 50.0)
        .left_from(ids.times_label, 30.0)
        .label("Rows")
        .set(ids.rows_ddl, ui)
        .map(|idx| board_sizes[idx])
        .unwrap_or(gc.rows);
    gc.cols = DropDownList::new(&board_sizes, Some(gc.cols.to_index()))
        .w_h(50.0, 50.0)
        .right_from(ids.times_label, 30.0)
        .label("Cols")
        .set(ids.cols_ddl, ui)
        .map(|idx| board_sizes[idx])
        .unwrap_or(gc.cols);

    let player_kinds = PlayerKind::all_values();
    gc.black_player = DropDownList::new(&player_kinds, Some(gc.black_player.to_index()))
        .w_h(150.0, 50.0)
        .down_from(ids.times_label, 40.0)
        .left_from(ids.times_label, 30.0)
        .label("Black Player")
        .set(ids.black_player_ddl, ui)
        .map(|idx| player_kinds[idx])
        .unwrap_or(gc.black_player);
    gc.white_player = DropDownList::new(&player_kinds, Some(gc.white_player.to_index()))
        .w_h(150.0, 50.0)
        .down_from(ids.times_label, 40.0)
        .right_from(ids.times_label, 30.0)
        .label("White Player")
        .set(ids.white_player_ddl, ui)
        .map(|idx| player_kinds[idx])
        .unwrap_or(gc.white_player);

    let clicked = Button::new()
        .w_h(200.0, 50.0)
        .down_from(ids.times_label, 130.0)
        .align_middle_x_of(ids.times_label)
        .label("start")
        .set(ids.start_button, ui)
        .was_clicked();

    if clicked {
        let new_state = State::Play(PlayState::new(Size(gc.rows.to_value(), gc.cols.to_value()),
                                                   gc.black_player,
                                                   gc.white_player));
        Some(new_state)
    } else {
        None
    }
}
