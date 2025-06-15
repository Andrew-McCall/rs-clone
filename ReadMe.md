# rs-clone

A Rust CLI tool to copy and organize video folders with subtitles.

## Features

- Select and rename source folders before copying
- Copy videos, subtitles, or both
- Preserve folder structure
- Simple TOML config for paths and mappings

## Usage

1. Edit `.rs-clone.conf` with your source and destination folders.
2. Run with `cargo run --release`
3. Select folder and optionally rename destination
4. Choose which files to copy
5. Files are copied accordingly

## Config Example
.rs-clone.conf
```toml
[settings]
source_dir = "/path/to/source"
destination_dir = "/path/to/destination"

[mapping]
"example_src" = "example_dest"
```

## File Types
Videos: mp4, mkv, avi, mov, flv, wmv, webm
Subtitles: srt, ass, vtt, sub, ssa

## License
MIT License Copyright (c) 2025 Andrew McCall


