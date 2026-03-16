use anyhow::{Context, Result, bail};
use ewvx::reader;
use minifb::{Key, Window, WindowOptions};
use resvg::{tiny_skia, usvg};
use std::{env, fs};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        bail!("Usage: {} <input.ewvx> <frame index>", args[0]);
    }

    let input_path = &args[1];
    let frame_index: usize = args[2]
        .parse()
        .with_context(|| format!("Invalid frame index: {}", args[2]))?;

    let content = fs::read_to_string(input_path)
        .with_context(|| format!("Failed to read ewvx file: {input_path}"))?;

    let ewvx = reader::read(&content).context("Failed to parse ewvx data")?;

    let width = ewvx.meta.width as usize;
    let height = ewvx.meta.height as usize;

    let frame = ewvx
        .frames
        .get(frame_index)
        .with_context(|| {
            format!(
                "Frame index {frame_index} out of range (file has {} frames)",
                ewvx.frames.len()
            )
        })?;

    let buffer = render_frame(&frame.svg, frame.index, width, height)?;

    let title = format!("EWVX Extract — frame {frame_index}");
    let mut window = Window::new(&title, width, height, WindowOptions::default())
        .context("Failed to create window")?;

    window.set_target_fps(30);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer, width, height)
            .context("Failed to update window buffer")?;
    }

    Ok(())
}

fn render_frame(svg_str: &str, index: usize, width: usize, height: usize) -> Result<Vec<u32>> {
    let tree = usvg::Tree::from_data(svg_str.as_bytes(), &usvg::Options::default())
        .with_context(|| format!("Failed to parse SVG for frame {index}"))?;

    let mut pixmap = tiny_skia::Pixmap::new(width as u32, height as u32)
        .context("Failed to create pixmap")?;

    pixmap.fill(tiny_skia::Color::TRANSPARENT);
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let buffer: Vec<u32> = pixmap
        .data()
        .chunks(4)
        .map(|c| ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32))
        .collect();

    Ok(buffer)
}
