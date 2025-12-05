# Add New Detection Method

Guide for implementing a new detection flag or method in losselot.

## Instructions

When the user wants to add a new detection capability:

1. **Understand the detection goal**: Ask what pattern they want to detect (e.g., specific encoder artifact, frequency pattern, metadata anomaly)

2. **Choose the right module**:
   - `src/analyzer/spectral.rs` - For frequency-domain analysis (FFT-based)
   - `src/analyzer/binary.rs` - For metadata/header analysis
   - `src/mp3/lame.rs` - For LAME-specific header parsing
   - `src/mp3/frame.rs` - For MP3 frame-level analysis

3. **Implementation pattern**:
   ```rust
   // In the appropriate analyze() function:

   // 1. Add detection logic
   let detected = /* your detection logic */;

   // 2. Add to flags if detected
   if detected {
       flags.push("your_flag_name".to_string());
   }

   // 3. Contribute to score if appropriate
   if detected {
       score += POINTS_FOR_THIS_DETECTION;
   }
   ```

4. **Update documentation**:
   - Add flag to CLAUDE.md detection flags list
   - Add explanation to HTML report if visualization needed

5. **Add tests**:
   ```rust
   #[test]
   fn test_new_detection() {
       // Test with known positive and negative cases
   }
   ```

6. **Test with real files**:
   - Generate appropriate test files
   - Verify detection accuracy
   - Check for false positives

## Current detection flags for reference
Read `src/analyzer/spectral.rs` and `src/analyzer/binary.rs` for the current flag implementations.

$ARGUMENTS
