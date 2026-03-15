use anyhow::{Context, Result, bail};
use ewvx::parser::parse;
use minifb::{Window, WindowOptions};
use resvg::{tiny_skia, usvg};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{env, fs, thread};

const LOOKAHEAD: usize = 16;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        bail!("Usage: {} <input.ewvx>", args[0]);
    }

    let content = fs::read_to_string(&args[1])
        .with_context(|| format!("Failed to read ewvx file: {}", args[1]))?;

    let ewvx_data = parse(&content).context("Failed to parse ewvx data")?;

    let width = ewvx_data.meta.width as usize;
    let height = ewvx_data.meta.height as usize;

    let frame_duration = Duration::from_secs_f64(1.0 / ewvx_data.meta.fps);

    let (sync_sender, receiver) = mpsc::sync_channel::<Result<Vec<u32>>>(LOOKAHEAD);
    let frames = ewvx_data.frames;

    let render_handle = thread::spawn(move || {
        let mut pixmap =
            tiny_skia::Pixmap::new(width as u32, height as u32).expect("Failed to create pixmap");

        for frame in &frames {
            let result = render_frame(&frame.svg, frame.index, &mut pixmap);
            if sync_sender.send(result).is_err() {
                break;
            }
        }
    });

    let mut window = Window::new("EWVX Player", width, height, WindowOptions::default())
        .context("Failed to create window")?;

    for buffer in receiver {
        if !window.is_open() {
            break;
        }

        let start = Instant::now();
        let buffer = buffer?;

        window
            .update_with_buffer(&buffer, width, height)
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
    let tree = usvg::Tree::from_data(svg_str.as_bytes(), &usvg::Options::default())
        .with_context(|| format!("Failed to parse SVG for frame {}", index))?;

    pixmap.fill(tiny_skia::Color::TRANSPARENT);
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let buffer: Vec<u32> = pixmap
        .data()
        .chunks(4)
        .map(|c| ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32))
        .collect();

    Ok(buffer)
}
