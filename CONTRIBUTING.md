# Contributing

Utter is a macOS Apple Silicon dictation app. Patches that claim Windows or Intel support need a speech engine that runs there, and a CI job that builds a real installer. Do not upload a placeholder file with an installer name.

## Before you open a pull request

- `cargo test --bin utter --manifest-path apps/desktop/src-tauri/Cargo.toml`
- `node --test site/assets/*.test.js`
- If you change `apps/desktop/src/assets/*.js`, run `npm run build` in `apps/desktop` and commit the bundles. The app loads the bundles, not the source files.

## Product rules

- Core dictation does not make a network call. On-device recognition stays required.
- Do not log transcript text or API keys.
- Do not put secrets in `config.toml`, `localStorage`, or the repo.
- Do not advertise a platform the app cannot dictate on.

## Reporting security issues

Use GitHub private vulnerability reporting on `NeelShah1505/Utter`. Do not file a public issue for a vulnerability.
