#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_DIR/tmp"
OUTPUT_FILE="$OUTPUT_DIR/screenshot.png"
WINDOW_TITLE="Sharaku"

mkdir -p "$OUTPUT_DIR"

case "$(uname -s)" in
  Linux)
    if ! command -v xdotool >/dev/null 2>&1; then
      echo "Error: xdotool is not installed." >&2
      echo "Install with: sudo apt install xdotool" >&2
      exit 2
    fi
    if ! command -v import >/dev/null 2>&1; then
      echo "Error: ImageMagick (import) is not installed." >&2
      echo "Install with: sudo apt install imagemagick" >&2
      exit 2
    fi

    WINDOW_ID=$(xdotool search --name "$WINDOW_TITLE" 2>/dev/null | head -n 1)
    if [ -z "$WINDOW_ID" ]; then
      echo "Error: Window '$WINDOW_TITLE' not found." >&2
      echo "Make sure the app is running with: pnpm run tauri dev" >&2
      exit 1
    fi

    import -window "$WINDOW_ID" "$OUTPUT_FILE"
    ;;

  Darwin)
    WINDOW_ID=$(osascript -e "
      tell application \"System Events\"
        repeat with proc in every process
          repeat with w in every window of proc
            if name of w contains \"$WINDOW_TITLE\" then
              return id of w
            end if
          end repeat
        end repeat
      end tell
      return \"\"
    " 2>/dev/null || true)

    if [ -z "$WINDOW_ID" ]; then
      echo "Error: Window '$WINDOW_TITLE' not found." >&2
      echo "Make sure the app is running with: pnpm run tauri dev" >&2
      exit 1
    fi

    screencapture -l "$WINDOW_ID" "$OUTPUT_FILE"
    ;;

  *)
    echo "Error: Unsupported OS: $(uname -s)" >&2
    exit 2
    ;;
esac

echo "Screenshot saved to $OUTPUT_FILE"
