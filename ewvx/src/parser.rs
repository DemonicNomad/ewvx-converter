use anyhow::{Context, Result, bail};
use crate::types::{EwvxData, EwvxFrame, EwvxMeta};

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

    Ok(EwvxData { meta, frames })
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
