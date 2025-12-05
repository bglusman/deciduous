#!/bin/bash
#
# Generate Test MP3 Files for Losselot Demo
#
# This script creates various MP3 files with different encoding scenarios
# to demonstrate Losselot's detection capabilities:
#
#   1. Clean files (legitimate encodes from lossless)
#   2. Transcoded files (128kbps source re-encoded as 320kbps)
#   3. Re-encoded files (multiple LAME passes)
#   4. Mixed encoder chain (LAME → FFmpeg)
#   5. YouTube-style rips (low quality source upscaled)
#
# Requirements: ffmpeg, lame, sox
#
# Usage: ./generate_test_files.sh
#

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/demo_files"

log() { echo -e "${GREEN}[+]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[!]${NC} $1"; }

check_deps() {
    local missing=()
    command -v ffmpeg >/dev/null 2>&1 || missing+=("ffmpeg")
    command -v lame >/dev/null 2>&1 || missing+=("lame")

    if [[ ${#missing[@]} -gt 0 ]]; then
        error "Missing dependencies: ${missing[*]}"
        echo "Install with: brew install ${missing[*]}"
        exit 1
    fi
}

# Generate a test tone WAV file (source material)
generate_source_wav() {
    local output="$1"
    local duration="${2:-10}"

    log "Generating source WAV with full frequency content..."

    # Use ffmpeg to generate a complex test signal with:
    # - Multiple sine waves at different frequencies
    # - White noise for broadband content
    # - This simulates music-like frequency content
    ffmpeg -y \
        -f lavfi -i "sine=frequency=440:duration=$duration" \
        -f lavfi -i "sine=frequency=880:duration=$duration" \
        -f lavfi -i "sine=frequency=1760:duration=$duration" \
        -f lavfi -i "sine=frequency=3520:duration=$duration" \
        -f lavfi -i "sine=frequency=7040:duration=$duration" \
        -f lavfi -i "sine=frequency=14080:duration=$duration" \
        -f lavfi -i "anoisesrc=d=$duration:c=pink:a=0.1" \
        -filter_complex "[0][1][2][3][4][5][6]amix=inputs=7:duration=first[out]" \
        -map "[out]" \
        -ar 44100 \
        -sample_fmt s16 \
        "$output" 2>/dev/null

    log "Source WAV created: $output"
}

# 1. Clean legitimate 320kbps encode
create_clean_320() {
    local source="$1"
    local output="$OUTPUT_DIR/01_clean_320kbps.mp3"

    log "Creating clean 320kbps encode..."
    lame --preset insane -q 0 "$source" "$output" 2>/dev/null
    log "Created: $output"
}

# 2. Clean legitimate 192kbps VBR V0
create_clean_v0() {
    local source="$1"
    local output="$OUTPUT_DIR/02_clean_v0_vbr.mp3"

    log "Creating clean V0 VBR encode..."
    lame -V 0 -q 0 "$source" "$output" 2>/dev/null
    log "Created: $output"
}

# 3. Transcode: 128kbps source re-encoded as 320kbps (CLASSIC FAKE)
create_transcode_128_to_320() {
    local source="$1"
    local temp_128="$OUTPUT_DIR/.temp_128.mp3"
    local temp_wav="$OUTPUT_DIR/.temp_decoded.wav"
    local output="$OUTPUT_DIR/03_FAKE_128_to_320.mp3"

    log "Creating TRANSCODE: 128kbps → WAV → 320kbps..."

    # Step 1: Encode to 128kbps (creates 16kHz lowpass)
    lame -b 128 -q 5 "$source" "$temp_128" 2>/dev/null

    # Step 2: Decode back to WAV
    ffmpeg -y -i "$temp_128" "$temp_wav" 2>/dev/null

    # Step 3: Re-encode as "320kbps" - but lowpass is still 16kHz!
    lame --preset insane -q 0 "$temp_wav" "$output" 2>/dev/null

    rm -f "$temp_128" "$temp_wav"
    log "Created: $output (TRANSCODE - lowpass reveals 128kbps source)"
}

# 4. Re-encoded multiple times with LAME
create_multiple_lame_passes() {
    local source="$1"
    local temp1="$OUTPUT_DIR/.temp_pass1.mp3"
    local temp2="$OUTPUT_DIR/.temp_pass2.wav"
    local temp3="$OUTPUT_DIR/.temp_pass2.mp3"
    local temp4="$OUTPUT_DIR/.temp_pass3.wav"
    local output="$OUTPUT_DIR/04_REENCODED_3x_lame.mp3"

    log "Creating RE-ENCODED file: 3 LAME passes..."

    # Pass 1: Initial encode at 256kbps
    lame -b 256 -q 2 "$source" "$temp1" 2>/dev/null

    # Pass 2: Decode and re-encode at 320kbps
    ffmpeg -y -i "$temp1" "$temp2" 2>/dev/null
    lame --preset insane "$temp2" "$temp3" 2>/dev/null

    # Pass 3: Decode and re-encode again at 320kbps
    ffmpeg -y -i "$temp3" "$temp4" 2>/dev/null
    lame --preset insane "$temp4" "$output" 2>/dev/null

    rm -f "$temp1" "$temp2" "$temp3" "$temp4"
    log "Created: $output (3 LAME signatures detectable)"
}

# 5. Mixed encoder chain: LAME → FFmpeg
create_lame_to_ffmpeg_chain() {
    local source="$1"
    local temp_lame="$OUTPUT_DIR/.temp_lame.mp3"
    local temp_wav="$OUTPUT_DIR/.temp_ffmpeg.wav"
    local output="$OUTPUT_DIR/05_CHAIN_lame_to_ffmpeg.mp3"

    log "Creating ENCODING CHAIN: LAME → FFmpeg..."

    # Step 1: Encode with LAME
    lame -V 2 "$source" "$temp_lame" 2>/dev/null

    # Step 2: Decode
    ffmpeg -y -i "$temp_lame" "$temp_wav" 2>/dev/null

    # Step 3: Re-encode with FFmpeg (leaves Lavf signature)
    ffmpeg -y -i "$temp_wav" -codec:a libmp3lame -b:a 320k "$output" 2>/dev/null

    rm -f "$temp_lame" "$temp_wav"
    log "Created: $output (LAME + FFmpeg signatures)"
}

# 6. YouTube-style rip (128kbps AAC equivalent quality)
create_youtube_style() {
    local source="$1"
    local temp_aac="$OUTPUT_DIR/.temp_aac.m4a"
    local temp_wav="$OUTPUT_DIR/.temp_youtube.wav"
    local output="$OUTPUT_DIR/06_YOUTUBE_style_rip.mp3"

    log "Creating YOUTUBE-STYLE rip (128kbps AAC → 320kbps MP3)..."

    # Step 1: Encode as low-quality AAC (simulates YouTube audio)
    ffmpeg -y -i "$source" -c:a aac -b:a 128k "$temp_aac" 2>/dev/null

    # Step 2: Decode
    ffmpeg -y -i "$temp_aac" "$temp_wav" 2>/dev/null

    # Step 3: Re-encode as "high quality" MP3
    lame --preset insane "$temp_wav" "$output" 2>/dev/null

    rm -f "$temp_aac" "$temp_wav"
    log "Created: $output (low quality source upscaled)"
}

# 7. Multiple FFmpeg passes
create_multiple_ffmpeg_passes() {
    local source="$1"
    local temp1="$OUTPUT_DIR/.temp_ff1.mp3"
    local temp2="$OUTPUT_DIR/.temp_ff2.mp3"
    local output="$OUTPUT_DIR/07_FFMPEG_processed_3x.mp3"

    log "Creating file with multiple FFmpeg processings..."

    # Pass 1
    ffmpeg -y -i "$source" -codec:a libmp3lame -b:a 256k "$temp1" 2>/dev/null

    # Pass 2 (re-encode)
    ffmpeg -y -i "$temp1" -codec:a libmp3lame -b:a 320k "$temp2" 2>/dev/null

    # Pass 3 (re-encode again)
    ffmpeg -y -i "$temp2" -codec:a libmp3lame -b:a 320k "$output" 2>/dev/null

    rm -f "$temp1" "$temp2"
    log "Created: $output (multiple FFmpeg Lavf signatures)"
}

# 8. Borderline suspect (192kbps source to 256kbps)
create_borderline_suspect() {
    local source="$1"
    local temp_192="$OUTPUT_DIR/.temp_192.mp3"
    local temp_wav="$OUTPUT_DIR/.temp_border.wav"
    local output="$OUTPUT_DIR/08_SUSPECT_192_to_256.mp3"

    log "Creating borderline SUSPECT file (192 → 256kbps)..."

    # Encode at 192kbps
    lame -b 192 "$source" "$temp_192" 2>/dev/null

    # Decode and re-encode at 256kbps
    ffmpeg -y -i "$temp_192" "$temp_wav" 2>/dev/null
    lame -b 256 "$temp_wav" "$output" 2>/dev/null

    rm -f "$temp_192" "$temp_wav"
    log "Created: $output (borderline quality loss)"
}

# 9. Multi-codec "laundering" chain: WAV → MP3 128k → AAC → OGG → MP3 320k
# Simulates someone trying to hide source quality by converting through formats
create_multi_codec_laundering() {
    local source="$1"
    local temp_mp3_128="$OUTPUT_DIR/.temp_launder_mp3.mp3"
    local temp_wav1="$OUTPUT_DIR/.temp_launder_wav1.wav"
    local temp_aac="$OUTPUT_DIR/.temp_launder_aac.m4a"
    local temp_wav2="$OUTPUT_DIR/.temp_launder_wav2.wav"
    local temp_ogg="$OUTPUT_DIR/.temp_launder_ogg.ogg"
    local temp_wav3="$OUTPUT_DIR/.temp_launder_wav3.wav"
    local output="$OUTPUT_DIR/09_LAUNDERED_mp3_aac_ogg_mp3.mp3"

    log "Creating LAUNDERED file: WAV → MP3 128k → AAC → OGG → MP3 320k..."

    # Step 1: Encode to low-quality MP3 (128kbps) - this is the "source"
    lame -b 128 -q 5 "$source" "$temp_mp3_128" 2>/dev/null

    # Step 2: Decode and convert to AAC (someone "upgrading" to AAC)
    ffmpeg -y -i "$temp_mp3_128" "$temp_wav1" 2>/dev/null
    ffmpeg -y -i "$temp_wav1" -c:a aac -b:a 256k "$temp_aac" 2>/dev/null

    # Step 3: Decode AAC and convert to OGG Vorbis (more "upgrading")
    ffmpeg -y -i "$temp_aac" "$temp_wav2" 2>/dev/null
    ffmpeg -y -i "$temp_wav2" -c:a libvorbis -q:a 6 "$temp_ogg" 2>/dev/null

    # Step 4: Decode OGG and "upgrade" to 320kbps MP3 (final "high quality" output)
    ffmpeg -y -i "$temp_ogg" "$temp_wav3" 2>/dev/null
    lame --preset insane "$temp_wav3" "$output" 2>/dev/null

    rm -f "$temp_mp3_128" "$temp_wav1" "$temp_aac" "$temp_wav2" "$temp_ogg" "$temp_wav3"
    log "Created: $output (4 lossy encodings through 3 codecs - quality destroyed)"
}

main() {
    echo ""
    echo "╔════════════════════════════════════════════════════════════════════╗"
    echo "║           Losselot Test File Generator                             ║"
    echo "║           Creating demo files for all detection scenarios          ║"
    echo "╚════════════════════════════════════════════════════════════════════╝"
    echo ""

    check_deps

    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    rm -f "$OUTPUT_DIR"/*.mp3  # Clean old files

    # Generate source material
    SOURCE_WAV="$OUTPUT_DIR/.source.wav"
    generate_source_wav "$SOURCE_WAV" 10

    echo ""
    log "Creating test files..."
    echo ""

    # Create all test scenarios
    create_clean_320 "$SOURCE_WAV"
    create_clean_v0 "$SOURCE_WAV"
    create_transcode_128_to_320 "$SOURCE_WAV"
    create_multiple_lame_passes "$SOURCE_WAV"
    create_lame_to_ffmpeg_chain "$SOURCE_WAV"
    create_youtube_style "$SOURCE_WAV"
    create_multiple_ffmpeg_passes "$SOURCE_WAV"
    create_borderline_suspect "$SOURCE_WAV"
    create_multi_codec_laundering "$SOURCE_WAV"

    # Cleanup
    rm -f "$SOURCE_WAV"
    rm -f "$OUTPUT_DIR"/.temp*

    echo ""
    echo "════════════════════════════════════════════════════════════════════"
    log "All test files created in: $OUTPUT_DIR"
    echo ""
    echo "Test files created:"
    echo ""
    ls -la "$OUTPUT_DIR"/*.mp3 2>/dev/null | awk '{print "  " $NF}' | xargs -I {} basename {}
    echo ""
    echo "Run Losselot GUI to analyze:"
    echo ""
    echo "  ./target/release/losselot --gui"
    echo ""
    echo "Then select the demo_files folder when the file picker opens."
    echo ""
    echo "The HTML report will open automatically showing:"
    echo "  - Encoding chain timeline visualization"
    echo "  - Spectral damage annotations"
    echo "  - Re-encoding detection flags"
    echo ""
    echo "Or double-click the losselot binary to launch GUI mode directly."
    echo ""
}

main "$@"
