pub struct EwvxData {
    meta: EwvxMeta,
    frames: Vec<String>
}

pub struct EwvxMeta {
    fps: f32,
    ente: bool
}

pub fn parse(input: &str) -> EwvxData {
    let meta_start = input.find("<meta-ente>").unwrap();
    let meta_end = input.find("</meta-ente>").unwrap();

    let meta = parse_meta_ente(&input[meta_start..meta_end]);

    EwvxData {
        meta,
        frames: vec![]
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
