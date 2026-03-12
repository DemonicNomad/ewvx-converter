mod decode;
mod trace;

use std::io::{BufWriter, Write};
use std::thread;

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
    let total = frames.len();

    // PARALLELISIERUNG JUHUU
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
            writeln!(w, "{}", svg).unwrap();
            println!("Sehr effizienter Trace von Frame {}", index);
        }
    }

    w.flush().unwrap();
    println!("Frame-Anzahl {}", total);
}
