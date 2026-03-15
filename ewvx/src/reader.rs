use crate::types::{EwvxData, EwvxFrame, EwvxMeta, EwvxSegment, EwvxTrack, EwvxTrackInfo};
use anyhow::{Context, Result, bail};

/// Parses an EWVX v2.0 XML string into an [`EwvxData`].
///
/// Validates the `version="2.0"` attribute on the root `<video>` element,
/// then extracts metadata, frames, and optional audio tracks.
pub fn read(input: &str) -> Result<EwvxData> {
    validate_version(input)?;

    let meta_start = input.find("<meta-ente>").context("Missing <meta-ente>")?;
    let meta_end = input.find("</meta-ente>").context("Missing </meta-ente>")?;
    let meta = read_meta_ente(&input[meta_start..meta_end])?;

    let frames_start = input.find("<frames>").context("Missing <frames>")?;
    let frames = read_frames(&input[frames_start..])?;

    let audio = match input.find("<audio>") {
        Some(start) => {
            let end = input.find("</audio>").context("Missing </audio>")?;
            read_audio(&input[start..end])?
        }
        None => Vec::new(),
    };

    Ok(EwvxData {
        meta,
        frames,
        audio,
    })
}

fn validate_version(input: &str) -> Result<()> {
    let tag_start = input
        .find("<video")
        .context("Missing <video> root element")?;
    let tag_end = tag_start
        + input[tag_start..]
            .find('>')
            .context("Malformed <video> tag")?;
    let tag = &input[tag_start..tag_end];

    let version = read_attribute(tag, "version").context("Missing version attribute on <video>")?;
    if version != "2.0" {
        bail!("Unsupported EWVX version: {version} (expected 2.0)");
    }
    Ok(())
}

fn read_meta_ente(section: &str) -> Result<EwvxMeta> {
    Ok(EwvxMeta {
        title: read_optional_tag(section, "title"),
        author: read_optional_tag(section, "author"),
        created: read_optional_tag(section, "created"),
        description: read_optional_tag(section, "description"),
        fps: read_required_tag(section, "fps")?,
        width: read_required_tag(section, "width")?,
        height: read_required_tag(section, "height")?,
        frame_count: read_required_tag(section, "frame-count")?,
        duration: read_required_tag(section, "duration")?,
        ente: read_required_tag(section, "ente")?,
    })
}

fn read_frames(input: &str) -> Result<Vec<EwvxFrame>> {
    let mut frames = Vec::new();

    for (i, part) in input.split("<frame ").skip(1).enumerate() {
        let index = read_attribute(part, "index")?
            .parse::<usize>()
            .with_context(|| format!("Invalid index on frame {i}"))?;

        let content_start = part
            .find('>')
            .with_context(|| format!("Malformed <frame> at frame {i}"))?;
        let content = &part[content_start + 1..];
        let end = content
            .find("</frame>")
            .with_context(|| format!("Missing </frame> for frame {i}"))?;

        frames.push(EwvxFrame {
            index,
            svg: content[..end].to_string(),
        });
    }

    if frames.is_empty() {
        bail!("No frames found in EWVX file");
    }

    Ok(frames)
}

fn read_audio(input: &str) -> Result<Vec<EwvxTrack>> {
    let mut tracks = Vec::new();

    for (i, part) in input.split("<track ").skip(1).enumerate() {
        let track = read_track(part).with_context(|| format!("Failed to parse audio track {i}"))?;
        tracks.push(track);
    }

    Ok(tracks)
}

fn read_track(part: &str) -> Result<EwvxTrack> {
    let id = read_attribute(part, "id")?
        .parse::<u32>()
        .context("Invalid track id")?;
    let lang = read_optional_attribute(part, "lang");

    let info_start = part.find("<track-info>").context("Missing <track-info>")?;
    let info_end = part
        .find("</track-info>")
        .context("Missing </track-info>")?;
    let info = read_track_info(&part[info_start..info_end + "</track-info>".len()])?;

    let seg_start = part.find("<segments>").context("Missing <segments>")?;
    let seg_end = part.find("</segments>").context("Missing </segments>")?;
    let segments = read_segments(&part[seg_start..seg_end])?;

    Ok(EwvxTrack {
        id,
        lang,
        info,
        segments,
    })
}

fn read_track_info(section: &str) -> Result<EwvxTrackInfo> {
    Ok(EwvxTrackInfo {
        sample_rate: read_required_tag(section, "sample-rate")?,
        bit_depth: read_required_tag(section, "bit-depth")?,
        channels: read_required_tag(section, "channels")?,
        sample_format: read_required_tag(section, "sample-format")?,
        endianness: read_optional_tag(section, "endianness")
            .unwrap_or_else(|| "little".to_string()),
        total_samples: read_required_tag(section, "total-samples")?,
    })
}

fn read_segments(section: &str) -> Result<Vec<EwvxSegment>> {
    let mut segments = Vec::new();

    for (i, part) in section.split("<segment ").skip(1).enumerate() {
        let segment = read_segment(part).with_context(|| format!("Failed to parse segment {i}"))?;
        segments.push(segment);
    }

    Ok(segments)
}

fn read_segment(part: &str) -> Result<EwvxSegment> {
    let index = read_attribute(part, "index")?
        .parse::<usize>()
        .context("Invalid segment index")?;
    let timestamp = read_attribute(part, "timestamp")?
        .parse::<f64>()
        .context("Invalid segment timestamp")?;
    let sample_offset = read_attribute(part, "sample-offset")?
        .parse::<u64>()
        .context("Invalid segment sample-offset")?;
    let sample_count = read_attribute(part, "sample-count")?
        .parse::<u32>()
        .context("Invalid segment sample-count")?;

    let samples_start = part.find("<samples>").context("Missing <samples>")?;
    let samples_end = part.find("</samples>").context("Missing </samples>")?;
    let samples_str = &part[samples_start + "<samples>".len()..samples_end];

    let samples: Vec<i32> = samples_str
        .split_whitespace()
        .map(|s| s.parse::<i32>())
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Invalid PCM sample values")?;

    Ok(EwvxSegment {
        index,
        timestamp,
        sample_offset,
        sample_count,
        samples,
    })
}

fn read_required_tag<T: std::str::FromStr>(input: &str, tag: &str) -> Result<T>
where
    T::Err: std::fmt::Display,
{
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = input
        .find(&open)
        .with_context(|| format!("Missing <{tag}>"))?;
    let end = input
        .find(&close)
        .with_context(|| format!("Missing </{tag}>"))?;
    let value = input[start + open.len()..end].trim();
    value
        .parse::<T>()
        .map_err(|e| anyhow::anyhow!("Invalid <{tag}> value: {e}"))
}

fn read_optional_tag(input: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = input.find(&open)?;
    let end = input.find(&close)?;
    Some(xml_unescape(input[start + open.len()..end].trim()))
}

fn read_attribute(input: &str, name: &str) -> Result<String> {
    let needle = format!("{name}=\"");
    let start = input
        .find(&needle)
        .with_context(|| format!("Missing {name} attribute"))?;
    let value_start = start + needle.len();
    let value_end = input[value_start..]
        .find('"')
        .with_context(|| format!("Malformed {name} attribute"))?;
    Ok(input[value_start..value_start + value_end].to_string())
}

fn read_optional_attribute(input: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = input.find(&needle)?;
    let value_start = start + needle.len();
    let value_end = input[value_start..].find('"')?;
    Some(xml_unescape(&input[value_start..value_start + value_end]))
}

fn xml_unescape(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}
