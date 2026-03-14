use anyhow::{Context, Result, bail};

pub struct EwvxData {
    pub meta: EwvxMeta,
    pub frames: Vec<String>
}

pub struct EwvxMeta {
    pub fps: f32,
    pub ente: bool
}

pub fn parse(input: &str) -> Result<EwvxData> {
    let meta_start = input.find("<meta-ente>")
        .context("Missing <meta-ente> tag in EWVX file")?;
    let meta_end = input.find("</meta-ente>")
        .context("Missing </meta-ente> closing tag in EWVX file")?;
    let frames_start = input.find("<frames>")
        .context("Missing <frames> tag in EWVX file")?;

    let meta = parse_meta_ente(&input[meta_start..meta_end])?;
    let frames = parse_frames(&input[frames_start..])?;

    Ok(EwvxData {
        meta,
        frames
    })
}

fn parse_meta_ente(meta_input: &str) -> Result<EwvxMeta> {
    let fps_start = meta_input.find("<fps>")
        .context("Missing <fps> tag in meta-ente")?;
    let fps_end = meta_input.find("</fps>")
        .context("Missing </fps> closing tag in meta-ente")?;
    let fps = meta_input[fps_start + 5..fps_end]
        .trim().parse::<f32>()
        .context("Failed to parse fps value as f32")?;

    let ente_start = meta_input.find("<ente>")
        .context("Missing <ente> tag in meta-ente")?;
    let ente_end = meta_input.find("</ente>")
        .context("Missing </ente> closing tag in meta-ente")?;
    let ente = meta_input[ente_start + 6..ente_end]
        .trim().parse::<bool>()
        .context("Failed to parse ente value as bool")?;

    Ok(EwvxMeta {
        fps,
        ente
    })
}

fn parse_frames(frame_input: &str) -> Result<Vec<String>> {
    let mut frames: Vec<String> = Vec::new();
    let frame_raw = frame_input.split("<frame>").skip(1).collect::<Vec<&str>>();

    if frame_raw.is_empty() {
        bail!("No frames found in EWVX file");
    }

    for (i, frame) in frame_raw.iter().enumerate() {
        let end = frame.find("</frame>")
            .with_context(|| format!("Missing </frame> closing tag for frame {}", i))?;
        frames.push(frame[..end].to_string());
    }

    Ok(frames)
}
