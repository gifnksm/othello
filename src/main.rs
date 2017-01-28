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

#![cfg_attr(feature="nightly", feature(plugin))]
#![cfg_attr(feature="nightly", plugin(clippy))]
#![cfg_attr(feature="nightly", warn(mut_mut))]
#![cfg_attr(feature="nightly", warn(string_add))]
#![cfg_attr(feature="nightly", warn(string_add_assign))]

#![cfg_attr(feature="nightly", feature(windows_subsystem))]
#![cfg_attr(feature="nightly", windows_subsystem="windows")]

#[macro_use]
extern crate conrod;
extern crate find_folder;
extern crate rand;
extern crate vecmath;

use conrod::UiBuilder;
use conrod::backend::piston::Window;
use conrod::backend::piston::event::UpdateEvent;
use conrod::backend::piston::window::{self, GlyphCache, WindowEvents, WindowSettings};
use conrod::image::Map as ImageMap;
use view::Ids;
use view_model::App;

mod model;
mod view;
mod view_model;

fn main() {
    const WIDTH: u32 = 1024;
    const HEIGHT: u32 = 768;
    let mut window: Window = WindowSettings::new("Othello", [WIDTH, HEIGHT])
        .exit_on_esc(true)
        .vsync(true)
        .build()
        .expect("failed to build PistonWindow");

    let mut events = WindowEvents::new();

    let mut ui = UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .expect("faild to find assets folder");
    let font_path = &assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    let _ = ui.fonts.insert_from_file(font_path).expect("failed to load font");

    let mut text_texture_cache = GlyphCache::new(&mut window, WIDTH, HEIGHT);
    let image_map = ImageMap::new();

    let mut app = App::default();
    let mut ids = Ids::new(ui.widget_id_generator());

    while let Some(event) = window.next_event(&mut events) {
        if let Some(e) = window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        let _ = event.update(|_| {
            let mut ui = ui.set_widgets();
            view::set_widgets(&mut ui, &mut ids, &mut app)
        });

        let _ = window.draw_2d(&event,
                               |c, g| if let Some(primitives) = ui.draw_if_changed() {
                                   fn texture_from_image<T>(img: &T) -> &T {
                                       img
                                   };
                                   window::draw(c,
                                                g,
                                                primitives,
                                                &mut text_texture_cache,
                                                &image_map,
                                                texture_from_image);
                               });
    }
}
