use anyhow::{Context, Result, bail};
use crate::types::{EwvxData, EwvxFrame, EwvxMeta, EwvxTrack, EwvxTrackInfo, EwvxSegment};

/// Parses an EWVX v2.0 XML string into an [`EwvxData`].
///
/// Validates the `version="2.0"` attribute on the root `<video>` element,
/// then extracts metadata, frames, and optional audio tracks.
pub fn parse(input: &str) -> Result<EwvxData> {
    validate_version(input)?;

    let meta_start = input.find("<meta-ente>")
        .context("Missing <meta-ente> tag")?;
    let meta_end = input.find("</meta-ente>")
        .context("Missing </meta-ente> closing tag")?;
    let frames_start = input.find("<frames>")
        .context("Missing <frames> tag")?;

    let meta = parse_meta_ente(&input[meta_start..meta_end])?;
    let frames = parse_frames(&input[frames_start..])?;
    let audio = match input.find("<audio>") {
        Some(audio_start) => {
            let audio_end = input.find("</audio>")
                .context("Missing </audio> closing tag")?;
            parse_audio(&input[audio_start..audio_end])?
        }
        None => Vec::new(),
    };

    Ok(EwvxData { meta, frames, audio })
}

fn validate_version(input: &str) -> Result<()> {
    let video_tag = input.find("<video")
        .context("Missing <video> root element")?;
    let tag_end = input[video_tag..].find('>')
        .context("Malformed <video> tag")?;
    let tag = &input[video_tag..video_tag + tag_end];

    if let Some(pos) = tag.find("version=\"") {
        let version_start = pos + 9;
        let version_end = tag[version_start..].find('"')
            .context("Malformed version attribute")?;
        let version = &tag[version_start..version_start + version_end];
        if version != "2.0" {
            bail!("Unsupported EWVX version: {} (expected 2.0)", version);
        }
    } else {
        bail!("Missing version attribute on <video> element");
    }

    Ok(())
}

fn parse_meta_ente(meta_input: &str) -> Result<EwvxMeta> {
    let fps = parse_required_tag::<f64>(meta_input, "fps")
        .context("Failed to parse fps")?;
    let width = parse_required_tag::<u32>(meta_input, "width")
        .context("Failed to parse width")?;
    let height = parse_required_tag::<u32>(meta_input, "height")
        .context("Failed to parse height")?;
    let frame_count = parse_required_tag::<u32>(meta_input, "frame-count")
        .context("Failed to parse frame-count")?;
    let duration = parse_required_tag::<f64>(meta_input, "duration")
        .context("Failed to parse duration")?;
    let ente = parse_required_tag::<bool>(meta_input, "ente")
        .context("Failed to parse ente")?;

    let title = parse_optional_tag(meta_input, "title");
    let author = parse_optional_tag(meta_input, "author");
    let created = parse_optional_tag(meta_input, "created");
    let description = parse_optional_tag(meta_input, "description");

    Ok(EwvxMeta {
        title,
        author,
        created,
        description,
        fps,
        width,
        height,
        frame_count,
        duration,
        ente,
    })
}

fn parse_required_tag<T: std::str::FromStr>(input: &str, tag: &str) -> Result<T>
where
    T::Err: std::fmt::Display,
{
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = input.find(&open)
        .with_context(|| format!("Missing <{}> tag", tag))?;
    let end = input.find(&close)
        .with_context(|| format!("Missing </{}> tag", tag))?;
    let value = input[start + open.len()..end].trim();
    value.parse::<T>()
        .map_err(|e| anyhow::anyhow!("Failed to parse <{}>: {}", tag, e))
}

fn parse_optional_tag(input: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = input.find(&open)?;
    let end = input.find(&close)?;
    Some(input[start + open.len()..end].trim().to_string())
}

fn parse_frames(frame_input: &str) -> Result<Vec<EwvxFrame>> {
    let mut frames = Vec::new();

    let parts: Vec<&str> = frame_input.split("<frame ").skip(1).collect();

    if parts.is_empty() {
        bail!("No frames found in EWVX file");
    }

    for (i, part) in parts.iter().enumerate() {
        let index = parse_frame_index(part)
            .with_context(|| format!("Failed to parse index for frame {}", i))?;

        let content_start = part.find('>')
            .with_context(|| format!("Malformed <frame> tag at frame {}", i))?;
        let content = &part[content_start + 1..];

        let end = content.find("</frame>")
            .with_context(|| format!("Missing </frame> closing tag for frame {}", i))?;

        frames.push(EwvxFrame {
            index,
            svg: content[..end].to_string(),
        });
    }

    Ok(frames)
}

fn parse_frame_index(tag_content: &str) -> Result<usize> {
    let idx_start = tag_content.find("index=\"")
        .context("Missing index attribute on <frame>")?;
    let value_start = idx_start + 7;
    let value_end = tag_content[value_start..].find('"')
        .context("Malformed index attribute on <frame>")?;
    tag_content[value_start..value_start + value_end]
        .parse::<usize>()
        .context("Failed to parse frame index as integer")
}

fn parse_audio(audio_input: &str) -> Result<Vec<EwvxTrack>> {
    let mut tracks = Vec::new();
    let parts: Vec<&str> = audio_input.split("<track ").skip(1).collect();

    for (i, part) in parts.iter().enumerate() {
        let track = parse_track(part)
            .with_context(|| format!("Failed to parse audio track {}", i))?;
        tracks.push(track);
    }

    Ok(tracks)
}

fn parse_track(part: &str) -> Result<EwvxTrack> {
    let id = parse_attribute(part, "id")?
        .parse::<u32>()
        .context("Failed to parse track id")?;
    let lang = parse_optional_attribute(part, "lang");

    let info_start = part.find("<track-info>")
        .context("Missing <track-info>")?;
    let info_end = part.find("</track-info>")
        .context("Missing </track-info>")?;
    let info_section = &part[info_start..info_end + "</track-info>".len()];
    let info = parse_track_info(info_section)?;

    let segments_start = part.find("<segments>")
        .context("Missing <segments>")?;
    let segments_end = part.find("</segments>")
        .context("Missing </segments>")?;
    let segments_section = &part[segments_start..segments_end];
    let segments = parse_segments(segments_section)?;

    Ok(EwvxTrack { id, lang, info, segments })
}

fn parse_track_info(section: &str) -> Result<EwvxTrackInfo> {
    let sample_rate = parse_required_tag::<u32>(section, "sample-rate")
        .context("Failed to parse sample-rate")?;
    let bit_depth = parse_required_tag::<u16>(section, "bit-depth")
        .context("Failed to parse bit-depth")?;
    let channels = parse_required_tag::<u16>(section, "channels")
        .context("Failed to parse channels")?;
    let total_samples = parse_required_tag::<u64>(section, "total-samples")
        .context("Failed to parse total-samples")?;

    let sample_format = parse_required_tag::<String>(section, "sample-format")
        .context("Failed to parse sample-format")?;
    let endianness = parse_optional_tag(section, "endianness")
        .unwrap_or_else(|| "little".to_string());

    Ok(EwvxTrackInfo {
        sample_rate,
        bit_depth,
        channels,
        sample_format,
        endianness,
        total_samples,
    })
}

fn parse_segments(section: &str) -> Result<Vec<EwvxSegment>> {
    let mut segments = Vec::new();
    let parts: Vec<&str> = section.split("<segment ").skip(1).collect();

    for (i, part) in parts.iter().enumerate() {
        let segment = parse_segment(part)
            .with_context(|| format!("Failed to parse segment {}", i))?;
        segments.push(segment);
    }

    Ok(segments)
}

fn parse_segment(part: &str) -> Result<EwvxSegment> {
    let index = parse_attribute(part, "index")?
        .parse::<usize>()
        .context("Failed to parse segment index")?;
    let timestamp = parse_attribute(part, "timestamp")?
        .parse::<f64>()
        .context("Failed to parse segment timestamp")?;
    let sample_offset = parse_attribute(part, "sample-offset")?
        .parse::<u64>()
        .context("Failed to parse segment sample-offset")?;
    let sample_count = parse_attribute(part, "sample-count")?
        .parse::<u32>()
        .context("Failed to parse segment sample-count")?;

    let samples_start = part.find("<samples>")
        .context("Missing <samples>")?;
    let samples_end = part.find("</samples>")
        .context("Missing </samples>")?;
    let samples_str = &part[samples_start + "<samples>".len()..samples_end];

    let samples: Vec<i32> = samples_str.split_whitespace()
        .map(|s| s.parse::<i32>())
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Failed to parse PCM sample values")?;

    Ok(EwvxSegment { index, timestamp, sample_offset, sample_count, samples })
}

fn parse_attribute(tag_content: &str, name: &str) -> Result<String> {
    let needle = format!("{}=\"", name);
    let start = tag_content.find(&needle)
        .with_context(|| format!("Missing {} attribute", name))?;
    let value_start = start + needle.len();
    let value_end = tag_content[value_start..].find('"')
        .with_context(|| format!("Malformed {} attribute", name))?;
    Ok(tag_content[value_start..value_start + value_end].to_string())
}

fn parse_optional_attribute(tag_content: &str, name: &str) -> Option<String> {
    let needle = format!("{}=\"", name);
    let start = tag_content.find(&needle)?;
    let value_start = start + needle.len();
    let value_end = tag_content[value_start..].find('"')?;
    Some(tag_content[value_start..value_start + value_end].to_string())
}
