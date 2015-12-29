extern crate piston_window;
use piston_window::{PistonWindow, Transformed, WindowSettings};
use piston_window::types::Color;

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

const WINDOW_WIDTH: u32 = (BOARD_H_MARGIN * 2.0 + BOARD_WIDTH + 0.5) as u32;
const WINDOW_HEIGHT: u32 = (BOARD_V_MARGIN * 2.0 + BOARD_HEIGHT + 0.5) as u32;

const BLACK: Color = [0.0, 0.0, 0.0, 1.0];
const WHITE: Color = [1.0, 1.0, 1.0, 1.0];
const GREEN: Color = [0.0, 0.5, 0.0, 1.0];

fn main() {
    let window: PistonWindow = WindowSettings::new("Othello", (WINDOW_WIDTH, WINDOW_HEIGHT))
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    for e in window {
        e.draw_2d(|c, g| {
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
                    match (x + y) % 3 {
                        0 => piston_window::ellipse(BLACK, [fx, fy, DISK_DIAMETER, DISK_DIAMETER], board_trans, g),
                        1 => piston_window::ellipse(WHITE, [fx, fy, DISK_DIAMETER, DISK_DIAMETER], board_trans, g),
                        _ => {}
                    }
                }
            }
        });
    }
}
