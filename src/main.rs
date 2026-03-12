mod decode;
mod trace;

use std::io::{BufWriter, Write};

fn main() {
    ffmpeg_next::init().expect("Kein ffmpeg blablabla");

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        std::process::exit(1);
    }

    let input = &args[1];
    let output = &args[2];

    run(input, output)
}

//Wer muss schon parallelisieren
fn run(input: &str, output: &str) -> () {
    if std::fs::metadata(input).is_err() {
        panic!("AHHHHHHH");
    }

    let fps = decode::get_fps(input);
    println!("FPS: {:.6}", fps);

    let file = std::fs::File::create(output).unwrap();
    let mut w = BufWriter::new(file);

    let frames = decode::decode_frames(input);

    for frame in frames {
        let index = frame.index;
        let svg = trace::trace_frame(frame);
        writeln!(w, "{}", svg).unwrap();
        println!("Sehr effizienter Trace von Frame {}", index);
    }

    w.flush().unwrap();
}
