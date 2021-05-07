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

use crate::view::Ids;
use crate::view_model::App;
use conrod_core::UiBuilder;
use conrod_core::{image::Map as ImageMap, text::Font};
use conrod_glium::Renderer;
use glium::glutin::{event, event_loop};
use glium::glutin::{window::WindowBuilder, ContextBuilder};
use glium::texture::Texture2d;
use glium::{self, Display, Surface};

mod model;
mod view;
mod view_model;

fn main() {
    const WIDTH: u32 = 1024;
    const HEIGHT: u32 = 768;

    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Othello")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));
    let context = ContextBuilder::new().with_vsync(true).with_multisampling(4);
    let display = Display::new(window, context, &event_loop).expect("failed to create Display");
    let mut ui = UiBuilder::new([f64::from(WIDTH), f64::from(HEIGHT)]).build();

    let font = Font::from_bytes(ttf_noto_sans::REGULAR).expect("failed to create font");
    let _ = ui.fonts.insert(font);

    let mut renderer = Renderer::new(&display).expect("failed to create Renderer");
    let image_map = ImageMap::<Texture2d>::new();

    let mut app = App::default();
    let mut ids = Ids::new(ui.widget_id_generator());

    // Start the loop:
    //
    // - Send available events to the `Ui`.
    // - Update the widgets via the `conrod_example_shared::gui` fn.
    // - Render the current state of the `Ui`.
    // - Repeat.
    run_loop(display, event_loop, move |request, display| {
        match request {
            Request::Event {
                event,
                should_update_ui,
                should_exit,
            } => {
                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = convert_event(&event, &display.gl_window().window()) {
                    ui.handle_event(event);
                    *should_update_ui = true;
                }

                match event {
                    glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                        // Break from the loop upon `Escape`.
                        glium::glutin::event::WindowEvent::CloseRequested
                        | glium::glutin::event::WindowEvent::KeyboardInput {
                            input:
                                glium::glutin::event::KeyboardInput {
                                    virtual_keycode:
                                        Some(glium::glutin::event::VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *should_exit = true,
                        _ => {}
                    },
                    _ => {}
                }
            }
            Request::SetUi { needs_redraw } => {
                // Instantiate a GUI demonstrating every widget type provided by conrod.
                let ui = &mut ui.set_widgets();
                view::set_widgets(ui, &mut ids, &mut app);

                *needs_redraw = ui.has_changed();
            }
            Request::Redraw => {
                // Render the `Ui` and then display it on the screen.
                let primitives = ui.draw();

                renderer.fill(display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    })
}

#[derive(Debug)]
pub enum Request<'a, 'b: 'a> {
    Event {
        event: &'a event::Event<'b, ()>,
        should_update_ui: &'a mut bool,
        should_exit: &'a mut bool,
    },
    SetUi {
        needs_redraw: &'a mut bool,
    },
    Redraw,
}

/// In most of the examples the `glutin` crate is used for providing the window context and
/// events while the `glium` crate is used for displaying `conrod_core::render::Primitives` to the
/// screen.
///
/// This function simplifies some of the boilerplate involved in limiting the redraw rate in the
/// glutin+glium event loop.
pub fn run_loop<F>(display: Display, event_loop: event_loop::EventLoop<()>, mut callback: F) -> !
where
    F: 'static + FnMut(Request, &Display),
{
    let sixteen_ms = std::time::Duration::from_millis(16);
    let mut next_update = None;
    let mut ui_update_needed = false;
    event_loop.run(move |event, _, control_flow| {
        {
            let mut should_update_ui = false;
            let mut should_exit = false;
            callback(
                Request::Event {
                    event: &event,
                    should_update_ui: &mut should_update_ui,
                    should_exit: &mut should_exit,
                },
                &display,
            );
            ui_update_needed |= should_update_ui;
            if should_exit {
                *control_flow = event_loop::ControlFlow::Exit;
                return;
            }
        }

        // We don't want to draw any faster than 60 FPS, so set the UI only on every 16ms, unless:
        // - this is the very first event, or
        // - we didn't request update on the last event and new events have arrived since then.
        let should_set_ui_on_main_events_cleared = next_update.is_none() && ui_update_needed;
        match (&event, should_set_ui_on_main_events_cleared) {
            (event::Event::NewEvents(event::StartCause::Init { .. }), _)
            | (event::Event::NewEvents(event::StartCause::ResumeTimeReached { .. }), _)
            | (event::Event::MainEventsCleared, true) => {
                next_update = Some(std::time::Instant::now() + sixteen_ms);
                ui_update_needed = false;

                let mut needs_redraw = false;
                callback(
                    Request::SetUi {
                        needs_redraw: &mut needs_redraw,
                    },
                    &display,
                );
                if needs_redraw {
                    display.gl_window().window().request_redraw();
                } else {
                    // We don't need to redraw anymore until more events arrives.
                    next_update = None;
                }
            }
            _ => {}
        }
        if let Some(next_update) = next_update {
            *control_flow = event_loop::ControlFlow::WaitUntil(next_update);
        } else {
            *control_flow = event_loop::ControlFlow::Wait;
        }

        // Request redraw if needed.
        match &event {
            event::Event::RedrawRequested(_) => {
                callback(Request::Redraw, &display);
            }
            _ => {}
        }
    })
}

conrod_winit::v023_conversion_fns!();
