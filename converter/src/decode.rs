use anyhow::{Context, Result};
use ffmpeg_next::format::{Pixel, input};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context as ScalingContext, flag::Flags};
use ffmpeg_next::util::frame::video::Video;

#[derive(Default)]
pub struct FrameData {
    pub index: usize,
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn get_fps(path: &str) -> Result<f64> {
    let input_context = input(&path)
        .with_context(|| format!("Failed to open input file: {}", path))?;
    let stream = input_context
        .streams()
        .best(Type::Video)
        .with_context(|| format!("No video stream found in: {}", path))?;

    let avg = stream.avg_frame_rate();

    let fps = avg.numerator() as f64 / avg.denominator() as f64;

    Ok(fps)
}

pub fn decode_frames(path: &str) -> Result<Vec<FrameData>> {
    let mut input_context = input(&path)
        .with_context(|| format!("Failed to open input file: {}", path))?;

    let video_stream_index = {
        let stream = input_context
            .streams()
            .best(Type::Video)
            .with_context(|| format!("No video stream found in: {}", path))?;
        stream.index()
    };

    let mut decoder = {
        let stream = input_context
            .stream(video_stream_index)
            .context("Failed to get video stream by index")?;
        let context =
            ffmpeg_next::codec::context::Context::from_parameters(stream.parameters())
                .context("Failed to create codec context")?;
        context.decoder().video()
            .context("Failed to create video decoder")?
    };

    let width = decoder.width();
    let height = decoder.height();

    let mut scaler = ScalingContext::get(
        decoder.format(),
        width,
        height,
        Pixel::RGBA,
        width,
        height,
        Flags::BICUBIC,
    )
    .context("Failed to create scaling context")?;

    let mut frames = Vec::new();
    let mut frame_index = 0usize;
    let mut decoded_frame = Video::empty();
    let mut rgb_frame = Video::empty();

    for (stream, packet) in input_context.packets() {
        if stream.index() != video_stream_index {
            continue;
        }

        decoder.send_packet(&packet)
            .with_context(|| format!("Failed to send packet for frame {}", frame_index))?;
        while decoder.receive_frame(&mut decoded_frame).is_ok() {
            scaler.run(&decoded_frame, &mut rgb_frame)
                .with_context(|| format!("Failed to scale frame {}", frame_index))?;

            let rgba = stride_ka(rgb_frame.data(0), width, height, rgb_frame.stride(0));
            frames.push(FrameData {
                index: frame_index,
                rgba,
                width,
                height,
            });

            frame_index += 1;
        }
    }

    Ok(frames)
}

fn stride_ka(data: &[u8], width: u32, height: u32, stride: usize) -> Vec<u8> {
    let row_bytes = width as usize * 4;
    if stride == row_bytes {
        return data[..row_bytes * height as usize].to_vec();
    }
    let mut out = Vec::with_capacity(row_bytes * height as usize);
    for y in 0..height as usize {
        let start = y * stride;
        out.extend_from_slice(&data[start..start + row_bytes]);
    }
    out
}
