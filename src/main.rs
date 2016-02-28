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

#![cfg_attr(feature="dev", feature(plugin))]
#![cfg_attr(feature="dev", plugin(clippy))]
#![cfg_attr(feature="dev", warn(mut_mut))]
#![cfg_attr(feature="dev", warn(string_add))]
#![cfg_attr(feature="dev", warn(string_add_assign))]

extern crate board_game_geom as geom;
#[macro_use]
extern crate conrod;
extern crate find_folder;
extern crate piston_window;
extern crate rand;
extern crate vecmath;

use std::cell::RefCell;
use std::rc::Rc;

use conrod::{Theme, Widget};

use piston_window::{Glyphs, PistonWindow, UpdateEvent, WindowSettings};

use view_model::App;
use view::Ui;

mod model;
mod view;
mod view_model;

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

fn main() {
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
        let font_path = &assets.join("FiraSans-Regular.ttf");
        let factory = window.factory.borrow().clone();
        let glyph_cache = Glyphs::new(font_path, factory).unwrap();
        let theme = Theme::default();
        Ui::new(glyph_cache, theme)
    };

    let app_ref = Rc::new(RefCell::new(App::default()));
    for event in window {
        ui.handle_event(&event);

        let _ = event.update(|_| ui.set_widgets(|ui| view::set_widgets(ui, app_ref.clone())));
        event.draw_2d(|c, g| ui.draw_if_changed(c, g));
    }
}
