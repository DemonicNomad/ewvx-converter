mod decode;
mod trace;

use anyhow::{Context, Result, bail};
use ewvx::writer::EwvxWriter;
use std::io::BufWriter;
use std::thread;

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

    let decode::DecodedVideo {
        meta,
        frames,
        audio_tracks,
    } = decode::decode_all(input)?;

    if frames.is_empty() {
        bail!("No frames decoded from input");
    }

    let file = std::fs::File::create(output)
        .with_context(|| format!("Failed to create output: {}", output))?;
    let buf = BufWriter::new(file);

    let mut writer = EwvxWriter::new(buf, &meta)
        .context("Failed to write EWVX header")?;

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
            let (index, svg) = handle
                .join()
                .map_err(|_| anyhow::anyhow!("Thread panicked while tracing frame"))?;
            let svg = svg.with_context(|| format!("Failed to trace frame {}", index))?;
            writer.write_frame(index, &svg)
                .with_context(|| format!("Failed to write frame {}", index))?;
        }
    }

    let mut writer = writer.end_frames().context("Failed to close frames")?;

    if !audio_tracks.is_empty() {
        writer
            .write_audio(&audio_tracks)
            .context("Failed to write audio tracks")?;
    }

    writer.finish().context("Failed to close EWVX document")?;
    Ok(())
}
