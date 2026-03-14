use std::io::Write;
use anyhow::{Context, Result};
use crate::types::EwvxMeta;

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

pub fn write_footer(w: &mut impl Write) -> Result<()> {
    writeln!(w, "  </frames>")
        .context("Failed to write </frames> tag")?;
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
