use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    keyboard::KeyCode, window::WindowId,
};

pub struct App {
    input: winit_input_helper::WinitInputHelper,
    window: Option<winit::window::Window>,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: winit_input_helper::WinitInputHelper::new(),
            window: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = winit::window::Window::default_attributes();
        let window = event_loop.create_window(window_attributes).unwrap();
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
