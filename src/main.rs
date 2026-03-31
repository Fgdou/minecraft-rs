mod render;

use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop};

use crate::render::RenderContext;

struct App {
    render_context: Option<RenderContext>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.render_context = Some(RenderContext::new(event_loop));
    }

    fn about_to_wait(&mut self, _: &winit::event_loop::ActiveEventLoop) {
        if let Some(context) = &self.render_context {
            context.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(_) => {
                if let Some(context) = &mut self.render_context {
                    context.resize();
                }
            },
            WindowEvent::RedrawRequested => {
                if let Some(context) = &self.render_context {
                    context.draw();
                }
            },
            _ => {}
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            render_context: None,
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
