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
        self.window = Some(window);
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        self.input.step();
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _wid: WindowId, event: WindowEvent) {
        self.input.process_window_event(&event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let window = match &mut self.window {
            Some(canvas) => canvas,
            None => return,
        };

        if self.input.close_requested() {
            event_loop.exit();
        }

        if self.input.key_pressed(KeyCode::Escape) {
            event_loop.exit();
        }

        window.request_redraw();

        self.input.end_step();
    }
}
