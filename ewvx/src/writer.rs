use crate::types::{EwvxMeta, EwvxTrack};
use anyhow::{Context, Result};
use std::io::{self, Write};
use std::marker::PhantomData;

/// Typestate: accepting `<frame>` elements.
pub struct WritingFrames;
/// Typestate: `<frames>` closed — may write `<audio>` or [`finish`](EwvxWriter::finish).
pub struct FramesDone;

/// EWVX v2.0 writer
///
/// ```text
/// EwvxWriter::new          → EwvxWriter<W, WritingFrames>
///   .write_frame()         → &mut self  (repeatable)
///   .end_frames()          → EwvxWriter<W, FramesDone>    (consumes self)
///     .write_audio()       → &mut self  (optional, at most once)
///     .finish()            → W          (consumes self)
/// ```
pub struct EwvxWriter<W: Write, S> {
    w: W,
    _state: PhantomData<S>,
}

impl<W: Write> EwvxWriter<W, WritingFrames> {
    /// Creates a new writer, emitting the XML declaration, `<video>`,
    /// the full `<meta-ente>` block, and the opening `<frames>` tag.
    pub fn new(mut w: W, meta: &EwvxMeta) -> Result<Self> {
        emit_header(&mut w, meta).context("writing EWVX header")?;
        Ok(Self {
            w,
            _state: PhantomData,
        })
    }

    /// Writes a single `<frame index="N">` element.
    ///
    /// Any leading `<?xml …?>` declaration or `<!-- … -->` comment in the SVG
    /// string is stripped automatically.
    pub fn write_frame(&mut self, index: usize, svg: &str) -> Result<()> {
        emit_frame(&mut self.w, index, svg).with_context(|| format!("writing frame {index}"))
    }

    /// Closes the `<frames>` element and transitions to [`FramesDone`],
    /// where [`write_audio`](EwvxWriter::write_audio) or
    /// [`finish`](EwvxWriter::finish) can be called.
    pub fn end_frames(mut self) -> Result<EwvxWriter<W, FramesDone>> {
        writeln!(self.w, "  </frames>").context("closing frames")?;
        Ok(EwvxWriter {
            w: self.w,
            _state: PhantomData,
        })
    }
}

impl<W: Write> EwvxWriter<W, FramesDone> {
    /// Writes the optional `<audio>` block containing the given tracks.
    ///
    /// Call at most once before [`finish`](Self::finish).
    pub fn write_audio(&mut self, tracks: &[EwvxTrack]) -> Result<()> {
        emit_audio(&mut self.w, tracks).context("writing audio")
    }

    /// Closes the `<video>` root element and returns the inner writer.
    pub fn finish(mut self) -> Result<W> {
        writeln!(self.w, "</video>").context("closing document")?;
        Ok(self.w)
    }
}

fn emit_header(w: &mut impl Write, meta: &EwvxMeta) -> io::Result<()> {
    writeln!(w, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(w, r#"<video version="2.0" xmlns="ente-schema:ewvx:2.0">"#)?;

    writeln!(w, "  <meta-ente>")?;
    if let Some(v) = &meta.title {
        writeln!(w, "    <title>{}</title>", xml_escape(v))?;
    }
    if let Some(v) = &meta.author {
        writeln!(w, "    <author>{}</author>", xml_escape(v))?;
    }
    if let Some(v) = &meta.created {
        writeln!(w, "    <created>{}</created>", xml_escape(v))?;
    }
    if let Some(v) = &meta.description {
        writeln!(w, "    <description>{}</description>", xml_escape(v))?;
    }
    writeln!(w, "    <fps>{:.6}</fps>", meta.fps)?;
    writeln!(w, "    <width>{}</width>", meta.width)?;
    writeln!(w, "    <height>{}</height>", meta.height)?;
    writeln!(w, "    <frame-count>{}</frame-count>", meta.frame_count)?;
    writeln!(w, "    <duration>{:.6}</duration>", meta.duration)?;
    writeln!(w, "    <ente>{}</ente>", meta.ente)?;
    writeln!(w, "  </meta-ente>")?;

    writeln!(w, "  <frames>")?;
    Ok(())
}

fn emit_frame(w: &mut impl Write, index: usize, svg: &str) -> io::Result<()> {
    let svg = strip_xml_preamble(svg);
    writeln!(w, r#"    <frame index="{index}">"#)?;
    for line in svg.trim().lines() {
        writeln!(w, "      {line}")?;
    }
    writeln!(w, "    </frame>")?;
    Ok(())
}

fn emit_audio(w: &mut impl Write, tracks: &[EwvxTrack]) -> io::Result<()> {
    writeln!(w, "  <audio>")?;

    for track in tracks {
        write!(w, r#"    <track id="{}""#, track.id)?;
        if let Some(lang) = &track.lang {
            write!(w, r#" lang="{}""#, xml_escape_attr(lang))?;
        }
        writeln!(w, ">")?;

        let info = &track.info;
        writeln!(w, "      <track-info>")?;
        writeln!(w, "        <sample-rate>{}</sample-rate>", info.sample_rate)?;
        writeln!(w, "        <bit-depth>{}</bit-depth>", info.bit_depth)?;
        writeln!(w, "        <channels>{}</channels>", info.channels)?;
        writeln!(
            w,
            "        <sample-format>{}</sample-format>",
            info.sample_format
        )?;
        writeln!(w, "        <endianness>{}</endianness>", info.endianness)?;
        writeln!(
            w,
            "        <total-samples>{}</total-samples>",
            info.total_samples
        )?;
        writeln!(w, "      </track-info>")?;

        writeln!(w, "      <segments>")?;
        for seg in &track.segments {
            writeln!(
                w,
                r#"        <segment index="{}" timestamp="{:.6}" sample-offset="{}" sample-count="{}">"#,
                seg.index, seg.timestamp, seg.sample_offset, seg.sample_count
            )?;

            write!(w, "          <samples>")?;
            for (i, sample) in seg.samples.iter().enumerate() {
                if i > 0 {
                    write!(w, " ")?;
                }
                write!(w, "{sample}")?;
            }
            writeln!(w, "</samples>")?;

            writeln!(w, "        </segment>")?;
        }
        writeln!(w, "      </segments>")?;
        writeln!(w, "    </track>")?;
    }

    writeln!(w, "  </audio>")?;
    Ok(())
}

fn strip_xml_preamble(svg: &str) -> &str {
    let mut s = svg.trim_start();
    if s.starts_with("<?xml") {
        if let Some(pos) = s.find("?>") {
            s = s[pos + 2..].trim_start();
        }
    }
    if s.starts_with("<!--") {
        if let Some(pos) = s.find("-->") {
            s = s[pos + 3..].trim_start();
        }
    }
    s
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn xml_escape_attr(s: &str) -> String {
    xml_escape(s).replace('"', "&quot;")
}
