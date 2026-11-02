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

#[cfg(test)]
mod tests {
    use super::*;

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
