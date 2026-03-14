mod parser;

use std::{env, fs, thread};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use anyhow::{Context, Result, bail};
use minifb::{Window, WindowOptions};
use resvg::{tiny_skia, usvg};
use crate::parser::parse;

const PRERENDER_FRAMES: usize = 16;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        bail!("Usage: {} <input.ewvx>", args[0]);
    }

    let content = fs::read_to_string(&args[1])
        .with_context(|| format!("Failed to read ewvx file: {}", args[1]))?;

    let ewvx_data = parse(&content)
        .context("Failed to parse ewvx data")?;

    let first_tree = usvg::Tree::from_data(
        ewvx_data.frames[0].as_bytes(),
        &usvg::Options::default(),
    ).context("Failed to parse first SVG frame")?;
    let width = first_tree.size().width() as usize;
    let height = first_tree.size().height() as usize;

    let frame_duration = Duration::from_secs_f32(1.0 / ewvx_data.meta.fps);

    let (sync_sender, receiver) = mpsc::sync_channel::<Result<Vec<u32>>>(PRERENDER_FRAMES);
    let frames = ewvx_data.frames;

    let render_handle = thread::spawn(move || {
        let mut pixmap = tiny_skia::Pixmap::new(width as u32, height as u32)
            .expect("Failed to create pixmap");

        for (i, svg_str) in frames.iter().enumerate() {
            let result = render_frame(svg_str, i, &mut pixmap);
            if sync_sender.send(result).is_err() {
                break;
            }
        }
    });

    let mut window = Window::new("EWVX Player", width, height, WindowOptions::default())
        .context("Failed to create window")?;

    for buffer in receiver {
        if !window.is_open() { break; }

        let start = Instant::now();
        let buffer = buffer?;

        window.update_with_buffer(&buffer, width, height)
            .context("Failed to update window buffer")?;

        let elapsed = start.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }

    let _ = render_handle.join();
    Ok(())
}

fn render_frame(svg_str: &str, index: usize, pixmap: &mut tiny_skia::Pixmap) -> Result<Vec<u32>> {
    let tree = usvg::Tree::from_data(
        svg_str.as_bytes(),
        &usvg::Options::default(),
    ).with_context(|| format!("Failed to parse SVG for frame {}", index))?;

    pixmap.fill(tiny_skia::Color::TRANSPARENT);
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // RGBA u8 -> 0RGB u32
    let buffer: Vec<u32> = pixmap.data().chunks(4)
        .map(|c| ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32))
        .collect();

    Ok(buffer)
}
