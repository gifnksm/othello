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
extern crate opengl_graphics;
extern crate piston;
extern crate rand;
extern crate sdl2_window;
extern crate ttf_noto_sans;
extern crate vecmath;

use conrod::{Scalar, UiBuilder};
use conrod::backend::piston::{draw, event};
use conrod::image::Map as ImageMap;
use conrod::text::{FontCollection, GlyphCache};
use conrod::text::rt::Rect;
use opengl_graphics::{Format, GlGraphics, OpenGL, Texture, TextureSettings, UpdateTexture};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderEvent, UpdateEvent};
use piston::window::{Window, WindowSettings};
use sdl2_window::Sdl2Window;
use view::Ids;
use view_model::App;

mod model;
mod view;
mod view_model;

fn main() {
    const WIDTH: u32 = 1024;
    const HEIGHT: u32 = 768;

    let opengl = OpenGL::V2_1;
    let mut window: Sdl2Window = WindowSettings::new("Othello", [WIDTH, HEIGHT])
        .opengl(opengl)
        .srgb(false)
        .exit_on_esc(true)
        .vsync(true)
        .build()
        .expect("failed to build PistonWindow");
    let mut gl_graphics = GlGraphics::new(opengl);

    let mut ui = UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    let font_collection = FontCollection::from_bytes(ttf_noto_sans::REGULAR);
    let _ = ui.fonts.insert(font_collection.into_font().expect("failed to into_font"));

    let mut text_vertex_data = vec![];
    let (mut glyph_cache, mut text_texture_cache) = {
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        let cache = GlyphCache::new(WIDTH, HEIGHT, SCALE_TOLERANCE, POSITION_TOLERANCE);
        let buffer_len = WIDTH as usize * HEIGHT as usize;
        let init = vec![128; buffer_len];
        let settings = TextureSettings::new();
        let texture = Texture::from_memory_alpha(&init, WIDTH, HEIGHT, &settings)
            .expect("failed to create Texture");
        (cache, texture)
    };

    let image_map = ImageMap::new();

    let mut app = App::default();
    let mut ids = Ids::new(ui.widget_id_generator());

    let mut events = Events::new(EventSettings::new());

    while let Some(event) = events.next(&mut window) {
        let size = window.size();
        let (win_w, win_h) = (size.width as Scalar, size.height as Scalar);
        if let Some(e) = event::convert(event.clone(), win_w, win_h) {
            ui.handle_event(e);
        }

        let _ = event.update(|_| {
            let mut ui = ui.set_widgets();
            view::set_widgets(&mut ui, &mut ids, &mut app)
        });

        if let Some(args) = event.render_args() {
            gl_graphics.draw(args.viewport(),
                             |ctx, g2d| if let Some(primitives) = ui.draw_if_changed() {
                                 let cache_queued_glyphs = |_graphics: &mut GlGraphics,
                                                            cache: &mut Texture,
                                                            rect: Rect<u32>,
                                                            data: &[u8]| {
                    let offset = [rect.min.x, rect.min.y];
                    let size = [rect.width(), rect.height()];
                    let format = Format::Rgba8;
                    text_vertex_data.clear();
                    text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
                    UpdateTexture::update(cache,
                                          &mut (),
                                          format,
                                          &text_vertex_data[..],
                                          offset,
                                          size)
                        .expect("failed to update texture")
                };
                                 fn texture_from_image<T>(img: &T) -> &T {
                                     img
                                 };
                                 draw::primitives(primitives,
                                                  ctx,
                                                  g2d,
                                                  &mut text_texture_cache,
                                                  &mut glyph_cache,
                                                  &image_map,
                                                  cache_queued_glyphs,
                                                  texture_from_image);

                             });
        }
    }
}
