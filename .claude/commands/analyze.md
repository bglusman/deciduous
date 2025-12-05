# Analyze Audio Files

Run losselot analysis on audio files to detect transcoding.

## Instructions

1. Build the project in release mode if not already built:
   ```bash
   cargo build --release
   ```

2. Run analysis on the target specified by the user. If no target specified, use `examples/demo_files/` or ask the user.

3. Parse the output and explain:
   - The verdict (OK, SUSPECT, TRANSCODE)
   - The score breakdown (binary vs spectral)
   - Key flags detected
   - What the flags mean for this specific file

4. If the user wants more detail, suggest using `--no-spectral` for faster binary-only analysis or generating an HTML report with `-o report.html`.

## Common options
- `--no-spectral` - Skip FFT analysis (faster)
- `--threshold N` - Custom transcode threshold (default: 65)
- `-o file.html` - Generate HTML report
- `-o file.json` - Generate JSON output
- `--jobs N` - Parallel jobs

$ARGUMENTS
