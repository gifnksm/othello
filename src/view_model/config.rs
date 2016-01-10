use conrod::color::{self, Color};

use model::PlayerKind;

#[derive(Copy, Clone, Debug)]
pub struct GameConfig {
    pub rows: i32,
    pub cols: i32,
    pub black_player: PlayerKind,
    pub white_player: PlayerKind,
}

impl Default for GameConfig {
    fn default() -> GameConfig {
        GameConfig {
            rows: 8,
            cols: 8,
            black_player: PlayerKind::Human,
            white_player: PlayerKind::Human,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ViewConfig {
    pub frame_width: f64,
    pub cell_size: f64,
    pub disk_radius: f64,
    pub dot_radius: f64,
    pub board_margin: f64,
    pub indicator_text_width: f64,

    pub frame_color: Color,
    pub board_color: Color,
    pub white_color: Color,
    pub black_color: Color,
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
