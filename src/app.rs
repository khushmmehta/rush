mod components;
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
    dt: f32,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: WinitInputHelper::new(),
            window: None,
            engine: None,
            dt: 0.0,
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

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.input.process_device_event(&event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _wid: WindowId, event: WindowEvent) {
        let engine = match &mut self.engine {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::Resized(size) => engine.resize(size),
            WindowEvent::RedrawRequested => {
                engine.update(self.dt);
                match engine.render() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("{e}");
                        event_loop.exit();
                    }
                }
            }
            _ => {}
        }

        self.input.process_window_event(&event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let engine = match &mut self.engine {
            Some(canvas) => canvas,
            None => return,
        };

        if self.input.close_requested() {
            event_loop.exit();
        }

        if self.input.mouse_held(winit::event::MouseButton::Right) {
            engine
                .camera_controller
                .handle_mouse(self.input.mouse_diff());
        }

        engine.camera_controller.process_keyboard(&self.input);
        if self.input.key_pressed(KeyCode::Escape) {
            event_loop.exit();
        }

        self.input.end_step();
        self.dt = self.input.delta_time().unwrap_or_default().as_secs_f32();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
