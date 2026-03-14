/// Representation of an EWVX file.
pub struct EwvxData {
    /// Video metadata.
    pub meta: EwvxMeta,
    /// Ordered list of frames.
    pub frames: Vec<EwvxFrame>,
    /// Optional audio tracks.
    pub audio: Vec<EwvxTrack>,
}

/// Metadata from the `<meta-ente>` element.
pub struct EwvxMeta {
    /// Optional video title.
    pub title: Option<String>,
    /// Optional author name.
    pub author: Option<String>,
    /// Optional creation timestamp.
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

/// A single audio track.
pub struct EwvxTrack {
    /// Zero-based track ID.
    pub id: u32,
    /// Optional BCP 47 language tag.
    pub lang: Option<String>,
    /// Audio format parameters.
    pub info: EwvxTrackInfo,
    /// Ordered list of audio segments.
    pub segments: Vec<EwvxSegment>,
}

/// Audio format parameters for a track.
pub struct EwvxTrackInfo {
    /// Samples per second (e.g. 44100, 48000).
    pub sample_rate: u32,
    /// Bits per sample (8, 16, 24, or 32).
    pub bit_depth: u16,
    /// Number of audio channels.
    pub channels: u16,
    /// Sample format: `"int"` or `"float"`.
    pub sample_format: String,
    /// Byte order: `"little"` or `"big"`.
    pub endianness: String,
    /// Total number of samples per channel.
    pub total_samples: u64,
}

/// A contiguous chunk of interleaved PCM samples.
pub struct EwvxSegment {
    /// Zero-based segment index.
    pub index: usize,
    /// Start time of this segment in seconds.
    pub timestamp: f64,
    /// Sample offset from the start of the track (per channel).
    pub sample_offset: u64,
    /// Number of samples per channel in this segment.
    pub sample_count: u32,
    /// Interleaved PCM sample values (L R L R … for stereo).
    pub samples: Vec<i32>,
}
