---
layout: default
title: Home
---

# Losselot

Detects fake lossless audio files - FLACs and WAVs that were actually made from MP3s.

---

## Try It Now

**[Browser Analyzer](analyzer.html)** - Upload files, analyze in your browser, no server needed

**[Interactive Demo](demo/)** - See real decision graph data from development

---

## What It Does

When someone converts an MP3 to FLAC, the high frequencies that MP3 removed don't come back. Losselot detects this:

1. **Spectral Analysis** - FFT finds where frequencies cut off
2. **Binary Analysis** - Checks for encoder signatures (LAME, FFmpeg, etc.)
3. **Lo-fi Detection** - Distinguishes MP3 cutoff from natural tape rolloff

### Verdicts

| Score | Verdict | Meaning |
|-------|---------|---------|
| 0-34 | OK | Clean file |
| 35-64 | SUSPECT | Possibly transcoded |
| 65-100 | TRANSCODE | Definitely from lossy source |

---

## Install & Run

```bash
git clone https://github.com/notactuallytreyanastasio/losselot.git
cd losselot
cargo build --release

# Analyze files
./target/release/losselot ~/Music/

# Web UI with spectrograms
./target/release/losselot serve ~/Music/ --port 3000
```

---

## Key Detection Flags

**Spectral indicators:**
- `severe_hf_damage` - Major high-frequency loss
- `hf_cutoff_detected` - Sharp frequency cutoff found
- `dead_ultrasonic_band` - No content above 20kHz
- `cfcc_cliff` - Cross-frequency coherence cliff (MP3 signature)

**Re-encoding indicators:**
- `multi_encoder_sigs` - Multiple encoder signatures found
- `encoding_chain(LAME → FFmpeg)` - Detected processing chain
- `lowpass_bitrate_mismatch` - Lowpass doesn't match claimed quality

---

## Also in This Repo

### [Decision Graph](decision-graph)

A queryable record of every decision made during development. Not documentation - actual decision nodes stored in SQLite.

```bash
./losselot db nodes    # See all decisions
./losselot db edges    # See relationships
./losselot db graph    # Export as JSON
```

### [Claude Tooling](claude-tooling)

Slash commands and context recovery for AI-assisted development. The decision graph persists across session boundaries.

### [Development Story](story)

How this tool evolved from a simple FFT check to multi-method analysis with lo-fi detection.

---

## Output Formats

```bash
# Terminal output (default)
./losselot ~/Music/

# JSON for scripting
./losselot ~/Music/ --format json

# HTML report
./losselot ~/Music/ --format html > report.html

# CSV for spreadsheets
./losselot ~/Music/ --format csv > report.csv
```

---

## Architecture

```
src/
├── main.rs         # CLI + parallel execution
├── serve.rs        # Web UI server
├── analyzer/
│   ├── mod.rs      # Score combination
│   ├── spectral.rs # FFT analysis
│   └── binary.rs   # Encoder detection
└── report/         # Output formats
```

Key dependencies: `symphonia` (audio decode), `rustfft` (FFT), `rayon` (parallel), `diesel` (SQLite)

---

## Exit Codes

- `0` - All files clean
- `1` - At least one suspect
- `2` - At least one definite transcode

---

[GitHub Repository](https://github.com/notactuallytreyanastasio/losselot)
