# Ente Wurzel? Video! XML

XML-based video format where each frame is an SVG element.

## Features

- Converts standard video files to `.ewvx` format via frame-by-frame SVG tracing
- Plays `.ewvx` files 

## Installation (Glaube ich, ka)

### Linux

```bash
sudo apt install build-essential nasm pkg-config libclang-dev

# Build
git clone https://github.com/your-user/ewvx-converter.git
cd ewvx-converter
cargo build --release
```

### Windows

```powershell
# Dependencies: ????

git clone https://github.com/your-user/ewvx-converter.git
cd ewvx-converter
cargo build --release
```

## Usage

```bash
# Convert a video to ewvx
ewvx-converter input.mp4 output.ewvx

# Play an ewvx file
ewvx-player output.ewvx
```

## Post 1.0 TODO

- [ ] Multi-threaded player rendering
- [ ] Improve metadata (e.g. dimensions, duration)
- [ ] Add audio support