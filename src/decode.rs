use ffmpeg_next::format::{input};
use ffmpeg_next::media::Type;

use crate::error::ConvertError;

pub fn get_fps(path: &str) -> Result<f64, ConvertError> {
    let input_context = input(&path)?;
    let stream = input_context
        .streams()
        .best(Type::Video)
        .ok_or_else(|| ConvertError::Arg("Kein Stream".into()))?;

    let avg = stream.avg_frame_rate();
    if avg.denominator() == 0 {
        return Err(ConvertError::Arg("FALSCH".into()))
    }
    let fps = avg.numerator() as f64 / avg.denominator() as f64;
    if fps <= 0.0 {
        return Err(ConvertError::Arg("FALSCH".into()))
    }

    Ok(fps)
}

