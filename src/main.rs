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
#![warn(clippy::mut_mut)]
#![warn(clippy::string_add)]
#![warn(clippy::string_add_assign)]
#![windows_subsystem = "windows"]

#[macro_use]
extern crate conrod;
#[macro_use]
extern crate conrod_derive;

use crate::view::Ids;
use crate::view_model::App;
use conrod::backend::glium::glium::glutin::{
    ContextBuilder, Event, KeyboardInput, VirtualKeyCode, WindowBuilder, WindowEvent,
};
use conrod::backend::glium::glium::texture::Texture2d;
use conrod::backend::glium::glium::{self, Display, Surface};
use conrod::backend::glium::Renderer;
use conrod::image::Map as ImageMap;
use conrod::text::FontCollection;
use conrod::UiBuilder;
use std::fmt;

mod model;
mod view;
mod view_model;

fn main() {
    const WIDTH: u32 = 1024;
    const HEIGHT: u32 = 768;

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Othello")
        .with_dimensions(WIDTH, HEIGHT);
    let context = ContextBuilder::new().with_vsync(true).with_multisampling(4);
    let display = Display::new(window, context, &event_loop.raw).expect("failed to create Display");
    let mut ui = UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    let font_collection = FontCollection::from_bytes(ttf_noto_sans::REGULAR);
    let _ = ui
        .fonts
        .insert(font_collection.into_font().expect("failed to into_font"));

    let mut renderer = Renderer::new(&display).expect("failed to create Renderer");
    let image_map = ImageMap::<Texture2d>::new();

    let mut app = App::default();
    let mut ids = Ids::new(ui.widget_id_generator());
    'main: loop {
        for event in event_loop.next() {
            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                ui.handle_event(event);
                event_loop.needs_update();
            }

            match event {
                Event::WindowEvent { event, .. } => match event {
                    // Break from the loop upon `Escape`.
                    WindowEvent::Closed
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => break 'main,
                    _ => (),
                },
                _ => (),
            }
        }

        {
            let ui = &mut ui.set_widgets();
            view::set_widgets(ui, &mut ids, &mut app);
        }

        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();

            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}

/// In most of the examples the `glutin` crate is used for providing the window context and
/// events while the `glium` crate is used for displaying `conrod::render::Primitives` to the
/// screen.
///
/// This `Iterator`-like type simplifies some of the boilerplate involved in setting up a
/// glutin+glium event loop that works efficiently with conrod.
pub struct EventLoop {
    raw: glium::glutin::EventsLoop,
    ui_needs_update: bool,
    last_update: std::time::Instant,
}

impl fmt::Debug for EventLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EventLoop {{ raw: ..., ui_needs_update: {:?}, last_update: {:?} }}",
            self.ui_needs_update, self.last_update
        )
    }
}

impl EventLoop {
    pub fn new() -> Self {
        EventLoop {
            raw: glium::glutin::EventsLoop::new(),
            last_update: std::time::Instant::now(),
            ui_needs_update: true,
        }
    }

    /// Produce an iterator yielding all available events.
    pub fn next(&mut self) -> Vec<Event> {
        // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
        // since the last yield.
        let last_update = self.last_update;
        let sixteen_ms = std::time::Duration::from_millis(16);
        let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        // Collect all pending events.
        let mut events = Vec::new();
        self.raw.poll_events(|event| events.push(event));

        // If there are no events and the `Ui` does not need updating, wait for the next event.
        if events.is_empty() && !self.ui_needs_update {
            self.raw.run_forever(|event| {
                events.push(event);
                glium::glutin::ControlFlow::Break
            });
        }

        self.ui_needs_update = false;
        self.last_update = std::time::Instant::now();

        events
    }

    /// Notifies the event loop that the `Ui` requires another update whether or not there are any
    /// pending events.
    ///
    /// This is primarily used on the occasion that some part of the `Ui` is still animating and
    /// requires further updates to do so.
    pub fn needs_update(&mut self) {
        self.ui_needs_update = true;
    }
}
