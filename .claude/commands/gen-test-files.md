# Generate Test Audio Files

Generate test audio files with various encoding scenarios for testing losselot detection.

## Instructions

1. Check if required tools are installed:
   ```bash
   which ffmpeg lame sox
   ```

2. If tools are missing, inform the user they need:
   - `ffmpeg` - For audio conversion
   - `lame` - For MP3 encoding
   - `sox` - For audio generation

3. Run the test file generator:
   ```bash
   ./examples/generate_test_files.sh
   ```

4. Explain what files were created:
   - **Clean files**: Legitimate encodes at various bitrates
   - **Transcoded files**: Low bitrate → high bitrate (detectable)
   - **Re-encoded files**: Multiple encoding passes
   - **Chain files**: Multiple different encoders (LAME → FFmpeg)

5. Suggest running analysis on the generated files to verify detection.

## Generated file types
- `clean_320.mp3` - Clean 320kbps encode
- `clean_v0.mp3` - Clean V0 VBR encode
- `transcoded_128_to_320.mp3` - 128kbps transcoded to 320kbps
- `reencoded_x2.mp3` - Double LAME encoding
- `chain_lame_ffmpeg.mp3` - LAME → FFmpeg chain
- `youtube_style.mp3` - AAC → MP3 (YouTube rip simulation)

$ARGUMENTS
