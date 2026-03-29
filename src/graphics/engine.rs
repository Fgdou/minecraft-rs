use std::sync::{Arc, Mutex};

use winit::window::Window;

pub struct Engine {
    context: Arc<Mutex<Option<RenderContext>>>,
}
pub struct RenderContext {
    window: Arc<Window>,
}

impl Engine {
    pub fn new(window: &mut crate::Window) -> Self {
        let context = Arc::new(Mutex::new(None::<RenderContext>));

        {
            let context = context.clone();
            window.set_resize_callback(Box::new(move |window| {
                let mut context_option = context.lock().unwrap();
                let old_context = context_option.take();
                *context_option = Some(resize(old_context, window));
            }));
        }

        {
            let context = context.clone();
            window.set_redraw_callback(Box::new(move || {
                if let Some(ctx) = context.lock().unwrap().as_ref() {
                    ctx.draw();
                }
            }));
        }

        Self {
            context,
        }
    }
}

fn resize(old_context: Option<RenderContext>, window: Arc<winit::window::Window>) -> RenderContext {
    println!("resize");
    RenderContext { 
        window
    }
}

impl RenderContext {
    fn draw(&self) {
        println!("draw")
    }
}