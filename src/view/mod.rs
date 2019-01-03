use crate::view_model::{App, State};
use conrod::UiCell;

mod play;
mod start;
mod widget;

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
        black_indicator,
        white_indicator,
        stop_button,
    }
}

pub fn set_widgets(ui: &mut UiCell<'_>, ids: &mut Ids, app: &mut App) {
    let new_state = match app.state {
        State::Start => start::set_widgets(ui, ids, &mut app.game_config, &app.view_config),
        State::Play(ref mut play) => {
            play::set_widgets(ui, ids, &app.game_config, &app.view_config, play)
        }
    };

    if let Some(new_state) = new_state {
        app.state = new_state;
    }
}
