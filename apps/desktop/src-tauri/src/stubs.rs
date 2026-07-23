// Suppress dead_code warnings on Phase 2/3/4/5 stub items.
// These are intentional forward declarations — they will be used once the
// corresponding phase is implemented. Suppressing here keeps `cargo check`
// output clean so real warnings don't get lost in noise.
#![allow(dead_code)]
