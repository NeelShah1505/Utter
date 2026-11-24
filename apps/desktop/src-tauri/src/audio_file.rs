// audio_file.rs — Transcribe a recording from disk.
//
// Decodes common audio files (WAV, MP3, M4A/AAC, ALAC, FLAC, OGG Vorbis, AIFF,
// CAF) to the 16 kHz mono f32 that Whisper wants, then runs one inference.
// Everything happens on this machine: the file is read, decoded and
// transcribed locally, and nothing is uploaded.
//
// Pure Rust decoding via symphonia, so there is no ffmpeg to ship or trust.

use crate::engine::whisper_cpp::WhisperCppEngine;
use crate::engine::{AsrEngine, EngineConfig, EngineError};
use std::borrow::Cow;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub const TARGET_RATE: u32 = 16_000;

/// Two hours of 16 kHz mono f32 is about 460 MB of samples. Refuse more than
/// that rather than exhaust memory on a mistakenly dropped file.
pub const MAX_SECONDS: f32 = 2.0 * 60.0 * 60.0;

#[derive(Debug, thiserror::Error)]
pub enum AudioFileError {
    #[error("Could not open that file: {0}")]
    Open(String),
    #[error("That file type is not supported. Use WAV, MP3, M4A, AAC, FLAC, OGG, AIFF or CAF.")]
    Unsupported,
    #[error("That file has no audio in it.")]
    NoAudio,
    #[error("That recording is longer than two hours. Split it and try again.")]
    TooLong,
    #[error("Could not decode the audio: {0}")]
    Decode(String),
}

pub struct DecodedAudio {
    /// 16 kHz mono samples.
    pub samples: Vec<f32>,
    /// Duration of the original recording.
    pub seconds: f32,
}

pub struct FileTranscript {
    pub text: String,
    pub audio_seconds: f32,
}

/// Decode `path` to 16 kHz mono f32.
pub fn decode_to_16k_mono(path: &Path) -> Result<DecodedAudio, AudioFileError> {
    let file = std::fs::File::open(path).map_err(|e| AudioFileError::Open(e.to_string()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|_| AudioFileError::Unsupported)?;
    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or(AudioFileError::NoAudio)?;
    let track_id = track.id;
    let mut rate = track.codec_params.sample_rate.unwrap_or(0);
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|_| AudioFileError::Unsupported)?;

    let mut mono: Vec<f32> = Vec::new();
    let mut scratch: Option<SampleBuffer<f32>> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            // End of stream is reported as an unexpected EOF.
            Err(SymError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            // A chained stream starts over; the first logical stream is enough.
            Err(SymError::ResetRequired) => break,
            Err(e) => return Err(AudioFileError::Decode(e.to_string())),
        };
        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            // One damaged packet should not sink the whole recording.
            Err(SymError::DecodeError(_)) => continue,
            Err(SymError::IoError(_)) => break,
            Err(e) => return Err(AudioFileError::Decode(e.to_string())),
        };

        let spec = *decoded.spec();
        // Trust the decoder, not the container. An ALAC .m4a can declare
        // 44.1 kHz in its header while the decoder produces 22.05 kHz; using the
        // header rate would play the recording at double speed into the model.
        rate = spec.rate;
        let channels = spec.channels.count().max(1);
        let needed = decoded.capacity() * channels;
        if scratch
            .as_ref()
            .map(|b| b.capacity() < needed)
            .unwrap_or(true)
        {
            scratch = Some(SampleBuffer::<f32>::new(decoded.capacity() as u64, spec));
        }
        let buf = scratch.as_mut().expect("scratch buffer was just created");
        buf.copy_interleaved_ref(decoded);

        for frame in buf.samples().chunks(channels) {
            mono.push(frame.iter().sum::<f32>() / channels as f32);
        }

        if rate > 0 && mono.len() as f32 / rate as f32 > MAX_SECONDS {
            return Err(AudioFileError::TooLong);
        }
    }

    if mono.is_empty() || rate == 0 {
        return Err(AudioFileError::NoAudio);
    }

    let seconds = mono.len() as f32 / rate as f32;
    Ok(DecodedAudio {
        samples: resample(&mono, rate, TARGET_RATE),
        seconds,
    })
}

/// Resample mono audio.
///
/// When downsampling, a moving average over the decimation window runs first.
/// It is a cheap low-pass: without it, everything above the new Nyquist
/// frequency (8 kHz for Whisper) folds back into the speech band as noise.
/// Linear interpolation then picks the output samples.
pub fn resample(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || input.is_empty() || from == 0 || to == 0 {
        return input.to_vec();
    }

    let window = (from as f64 / to as f64).round() as usize;
    let filtered: Cow<[f32]> = if from > to && window > 1 {
        let mut out = Vec::with_capacity(input.len());
        // f64 accumulator: a running sum over millions of f32 samples drifts.
        let mut acc = 0.0f64;
        for i in 0..input.len() {
            acc += input[i] as f64;
            if i >= window {
                acc -= input[i - window] as f64;
            }
            out.push((acc / (i + 1).min(window) as f64) as f32);
        }
        Cow::Owned(out)
    } else {
        Cow::Borrowed(input)
    };

    let ratio = to as f64 / from as f64;
    let out_len = ((filtered.len() as f64) * ratio).round() as usize;
    let last = filtered.len() - 1;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f64 / ratio;
        let i0 = (src.floor() as usize).min(last);
        let i1 = (i0 + 1).min(last);
        let frac = (src - i0 as f64) as f32;
        out.push(filtered[i0] * (1.0 - frac) + filtered[i1] * frac);
    }
    out
}

/// Decode `path` and transcribe it with a Whisper model.
///
/// `model_path` may be empty: the engine then uses any installed model.
/// `on_progress` receives whole percentages from whisper.cpp as it works.
pub fn transcribe_path(
    path: &Path,
    model_path: &str,
    prompt_hint: &str,
    on_progress: Option<Box<dyn FnMut(i32) + Send + 'static>>,
) -> Result<FileTranscript, String> {
    let decoded = decode_to_16k_mono(path).map_err(|e| e.to_string())?;

    let engine = WhisperCppEngine::init(&EngineConfig {
        model_path: model_path.to_string(),
        prompt_hint: prompt_hint.to_string(),
    })
    .map_err(|e| match e {
        EngineError::ModelNotFound(_) => {
            "Transcribing a file needs a Whisper model. Download one in Settings → Transcription engine, then try again.".to_string()
        }
        other => other.to_string(),
    })?;

    let text = engine
        .transcribe_16k(&decoded.samples, on_progress)
        .map_err(|e| e.to_string())?;

    Ok(FileTranscript {
        text,
        audio_seconds: decoded.seconds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file path inside a directory that no other test shares. A counter
    /// (not just the clock) keeps parallel tests apart: two tests can start in
    /// the same microsecond, and each removes its own directory when done.
    fn temp_file(name: &str) -> std::path::PathBuf {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("utter-audio-test-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn resample_scales_length_and_keeps_level() {
        // A constant signal must stay constant through the low-pass + interpolation.
        let input = vec![0.5f32; 48_000];
        let out = resample(&input, 48_000, 16_000);
        assert!((out.len() as i64 - 16_000).abs() <= 1);
        assert!(out.iter().skip(8).all(|s| (s - 0.5).abs() < 1e-4));

        // Same rate and empty input pass straight through.
        assert_eq!(resample(&[0.1, 0.2], 16_000, 16_000), vec![0.1, 0.2]);
        assert!(resample(&[], 48_000, 16_000).is_empty());

        // Upsampling doubles the length.
        assert_eq!(resample(&[0.0, 1.0, 0.0, 1.0], 8_000, 16_000).len(), 8);
    }

    #[test]
    fn resample_suppresses_content_above_nyquist() {
        // 20 kHz tone at 48 kHz is far above the 8 kHz Nyquist of 16 kHz output.
        // Without a low-pass it aliases into the speech band at full strength.
        let n = 48_000;
        let tone: Vec<f32> = (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * 20_000.0 * i as f32 / 48_000.0).sin())
            .collect();
        let out = resample(&tone, 48_000, 16_000);
        let rms = (out.iter().map(|s| s * s).sum::<f32>() / out.len() as f32).sqrt();
        assert!(
            rms < 0.35,
            "alias energy should be well below the 0.707 input rms, got {rms}"
        );
    }

    #[test]
    fn decodes_stereo_wav_to_16k_mono() {
        let path = temp_file("stereo.wav");
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44_100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&path, spec).unwrap();
        for _ in 0..44_100 {
            w.write_sample((0.25 * 32767.0) as i16).unwrap(); // left
            w.write_sample((0.75 * 32767.0) as i16).unwrap(); // right
        }
        w.finalize().unwrap();

        let decoded = decode_to_16k_mono(&path).unwrap();
        assert!(
            (decoded.seconds - 1.0).abs() < 0.01,
            "seconds = {}",
            decoded.seconds
        );
        assert!((decoded.samples.len() as i64 - 16_000).abs() <= 2);
        // Downmix is the average of the two channels.
        let mid = decoded.samples[8_000];
        assert!((mid - 0.5).abs() < 0.01, "mono level = {mid}");

        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    #[test]
    fn rejects_files_that_are_not_audio() {
        let path = temp_file("notes.wav");
        std::fs::write(
            &path,
            b"this is not audio, it is a text file with a wav name",
        )
        .unwrap();
        assert!(decode_to_16k_mono(&path).is_err());

        let missing = path.with_file_name("missing.m4a");
        assert!(matches!(
            decode_to_16k_mono(&missing),
            Err(AudioFileError::Open(_))
        ));
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    /// End-to-end: decode a real recording and transcribe it. Ignored by
    /// default (needs a model and an audio file):
    ///   UTTER_TEST_MODEL=.../ggml-base.en.bin UTTER_TEST_AUDIO=.../clip.m4a \
    ///     cargo test --bin utter -- --ignored --nocapture transcribes_a_recording
    #[test]
    #[ignore]
    fn transcribes_a_recording() {
        let model = std::env::var("UTTER_TEST_MODEL").expect("set UTTER_TEST_MODEL");
        let audio = std::env::var("UTTER_TEST_AUDIO").expect("set UTTER_TEST_AUDIO");
        let hint = std::env::var("UTTER_TEST_HINT").unwrap_or_default();
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<i32>::new()));
        let sink = seen.clone();
        let t0 = std::time::Instant::now();
        let out = transcribe_path(
            Path::new(&audio),
            &model,
            &hint,
            Some(Box::new(move |p| sink.lock().unwrap().push(p))),
        )
        .expect("transcribe");
        eprintln!("TRANSCRIPT: {}", out.text);
        eprintln!("AUDIO_SEC: {:.2}", out.audio_seconds);
        eprintln!("ELAPSED_MS: {}", t0.elapsed().as_millis());
        eprintln!("PROGRESS: {:?}", seen.lock().unwrap());
        assert!(!out.text.trim().is_empty());
    }
}
