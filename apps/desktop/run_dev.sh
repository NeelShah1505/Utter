#!/bin/bash
# (App is already built, skipping build to prevent macOS permission resets)

# Ensure the log file exists
touch /tmp/utter.log
echo "--- Starting Utter ---" > /tmp/utter.log

# Launch the app properly through macOS LaunchServices (required for Speech Recognition TCC prompts)
# Use absolute path to avoid CWD issues
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
open -n "$DIR/src-tauri/target/debug/bundle/macos/Utter.app" --stdout /tmp/utter.log --stderr /tmp/utter.log

# Ensure the app is killed when the user presses Ctrl+C
trap "pkill -f 'Utter.app/Contents/MacOS/utter'; exit" SIGINT SIGTERM EXIT

# Stream the logs to the terminal
echo "App launched. Streaming logs..."
tail -f /tmp/utter.log
