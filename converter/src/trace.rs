use anyhow::{Result};
use vtracer::{ColorImage, ColorMode, Config, convert};

use crate::decode::FrameData;

pub fn trace_frame(frame: FrameData) -> Result<String> {
    let img = ColorImage {
        pixels: frame.rgba,
        width: frame.width as usize,
        height: frame.height as usize,
    };

    let config = Config {
        color_mode: ColorMode::Color,
        max_iterations: 1,
        ..Config::default()
    };

    let svg = convert(img, config).unwrap();
    Ok(svg.to_string())
}
