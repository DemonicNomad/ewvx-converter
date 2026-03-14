use std::io::Write;
use anyhow::{Context, Result};

pub fn write_meta_ente(w: &mut impl Write, fps: f64) -> Result<()> {
    writeln!(w, r#"<?xml version="1.0" encoding="UTF-8"?>"#).context("Failed to write XML declaration")?;
    writeln!(w, "<video>").context("Failed to write <video> tag")?;
    writeln!(w, "  <meta-ente>").context("Failed to write <meta-ente> tag")?;
    writeln!(w, "    <fps>{:.6}</fps>", fps).context("Failed to write fps")?;
    writeln!(w, "    <ente>true</ente>").context("Failed to write ente")?;
    writeln!(w, "  </meta-ente>").context("Failed to write </meta-ente> tag")?;
    writeln!(w, "  <frames>").context("Failed to write <frames> tag")?;
    Ok(())
}

pub fn write_frame(w: &mut impl Write, svg: &str) -> Result<()> {
    let svg = strip_svg_xml_declare(svg);
    let svg = strip_start_comment(svg);
    writeln!(w, "    <frame>").context("Failed to write <frame> tag")?;

    for line in svg.trim().lines() {
        writeln!(w, "      {}", line).context("Failed to write frame SVG content")?;
    }
    
    writeln!(w, "    </frame>").context("Failed to write </frame> tag")?;
    Ok(())
}

pub fn write_frame_end(w: &mut impl Write) -> Result<()> {
    writeln!(w, "  </frames>").context("Failed to write </frames> tag")?;
    writeln!(w, "</video>").context("Failed to write </video> tag")?;
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
