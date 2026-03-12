use std::io::{Write};

pub fn write_meta_ente(w: &mut impl Write, fps: f64) -> () {
    writeln!(w, r#"<?xml version="1.0" encoding="UTF-8"?>"#).unwrap();
    writeln!(w, "<video>").unwrap();
    writeln!(w, "  <meta-ente>").unwrap();
    writeln!(w, "    <fps>{:.6}</fps>", fps).unwrap();
    writeln!(w, "    <ente>true</ente>").unwrap();
    writeln!(w, "  </meta-ente>").unwrap();
    writeln!(w, "  <frames>").unwrap();
}

pub fn write_frame(w: &mut impl Write, svg: &str) -> () {
    let svg = strip_svg_xml_declare(svg);
    let svg = strip_start_comment(svg);
    writeln!(w, "    <frame>").unwrap();

    for line in svg.trim().lines() {
        writeln!(w, "      {}", line).unwrap();
    }
    writeln!(w, "    </frame>").unwrap();
}

pub fn write_frame_end(w: &mut impl Write) -> () {
    writeln!(w, "  </frames>").unwrap();
    writeln!(w, "</video>").unwrap();
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
