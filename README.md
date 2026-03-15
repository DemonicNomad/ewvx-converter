# Ente Wurzel? Video! XML

XML-based video format where each frame is an SVG element.

## Features

- Converts standard video files to `.ewvx` format via frame-by-frame SVG tracing
- Plays `.ewvx` files 

## Usage

```bash
# Convert a video to ewvx
ewvx-converter input.mp4 output.ewvx

# Play an ewvx file
ewvx-player output.ewvx
```

## 2.0 TODO

- [x] 2.0 Schema
- [x] Restructure workspace to separate ewvx format data
- [x] Update ewvx package for v2.0 schema
- [x] Update converter to output v2.0 schema
- [ ] Update player to read v2.0 schema
