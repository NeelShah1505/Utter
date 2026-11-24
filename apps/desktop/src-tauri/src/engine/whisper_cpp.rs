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

/// whisper.cpp refuses anything under 1.0 s. Pad short recordings to 1.5 s.
const MIN_INFERENCE_SAMPLES: usize = (WHISPER_SAMPLE_RATE as usize * 3) / 2;
/// Silence placed before a short recording so its first sound is not clipped.
const LEAD_IN_SAMPLES: usize = WHISPER_SAMPLE_RATE as usize / 5;

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
        self.transcribe_16k(&audio, None)
    }
}

impl WhisperCppEngine {
    /// Run one inference over 16 kHz mono samples and return the text.
    ///
    /// Dictation calls this from `end_session`; file transcription calls it
    /// directly with a progress callback (whole percentages from whisper.cpp).
    pub fn transcribe_16k(
        &self,
        audio: &[f32],
        on_progress: Option<Box<dyn FnMut(i32) + Send + 'static>>,
    ) -> Result<String, EngineError> {
        let Some(prepared) = prepare_audio(audio) else {
            log::info!("whisper_cpp: no speech in the recording; nothing to transcribe");
            return Ok(String::new());
        };
        let segments = self.run_inference(&prepared, on_progress)?;
        Ok(assemble_transcript(&segments))
    }

    /// One whisper.cpp pass. Returns each segment with the model's own
    /// confidence in it (mean probability of its text tokens).
    fn run_inference(
        &self,
        audio: &[f32],
        on_progress: Option<Box<dyn FnMut(i32) + Send + 'static>>,
    ) -> Result<Vec<Segment>, EngineError> {
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
        // Never emit sound descriptions such as "(clapping)" or "[MUSIC]":
        // this is dictation, and they would be typed into the document.
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);
        if !self.prompt_hint.is_empty() {
            params.set_initial_prompt(&self.prompt_hint);
        }
        if let Some(cb) = on_progress {
            params.set_progress_callback_safe(cb);
        }

        state
            .full(params, audio)
            .map_err(|e| EngineError::Session(format!("whisper inference failed: {e}")))?;

        let n = state
            .full_n_segments()
            .map_err(|e| EngineError::Session(e.to_string()))?;
        let mut out = Vec::with_capacity(n.max(0) as usize);
        for i in 0..n {
            let Ok(text) = state.full_get_segment_text_lossy(i) else {
                continue;
            };
            let mut sum = 0.0f32;
            let mut count = 0u32;
            for t in 0..state.full_n_tokens(i).unwrap_or(0) {
                // Skip control tokens like [_BEG_] and timestamps.
                let is_text = state
                    .full_get_token_text_lossy(i, t)
                    .map(|s| !s.starts_with("[_"))
                    .unwrap_or(false);
                if is_text {
                    if let Ok(p) = state.full_get_token_prob(i, t) {
                        sum += p;
                        count += 1;
                    }
                }
            }
            out.push(Segment {
                text,
                confidence: if count > 0 { sum / count as f32 } else { 0.0 },
            });
        }
        Ok(out)
    }
}

/// One decoded segment and how sure the model was of it (0..1).
#[derive(Debug, Clone)]
struct Segment {
    text: String,
    confidence: f32,
}

/// Audio ready for the model, or `None` when there is no speech in it.
///
/// Two things Whisper gets wrong on its own are handled here, before inference:
/// - handed silence or noise it invents words, so audio without speech never
///   reaches it (see `logic::has_speech`);
/// - whisper.cpp discards input shorter than one second without a word of
///   warning, so a quick "Yes." would come back empty. Short recordings get a
///   little silence in front and are padded out behind.
fn prepare_audio(audio: &[f32]) -> Option<std::borrow::Cow<'_, [f32]>> {
    if !crate::logic::has_speech(audio, WHISPER_SAMPLE_RATE) {
        return None;
    }
    if audio.len() >= MIN_INFERENCE_SAMPLES {
        return Some(std::borrow::Cow::Borrowed(audio));
    }
    let mut padded = Vec::with_capacity(MIN_INFERENCE_SAMPLES + LEAD_IN_SAMPLES);
    padded.resize(LEAD_IN_SAMPLES, 0.0);
    padded.extend_from_slice(audio);
    let target = padded.len().max(MIN_INFERENCE_SAMPLES);
    padded.resize(target, 0.0);
    Some(std::borrow::Cow::Owned(padded))
}

/// A transcript this short, with the model this unsure, is far more likely an
/// invention than something the user said. Measured on this model: real
/// one-word utterances ("No.", "Yes.", "Stop.") score 0.56–1.00 across five
/// voices; the word it invents for silence ("you") scores 0.23.
const DOUBTFUL_MAX_WORDS: usize = 3;
const DOUBTFUL_CONFIDENCE: f32 = 0.40;

/// Join segments into the final transcript, dropping what is not speech.
fn assemble_transcript(segments: &[Segment]) -> String {
    let joined = segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join("");
    let text = crate::logic::strip_sound_annotations(&joined);
    if text.is_empty() {
        return text;
    }

    // Confidence of the whole transcript, weighted by how much text each
    // segment contributed.
    let (mut weighted, mut weight) = (0.0f32, 0.0f32);
    for s in segments {
        let w = s.text.trim().chars().count() as f32;
        weighted += s.confidence * w;
        weight += w;
    }
    let confidence = if weight > 0.0 { weighted / weight } else { 0.0 };

    if text.split_whitespace().count() <= DOUBTFUL_MAX_WORDS && confidence < DOUBTFUL_CONFIDENCE {
        log::info!(
            "whisper_cpp: dropped a {}-word transcript at confidence {confidence:.2}",
            text.split_whitespace().count()
        );
        return String::new();
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(text: &str, confidence: f32) -> Segment {
        Segment {
            text: text.to_string(),
            confidence,
        }
    }

    /// Speech-shaped test audio: five 150 ms tone bursts over quiet room noise.
    fn speechlike(seconds: f32) -> Vec<f32> {
        let rate = WHISPER_SAMPLE_RATE as usize;
        let n = (rate as f32 * seconds) as usize;
        let mut x = 0x2545_f491u32;
        let mut out: Vec<f32> = (0..n)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                0.0008 * ((x as f32 / u32::MAX as f32) * 2.0 - 1.0)
            })
            .collect();
        let burst = rate * 150 / 1000;
        let mut start = rate / 20;
        while start + burst < n {
            for i in 0..burst {
                let t = i as f32 / rate as f32;
                out[start + i] += 0.1 * (2.0 * std::f32::consts::PI * 180.0 * t).sin();
            }
            start += rate / 4;
        }
        out
    }

    #[test]
    fn silence_and_noise_never_reach_the_model() {
        assert!(prepare_audio(&[]).is_none());
        assert!(
            prepare_audio(&vec![0.0; 48_000]).is_none(),
            "digital silence"
        );
        let hum: Vec<f32> = (0..48_000)
            .map(|i| 0.02 * (2.0 * std::f32::consts::PI * 60.0 * i as f32 / 16_000.0).sin())
            .collect();
        assert!(prepare_audio(&hum).is_none(), "steady hum");
    }

    #[test]
    fn short_recordings_are_padded_past_whispers_one_second_minimum() {
        // A quick one-word dictation is about half a second long.
        let word = speechlike(0.5);
        let prepared = prepare_audio(&word).expect("speech must pass the gate");
        assert!(
            prepared.len() >= MIN_INFERENCE_SAMPLES,
            "whisper.cpp silently discards anything under one second"
        );
        // Lead-in silence, then the recording itself, untouched.
        assert!(prepared[..LEAD_IN_SAMPLES].iter().all(|s| *s == 0.0));
        assert_eq!(
            &prepared[LEAD_IN_SAMPLES..LEAD_IN_SAMPLES + word.len()],
            &word[..]
        );

        // Longer recordings are passed through without copying or padding.
        let sentence = speechlike(4.0);
        let prepared = prepare_audio(&sentence).expect("speech must pass the gate");
        assert_eq!(prepared.len(), sentence.len());
        assert!(matches!(prepared, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn transcript_assembly_keeps_speech_and_drops_inventions() {
        // Real short utterances, at the confidences measured for them.
        assert_eq!(assemble_transcript(&[seg(" Yes.", 0.80)]), "Yes.");
        assert_eq!(assemble_transcript(&[seg(" No.", 0.56)]), "No.");
        assert_eq!(
            assemble_transcript(&[seg(" Thank you.", 0.88)]),
            "Thank you."
        );

        // What the model invents for silence, at the confidence measured for it.
        assert_eq!(assemble_transcript(&[seg(" you", 0.23)]), "");

        // Sound descriptions are never text, however sure the model is.
        assert_eq!(assemble_transcript(&[seg(" (machine whirring)", 0.95)]), "");
        assert_eq!(assemble_transcript(&[seg(" [BLANK_AUDIO]", 0.99)]), "");

        // Segments join into one transcript.
        assert_eq!(
            assemble_transcript(&[seg(" Book an appointment", 0.9), seg(" for 10 p.m.", 0.9)]),
            "Book an appointment for 10 p.m."
        );

        // A longer transcript is never dropped for low confidence: unsure is
        // not the same as invented, and losing a sentence is worse.
        let long = " remind me to send the report before friday";
        assert_eq!(assemble_transcript(&[seg(long, 0.2)]), long.trim());

        assert_eq!(assemble_transcript(&[]), "");
    }

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

    /// Probe: what does the model do with a set of clips, and how sure is it?
    /// Prints the speech gate verdict, every raw segment with its confidence
    /// (bypassing the gate), and the final transcript. Used to choose the
    /// hallucination defenses from data. Ignored by default:
    ///   UTTER_TEST_MODEL=... UTTER_PROBE_WAVS=a.wav:b.wav \
    ///     cargo test --bin utter -- --ignored --nocapture whisper_probe
    #[test]
    #[ignore]
    fn whisper_probe() {
        let model = std::env::var("UTTER_TEST_MODEL").expect("set UTTER_TEST_MODEL");
        let wavs = std::env::var("UTTER_PROBE_WAVS").expect("set UTTER_PROBE_WAVS");
        let engine = WhisperCppEngine::init(&EngineConfig {
            model_path: model,
            prompt_hint: std::env::var("UTTER_TEST_HINT").unwrap_or_default(),
        })
        .expect("init engine");
        for wav in wavs.split(':').filter(|w| !w.is_empty()) {
            let (mono, rate) = load_wav_mono(wav);
            let audio = resample_to_16k(&mono, rate);
            let gate = crate::logic::has_speech(&audio, WHISPER_SAMPLE_RATE);
            let raw = engine.run_inference(&audio, None).expect("inference");
            let raw_txt: Vec<String> = raw
                .iter()
                .map(|s| format!("{:?}@{:.2}", s.text.trim(), s.confidence))
                .collect();
            // What the model says on the prepared (gated + padded) path.
            let seen: Vec<String> = match prepare_audio(&audio) {
                Some(a) => engine
                    .run_inference(&a, None)
                    .expect("inference")
                    .iter()
                    .map(|s| format!("{:?}@{:.2}", s.text.trim(), s.confidence))
                    .collect(),
                None => vec!["(gated)".into()],
            };
            let fin = engine.transcribe_16k(&audio, None).expect("transcribe");
            let name = std::path::Path::new(wav)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            eprintln!(
                "PROBE {name:<22} gate={gate:<5} model=[{}] final={fin:?} ungated=[{}]",
                seen.join(", "),
                raw_txt.join(", ")
            );
        }
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
