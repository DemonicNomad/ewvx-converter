/// Complete in-memory representation of an EWVX file.
pub struct EwvxData {
    /// Video metadata.
    pub meta: EwvxMeta,
    /// Ordered list of frames.
    pub frames: Vec<EwvxFrame>,
}

/// Metadata from the `<meta-ente>` element.
pub struct EwvxMeta {
    /// Optional video title.
    pub title: Option<String>,
    /// Optional author name.
    pub author: Option<String>,
    /// Optional ISO 8601 creation timestamp.
    pub created: Option<String>,
    /// Optional free-text description.
    pub description: Option<String>,
    /// Frames per second.
    pub fps: f64,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Total number of frames.
    pub frame_count: u32,
    /// Total duration in seconds.
    pub duration: f64,
    /// Ente.
    pub ente: bool,
}

/// A single video frame.
pub struct EwvxFrame {
    /// Zero-based frame index.
    pub index: usize,
    /// Raw SVG content for this frame.
    pub svg: String,
}
