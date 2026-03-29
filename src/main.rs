mod graphics;
use crate::graphics::{Engine, Window};


fn main() {
    let mut window = Window::new();
    let engine = Engine::new(&mut window);
    println!("Hello, world!");

    window.run();
}
