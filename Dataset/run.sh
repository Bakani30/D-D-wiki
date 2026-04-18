#!/usr/bin/env bash
set -e
DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"
exec "$DIR/.venv/bin/python" prepare_lexicon.py "$@"
