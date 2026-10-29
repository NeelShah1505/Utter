# Releasing Utter

1.0 ships one artifact: `app-macos-arm64.dmg` (macOS, Apple Silicon). The release
runs on a pushed semver tag (`v*.*.*`) via `.github/workflows/build-release.yml`.

## Why signing matters

Without a Developer ID signature and Apple notarization, macOS Gatekeeper blocks a
normal double-click of the downloaded app. The user must right-click → Open and
confirm a scary dialog. For a Hacker News launch where strangers install it, that
friction is a launch blocker (finding F-014). The steps below make the dmg
double-clickable on a clean machine.

If the secrets are not set (e.g. a fork, or before you obtain a certificate), the
build still succeeds with ad-hoc signing — it just will not pass Gatekeeper.

## One-time prerequisites

1. An Apple Developer account ($99/year).
2. A **Developer ID Application** certificate (NOT "Apple Distribution" — that is
   for the App Store). Create it in the Apple Developer portal → Certificates, or
   from Xcode → Settings → Accounts → Manage Certificates → + → Developer ID
   Application.
3. An **app-specific password** for your Apple ID, created at
   <https://appleid.apple.com> → Sign-In and Security → App-Specific Passwords.
   This is used for notarization, not your real Apple ID password.
4. Your **Team ID** (10 characters), shown in the Apple Developer portal under
   Membership.

## Export the certificate for CI

Export the Developer ID Application cert **with its private key** from Keychain
Access as a `.p12` file, set a password on it, then base64-encode it:

```sh
base64 -i DeveloperID.p12 | pbcopy   # now in your clipboard
```

## GitHub secrets to create

In the repo: Settings → Secrets and variables → Actions → New repository secret.
Create each of these (the workflow already references them):

| Secret | Value |
|--------|-------|
| `APPLE_CERTIFICATE` | the base64 string from the step above |
| `APPLE_CERTIFICATE_PASSWORD` | the password you set on the `.p12` |
| `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: Your Name (TEAMID)` — the exact name as shown by `security find-identity -v -p codesigning` |
| `APPLE_ID` | your Apple ID email |
| `APPLE_PASSWORD` | the app-specific password (not your login password) |
| `APPLE_TEAM_ID` | your 10-character Team ID |

Do not commit any of these values. None are invented in this repo.

## Cut a release

```sh
git tag v0.1.0        # or v0.1.0-rc.1 for a prerelease (the `-` marks it prerelease)
git push origin v0.1.0
```

CI builds, signs, notarizes, staples, renames the dmg to `app-macos-arm64.dmg`,
computes `app-macos-arm64.dmg.sha256`, and publishes a GitHub Release. The site's
download button resolves to that asset.

## Verify the signed build (do this once, on a clean machine)

```sh
# After downloading the released dmg on a Mac that never built Utter:
spctl -a -vvv -t install app-macos-arm64.dmg      # expect: accepted, source=Notarized Developer ID
codesign -dv --verbose=4 /Applications/Utter.app  # expect: Authority=Developer ID Application...
xcrun stapler validate /Applications/Utter.app    # expect: The validate action worked
```

## Known caveats to confirm on the first notarized build (UNVERIFIED here)

- **Entitlements / hardened runtime.** `entitlements.plist` currently contains a
  sandbox-only `com.apple.security.temporary-exception.mach-lookup.global-name`
  entry. The app is not sandboxed (no `com.apple.security.app-sandbox`), so that
  entry is inert. Synthetic keyboard input (text insertion) needs the user's
  Accessibility grant in System Settings, not an entitlement. Microphone needs the
  `NSMicrophoneUsageDescription` string (present in `Info.plist`). If notarization
  rejects an entitlement, remove the mach-lookup exception first.
- **Tauri updater.** Not configured. If you add auto-update later, generate an
  updater keypair (`cargo tauri signer generate`) and add the public key to
  `tauri.conf.json` and the private key as a secret. Out of scope for 1.0.
- **Keychain items** saved by an ad-hoc dev build are not readable by a signed
  build (different code-signing identity). This only affects cleanup API keys you
  saved while developing; re-enter them once on the signed build (finding F-007).
