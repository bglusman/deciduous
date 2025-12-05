# Start Interactive Web UI

Launch the losselot web server for interactive analysis with visualizations.

## Instructions

1. Build the project if needed:
   ```bash
   cargo build --release
   ```

2. Start the server. Use the path provided by the user, or default to current directory:
   ```bash
   cargo run --release -- serve <path> --port 3000
   ```

3. Inform the user:
   - The server is running at http://localhost:3000
   - They can browse and analyze files interactively
   - The UI shows spectrograms, bitrate timelines, and detection details
   - They can adjust thresholds in real-time

4. The server will run in the background. Remind user to stop it when done (Ctrl+C).

## UI Features
- File browser for the served directory
- Real-time analysis on file selection
- Interactive spectrogram visualization
- Bitrate timeline graphs
- Threshold adjustment sliders
- Detection flag explanations

$ARGUMENTS
