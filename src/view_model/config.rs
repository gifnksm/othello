use super::BoardSize;
use crate::model::PlayerKind;
use conrod::color::{self, Color};

#[derive(Copy, Clone, Debug)]
pub struct GameConfig {
    pub rows: BoardSize,
    pub cols: BoardSize,
    pub black_player: PlayerKind,
    pub white_player: PlayerKind,
}

impl Default for GameConfig {
    fn default() -> GameConfig {
        GameConfig {
            rows: BoardSize::N8,
            cols: BoardSize::N8,
            black_player: PlayerKind::Human,
            white_player: PlayerKind::Human,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ViewConfig {
    pub border_width: f64,
    pub cell_size: f64,
    pub disk_radius_ratio: f64,
    pub dot_radius: f64,
    pub board_margin: f64,
    pub indicator_text_width: f64,
    pub indicator_width: f64,

    pub border_color: Color,
    pub board_color: Color,
    pub white_color: Color,
    pub black_color: Color,
}

impl Default for ViewConfig {
    fn default() -> ViewConfig {
        ViewConfig {
            border_width: 1.0,
            cell_size: 80.0,
            disk_radius_ratio: 0.4,
            dot_radius: 6.0,
            board_margin: 40.0,
            indicator_text_width: 90.0,
            indicator_width: 240.0,

            border_color: color::BLACK,
            board_color: color::rgba(0.0, 0.5, 0.0, 1.0),
            white_color: color::WHITE,
            black_color: color::BLACK,
        }
    }
}
