# Explain Scoring System

Help understand how losselot scores and classifies audio files.

## Instructions

Read the analyzer module to provide accurate, up-to-date information:

1. Read `src/analyzer/mod.rs` to get current threshold values and scoring logic
2. Read `src/analyzer/spectral.rs` for spectral scoring details
3. Read `src/analyzer/binary.rs` for binary scoring details

Then explain to the user:

### Scoring Components

**Binary Analysis (from MP3 metadata):**
- Lowpass filter frequency vs declared bitrate mismatch
- Multiple encoder signatures (LAME, FFmpeg, Fraunhofer)
- Re-encoding detection (same encoder multiple times)
- Frame size variance (CBR vs VBR patterns)

**Spectral Analysis (from FFT):**
- High-frequency energy drops (compare 10-15kHz vs 17-20kHz)
- Ultrasonic content (19-22kHz range)
- Frequency rolloff steepness
- Stereo correlation patterns

**Agreement Bonus:**
- When both binary and spectral methods independently suggest transcoding
- Adds confidence to the verdict

### Verdict Thresholds
- OK: 0-34 points
- SUSPECT: 35-64 points
- TRANSCODE: 65-100 points

If the user has a specific file or score they're confused about, offer to analyze it and break down the scoring.

$ARGUMENTS
