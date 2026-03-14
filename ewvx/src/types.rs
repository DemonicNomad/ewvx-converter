pub struct EwvxData {
    pub meta: EwvxMeta,
    pub frames: Vec<EwvxFrame>,
}

pub struct EwvxMeta {
    pub title: Option<String>,
    pub author: Option<String>,
    pub created: Option<String>,
    pub description: Option<String>,
    pub fps: f64,
    pub width: u32,
    pub height: u32,
    pub frame_count: u32,
    pub duration: f64,
    pub ente: bool,
}

pub struct EwvxFrame {
    pub index: usize,
    pub svg: String,
}
