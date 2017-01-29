use conrod::UiCell;

use view_model::{App, State};

mod othello_disk;
mod start;
mod play;

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
        indicator_label_texts[],
        indicator_player_texts[],
    }
}

pub fn set_widgets(ui: &mut UiCell, ids: &mut Ids, app: &mut App) {
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
