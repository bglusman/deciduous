//! Spectral analysis of MP3 files
//!
//! Uses FFT to analyze frequency content and detect transcoding:
//! - Measures energy in frequency bands (10-15kHz, 15-20kHz, 17-20kHz)
//! - Transcodes have a characteristic "cliff" where high frequencies die
//! - Legitimate high-bitrate files have gradual rolloff

use rustfft::{num_complex::Complex, FftPlanner};
use serde::Serialize;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

const FFT_SIZE: usize = 8192;
const SAMPLE_RATE: u32 = 44100;

#[derive(Debug, Clone, Default, Serialize)]
pub struct SpectralDetails {
    /// RMS level of full signal (dB)
    pub rms_full: f64,
    /// RMS level of 10-15kHz band (dB)
    pub rms_mid_high: f64,
    /// RMS level of 15-20kHz band (dB)
    pub rms_high: f64,
    /// RMS level of 17-20kHz band (dB)
    pub rms_upper: f64,
    /// RMS level of 19-20kHz band (dB)
    pub rms_19_20k: f64,
    /// RMS level of 20-22kHz band (dB) - ultrasonic, key for 320k detection
    pub rms_ultrasonic: f64,
    /// Drop from full to high band (dB)
    pub high_drop: f64,
    /// Drop from mid-high to upper band (dB)
    pub upper_drop: f64,
    /// Drop from 19-20kHz to 20-22kHz (dB) - key for 320k detection
    pub ultrasonic_drop: f64,
    /// Spectral flatness in 19-21kHz (1.0 = noise-like, 0.0 = tonal/empty)
    pub ultrasonic_flatness: f64,
}

#[derive(Debug, Clone, Default)]
pub struct SpectralResult {
    pub score: u32,
    pub flags: Vec<String>,
    pub details: SpectralDetails,
}

/// Hanning window function
fn hanning_window(size: usize) -> Vec<f64> {
    (0..size)
        .map(|i| {
            0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / (size - 1) as f64).cos())
        })
        .collect()
}

/// Convert linear magnitude to dB
fn to_db(value: f64) -> f64 {
    if value <= 0.0 {
        -96.0
    } else {
        20.0 * value.log10()
    }
}

/// Calculate RMS of a slice
fn rms(samples: &[f64]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = samples.iter().map(|&x| x * x).sum();
    (sum_sq / samples.len() as f64).sqrt()
}

/// Decode audio to PCM samples using symphonia (supports MP3, FLAC, WAV, OGG, etc.)
fn decode_audio(data: &[u8]) -> Option<(Vec<f64>, u32)> {
    let cursor = std::io::Cursor::new(data.to_vec());
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

    // Don't provide a hint - let symphonia auto-detect the format
    let hint = Hint::new();

    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .ok()?;

    let mut format = probed.format;
    let track = format.default_track()?;
    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(SAMPLE_RATE);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts)
        .ok()?;

    let mut samples = Vec::new();
    let mut sample_buf: Option<SampleBuffer<f32>> = None;

    // Decode up to ~15 seconds from middle of file
    let max_samples = (sample_rate as usize) * 15;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if sample_buf.is_none() {
            let spec = *decoded.spec();
            let duration = decoded.capacity() as u64;
            sample_buf = Some(SampleBuffer::new(duration, spec));
        }

        if let Some(ref mut buf) = sample_buf {
            // Get channel count before moving decoded
            let channel_count = decoded.spec().channels.count();
            buf.copy_interleaved_ref(decoded);

            // Convert to mono f64
            for chunk in buf.samples().chunks(channel_count) {
                let mono: f64 = chunk.iter().map(|&s| s as f64).sum::<f64>() / channel_count as f64;
                samples.push(mono);
            }

            if samples.len() >= max_samples {
                break;
            }
        }
    }

    if samples.is_empty() {
        return None;
    }

    Some((samples, sample_rate))
}

/// Calculate spectral flatness (Wiener entropy)
/// Returns 1.0 for white noise, 0.0 for pure tone or silence
fn spectral_flatness(magnitudes: &[f64]) -> f64 {
    if magnitudes.is_empty() {
        return 0.0;
    }

    let n = magnitudes.len() as f64;

    // Geometric mean (via log to avoid underflow)
    let log_sum: f64 = magnitudes.iter().map(|&x| (x + 1e-10).ln()).sum();
    let geo_mean = (log_sum / n).exp();

    // Arithmetic mean
    let arith_mean: f64 = magnitudes.iter().sum::<f64>() / n;

    if arith_mean <= 0.0 {
        return 0.0;
    }

    geo_mean / arith_mean
}

/// Calculate energy in a frequency band using FFT results
fn band_energy(fft_result: &[Complex<f64>], sample_rate: u32, low_hz: u32, high_hz: u32) -> f64 {
    let bin_resolution = sample_rate as f64 / FFT_SIZE as f64;
    let low_bin = (low_hz as f64 / bin_resolution) as usize;
    let high_bin = (high_hz as f64 / bin_resolution).min((FFT_SIZE / 2) as f64) as usize;

    let mut energy = 0.0;
    for bin in low_bin..=high_bin.min(fft_result.len() - 1) {
        let mag = fft_result[bin].norm();
        energy += mag * mag;
    }

    energy.sqrt()
}

/// Perform spectral analysis on MP3 data
pub fn analyze(data: &[u8], _declared_sample_rate: u32) -> SpectralResult {
    let mut result = SpectralResult::default();

    // Decode audio to PCM (supports MP3, FLAC, WAV, OGG, etc.)
    let (samples, sample_rate) = match decode_audio(data) {
        Some(s) => s,
        None => return result,
    };

    if samples.len() < FFT_SIZE {
        return result;
    }

    // Calculate overall RMS
    let rms_full = to_db(rms(&samples));
    result.details.rms_full = rms_full;

    // Set up FFT
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);
    let window = hanning_window(FFT_SIZE);

    // Process overlapping windows and average the results
    let hop_size = FFT_SIZE / 2;
    let num_windows = (samples.len() - FFT_SIZE) / hop_size + 1;

    let mut avg_full = 0.0;
    let mut avg_mid_high = 0.0;
    let mut avg_high = 0.0;
    let mut avg_upper = 0.0;
    let mut avg_19_20k = 0.0;
    let mut avg_ultrasonic = 0.0;

    // For spectral flatness calculation
    let mut ultrasonic_magnitudes: Vec<f64> = Vec::new();

    for i in 0..num_windows {
        let start = i * hop_size;
        let end = start + FFT_SIZE;

        if end > samples.len() {
            break;
        }

        // Apply window and convert to complex
        let mut buffer: Vec<Complex<f64>> = samples[start..end]
            .iter()
            .zip(window.iter())
            .map(|(&s, &w)| Complex::new(s * w, 0.0))
            .collect();

        // Perform FFT
        fft.process(&mut buffer);

        // Calculate band energies (all from FFT for fair comparison)
        avg_full += band_energy(&buffer, sample_rate, 20, 20000); // Full audible range
        avg_mid_high += band_energy(&buffer, sample_rate, 10000, 15000);
        avg_high += band_energy(&buffer, sample_rate, 15000, 20000);
        avg_upper += band_energy(&buffer, sample_rate, 17000, 20000);
        avg_19_20k += band_energy(&buffer, sample_rate, 19000, 20000);
        avg_ultrasonic += band_energy(&buffer, sample_rate, 20000, 22000);

        // Collect magnitudes in 19-21kHz for flatness calculation
        let bin_resolution = sample_rate as f64 / FFT_SIZE as f64;
        let low_bin = (19000.0 / bin_resolution) as usize;
        let high_bin = (21000.0 / bin_resolution).min((FFT_SIZE / 2) as f64) as usize;
        for bin in low_bin..=high_bin.min(buffer.len() - 1) {
            ultrasonic_magnitudes.push(buffer[bin].norm());
        }
    }

    let num_windows = num_windows.max(1) as f64;
    avg_full /= num_windows;
    avg_mid_high /= num_windows;
    avg_high /= num_windows;
    avg_upper /= num_windows;
    avg_19_20k /= num_windows;
    avg_ultrasonic /= num_windows;

    // Convert to dB
    result.details.rms_full = to_db(avg_full);
    result.details.rms_mid_high = to_db(avg_mid_high);
    result.details.rms_high = to_db(avg_high);
    result.details.rms_upper = to_db(avg_upper);
    result.details.rms_19_20k = to_db(avg_19_20k);
    result.details.rms_ultrasonic = to_db(avg_ultrasonic);

    // Calculate drops (positive = high band is quieter, which is normal)
    result.details.high_drop = result.details.rms_full - result.details.rms_high;
    result.details.upper_drop = result.details.rms_mid_high - result.details.rms_upper;
    result.details.ultrasonic_drop = result.details.rms_19_20k - result.details.rms_ultrasonic;

    // Calculate spectral flatness in 19-21kHz range
    // Flatness = geometric_mean / arithmetic_mean (1.0 = white noise, 0.0 = pure tone/silence)
    result.details.ultrasonic_flatness = spectral_flatness(&ultrasonic_magnitudes);

    // Score based on analysis
    // Tuned to detect lossy origins in "lossless" files
    //
    // Key insight: upper_drop (difference between 10-15kHz and 17-20kHz bands)
    // is the most diagnostic metric for lossy damage:
    // - Real lossless: ~4-6 dB (gradual natural rolloff)
    // - Lossy 320k: ~8-12 dB (slight damage)
    // - Lossy 192k: ~12-20 dB (moderate damage)
    // - Lossy 128k MP3: ~40-70 dB (severe damage, hard cutoff)

    // Severe damage - almost certainly from low-bitrate lossy (MP3 128k or worse)
    if result.details.upper_drop > 40.0 {
        result.score += 50;
        result.flags.push("severe_hf_damage".to_string());
    }
    // Significant damage - likely from lossy source (192k or lower)
    else if result.details.upper_drop > 15.0 {
        result.score += 35;
        result.flags.push("hf_cutoff_detected".to_string());
    }
    // Mild damage - possibly from high-bitrate lossy (256k-320k)
    else if result.details.upper_drop > 10.0 {
        result.score += 20;
        result.flags.push("possible_lossy_origin".to_string());
    }

    // === 320k DETECTION ===
    // MP3 320k cuts at ~20kHz, leaving no content above that
    // Real lossless has content extending to 21-22kHz
    //
    // Key metrics from analysis:
    // - Real lossless: ultrasonic_drop ~1-2 dB, flatness ~0.98
    // - Fake 320k: ultrasonic_drop ~50+ dB, flatness ~0.10

    // Massive cliff at 20kHz - strong indicator of 320k transcode
    if result.details.ultrasonic_drop > 40.0 {
        result.score += 35;
        result.flags.push("cliff_at_20khz".to_string());
    } else if result.details.ultrasonic_drop > 25.0 {
        result.score += 25;
        result.flags.push("steep_20khz_cutoff".to_string());
    } else if result.details.ultrasonic_drop > 15.0 {
        result.score += 15;
        result.flags.push("possible_320k_origin".to_string());
    }

    // Low spectral flatness in 19-21kHz = empty/dead band
    // Real audio has noise-like content (flatness ~0.9+)
    // 320k transcode has almost nothing (flatness <0.5)
    if result.details.ultrasonic_flatness < 0.3 {
        result.score += 20;
        result.flags.push("dead_ultrasonic_band".to_string());
    } else if result.details.ultrasonic_flatness < 0.5 {
        result.score += 10;
        result.flags.push("weak_ultrasonic_content".to_string());
    }

    // Steep overall rolloff (full spectrum to 15-20kHz)
    if result.details.high_drop > 48.0 {
        result.score += 15;
        result.flags.push("steep_hf_rolloff".to_string());
    }

    // Silent upper frequencies (absolute check)
    if result.details.rms_upper < -50.0 {
        result.score += 15;
        result.flags.push("silent_17k+".to_string());
    }

    // Very quiet ultrasonic band (absolute check)
    if result.details.rms_ultrasonic < -70.0 {
        result.score += 10;
        result.flags.push("silent_20k+".to_string());
    }

    result
}
