#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}"

ARTIFACT_PATH="${ARTIFACT_PATH:-./dist/rust_hltb_bot.zip}"

if ! command -v aws >/dev/null 2>&1; then
  echo "aws CLI is required but was not found in PATH"
  exit 1
fi

aws lambda update-function-code \
  --function-name "hltb_rust" \
  --no-cli-pager \
  --zip-file "fileb://${ARTIFACT_PATH}"



