use super::Ids;
use super::othello_disk::OthelloDisk;
use conrod::{Borderable, Sizeable, UiCell, Widget};
use conrod::Positionable;
use conrod::color::Colorable;
use conrod::widget::{Canvas, Circle, Matrix, Rectangle, Text};
use conrod::widget::line::Style as LineStyle;
use model::{Point, Side};
use view_model::{GameConfig, PlayState, State, ViewConfig};

pub fn set_widgets(ui: &mut UiCell,
                   ids: &mut Ids,
                   gc: &GameConfig,
                   vc: &ViewConfig,
                   play: &mut PlayState)
                   -> Option<State> {
    play.listen_player();

    let cols = gc.cols.to_value();
    let rows = gc.rows.to_value();

    Canvas::new().color(vc.board_color).scroll_kids().set(ids.canvas, ui);

    let board_width = vc.cell_size * (cols as f64);
    let indicator_width = vc.cell_size + vc.indicator_text_width;
    let width = board_width + vc.board_margin * 2.0 + indicator_width + vc.board_margin;

    let board_height = vc.cell_size * (rows as f64);
    let indicator_height = vc.cell_size * 2.0;
    let height = vc.board_margin * 2.0 + f64::max(board_height, indicator_height);

    let style = LineStyle::new().thickness(0.0);
    Rectangle::outline_styled([width, height], style)
        .middle_of(ids.canvas)
        .set(ids.play_canvas, ui);

    let mut elements = Matrix::new(cols as usize, rows as usize)
        .top_left_with_margins_on(ids.play_canvas, vc.board_margin, vc.board_margin)
        .w_h(vc.cell_size * (cols as f64), vc.cell_size * (rows as f64))
        .set(ids.board, ui);

    while let Some(element) = elements.next(ui) {
        let pt = Point(element.row as u32, element.col as u32);

        let disk = {
            let mut disk = OthelloDisk::new();
            if let Some(turn) = play.turn() {
                if play.can_move(pt) && !play.has_player(turn) {
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
            if let Some(turn) = play.turn() {
                if !play.has_player(turn) {
                    play.make_move(pt);
                }
            }
        }
    }

    let x = vc.cell_size * ((cols / 4) as f64);
    let y = vc.cell_size * ((rows / 4) as f64);
    let signs = &[(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)];
    ids.dots.resize(signs.len(), &mut ui.widget_id_generator());
    for (&id, &(sx, sy)) in ids.dots.iter().zip(signs) {
        Circle::fill(vc.dot_radius)
            .x_y_relative_to(ids.board, sx * x, sy * y)
            .color(vc.border_color)
            .set(id, ui);
    }

    let sides = &[Side::Black, Side::White];
    ids.indicator_player_texts.resize(sides.len(), &mut ui.widget_id_generator());
    ids.indicator_label_icons.resize(sides.len(), &mut ui.widget_id_generator());
    ids.indicator_label_texts.resize(sides.len(), &mut ui.widget_id_generator());
    let iter = ids.indicator_player_texts
        .iter()
        .zip(ids.indicator_label_icons.iter())
        .zip(ids.indicator_label_texts.iter())
        .zip(sides);
    for (((&player_id, &icon_id), &text_id), &side) in iter {
        let kind = play.player_kind(side);
        let player = Text::new(kind.as_ref());
        let player = if player_id == ids.indicator_player_texts[0] {
            player.right_from(ids.board, vc.board_margin)
        } else {
            player.down_from(ids.indicator_label_icons[0], 10.0)
        };
        player.w(vc.cell_size + vc.indicator_text_width)
            .font_size(30)
            .set(player_id, ui);

        OthelloDisk::new()
            .down_from(player_id, 0.0)
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

    None
}
