# Losselot

Detect "lossless" audio files that were actually created from lossy sources.

## Quick Start

```bash
# Download for your platform from Releases, then:
chmod +x losselot-*
./losselot-darwin-arm64 examples/suspect.mp3
```

**Output:**
```
[TRANSCODE]  75%   255kbps  LAME3.97      possible_lossy_origin,cliff...  suspect.mp3
```

This example file scores **75% TRANSCODE** - it's a 256kbps MP3 that was encoded from an already-lossy source (~192-224kbps), not from the original WAV. The telltale sign: frequencies cut off at 19kHz instead of the 19.5kHz this bitrate should allow.

### Try It Yourself

```bash
# Analyze a single file
losselot myfile.flac

# Analyze your whole library
losselot ~/Music/

# Verbose output (see spectral details)
losselot -v myfile.flac

# Generate HTML report
losselot -o report.html ~/Music/
```

## The Problem

You download a FLAC or WAV file labeled as "lossless" - but how do you know it wasn't just an MP3 that someone converted? Once audio goes through lossy compression (MP3, AAC, etc.), the lost frequencies are gone forever. Converting to FLAC doesn't bring them back.

**Losselot detects these fake lossless files.**

## How It Works

Lossy codecs like MP3 work by removing high frequencies that are "less audible." A 128kbps MP3 typically cuts everything above ~16kHz. When you convert that MP3 to FLAC, the cutoff remains - it's a permanent scar.

Losselot performs **spectral analysis** to measure energy in different frequency bands:
- Real lossless audio has gradual, natural high-frequency rolloff
- Fake lossless (from MP3/AAC) has a sharp cliff where the original encoder cut frequencies

### What It Detects

| Source | Detection |
|--------|-----------|
| MP3 128kbps → FLAC | Easily detected (hard cutoff at ~16kHz) |
| MP3 192kbps → FLAC | Usually detected (cutoff at ~18kHz) |
| MP3 320kbps → FLAC | Detected via ultrasonic analysis (no content >20kHz) |
| AAC 128kbps → FLAC | Sometimes detected (AAC is more efficient) |
| MP3 → MP3 transcode | Detected via spectral + LAME header analysis |
| Real lossless | Shows 0% score, natural rolloff |

## Installation

**No dependencies required** - binaries are fully self-contained.

### Pre-built Binaries (Recommended)

Download from [Releases](https://github.com/notactuallytreyanastasio/losselot/releases):

| Platform | Binary |
|----------|--------|
| macOS Apple Silicon | `losselot-darwin-arm64` |
| macOS Intel | `losselot-darwin-amd64` |
| Linux x86_64 | `losselot-linux-amd64` |
| Windows x86_64 | `losselot-windows-amd64.exe` |

```bash
# macOS/Linux: make executable and run
chmod +x losselot-*
./losselot-darwin-arm64 --help

# Or move to PATH
sudo mv losselot-darwin-arm64 /usr/local/bin/losselot
losselot --help
```

### From Source

Requires [Rust](https://rustup.rs/) (no other dependencies):

```bash
git clone https://github.com/notactuallytreyanastasio/losselot.git
cd losselot
cargo build --release
./target/release/losselot examples/suspect.mp3
```

### Via Cargo

```bash
cargo install --git https://github.com/notactuallytreyanastasio/losselot
```

## Supported Formats

**Input formats:** FLAC, WAV, AIFF, MP3, M4A, AAC, OGG, Opus, WMA, ALAC

The primary use case is analyzing FLAC/WAV files, but Losselot can also detect MP3→MP3 transcodes using binary forensics (LAME header analysis).

## Understanding Results

### Verdicts

- **OK (0-34%)**: Appears to be genuine lossless
- **SUSPECT (35-64%)**: Might have lossy origins, worth investigating
- **TRANSCODE (65-100%)**: Almost certainly from a lossy source

### Flags

| Flag | Meaning |
|------|---------|
| `severe_hf_damage` | Major high frequency loss (probably from 128kbps or lower) |
| `hf_cutoff_detected` | Clear lossy cutoff pattern detected |
| `possible_lossy_origin` | Mild HF damage, possibly from high-bitrate lossy |
| `cliff_at_20khz` | Sharp cutoff at 20kHz (320kbps MP3 signature) |
| `steep_20khz_cutoff` | Significant drop at 20kHz boundary |
| `possible_320k_origin` | May have originated from 320kbps MP3 |
| `dead_ultrasonic_band` | No content above 20kHz (strong 320k indicator) |
| `weak_ultrasonic_content` | Low energy above 20kHz |
| `steep_hf_rolloff` | High frequencies drop off too sharply |
| `silent_17k+` | Upper frequencies (17-20kHz) are essentially silent |
| `silent_20k+` | Ultrasonic frequencies (20-22kHz) are silent |
| `lowpass_mismatch` | (MP3 only) LAME header lowpass doesn't match bitrate |

### Verbose Output

Use `-v` to see spectral details:
```
Spectral: full=59.1dB high=30.9dB upper=25.6dB ultrasonic=-39.4dB
Drops: upper=11.2dB ultrasonic=44.7dB | flatness_19-21k=0.015
```

Key metrics:
- **upper_drop**: Difference between 10-15kHz and 17-20kHz bands. Real lossless: ~4-8dB. Transcode from 128k: ~40-70dB.
- **ultrasonic_drop**: Difference between 19-20kHz and 20-22kHz. Real lossless: ~1-2dB. 320k transcode: ~40-50dB.
- **flatness**: Content complexity above 20kHz. Real lossless: ~0.8-0.99. 320k transcode: ~0.01-0.1.

## CLI Reference

```
losselot [OPTIONS] <PATH>

Arguments:
  <PATH>  File or directory to analyze

Options:
  -o, --output <FILE>      Output report file (.html, .csv, .json)
  -j, --jobs <NUM>         Number of parallel workers (default: CPU count)
      --no-spectral        Skip spectral analysis (faster, binary-only)
  -v, --verbose            Show detailed analysis
  -q, --quiet              Only show summary
      --threshold <NUM>    Transcode threshold percentage [default: 65]
  -h, --help               Print help
  -V, --version            Print version
```

### Exit Codes

- `0`: All files clean
- `1`: Some files suspect
- `2`: Transcodes detected

## Report Formats

### HTML
Dark-mode report with summary statistics, color-coded verdicts, and flag reference.

### CSV
```csv
verdict,filepath,bitrate_kbps,combined_score,spectral_score,binary_score,flags,encoder,lowpass
TRANSCODE,/path/to/fake.flac,0,80,80,0,severe_hf_damage,,
```

### JSON
```json
{
  "generated": "2024-01-01T00:00:00Z",
  "summary": {"total": 100, "ok": 85, "suspect": 10, "transcode": 5},
  "files": [...]
}
```

## Limitations

- **High-bitrate lossy is harder**: MP3 320kbps has cutoff near 20kHz, but ultrasonic analysis helps
- **Some codecs are stealthier**: AAC and Vorbis are more efficient than MP3, leaving less obvious damage
- **Dark/quiet recordings**: Low energy in high frequencies is normal for some content
- **Not 100% definitive**: Use as one data point, not absolute proof

## License

MIT
