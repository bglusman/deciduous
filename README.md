# Losselot

**Find out if your "lossless" audio files are actually lossless.**

Ever downloaded a FLAC or WAV and wondered if it's the real deal, or just an MP3 someone converted? Losselot tells you the truth in seconds.

![Losselot Demo](docs/demo.gif)

---

## Try It Now

Clone and run in 30 seconds:

```bash
git clone https://github.com/notactuallytreyanastasio/losselot.git
cd losselot
cargo build --release
./target/release/losselot serve examples/ --port 3000
```

Open [http://localhost:3000](http://localhost:3000) and explore the interactive UI. The `examples/` folder contains test files demonstrating different encoding scenarios - transcodes, re-encodes, and clean files.

**No test files yet?** Generate them:

```bash
cd examples && ./generate_test_files.sh && cd ..
./target/release/losselot serve examples/ --port 3000
```

---

## Download

| Platform | Download | Notes |
|----------|----------|-------|
| **Mac (Apple Silicon)** | [Download](https://github.com/notactuallytreyanastasio/losselot/releases/latest/download/losselot-darwin-arm64) | M1/M2/M3 Macs |
| **Mac (Intel)** | [Download](https://github.com/notactuallytreyanastasio/losselot/releases/latest/download/losselot-darwin-amd64) | Older Macs |
| **Windows** | [Download](https://github.com/notactuallytreyanastasio/losselot/releases/latest/download/losselot-windows-amd64.exe) | Windows 10/11 |
| **Linux (GUI)** | [Download AppImage](https://github.com/notactuallytreyanastasio/losselot/releases/latest/download/losselot-linux-amd64.AppImage) | Double-click to run |
| **Linux (CLI)** | [Download](https://github.com/notactuallytreyanastasio/losselot/releases/latest/download/losselot-linux-amd64) | Terminal only |

---

## How to Use

### Mac & Windows (GUI)

1. **Download** the file for your system
2. **Double-click** it (Mac users: right-click → Open the first time)
3. **Pick a folder** with your audio files
4. **View the report** that opens in your browser

That's it. No installation, no terminal commands.

### Linux AppImage

1. Download the `.AppImage` file
2. Right-click → Properties → Permissions → "Allow executing as program"
3. Double-click to run

---

## Understanding the Report

### Overview Dashboard

![Dashboard Overview](docs/basics.png)

**At the top:**
- **Summary cards** show how many files are Clean, Suspect, or Transcode at a glance
- **Verdict Distribution** pie chart gives you the big picture of your library's health
- **Score Distribution** bar chart shows every file ranked by how suspicious it is

**The Spectral Waterfall:**
Each row is one of your files. The columns show different frequency ranges:
- **Full** (20Hz-20kHz) - The whole audible spectrum
- **Mid-High** (10-15kHz) - Usually healthy even in lossy files
- **High** (15-20kHz) - Starts showing damage in lower bitrate MP3s
- **Upper** (17-20kHz) - Where medium bitrate damage shows
- **Ultrasonic** (20-22kHz) - The smoking gun for 320kbps detection

**What to look for:** Red/orange dots mark where frequency energy suddenly drops off. Real lossless audio has smooth gradients. Transcoded files have sharp cutoffs - that's the "scar" left by lossy compression.

### Detailed File Analysis

Click any file to see its full breakdown:

![File Analysis](docs/everything.png)

**Frequency Response Curve:**
Shows exactly where the audio cuts off. The pink shaded area is the frequency content. A sharp drop around 17-20kHz with a "-31dB DROP" annotation is the telltale sign of lossy compression. Real lossless files have a gentle, natural rolloff.

**Encoding Chain:**
The visualization shows the file's journey through different encoders. Each lossy pass destroys quality permanently - you can't get quality back by re-encoding at a higher bitrate.

**Spectrogram:**

![Spectrogram](docs/spectro.png)

A time vs. frequency heatmap showing the audio's spectral content over the first ~15 seconds. Brighter colors = more energy. Look for:
- **Horizontal cutoff lines** - Where frequencies suddenly stop (lossy compression damage)
- **Missing high frequencies** - Dark bands at the top indicate missing ultrasonic content
- **Consistent patterns** - Real music has varied spectral content; transcodes often show uniform damage

**Bitrate Timeline:**

![Bitrate Timeline](docs/bitrate_timeline.png)

Shows how bitrate varies over time for MP3 files. Useful for identifying:
- **VBR vs CBR** - VBR files show bitrate fluctuations, CBR shows a flat line
- **Suspicious patterns** - A "320kbps" file with mostly 128kbps frames is fake
- **Re-encoding artifacts** - Unusual bitrate distributions can indicate multiple encoding passes

**Stereo Correlation:**

![Stereo Correlation](docs/stereo_correlate.png)

Measures left/right channel similarity over time. The correlation value ranges from -1.0 to 1.0:
- **1.0 (Mono)** - Identical channels, may indicate fake stereo or mono source
- **0.7-0.95 (Normal Stereo)** - Typical for most music
- **0.3-0.7 (Wide Stereo)** - Significant separation between channels
- **< 0.3 (Very Wide/Phase Issues)** - Unusual, may indicate phase problems

---

## What the Verdicts Mean

| Verdict | Score | What it means |
|---------|-------|---------------|
| **CLEAN** | 0-34% | Looks like genuine lossless. Natural frequency rolloff, content above 20kHz present, single encoding pass. |
| **SUSPECT** | 35-64% | Something's off. Could be from a high-bitrate lossy source (256-320kbps), unusual audio content, or possible re-encoding. Worth investigating. |
| **TRANSCODE** | 65-100% | Almost certainly fake. Clear signs of lossy compression damage, or detected re-encoding chain. The "lossless" file was made from an MP3/AAC. |

---

## Understanding the Flags

Losselot tags files with specific flags to explain exactly what it found:

### Spectral Flags

| Flag | What it means |
|------|---------------|
| `severe_hf_damage` | Major frequency loss (probably from 128kbps or lower source) |
| `hf_cutoff_detected` | Clear lossy cutoff pattern found in high frequencies |
| `weak_ultrasonic_content` | Not enough content above 20kHz for claimed quality |
| `dead_ultrasonic_band` | Virtually no content in ultrasonic range - smoking gun |
| `silent_17k+` | Upper frequencies are basically silent |
| `steep_hf_rolloff` | Unnaturally sharp frequency cutoff |
| `possible_320k_origin` | Pattern matches 320kbps MP3 source |

### Re-encoding Flags

| Flag | What it means |
|------|---------------|
| `multi_encoder_sigs` | Multiple different encoder signatures detected in file |
| `encoding_chain(LAME → FFmpeg)` | File was encoded by LAME, then processed by FFmpeg |
| `lame_reencoded_x2` | LAME encoder signatures found 2 times (re-encoded once) |
| `lame_reencoded_x3` | LAME encoder signatures found 3 times (re-encoded twice) |
| `ffmpeg_processed_x2` | FFmpeg processed this file multiple times |
| `fraunhofer_reencoded_x2` | Fraunhofer encoder signatures found multiple times |

### Binary Analysis Flags

| Flag | What it means |
|------|---------------|
| `lowpass_bitrate_mismatch` | The lowpass frequency doesn't match the claimed bitrate |
| `encoder_quality_mismatch` | Encoder settings don't match file quality |

---

## Supported File Types

Losselot can analyze: **FLAC, WAV, AIFF, MP3, M4A, AAC, OGG, Opus, ALAC**

The main use case is checking FLAC/WAV files, but it can also detect:
- If an MP3 was transcoded from a lower-quality MP3
- If a file has been re-encoded multiple times
- If someone "laundered" audio through multiple format conversions

---

## How Detection Works

### 1. Spectral Analysis

Looks at the actual frequency content of your audio. Lossy codecs like MP3 remove high frequencies to save space:

| Source Quality | Typical Cutoff |
|----------------|----------------|
| 128kbps MP3 | ~16kHz |
| 192kbps MP3 | ~18kHz |
| 256kbps MP3 | ~19kHz |
| 320kbps MP3 | ~20kHz |
| True Lossless | ~22kHz (full range) |

Real lossless audio has content all the way up to 22kHz. When that content is missing, you know something's wrong. The "scars" from lossy compression are permanent - converting an MP3 to FLAC doesn't bring back lost frequencies.

### 2. Binary Analysis (MP3 files)

For MP3 files, Losselot reads the encoder metadata embedded in the file. The LAME encoder stores a "lowpass" value that reveals the original encoding settings.

**The smoking gun:** A "320kbps" MP3 with a lowpass of 16kHz was definitely transcoded from a 128kbps source. The encoder honestly reports what frequencies it kept, even when someone re-encoded to a higher bitrate.

### 3. Lo-Fi / Tape Detection (CFCC)

Not all high-frequency rolloff means lossy compression. Tape recordings, lo-fi productions, and vintage equipment have natural rolloff that shouldn't trigger false positives.

**The Problem:**
- MP3 encoders create a "brick wall" cutoff at a fixed frequency
- Tape/lo-fi has gradual rolloff that varies with dynamics
- Both result in missing high frequencies - but for different reasons

**The Solution: Cross-Frequency Coherence Coefficient (CFCC)**

CFCC measures correlation between adjacent frequency bands:
- **MP3**: Sudden decorrelation at the cutoff frequency (cliff)
- **Tape**: Gradual decorrelation that follows the music's dynamics

```
MP3 @ 160kbps:     Tape/Lo-Fi:
     |                    \
     |                     \
     |____                  \___
    16kHz                   varies
```

**Scoring:**
| Signal | Score Impact |
|--------|--------------|
| `cfcc_cliff` detected | +25 (strong transcode indicator) |
| `decorrelation_spike` | +15 (supports transcode) |
| `lofi_safe_natural_rolloff` | -15 (reduces false positives) |

**Known Codec Cutoffs:**
| Bitrate | Typical Cutoff |
|---------|----------------|
| 64-96 kbps | 10.5-12 kHz |
| 128 kbps | 14-16.5 kHz |
| 192 kbps | 16.5-18.5 kHz |
| 256 kbps | 18-19.5 kHz |
| 320 kbps | 19.5-21 kHz |

### 4. Mixed-Source Detection (Future)

Some productions mix lossy and lossless sources - for example, a beat made with MP3 samples plus live recording. The spectrogram shows "pillars" of high-frequency content punching through an otherwise limited bandwidth:

```
22kHz  |    ██     ██  ██      |  <- Lossless elements (cymbals, synths)
       |    ██     ██  ██      |
16kHz  |████████████████████████|  <- MP3 sample baseline
       |████████████████████████|
0 Hz   |████████████████████████|
       0:00              4:00
```

This indicates the final file contains pre-lossy source material regardless of its current format.

### 5. Re-encoding Detection

Losselot scans for multiple encoder signatures in the file header:

- **LAME signatures**: Counts occurrences of LAME encoder tags
- **FFmpeg/Lavf signatures**: Detects FFmpeg processing
- **Fraunhofer signatures**: Detects Fraunhofer/FhG encoder
- **Other encoders**: GOGO, BladeEnc, Shine, Helix, iTunes
- **Mixed chains**: Identifies when files pass through different encoders

**Why this matters:** Each lossy encoding pass causes cumulative damage. A file encoded at 128kbps, then "upgraded" to 320kbps, then converted to FLAC still only has 128kbps worth of actual audio quality.

---

## Command Line (Advanced)

For power users who prefer the terminal:

```bash
# Analyze a folder
./losselot ~/Music/

# Analyze a single file
./losselot suspicious-file.flac

# Interactive web UI
./losselot serve ~/Music/ --port 3000

# Save report to specific location
./losselot -o report.html ~/Downloads/

# Quick scan without spectral analysis
./losselot --no-spectral ~/Music/

# See all options
./losselot --help
```

**Exit codes for scripting:**
- `0` = All files clean
- `1` = Some files suspect
- `2` = Transcodes detected

---

## SQLite Database

Losselot stores analysis results in a SQLite database (`losselot.db`) for tracking, comparison, and historical review.

### Schema Versioning

Each analysis is tagged with a schema version that tracks which detection algorithms were used:

| Version | Name | Features |
|---------|------|----------|
| 1.0.0 | initial | binary_analysis, spectral_analysis |
| 1.1.0 | lofi-detection | + cutoff_variance, rolloff_slope, transition_width, natural_rolloff |

This allows comparing results across algorithm versions and identifying files that need re-analysis.

### Database Tables

```sql
-- Analysis results with all metrics
analysis_results (file_path, verdict, scores, spectral_details, ...)

-- Schema version history
schema_versions (version, name, features, introduced_at)

-- Project tasks for development tracking
project_tasks (plan_name, category, description, status, priority, ...)

-- Research notes
research_notes (topic, content, source, tags)
```

### Programmatic Access

```rust
use losselot::{Database, CURRENT_SCHEMA};

let db = Database::open()?;

// Store analysis result
db.insert_result(&result)?;

// Query results
let transcodes = db.get_results(Some("TRANSCODE"))?;

// Get summary statistics
let summary = db.get_summary()?;
println!("{} files analyzed, {} transcodes", summary.total, summary.transcode_count);
```

---

## Decision Graph

Losselot includes a built-in decision tracking system for documenting algorithm choices, research findings, and development rationale. This serves as living documentation that explains *why* things work the way they do.

![Decision Graph Overview](knowledge_graph.png)

### How It Works

The decision graph is a directed acyclic graph (DAG) stored in SQLite that tracks the evolution of detection algorithms:

**Node Types:**
| Type | Color | Purpose |
|------|-------|---------|
| **Goal** | Green | High-level objectives (e.g., "Improve lo-fi detection") |
| **Decision** | Yellow | Choice points with multiple approaches |
| **Option** | Cyan | Possible approaches to consider |
| **Action** | Red | Implemented changes with commit references |
| **Outcome** | Purple | Results of actions (success/failure/learning) |
| **Observation** | Gray | Data points, findings, technical notes |

**Edge Types:**
- `leads_to` (gray) - Natural progression between nodes
- `chosen` (green) - Selected option from a decision
- `rejected` (red dashed) - Option that was not selected, with rationale
- `requires` - Dependency relationship
- `blocks` / `enables` - Impediments or enablers

### Viewing the Graph

```bash
# Web UI (interactive, with tooltips)
make db-view
# Or manually:
./target/release/losselot serve . --port 3001
# Then open http://localhost:3001/graph

# CLI commands
make db-nodes    # List all nodes
make db-edges    # List all edges
make db-graph    # Full graph as JSON
```

### Adding to the Graph

```bash
# Create nodes
make goal T="Detect cassette tape artifacts"
make decision T="Choose detection algorithm"
make option T="Spectral slope analysis"
make action T="Implemented in commit abc123"
make outcome T="Passes 95% of test cases"
make obs T="MP3 cutoff is always brick-wall"

# Link nodes (FROM → TO)
make link FROM=1 TO=2                    # Basic link
make link FROM=2 TO=3 TYPE=chosen        # Mark as chosen
make link FROM=2 TO=4 TYPE=rejected REASON="Too complex"

# Update status
make status ID=1 S=completed
```

### Example: Lo-Fi Detection Decision

![Decision Node](decision.png)

The graph documents how we chose CFCC (Cross-Frequency Coherence) over Temporal Cutoff Variance:

1. **Goal**: Test lo-fi detection on charlie.flac
2. **Decision**: Choose approach for distinguishing MP3 brick-wall from tape rolloff
3. **Options**:
   - Approach A: Temporal Cutoff Variance (rejected - more complex)
   - Approach B: CFCC (chosen - works with existing FFT structure)
4. **Action**: Implemented CFCC in commit aa464b6
5. **Outcome**: Passes 157 tests, detects 25/29 transcodes

![Action Node](action.png)

### Why This Matters

Audio forensics involves many judgment calls. The decision graph:
- **Documents rationale** so future contributors understand *why* not just *what*
- **Tracks experiments** that didn't work (equally valuable)
- **Links code to reasoning** via commit references
- **Preserves institutional knowledge** that would otherwise be lost

---

## Build from Source

```bash
# Requires Rust (https://rustup.rs)
git clone https://github.com/notactuallytreyanastasio/losselot.git
cd losselot
cargo build --release
./target/release/losselot
```

### Generate Test Files

Want to test the detection? Generate sample files with various encoding scenarios:

```bash
cd examples
./generate_test_files.sh
```

This creates files demonstrating:
- Clean 320kbps and V0 VBR encodes
- 128kbps transcoded to 320kbps (classic fake)
- Multiple LAME re-encoding passes
- LAME → FFmpeg encoding chains
- YouTube-style rips (AAC → MP3)
- Multi-codec laundering (MP3 → AAC → OGG → MP3)

---

## License

MIT - Do whatever you want with it.
