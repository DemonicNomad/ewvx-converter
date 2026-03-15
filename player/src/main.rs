use anyhow::{Context, Result, bail};
use ewvx::reader;
use minifb::{Window, WindowOptions};
use rayon::prelude::*;
use resvg::{tiny_skia, usvg};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{env, fs, thread};

const PRERENDER_FRAMES: usize = 32;
const RENDER_BATCH: usize = 8;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        bail!("Usage: {} <input.ewvx>", args[0]);
    }

    let content = fs::read_to_string(&args[1])
        .with_context(|| format!("Failed to read ewvx file: {}", args[1]))?;

    let ewvx = reader::read(&content).context("Failed to parse ewvx data")?;

    let width = ewvx.meta.width as usize;
    let height = ewvx.meta.height as usize;
    let frame_duration = Duration::from_secs_f64(1.0 / ewvx.meta.fps);

    let (sender, receiver) =
        mpsc::sync_channel::<Result<Vec<u32>>>(PRERENDER_FRAMES);
    let frames = ewvx.frames;

    let render_handle = thread::spawn(move || {
        for chunk in frames.chunks(RENDER_BATCH) {
            let rendered: Vec<Result<Vec<u32>>> = chunk
                .par_iter()
                .map(|frame| {
                    let mut pixmap = tiny_skia::Pixmap::new(width as u32, height as u32)
                            .expect("Failed to create pixmap");
                    render_frame(&frame.svg, frame.index, &mut pixmap)
                })
                .collect();

            for result in rendered {
                if sender.send(result).is_err() {
                    return;
                }
            }
        }
    });

    let mut window = Window::new("EWVX Player", width, height, WindowOptions::default())
        .context("Failed to create window")?;

    let playback_start = Instant::now();
    let mut frame_num: u64 = 0;

    for buffer in receiver {
        if !window.is_open() {
            break;
        }

        let buffer = buffer?;

        window
            .update_with_buffer(&buffer, width, height)
            .context("Failed to update window buffer")?;

        frame_num += 1;
        let target = playback_start + frame_duration * frame_num as u32;
        let now = Instant::now();
        if now < target {
            thread::sleep(target - now);
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

    //??? u8 -> u32 I guess
    let buffer: Vec<u32> = pixmap
        .data()
        .chunks(4)
        .map(|c| ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32))
        .collect();

    Ok(buffer)
}
