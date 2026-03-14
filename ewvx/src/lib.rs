//! Shared library for the EWVX (Ente Wurzel Video XML) format.
//!
//! Provides the canonical data types, a v2.0 XML writer, and a v2.0 XML parser
//! used by both the `ewvx-converter` and `ewvx-player` binaries.
//! Currently no support for audio, will be added before 1.0.

pub mod types;
/// Streaming XML writer for the EWVX v2.0 format.
///
/// # Example
/// ```
/// use ewvx::types::EwvxMeta;
/// use ewvx::writer;
///
/// let meta = EwvxMeta {
///     title: None, author: None, created: None, description: None,
///     fps: 24.0, width: 100, height: 100,
///     frame_count: 1, duration: 1.0 / 24.0, ente: true,
/// };
///
/// let mut buf = Vec::new();
/// writer::write_header(&mut buf, &meta).unwrap();
/// writer::write_frame(&mut buf, 0, r#"<svg xmlns="http://www.w3.org/2000/svg"/>"#).unwrap();
/// writer::write_footer(&mut buf).unwrap();
///
/// let output = String::from_utf8(buf).unwrap();
/// assert!(output.contains(r#"version="2.0""#));
/// assert!(output.contains(r#"<frame index="0">"#));
/// ```
pub mod writer;
/// XML parser for the EWVX v2.0 format.
///
/// # Example
/// ```
/// use ewvx::parser;
///
/// let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
/// <video version="2.0" xmlns="ente-schema:ewvx:2.0">
///   <meta-ente>
///     <fps>24.000000</fps>
///     <width>100</width>
///     <height>100</height>
///     <frame-count>1</frame-count>
///     <duration>0.041667</duration>
///     <ente>true</ente>
///   </meta-ente>
///   <frames>
///     <frame index="0">
///       <svg xmlns="http://www.w3.org/2000/svg"/>
///     </frame>
///   </frames>
/// </video>"#;
///
/// let data = parser::parse(xml).unwrap();
/// assert_eq!(data.meta.fps, 24.0);
/// assert_eq!(data.meta.width, 100);
/// assert_eq!(data.frames.len(), 1);
/// assert_eq!(data.frames[0].index, 0);
/// ```
pub mod parser;
