use font::Text;
use raylib::{color::Color, ffi::Vector2, prelude::RaylibDraw};

mod font;

fn main() {
    let w = 1280;
    let h = 720;
    let rust_orange = Color::new(222, 165, 132, 255);
    let ray_white = Color::new(255, 255, 255, 255);
    let (mut rl, thread) = raylib::init().size(w, h).title("Font Demo").build();
    rl.set_target_fps(60);
    let Text(font, shader) = font::load_sdf_font(&mut rl, &thread).unwrap();
    let text_start = Vector2 { x: 20.0, y: 100.0 };
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(ray_white);
        d.draw_rectangle(w / 2 - 128, h / 2 - 128, 256, 256, rust_orange);
        d.draw_text_ex(&font, "Hello World!", text_start, 14.0, 1.0, rust_orange);
    }
    println!("Hello, world!");
}
