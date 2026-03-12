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

fn run(input: &str, output: &str) -> () {
    if std::fs::metadata(input).is_err() {
        panic!("AHHHHHHH");
    }

    let fps = decode::get_fps(input);
    println!("FPS: {:.6}", fps);

    let file = std::fs::File::create(output).unwrap();
    let mut w = BufWriter::new(file);

    let frames = decode::decode_frames(input);

    for frame in &frames {
        let mut first = true;
        for pixel in frame.rgba.chunks_exact(4) {
            if !first {
                write!(w, " ").unwrap();
            }
            write!(w, "{},{},{},{}", pixel[0], pixel[1], pixel[2], pixel[3]).unwrap();
            first = false;
        }
        writeln!(w).unwrap();

        println!("Frame {} ({}x{}, {} pixels)", frame.index, frame.width, frame.height, frame.rgba.len() / 4);
    }

    w.flush().unwrap();
    eprintln!("Frame-Anzahl {}", frames.len());
}
