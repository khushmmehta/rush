mod renderer;

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::{Window, WindowId},
};
use winit_input_helper::WinitInputHelper;

use crate::app::renderer::Engine;

pub struct App {
    input: WinitInputHelper,
    window: Option<Arc<Window>>,
    engine: Option<Engine>,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: WinitInputHelper::new(),
            window: None,
            engine: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.engine = Some(pollster::block_on(Engine::new(window.clone())).unwrap());

        window.request_redraw();
        self.window = Some(window);
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        self.input.step();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _wid: WindowId, event: WindowEvent) {
        let engine = match &mut self.engine {
            Some(canvas) => canvas,
            None => return,
        };

        if let WindowEvent::Resized(size) = event {
            engine.resize(size);
        }

        if self.input.process_window_event(&event) {
            match engine.render() {
                Ok(_) => {}
                Err(e) => {
                    log::error!("{e}");
                    event_loop.exit();
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.input.close_requested() {
            event_loop.exit();
        }

        if self.input.key_pressed(KeyCode::Escape) {
            event_loop.exit();
        }

        self.input.end_step();
        if let Some(window) = &self.window {
            window.request_redraw();
        }

        println!(
            "{:.3}",
            self.input.delta_time().unwrap().as_secs_f32() * 1000.0
        );
    }
}
