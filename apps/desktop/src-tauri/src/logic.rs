// logic.rs — Pure functions for dictation text handling.
//
// No I/O. These are the pieces that can delete user text or inject
// keystrokes, so they are unit-tested here.

/// How to turn `old` (already typed) into `new` with backspaces + a suffix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditPlan {
    pub backspaces: usize,
    pub suffix: String,
}

/// Longest common char prefix, then backspaces for the rest of `old`.
pub fn plan_edit(old: &str, new: &str) -> EditPlan {
    if old == new {
        return EditPlan {
            backspaces: 0,
            suffix: String::new(),
        };
    }

    let mut prefix = 0;
    let mut old_chars = old.chars();
    let mut new_chars = new.chars();
    while let (Some(a), Some(b)) = (old_chars.next(), new_chars.next()) {
        if a == b {
            prefix += 1;
        } else {
            break;
        }
    }

    let old_len = old.chars().count();
    let suffix: String = new.chars().skip(prefix).collect();
    EditPlan {
        backspaces: old_len.saturating_sub(prefix),
        suffix,
    }
}

/// Strip terminal escapes and control characters before synthetic typing.
///
/// Newlines become spaces. A Return key in Slack, Messages, or a shell
/// can send or execute. Utter types a single line.
pub fn sanitize_insertion(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for n in chars.by_ref() {
                    if n.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        if c == '\n' || c == '\r' || c == '\t' {
            if !out.ends_with(' ') {
                out.push(' ');
            }
            continue;
        }
        if c.is_control() {
            continue;
        }
        out.push(c);
    }
    out.trim().to_string()
}

/// Whole-token dictionary replacement. Match is ASCII case-insensitive.
/// The token may contain letters, digits, underscore, and apostrophe.
pub fn apply_dictionary(text: &str, rules: &[(String, String)]) -> String {
    if text.is_empty() || rules.is_empty() {
        return text.to_string();
    }

    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if is_word_char(c) {
            let start = i;
            i += 1;
            while i < chars.len() && is_word_char(chars[i]) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if let Some((_, repl)) = rules
                .iter()
                .find(|(from, _)| from.eq_ignore_ascii_case(&word))
            {
                out.push_str(repl);
            } else {
                out.push_str(&word);
            }
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '\''
}

/// App kind used only to bias the optional cleanup prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppKind {
    Code,
    Chat,
    Email,
    Pro,
}

impl AppKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AppKind::Code => "code",
            AppKind::Chat => "chat",
            AppKind::Email => "email",
            AppKind::Pro => "pro",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AppKind::Code => "Code Mode",
            AppKind::Chat => "Casual Chat",
            AppKind::Email => "Professional Email",
            AppKind::Pro => "Pro Document",
        }
    }

    pub fn icon(self) -> &'static str {
        self.as_str()
    }

    /// Cleanup instruction. Empty when smart routing is off — caller uses a generic prompt.
    pub fn cleanup_directive(self) -> &'static str {
        match self {
            AppKind::Code => " Format the text cleanly for code comments or technical prose. Suppress filler and preserve technical terms and casing.",
            AppKind::Chat => " Format the text as a short chat message. Do not add a greeting or a sign-off.",
            AppKind::Email => " Format the text as a clear professional note. Do not invent recipients, dates, or facts that were not spoken.",
            AppKind::Pro => " Fix grammar, remove filler words, and keep the original meaning.",
        }
    }
}

pub fn classify_app(bundle_id: &str, name: &str) -> AppKind {
    let bundle = bundle_id.to_lowercase();
    let name = name.to_lowercase();

    if bundle.contains("vscode")
        || bundle.contains("cursor")
        || bundle.contains("xcode")
        || bundle.contains("sublime")
        || bundle.contains("intellij")
        || bundle.contains("pycharm")
        || bundle.contains("terminal")
        || bundle.contains("iterm")
        || bundle.contains("ghostty")
        || bundle.contains("alacritty")
        || bundle.contains("kitty")
        || name.contains("cursor")
        || name.contains("code")
        || name.contains("terminal")
    {
        AppKind::Code
    } else if bundle.contains("slack")
        || bundle.contains("discord")
        || bundle.contains("whatsapp")
        || bundle.contains("telegram")
        || bundle.contains("messages")
        || bundle.contains("signal")
    {
        AppKind::Chat
    } else if bundle.contains("mail")
        || bundle.contains("outlook")
        || bundle.contains("superhuman")
        || bundle.contains("spark")
    {
        AppKind::Email
    } else {
        AppKind::Pro
    }
}

/// Loudness of a chunk of microphone samples as 0..1, for the live waveform.
///
/// RMS in dBFS, mapped so room tone (-60 dB) reads as 0 and firm speech
/// (-12 dB) reads as 1. Only this one number leaves the audio thread: the UI
/// never sees the samples themselves.
pub fn audio_level(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let mean_sq = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
    let rms = mean_sq.sqrt();
    if !rms.is_finite() || rms <= 0.0 {
        return 0.0;
    }
    let db = 20.0 * rms.log10();
    ((db + 60.0) / 48.0).clamp(0.0, 1.0)
}

/// True when mono audio contains something speech-like: a sustained run of
/// frames clearly above the noise floor.
///
/// Whisper does not answer silence with silence. Given an empty recording it
/// invents words ("you", "Thank you."), and given room tone or key clicks it
/// describes them ("(machine whirring)", "(clapping)"). In a dictation app
/// those would be typed into the user's document, so audio with no speech in
/// it must never reach the model.
///
/// The test is deliberately conservative — it should never reject real speech:
/// - frames are 20 ms; a frame counts when it is above an adaptive threshold;
/// - the threshold sits 9 dB over the noise floor (10th-percentile frame), but
///   never below -52 dBFS, and the floor is capped at -45 dBFS so continuous
///   loud speech cannot raise the bar against itself;
/// - speech needs 100 ms of consecutive frames, which a key click (a few
///   milliseconds) cannot produce;
/// - speech is never steady: its loud and quiet frames differ by well over
///   10 dB. A signal whose 90th- and 10th-percentile frames are within 6 dB
///   of each other is a fan, a hum or hiss, however loud it is.
pub fn has_speech(samples: &[f32], sample_rate: u32) -> bool {
    const FRAME_MS: usize = 20;
    const MIN_RUN_FRAMES: usize = 5;
    const ABS_FLOOR_DB: f32 = -52.0;
    const ABOVE_NOISE_DB: f32 = 9.0;
    const NOISE_CAP_DB: f32 = -45.0;
    const MIN_SPREAD_DB: f32 = 6.0;

    let frame = (sample_rate as usize * FRAME_MS / 1000).max(1);
    let levels: Vec<f32> = samples
        .chunks_exact(frame)
        .map(|c| {
            let mean_sq = c.iter().map(|s| s * s).sum::<f32>() / c.len() as f32;
            if mean_sq.is_finite() && mean_sq > 0.0 {
                10.0 * mean_sq.log10()
            } else {
                -120.0
            }
        })
        .collect();
    if levels.len() < MIN_RUN_FRAMES {
        return false;
    }

    let mut sorted = levels.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p10 = sorted[sorted.len() / 10];
    let p90 = sorted[(sorted.len() * 9 / 10).min(sorted.len() - 1)];
    if p90 - p10 < MIN_SPREAD_DB {
        return false; // stationary: noise, not speech
    }
    let threshold = (p10.min(NOISE_CAP_DB) + ABOVE_NOISE_DB).max(ABS_FLOOR_DB);

    let mut run = 0;
    for level in levels {
        if level > threshold {
            run += 1;
            if run >= MIN_RUN_FRAMES {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

/// Remove sound descriptions that a speech model sometimes emits in place of
/// words — "[BLANK_AUDIO]", "(clapping)", "*sighs*", "♪" — and tidy the spacing.
/// Nobody dictates those; they must never be typed.
pub fn strip_sound_annotations(text: &str) -> String {
    const MARKS: [char; 9] = ['[', ']', '(', ')', '*', '♪', '♫', '♬', '♩'];
    let tidy = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");

    let mut out = String::with_capacity(text.len());
    let mut depth_square = 0usize;
    let mut depth_round = 0usize;
    let mut in_star = false;
    let mut stray = false;
    for c in text.chars() {
        match c {
            '[' => depth_square += 1,
            ']' if depth_square > 0 => depth_square -= 1,
            '(' => depth_round += 1,
            ')' if depth_round > 0 => depth_round -= 1,
            ']' | ')' => stray = true,
            '*' => in_star = !in_star,
            '♪' | '♫' | '♬' | '♩' => {}
            _ if depth_square == 0 && depth_round == 0 && !in_star => out.push(c),
            _ => {}
        }
    }

    // Marks that do not pair up are stray characters, not annotations. Drop the
    // marks but keep the words, rather than swallow the rest of the sentence.
    if stray || depth_square > 0 || depth_round > 0 || in_star {
        let kept: String = text.chars().filter(|c| !MARKS.contains(c)).collect();
        return tidy(&kept);
    }
    tidy(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_level_maps_silence_to_zero_and_speech_toward_one() {
        assert_eq!(audio_level(&[]), 0.0);
        assert_eq!(audio_level(&[0.0; 512]), 0.0);

        let tone = |amp: f32| -> Vec<f32> {
            (0..1600)
                .map(|i| amp * (2.0 * std::f32::consts::PI * 220.0 * i as f32 / 16_000.0).sin())
                .collect()
        };
        let quiet = audio_level(&tone(0.004));
        let normal = audio_level(&tone(0.05));
        let loud = audio_level(&tone(0.6));
        assert!(quiet > 0.0 && quiet < 0.25, "quiet = {quiet}");
        assert!(normal > quiet && normal < 1.0, "normal = {normal}");
        assert_eq!(loud, 1.0);

        // Never NaN or out of range, whatever the input.
        let weird = audio_level(&[f32::NAN, 1.0, -1.0]);
        assert!((0.0..=1.0).contains(&weird));
    }

    /// Deterministic noise in -1..1 so the tests need no rand crate.
    fn noise(n: usize, amp: f32, seed: u32) -> Vec<f32> {
        let mut x = seed.max(1);
        (0..n)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                amp * ((x as f32 / u32::MAX as f32) * 2.0 - 1.0)
            })
            .collect()
    }

    /// Something speech-shaped: bursts of tone (syllables) with gaps between.
    fn syllables(amp: f32) -> Vec<f32> {
        let rate = 16_000usize;
        let mut out = noise(rate * 2, 0.0008, 3); // quiet room
        for syl in 0..5 {
            let start = rate / 4 + syl * rate / 4;
            for i in 0..(rate * 150 / 1000) {
                let t = i as f32 / rate as f32;
                out[start + i] += amp * (2.0 * std::f32::consts::PI * 180.0 * t).sin();
            }
        }
        out
    }

    #[test]
    fn has_speech_rejects_silence_room_tone_hum_and_clicks() {
        let rate = 16_000u32;
        let n = rate as usize * 3;

        assert!(!has_speech(&[], rate));
        assert!(!has_speech(&vec![0.0; n], rate), "digital silence");
        assert!(!has_speech(&noise(n, 0.0026, 7), rate), "room tone");

        let hum: Vec<f32> = (0..n)
            .map(|i| 0.02 * (2.0 * std::f32::consts::PI * 60.0 * i as f32 / rate as f32).sin())
            .collect();
        assert!(!has_speech(&hum, rate), "steady mains hum");
        // A loud desk fan: plenty of energy, none of it speech.
        assert!(!has_speech(&noise(n, 0.035, 5), rate), "loud steady fan");

        // Key clicks: 7 ms bursts, far apart. Loud, but never sustained.
        let mut clicks = noise(n, 0.001, 11);
        for k in 0..8 {
            let start = 2_000 + k * 5_600;
            for (i, s) in noise(112, 0.2, 13 + k as u32).into_iter().enumerate() {
                clicks[start + i] += s;
            }
        }
        assert!(!has_speech(&clicks, rate), "key clicks");
    }

    #[test]
    fn has_speech_accepts_normal_quiet_and_noisy_speech() {
        let rate = 16_000u32;
        assert!(has_speech(&syllables(0.1), rate), "normal speech level");
        assert!(has_speech(&syllables(0.012), rate), "quiet speech");

        // Speech over a loud steady fan.
        let mut noisy = syllables(0.15);
        for (s, n) in noisy.iter_mut().zip(noise(32_000, 0.02, 21)) {
            *s += n;
        }
        assert!(has_speech(&noisy, rate), "speech over fan noise");

        // Continuous loud talking with no real pauses must not raise the bar
        // against itself.
        let continuous: Vec<f32> = (0..32_000)
            .map(|i| {
                let t = i as f32 / 16_000.0;
                let syllable = 0.55 + 0.45 * (2.0 * std::f32::consts::PI * 4.0 * t).sin();
                0.2 * syllable * (2.0 * std::f32::consts::PI * 200.0 * t).sin()
            })
            .collect();
        assert!(has_speech(&continuous, rate), "continuous speech");
    }

    #[test]
    fn strips_sound_annotations_but_keeps_words() {
        assert_eq!(strip_sound_annotations("[BLANK_AUDIO]"), "");
        assert_eq!(strip_sound_annotations("(machine whirring)"), "");
        assert_eq!(strip_sound_annotations(" (clapping) "), "");
        assert_eq!(strip_sound_annotations("♪ ♪"), "");
        assert_eq!(
            strip_sound_annotations("Hello there. [MUSIC] Book it for 10 p.m. *sighs*"),
            "Hello there. Book it for 10 p.m."
        );
        // Plain text is untouched apart from spacing.
        assert_eq!(
            strip_sound_annotations("  Send  the report.  "),
            "Send the report."
        );
        // Unpaired marks are stray characters: keep the words.
        assert_eq!(
            strip_sound_annotations("call f(x and return"),
            "call fx and return"
        );
        assert_eq!(
            strip_sound_annotations("5 * 3 is fifteen"),
            "5 3 is fifteen"
        );
    }

    #[test]
    fn plan_edit_noop_and_append_and_revision() {
        assert_eq!(plan_edit("hi", "hi").backspaces, 0);
        let append = plan_edit("hello", "hello there");
        assert_eq!(append.backspaces, 0);
        assert_eq!(append.suffix, " there");

        let revise = plan_edit("the cat sat", "the dog sat");
        assert_eq!(revise.backspaces, 7); // "cat sat"
        assert_eq!(revise.suffix, "dog sat");
    }

    #[test]
    fn plan_edit_counts_chars_not_bytes() {
        let plan = plan_edit("café", "cafe");
        assert!(plan.backspaces >= 1);
        assert!(!plan.suffix.is_empty() || plan.backspaces > 0);
    }

    #[test]
    fn sanitize_strips_escapes_and_newlines() {
        let raw = "hello\nworld\u{1b}[31mred\u{1b}[0m\tnow";
        assert_eq!(sanitize_insertion(raw), "hello worldred now");
        assert_eq!(sanitize_insertion("  \n  "), "");
    }

    #[test]
    fn dictionary_is_whole_token_and_case_insensitive() {
        let rules = vec![("btw".into(), "by the way".into())];
        assert_eq!(
            apply_dictionary("BTW, see btwx later.", &rules),
            "by the way, see btwx later."
        );
    }

    #[test]
    fn classify_known_apps() {
        assert_eq!(
            classify_app("com.todesktop.230313mzl4w4u92", "Cursor"),
            AppKind::Code
        );
        assert_eq!(
            classify_app("com.tinyspeck.slackmacgap", "Slack"),
            AppKind::Chat
        );
        assert_eq!(classify_app("com.apple.mail", "Mail"), AppKind::Email);
        assert_eq!(classify_app("com.apple.Notes", "Notes"), AppKind::Pro);
        assert_eq!(
            classify_app("com.apple.Terminal", "Terminal"),
            AppKind::Code
        );
    }
}
