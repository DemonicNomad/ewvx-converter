pub struct EwvxData {
    pub meta: EwvxMeta,
    pub frames: Vec<String>
}

pub struct EwvxMeta {
    pub fps: f32,
    pub ente: bool
}

pub fn parse(input: &str) -> EwvxData {
    let meta_start = input.find("<meta-ente>").unwrap();
    let meta_end = input.find("</meta-ente>").unwrap();
    let frames_start = input.find("<frames>").unwrap();

    let meta = parse_meta_ente(&input[meta_start..meta_end]);
    let frames = parse_frames(&input[frames_start..]);

    EwvxData {
        meta,
        frames
    }
}

fn parse_meta_ente(meta_input: &str) -> EwvxMeta {
    let fps_start = meta_input.find("<fps>").unwrap();
    let fps = meta_input[fps_start + 5..meta_input.find("</fps>").unwrap()]
        .trim().parse::<f32>().unwrap();

    let ente_start = meta_input.find("<ente>").unwrap();
    let ente = meta_input[ente_start + 6..meta_input.find("</ente>").unwrap()]
        .trim().parse::<bool>().unwrap();

    EwvxMeta {
        fps,
        ente
    }
}

fn parse_frames(frame_input: &str) -> Vec<String> {
    let mut frames: Vec<String> = Vec::new();
    let frame_raw = frame_input.split("<frame>").skip(1).collect::<Vec<&str>>();

    for frame in frame_raw {
        frames.push(frame[..frame.find("</frame>").unwrap()].to_string());
    }

    frames
}
