mod parser;

use std::{env, fs};
use std::time::{Duration, Instant};
use minifb::{Window, WindowOptions};
use resvg::{tiny_skia, usvg};
use crate::parser::{parse};

fn main() {
    let args: Vec<String> = env::args().collect();
    let content = fs::read_to_string(&args[1]).unwrap();

    let ewvx_data = parse(&content);

    let first_tree = usvg::Tree::from_data(
        ewvx_data.frames[0].as_bytes(),
        &usvg::Options::default(),
    ).unwrap();
    let width = first_tree.size().width() as usize;
    let height = first_tree.size().height() as usize;

    let frame_duration = Duration::from_secs_f32(1.0 / ewvx_data.meta.fps);
    let mut window = Window::new("EWVX Player", width, height, WindowOptions::default()).unwrap();

    for svg_str in ewvx_data.frames {
        if !window.is_open() { break; }

        let start = Instant::now();

        let tree = usvg::Tree::from_data(
            svg_str.as_bytes(),
            &usvg::Options::default()
        ).unwrap();

        let size = tree.size();
        let mut pixmap = tiny_skia::Pixmap::new(
            size.width() as u32,
            size.height() as u32
        ).unwrap();

        resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

        // ??? RGBA u8 -> 0RGB u32(Bruder ich hab keine Ahnung)
        let buffer: Vec<u32> = pixmap.data().chunks(4)
            .map(|c| {
            ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32)
        }).collect();

        window.update_with_buffer(&buffer, size.width() as usize, size.height() as usize).unwrap();

        let elapsed = start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
}

