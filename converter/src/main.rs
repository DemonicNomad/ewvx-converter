mod decode;
mod trace;

use std::io::BufWriter;
use std::thread;
use anyhow::{bail, Context, Result};
use ewvx::types::EwvxMeta;
use ewvx::writer;

fn main() -> Result<()> {
    ffmpeg_next::init().context("Failed to initialize ffmpeg")?;

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        bail!("Usage: {} <input video> <output ewvx>", args[0]);
    }

    let input = &args[1];
    let output = &args[2];

    run(input, output)
}

fn run(input: &str, output: &str) -> Result<()> {
    if std::fs::metadata(input).is_err() {
        bail!("Input not found: {}", input);
    }

    let fps = decode::get_fps(input)?;
    let frames = decode::decode_frames(input)?;

    let (width, height) = frames.first()
        .map(|f| (f.width, f.height))
        .context("No frames decoded from input")?;

    let meta = EwvxMeta {
        title: None,
        author: None,
        created: None,
        description: None,
        fps,
        width,
        height,
        frame_count: frames.len() as u32,
        duration: frames.len() as f64 / fps,
        ente: false,
    };

    let file = std::fs::File::create(output)
        .with_context(|| format!("Failed to create output: {}", output))?;
    let mut w = BufWriter::new(file);

    writer::write_header(&mut w, &meta)?;

    for chunk in frames.into_iter().collect::<Vec<_>>().chunks_mut(8) {
        let frame_chunk: Vec<_> = chunk.iter_mut().map(|f| std::mem::take(f)).collect();

        let chunk_processed: Vec<_> = frame_chunk
            .into_iter()
            .map(|frame| {
                thread::spawn(move || {
                    let index = frame.index;
                    let svg = trace::trace_frame(frame);
                    (index, svg)
                })
            })
            .collect();

        for handle in chunk_processed {
            let (index, svg) = handle.join()
                .map_err(|_| anyhow::anyhow!("Thread panicked while tracing frame"))?;
            let svg = svg.with_context(|| format!("Failed to trace frame {}", index))?;
            writer::write_frame(&mut w, index, &svg)
                .with_context(|| format!("Failed to write frame {}", index))?;
            println!("Sehr effizientes Tracen von Frame {}", index);
        }
    }

    writer::write_footer(&mut w)?;
    Ok(())
}
