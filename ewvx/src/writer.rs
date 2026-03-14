use std::io::Write;
use anyhow::{Context, Result};
use crate::types::{EwvxMeta, EwvxTrack};

/// Writes the XML declaration, opening `<video>` tag, `<meta-ente>` block,
/// and opening `<frames>` tag. Call [`write_frame`] for each frame, then
/// [`write_frames_end`], optionally [`write_audio`], and finally [`write_end`].
pub fn write_header(w: &mut impl Write, meta: &EwvxMeta) -> Result<()> {
    writeln!(w, r#"<?xml version="1.0" encoding="UTF-8"?>"#)
        .context("Failed to write XML declaration")?;
    writeln!(w, r#"<video version="2.0" xmlns="ente-schema:ewvx:2.0">"#)
        .context("Failed to write <video> tag")?;
    writeln!(w, "  <meta-ente>")
        .context("Failed to write <meta-ente> tag")?;

    if let Some(title) = &meta.title {
        writeln!(w, "    <title>{}</title>", title)
            .context("Failed to write title")?;
    }
    if let Some(author) = &meta.author {
        writeln!(w, "    <author>{}</author>", author)
            .context("Failed to write author")?;
    }
    if let Some(created) = &meta.created {
        writeln!(w, "    <created>{}</created>", created)
            .context("Failed to write created")?;
    }
    if let Some(description) = &meta.description {
        writeln!(w, "    <description>{}</description>", description)
            .context("Failed to write description")?;
    }

    writeln!(w, "    <fps>{:.6}</fps>", meta.fps)
        .context("Failed to write fps")?;
    writeln!(w, "    <width>{}</width>", meta.width)
        .context("Failed to write width")?;
    writeln!(w, "    <height>{}</height>", meta.height)
        .context("Failed to write height")?;
    writeln!(w, "    <frame-count>{}</frame-count>", meta.frame_count)
        .context("Failed to write frame-count")?;
    writeln!(w, "    <duration>{:.6}</duration>", meta.duration)
        .context("Failed to write duration")?;
    writeln!(w, "    <ente>{}</ente>", meta.ente)
        .context("Failed to write ente")?;

    writeln!(w, "  </meta-ente>")
        .context("Failed to write </meta-ente> tag")?;
    writeln!(w, "  <frames>")
        .context("Failed to write <frames> tag")?;
    Ok(())
}

/// Writes a single `<frame index="N">` element. Any leading XML declaration
/// or comment in the SVG string is stripped automatically.
pub fn write_frame(w: &mut impl Write, index: usize, svg: &str) -> Result<()> {
    let svg = strip_svg_xml_declare(svg);
    let svg = strip_start_comment(svg);
    writeln!(w, r#"    <frame index="{}">"#, index)
        .context("Failed to write <frame> tag")?;

    for line in svg.trim().lines() {
        writeln!(w, "      {}", line)
            .context("Failed to write frame SVG content")?;
    }

    writeln!(w, "    </frame>")
        .context("Failed to write </frame> tag")?;
    Ok(())
}

/// Closes the `<frames>` element. Call after all [`write_frame`] calls,
/// before [`write_audio`] or [`write_end`].
pub fn write_frames_end(w: &mut impl Write) -> Result<()> {
    writeln!(w, "  </frames>")
        .context("Failed to write </frames> tag")?;
    Ok(())
}

/// Writes the optional `<audio>` block containing the given tracks.
/// Call between [`write_frames_end`] and [`write_end`].
pub fn write_audio(w: &mut impl Write, tracks: &[EwvxTrack]) -> Result<()> {
    writeln!(w, "  <audio>").context("Failed to write <audio>")?;

    for track in tracks {
        write!(w, r#"    <track id="{}""#, track.id)
            .context("Failed to write <track>")?;
        if let Some(lang) = &track.lang {
            write!(w, r#" lang="{}""#, lang).context("Failed to write track lang")?;
        }
        writeln!(w, ">").context("Failed to write <track> close")?;

        let info = &track.info;
        writeln!(w, "      <track-info>").context("Failed to write <track-info>")?;
        writeln!(w, "        <sample-rate>{}</sample-rate>", info.sample_rate)
            .context("Failed to write sample-rate")?;
        writeln!(w, "        <bit-depth>{}</bit-depth>", info.bit_depth)
            .context("Failed to write bit-depth")?;
        writeln!(w, "        <channels>{}</channels>", info.channels)
            .context("Failed to write channels")?;
        writeln!(w, "        <sample-format>{}</sample-format>", info.sample_format)
            .context("Failed to write sample-format")?;
        writeln!(w, "        <endianness>{}</endianness>", info.endianness)
            .context("Failed to write endianness")?;
        writeln!(w, "        <total-samples>{}</total-samples>", info.total_samples)
            .context("Failed to write total-samples")?;
        writeln!(w, "      </track-info>").context("Failed to write </track-info>")?;

        writeln!(w, "      <segments>").context("Failed to write <segments>")?;
        for seg in &track.segments {
            writeln!(w,
                r#"        <segment index="{}" timestamp="{:.6}" sample-offset="{}" sample-count="{}">"#,
                seg.index, seg.timestamp, seg.sample_offset, seg.sample_count
            ).context("Failed to write <segment>")?;

            let sample_str: String = seg.samples.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            writeln!(w, "          <samples>{}</samples>", sample_str)
                .context("Failed to write samples")?;

            writeln!(w, "        </segment>").context("Failed to write </segment>")?;
        }
        writeln!(w, "      </segments>").context("Failed to write </segments>")?;
        writeln!(w, "    </track>").context("Failed to write </track>")?;
    }

    writeln!(w, "  </audio>").context("Failed to write </audio>")?;
    Ok(())
}

/// Closes the `<video>` root element. Must be the last writer call.
pub fn write_end(w: &mut impl Write) -> Result<()> {
    writeln!(w, "</video>")
        .context("Failed to write </video> tag")?;
    Ok(())
}

fn strip_svg_xml_declare(svg: &str) -> &str {
    let trimmed = svg.trim_start();
    if trimmed.starts_with("<?xml") {
        if let Some(pos) = trimmed.find("?>") {
            return trimmed[pos + 2..].trim_start();
        }
    }
    trimmed
}

fn strip_start_comment(svg: &str) -> &str {
    let trimmed = svg.trim_start();
    if trimmed.starts_with("<!--") {
        if let Some(pos) = trimmed.find("-->") {
            return trimmed[pos + 3..].trim_start();
        }
    }
    trimmed
}
