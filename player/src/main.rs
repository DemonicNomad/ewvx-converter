use anyhow::{Context, Result, bail};
use ewvx::reader;
use ewvx::types::EwvxTrack;
use minifb::{Window, WindowOptions};
use rayon::prelude::*;
use resvg::{tiny_skia, usvg};
use rodio::{Player, buffer::SamplesBuffer, stream::DeviceSinkBuilder};
use std::num::NonZero;
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

    let device_sink = DeviceSinkBuilder::open_default_sink()
        .context("Failed to open audio output")?;
    let player = Player::connect_new(device_sink.mixer());

    if !ewvx.audio.is_empty() {
        let source = build_audio_source(&ewvx.audio[0])?;
        player.append(source);
        player.pause();
    }

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

    player.play();
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

    player.sleep_until_end();

    let _ = render_handle.join();
    Ok(())
}

fn build_audio_source(track: &EwvxTrack) -> Result<SamplesBuffer> {
    let info = &track.info;

    let total_interleaved =
        info.total_samples as usize * info.channels as usize;
    let mut pcm = Vec::<f32>::with_capacity(total_interleaved);

    for segment in &track.segments {
        for &sample in &segment.samples {
            pcm.push(sample as f32 / 32768.0);
        }
    }

    let channels = NonZero::new(info.channels)
        .context("channels must be > 0")?;
    let sample_rate = NonZero::new(info.sample_rate)
        .context("sample_rate must be > 0")?;

    Ok(SamplesBuffer::new(channels, sample_rate, pcm))
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
