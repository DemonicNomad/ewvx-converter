use anyhow::{Context, Result};
use ewvx::types::{EwvxMeta, EwvxSegment, EwvxTrack, EwvxTrackInfo};
use ffmpeg_next::format::sample::Type as SampleType;
use ffmpeg_next::format::{input, Pixel, Sample};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context as ScalingContext, flag::Flags};
use ffmpeg_next::util::frame::audio::Audio as AudioFrame;
use ffmpeg_next::util::frame::video::Video;

pub struct DecodedVideo {
    pub meta: EwvxMeta,
    pub frames: Vec<FrameData>,
    pub audio_tracks: Vec<EwvxTrack>,
}

#[derive(Default)]
pub struct FrameData {
    pub index: usize,
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

struct AudioDecState {
    stream_index: usize,
    decoder: ffmpeg_next::decoder::Audio,
    resampler: ffmpeg_next::software::resampling::context::Context,
    time_base: f64,
    lang: Option<String>,
    channels: u16,
    sample_rate: u32,
    segments: Vec<EwvxSegment>,
    seg_idx: usize,
    sample_offset: u64,
}

pub fn decode_all(path: &str) -> Result<DecodedVideo> {
    let mut input_context =
        input(path).with_context(|| format!("Failed to open input file: {path}"))?;

    let title = input_context
        .metadata()
        .get("title")
        .map(|s| s.to_string());
    let author = input_context
        .metadata()
        .get("artist")
        .map(|s| s.to_string())
        .or_else(|| input_context.metadata().get("author").map(|s| s.to_string()));
    let created = input_context
        .metadata()
        .get("creation_time")
        .map(|s| s.to_string())
        .or_else(|| input_context.metadata().get("date").map(|s| s.to_string()));
    let description = input_context
        .metadata()
        .get("comment")
        .map(|s| s.to_string())
        .or_else(|| {
            input_context.metadata()
                .get("description")
                .map(|s| s.to_string())
        });


    let video_stream_index = input_context
        .streams()
        .best(Type::Video)
        .context("No video stream found in input")?
        .index();

    let fps = {
        let avg = input_context.stream(video_stream_index).unwrap().avg_frame_rate();
        avg.numerator() as f64 / avg.denominator() as f64
    };

    let mut decoded_video = {
        let s = input_context.stream(video_stream_index).unwrap();
        ffmpeg_next::codec::context::Context::from_parameters(s.parameters())
            .context("Failed to create video codec context")?
            .decoder()
            .video()
            .context("Failed to create video decoder")?
    };

    let width = decoded_video.width();
    let height = decoded_video.height();

    let mut scaler = ScalingContext::get(
        decoded_video.format(),
        width,
        height,
        Pixel::RGBA,
        width,
        height,
        Flags::BICUBIC,
    )
    .context("Failed to create scaling context")?;


    let audio_info: Vec<(usize, Option<String>, f64)> = input_context
        .streams()
        .filter(|s| s.parameters().medium() == Type::Audio)
        .map(|s| {
            let lang = s.metadata().get("language").map(|v| v.to_string());
            let tb = s.time_base();
            let time_base = tb.numerator() as f64 / tb.denominator() as f64;
            (s.index(), lang, time_base)
        })
        .collect();

    let mut audio_states: Vec<AudioDecState> = Vec::new();

    for (index, language, time_base) in audio_info {
        let audio_stream = input_context.stream(index).unwrap();
        let decoded_audio =
            ffmpeg_next::codec::context::Context::from_parameters(audio_stream.parameters())
            .context("Failed to create audio codec context")?
            .decoder()
            .audio()
            .context("Failed to create audio decoder")?;

        let channels = decoded_audio.channels();
        let rate = decoded_audio.rate();
        let layout = decoded_audio.channel_layout();

        let resampler = ffmpeg_next::software::resampling::context::Context::get(
            decoded_audio.format(),
            layout,
            rate,
            Sample::I16(SampleType::Packed),
            layout,
            rate,
        )
        .context("Failed to create audio resampler")?;

        audio_states.push(AudioDecState {
            stream_index: index,
            decoder: decoded_audio,
            resampler,
            time_base,
            lang: language,
            channels,
            sample_rate: rate,
            segments: Vec::new(),
            seg_idx: 0,
            sample_offset: 0,
        });
    }


    let mut frames = Vec::new();
    let mut frame_index = 0usize;
    let mut raw_frame = Video::empty();
    let mut scaled = Video::empty();

    for (stream, packet) in input_context.packets() {
        let stream_index = stream.index();

        if stream_index == video_stream_index {
            decoded_video
                .send_packet(&packet)
                .with_context(|| format!("Failed to send video packet {frame_index}"))?;
            drain_video(
                &mut decoded_video,
                &mut scaler,
                &mut raw_frame,
                &mut scaled,
                &mut frames,
                &mut frame_index,
                width,
                height,
            )?;
        }

        if let Some(state) = audio_states.iter_mut().find(|s| s.stream_index == stream_index) {
            state.decoder.send_packet(&packet)?;
            drain_audio(state)?;
        }
    }

    let _ = decoded_video.send_eof();
    drain_video(
        &mut decoded_video,
        &mut scaler,
        &mut raw_frame,
        &mut scaled,
        &mut frames,
        &mut frame_index,
        width,
        height,
    )?;

    for state in &mut audio_states {
        let _ = state.decoder.send_eof();
        drain_audio(state)?;
    }


    let audio_tracks = audio_states
        .into_iter()
        .enumerate()
        .map(|(id, state)| EwvxTrack {
            id: id as u32,
            lang: state.lang,
            info: EwvxTrackInfo {
                sample_rate: state.sample_rate,
                bit_depth: 16,
                channels: state.channels,
                sample_format: "int".to_string(),
                endianness: if cfg!(target_endian = "little") {
                    "little"
                } else {
                    "big"
                }
                .to_string(),
                total_samples: state.sample_offset,
            },
            segments: state.segments,
        })
        .collect();

    Ok(DecodedVideo {
        meta: EwvxMeta {
            title,
            author,
            created,
            description,
            fps,
            width,
            height,
            frame_count: frames.len() as u32,
            duration: frames.len() as f64 / fps,
            ente: false,
        },
        frames,
        audio_tracks,
    })
}

fn drain_video(
    decoded_video: &mut ffmpeg_next::decoder::Video,
    scaler: &mut ScalingContext,
    raw: &mut Video,
    scaled: &mut Video,
    out: &mut Vec<FrameData>,
    index: &mut usize,
    width: u32,
    height: u32,
) -> Result<()> {
    while decoded_video.receive_frame(raw).is_ok() {
        scaler
            .run(raw, scaled)
            .with_context(|| format!("Failed to scale frame {index}"))?;
        let rgba = stride_ka(scaled.data(0), width, height, scaled.stride(0));
        out.push(FrameData {
            index: *index,
            rgba,
            width,
            height,
        });
        *index += 1;
    }
    Ok(())
}

fn drain_audio(state: &mut AudioDecState) -> Result<()> {
    let mut decoded = AudioFrame::empty();
    while state.decoder.receive_frame(&mut decoded).is_ok() {
        let mut resampled = AudioFrame::empty();
        state
            .resampler
            .run(&decoded, &mut resampled)
            .context("Failed to resample audio frame")?;

        let nb = resampled.samples();
        if nb == 0 {
            continue;
        }

        let channels = state.channels as usize;
        let byte_len = nb * channels * 2; // 2 bytes per i16 sample
        let raw = &resampled.data(0)[..byte_len];
        let samples: Vec<i32> = raw
            .chunks_exact(2)
            .map(|c| i16::from_ne_bytes([c[0], c[1]]) as i32)
            .collect();

        let timestamp = decoded
            .timestamp()
            .map(|pts| pts as f64 * state.time_base)
            .unwrap_or(state.sample_offset as f64 / state.sample_rate as f64);

        state.segments.push(EwvxSegment {
            index: state.seg_idx,
            timestamp,
            sample_offset: state.sample_offset,
            sample_count: nb as u32,
            samples,
        });

        state.seg_idx += 1;
        state.sample_offset += nb as u64;
    }
    Ok(())
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
