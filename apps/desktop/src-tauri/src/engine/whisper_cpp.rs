// engine/whisper_cpp.rs — whisper.cpp engine (whisper-rs bindings).
//
// Design: whisper is not a streaming recognizer. Utter already buffers the whole
// utterance (live typing is off by default), so we accumulate audio in
// feed_audio and run one inference in end_session. whisper requires 16 kHz mono
// f32; we resample if the mic delivered another rate. The user's dictionary is
// passed as whisper's initial_prompt so proper nouns and code terms (useState,
// kubectl, snake_case) are recognized during decoding, not patched afterward.

use super::{AsrEngine, EngineConfig, EngineError, SessionId};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const WHISPER_SAMPLE_RATE: u32 = 16_000;

struct Session {
    samples: Vec<f32>,
    sample_rate: u32,
}

pub struct WhisperCppEngine {
    ctx: WhisperContext,
    prompt_hint: String,
    sessions: Mutex<HashMap<SessionId, Session>>,
    next_id: AtomicU64,
}

/// Where the default model lives: ~/Library/Application Support/Utter/models.
pub fn default_model_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("Utter").join("models").join("ggml-base.en.bin"))
}

fn resolve_model_path(configured: &str) -> Result<PathBuf, EngineError> {
    // Explicit path wins.
    if !configured.is_empty() {
        let p = PathBuf::from(configured);
        return if p.exists() {
            Ok(p)
        } else {
            Err(EngineError::ModelNotFound(format!(
                "No speech model at {}. Download a Whisper model in Settings, or switch the engine to Apple.",
                p.display()
            )))
        };
    }

    // No explicit path: prefer base.en, else use any installed ggml-*.bin so a
    // user who only has small/large is not told to download.
    if let Some(base) = default_model_path() {
        if base.exists() {
            return Ok(base);
        }
        if let Some(dir) = base.parent() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                let mut found: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n.starts_with("ggml-") && n.ends_with(".bin"))
                            .unwrap_or(false)
                    })
                    .collect();
                found.sort();
                if let Some(first) = found.into_iter().next() {
                    return Ok(first);
                }
            }
        }
    }
    Err(EngineError::ModelNotFound(
        "No Whisper model installed. Download one in Settings → Transcription engine, or switch to Apple on-device.".into(),
    ))
}

/// Linear-interpolation resample to 16 kHz mono. Good enough for speech; a
/// higher-quality resampler can replace this without touching callers.
fn resample_to_16k(input: &[f32], from_rate: u32) -> Vec<f32> {
    if from_rate == WHISPER_SAMPLE_RATE || input.is_empty() {
        return input.to_vec();
    }
    let ratio = WHISPER_SAMPLE_RATE as f64 / from_rate as f64;
    let out_len = ((input.len() as f64) * ratio).round() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f64 / ratio;
        let i0 = src.floor() as usize;
        let i1 = (i0 + 1).min(input.len() - 1);
        let frac = (src - i0 as f64) as f32;
        out.push(input[i0] * (1.0 - frac) + input[i1] * frac);
    }
    out
}

impl AsrEngine for WhisperCppEngine {
    fn init(config: &EngineConfig) -> Result<Self, EngineError> {
        // Route whisper.cpp's chatty stderr through the `log` crate once, so it
        // obeys our log level instead of spamming the terminal / crash log.
        // (CPU backend: on this hardware it beat Metal for the small default
        // model — Metal's per-call GPU overhead dominated base.en's latency.)
        static LOG_HOOK: std::sync::Once = std::sync::Once::new();
        LOG_HOOK.call_once(whisper_rs::install_logging_hooks);

        let model_path = resolve_model_path(&config.model_path)?;
        let ctx = WhisperContext::new_with_params(
            &model_path.to_string_lossy(),
            WhisperContextParameters::default(),
        )
        .map_err(|e| EngineError::ModelLoad(e.to_string()))?;
        log::info!("whisper_cpp: loaded model {}", model_path.display());
        Ok(Self {
            ctx,
            prompt_hint: config.prompt_hint.clone(),
            sessions: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        })
    }

    fn start_session(
        &self,
        sample_rate: u32,
        _on_partial: Box<dyn Fn(String, bool) + Send + 'static>,
    ) -> Result<SessionId, EngineError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.sessions.lock().expect("sessions poisoned").insert(
            id,
            Session {
                samples: Vec::new(),
                sample_rate,
            },
        );
        log::debug!("whisper_cpp: session {id} started ({sample_rate} Hz)");
        Ok(id)
    }

    fn feed_audio(&self, session: SessionId, samples: &[f32]) -> Result<(), EngineError> {
        let mut map = self.sessions.lock().expect("sessions poisoned");
        let sess = map
            .get_mut(&session)
            .ok_or_else(|| EngineError::Session(format!("unknown session {session}")))?;
        sess.samples.extend_from_slice(samples);
        Ok(())
    }

    fn end_session(&self, session: SessionId) -> Result<String, EngineError> {
        let sess = {
            let mut map = self.sessions.lock().expect("sessions poisoned");
            map.remove(&session)
                .ok_or_else(|| EngineError::Session(format!("unknown session {session}")))?
        };

        let audio = resample_to_16k(&sess.samples, sess.sample_rate);
        if audio.is_empty() {
            return Ok(String::new());
        }

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| EngineError::Session(e.to_string()))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(4);
        params.set_language(Some("en"));
        params.set_translate(false);
        params.set_print_progress(false);
        params.set_print_special(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        if !self.prompt_hint.is_empty() {
            params.set_initial_prompt(&self.prompt_hint);
        }

        state
            .full(params, &audio)
            .map_err(|e| EngineError::Session(format!("whisper inference failed: {e}")))?;

        let n = state
            .full_n_segments()
            .map_err(|e| EngineError::Session(e.to_string()))?;
        let mut text = String::new();
        for i in 0..n {
            if let Ok(seg) = state.full_get_segment_text(i) {
                text.push_str(&seg);
            }
        }
        Ok(text.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_is_identity_at_16k_and_scales_rate() {
        let s = vec![0.1, 0.2, 0.3, 0.4];
        assert_eq!(resample_to_16k(&s, 16_000), s);
        // 48k -> 16k is 1/3 the samples (within rounding).
        let down = resample_to_16k(&vec![0.0; 48_000], 48_000);
        assert!((down.len() as i32 - 16_000).abs() <= 1);
        assert!(resample_to_16k(&[], 48_000).is_empty());
    }

    /// Decode a WAV to mono f32 plus its sample rate (downmixing if stereo).
    fn load_wav_mono(path: &str) -> (Vec<f32>, u32) {
        let mut reader = hound::WavReader::open(path).expect("open wav");
        let spec = reader.spec();
        let raw: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => reader
                .samples::<i16>()
                .map(|s| s.unwrap() as f32 / 32768.0)
                .collect(),
            hound::SampleFormat::Float => reader.samples::<f32>().map(|s| s.unwrap()).collect(),
        };
        let mono = if spec.channels > 1 {
            raw.chunks(spec.channels as usize)
                .map(|c| c.iter().sum::<f32>() / c.len() as f32)
                .collect()
        } else {
            raw
        };
        (mono, spec.sample_rate)
    }

    /// Run one clip through the real engine; returns (transcript, elapsed_ms).
    fn transcribe(engine: &WhisperCppEngine, mono: &[f32], rate: u32) -> (String, u128) {
        let sid = engine
            .start_session(rate, Box::new(|_, _| {}))
            .expect("start");
        engine.feed_audio(sid, mono).expect("feed");
        let t0 = std::time::Instant::now();
        let text = engine.end_session(sid).expect("end");
        (text, t0.elapsed().as_millis())
    }

    /// Offline transcription check against the real engine. Ignored by
    /// default (needs a model + a wav). Run with:
    ///   UTTER_TEST_MODEL=.../ggml-base.en.bin UTTER_TEST_WAV=.../clip.wav \
    ///     cargo test --bin utter -- --ignored --nocapture whisper_transcribes_wav
    #[test]
    #[ignore]
    fn whisper_transcribes_wav() {
        let model = std::env::var("UTTER_TEST_MODEL").expect("set UTTER_TEST_MODEL");
        let wav = std::env::var("UTTER_TEST_WAV").expect("set UTTER_TEST_WAV");
        let (mono, rate) = load_wav_mono(&wav);
        let engine = WhisperCppEngine::init(&EngineConfig {
            model_path: model,
            prompt_hint: std::env::var("UTTER_TEST_HINT").unwrap_or_default(),
        })
        .expect("init engine");
        let (text, ms) = transcribe(&engine, &mono, rate);
        eprintln!("TRANSCRIPT: {text}");
        eprintln!("ELAPSED_MS: {ms}");
        assert!(!text.is_empty(), "transcript should not be empty");
    }

    /// Manifest benchmark: load the model ONCE, run every clip, write JSONL.
    /// Manifest is tab-separated: id, voice, rate, category, wav_path, reference.
    ///   UTTER_TEST_MODEL=... UTTER_BENCH_MANIFEST=manifest.tsv UTTER_BENCH_OUT=out.jsonl \
    ///     cargo test --bin utter -- --ignored --nocapture whisper_bench_manifest
    #[test]
    #[ignore]
    fn whisper_bench_manifest() {
        use std::io::Write;
        let model = std::env::var("UTTER_TEST_MODEL").expect("set UTTER_TEST_MODEL");
        let manifest = std::env::var("UTTER_BENCH_MANIFEST").expect("set UTTER_BENCH_MANIFEST");
        let out_path = std::env::var("UTTER_BENCH_OUT").expect("set UTTER_BENCH_OUT");
        let hint = std::env::var("UTTER_TEST_HINT").unwrap_or_default();

        let t_load = std::time::Instant::now();
        let engine = WhisperCppEngine::init(&EngineConfig {
            model_path: model.clone(),
            prompt_hint: hint,
        })
        .expect("init engine");
        eprintln!("MODEL_LOAD_MS: {}", t_load.elapsed().as_millis());

        let mut out = std::fs::File::create(&out_path).expect("create out");
        let text = std::fs::read_to_string(&manifest).expect("read manifest");
        let mut n = 0;
        for line in text
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 6 {
                eprintln!("skip malformed line: {line}");
                continue;
            }
            let (id, voice, rate_tag, category, wav, reference) =
                (f[0], f[1], f[2], f[3], f[4], f[5]);
            let (mono, rate) = load_wav_mono(wav);
            let audio_sec = mono.len() as f32 / rate as f32;
            let (transcript, ms) = transcribe(&engine, &mono, rate);
            let row = serde_json::json!({
                "id": id, "voice": voice, "rate": rate_tag, "category": category,
                "reference": reference, "transcript": transcript,
                "elapsed_ms": ms, "audio_sec": audio_sec, "model": model,
            });
            writeln!(out, "{row}").expect("write row");
            n += 1;
        }
        eprintln!("BENCH_DONE: {n} clips -> {out_path}");
        assert!(n > 0, "manifest produced no rows");
    }
}
