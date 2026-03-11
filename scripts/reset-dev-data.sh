#!/usr/bin/env bash
set -euo pipefail

case "$(uname)" in
  Darwin) DEV_DATA_DIR="$HOME/Library/Application Support/com.sharaku.viewer.dev" ;;
  *)      DEV_DATA_DIR="$HOME/.local/share/com.sharaku.viewer.dev" ;;
esac

if [ ! -d "$DEV_DATA_DIR" ]; then
  echo "Dev data directory does not exist: $DEV_DATA_DIR"
  echo "Nothing to reset."
  exit 0
fi

rm -f "$DEV_DATA_DIR/sharaku.db" "$DEV_DATA_DIR/sharaku.db-shm" "$DEV_DATA_DIR/sharaku.db-wal"
echo "Deleted dev database files from $DEV_DATA_DIR"
