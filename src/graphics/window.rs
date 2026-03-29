use std::sync::Arc;

use vulkano::sync::event::Event;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop};

pub struct Window {
    window: Option<Arc<winit::window::Window>>,
    resized: bool,
    redraw_callback: Option<Box<dyn Fn()>>,
    resize_callback: Option<Box<dyn Fn(Arc<winit::window::Window>)>>,
    event_loop: Option<EventLoop<()>>
}

impl Window {
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();

        Self {
            window: None,
            resized: false,
            redraw_callback: None,
            resize_callback: None,
            event_loop: Some(event_loop),
        }
    }

    pub fn get_window(&self) -> Option<Arc<winit::window::Window>> {
        self.window.clone()
    }

    pub fn set_redraw_callback(&mut self, f: Box<dyn Fn()>) {
        self.redraw_callback = Some(f);
    }
    pub fn set_resize_callback(&mut self, f: Box<dyn Fn(Arc<winit::window::Window>)>) {
        self.resize_callback = Some(f);
    }
    pub fn get_eventloop(&self) -> Option<&EventLoop<()>> {
        self.event_loop.as_ref()
    }
    
    pub fn run(&mut self) {
        self.event_loop.take().unwrap().run_app(self).unwrap();
    }
}

impl ApplicationHandler for Window {
    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(Arc::new(event_loop.create_window(winit::window::Window::default_attributes()).unwrap()));
        self.resized = true;
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(_) => {
                self.resized = true;
            },
            WindowEvent::RedrawRequested => {
                if self.resized && let Some(callback) = &self.resize_callback && let Some(window) = &self.window {
                    self.resized = false;
                    callback(window.clone());
                }
                if let Some(callback) = &self.redraw_callback {
                    callback();
                }
            },
            _ => ()
        }
    }
}