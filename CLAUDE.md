# Losselot - Audio Forensics Tool

Losselot detects fake "lossless" audio files—files claiming to be lossless (FLAC, WAV, AIFF) but actually created from lossy sources (MP3, AAC). It uses dual analysis: binary metadata inspection and FFT-based spectral analysis.

## Quick Reference

```bash
# Build
cargo build --release

# Run tests
cargo test

# Analyze a file
cargo run -- path/to/file.flac

# Analyze a directory
cargo run -- ~/Music/

# Interactive web UI
cargo run -- serve ~/Music/ --port 3000

# Skip spectral analysis (faster)
cargo run -- --no-spectral path/to/file.flac

# Generate test files (requires ffmpeg, lame, sox)
./examples/generate_test_files.sh
```

## Architecture

```
src/
├── main.rs              # CLI entry, argument parsing, parallel execution
├── lib.rs               # Public API exports
├── serve.rs             # HTTP server for web UI
├── ui.html              # Embedded web UI (D3.js visualizations)
├── analyzer/
│   ├── mod.rs           # Core orchestration, score combination
│   ├── spectral.rs      # FFT frequency analysis (8192-sample windows)
│   └── binary.rs        # MP3 metadata, LAME headers, encoder signatures
├── mp3/
│   ├── mod.rs           # MP3 module exports
│   ├── frame.rs         # MP3 frame header parsing
│   └── lame.rs          # LAME/Xing header extraction
└── report/
    ├── mod.rs           # Report dispatcher
    ├── html.rs          # Interactive HTML report
    ├── json.rs          # JSON output
    └── csv.rs           # CSV output
```

## Scoring System

| Component | Max Points | Key Indicators |
|-----------|------------|----------------|
| Binary    | ~50        | Lowpass mismatch, multiple encoder signatures, frame variance |
| Spectral  | ~50        | High-frequency drops, missing ultrasonic, steep rolloff |
| Agreement | +15        | Bonus when both methods agree on transcode |

**Verdicts:**
- `OK` (0-34): Clean file
- `SUSPECT` (35-64): Possibly transcoded
- `TRANSCODE` (65-100): Definitely transcoded

## Key Detection Flags

**Spectral:** `severe_hf_damage`, `hf_cutoff_detected`, `weak_ultrasonic_content`, `dead_ultrasonic_band`, `silent_17k+`, `steep_hf_rolloff`

**Re-encoding:** `multi_encoder_sigs`, `encoding_chain(LAME → FFmpeg)`, `lame_reencoded_x2`, `ffmpeg_processed_x2`

**Binary:** `lowpass_bitrate_mismatch`, `encoder_quality_mismatch`

## Code Conventions

- Use `rustfmt` for formatting
- Tests go inline in each module using `#[test]`
- All analysis results must be serializable via `serde`
- Use `rayon` for parallel file processing
- Error handling: propagate with `?`, use `anyhow` for CLI errors
- Constants for thresholds are in `analyzer/mod.rs`

## Key Constants

```rust
// analyzer/mod.rs
const DEFAULT_SUSPECT_THRESHOLD: u8 = 35;
const DEFAULT_TRANSCODE_THRESHOLD: u8 = 65;
const AGREEMENT_BONUS: u8 = 15;

// spectral.rs
const FFT_SIZE: usize = 8192;  // ~186ms windows at 44.1kHz
```

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_threshold_boundaries
```

Test files can be generated with `./examples/generate_test_files.sh` which creates various encoding scenarios (clean, transcoded, re-encoded chains).

## Common Workflows

### Adding a new detection flag
1. Define flag string in `analyzer/spectral.rs` or `analyzer/binary.rs`
2. Add detection logic in the appropriate `analyze()` function
3. Push to `flags` vector in result
4. Update HTML report if visualization needed (`report/html.rs`)

### Adjusting thresholds
1. Modify constants in `analyzer/mod.rs`
2. Run tests to verify: `cargo test test_threshold`
3. Test with real files from `examples/demo_files/`

### Adding new audio format support
1. Symphonia handles most decoding - check if codec is supported
2. Add file extension to `SUPPORTED_EXTENSIONS` in `main.rs`
3. Test with sample files

## Exit Codes

- `0`: All files clean
- `1`: At least one suspect file
- `2`: At least one definite transcode

## Dependencies

Key crates:
- `symphonia`: Audio decoding
- `rustfft`: FFT for spectral analysis
- `clap`: CLI parsing
- `rayon`: Parallel processing
- `tiny_http`: Embedded web server
- `rfd`: GUI file picker (optional, behind `gui` feature)
