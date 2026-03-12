fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.ewvx>", args[0]);
        std::process::exit(1);
    }

    println!("TODO: play {}", args[1]);
}

