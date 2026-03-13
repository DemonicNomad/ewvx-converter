mod decode;
mod trace;
mod xml;

use std::io::BufWriter;
use std::thread;

fn main() {
    ffmpeg_next::init().expect("Kein ffmpeg blablabla");

    let args: Vec<String> = std::env::args().collect();

    let input = &args[1];
    let output = &args[2];

    run(input, output)
}

fn run(input: &str, output: &str) {
    if std::fs::metadata(input).is_err() {
        panic!("AHHHHHHH");
    }

    let fps = decode::get_fps(input);

    let file = std::fs::File::create(output).unwrap();
    let mut w = BufWriter::new(file);

    xml::write_meta_ente(&mut w, fps);

    let frames = decode::decode_frames(input);
    let total = frames.len();

    for chunk in frames.into_iter().collect::<Vec<_>>().chunks_mut(8) {
        let frame_chunk: Vec<_> = chunk.iter_mut().map(|f| std::mem::take(f)).collect();

        let chunk_processed: Vec<_> = frame_chunk
            .into_iter()
            .map(|frame| {
                thread::spawn(move || {
                    let index = frame.index;
                    let svg = trace::trace_frame(frame);
                    (index, svg)
                })
            })
            .collect();

        for handle in chunk_processed {
            let (index, svg) = handle.join().unwrap();
            xml::write_frame(&mut w, &svg);
            println!("Sehr effizientes Tracen von Frame {}", index);
        }
    }

    xml::write_frame_end(&mut w);
}
