#!/usr/bin/env bash
set -euo pipefail

TARGET_TRIPLE="${TARGET_TRIPLE:-x86_64-unknown-linux-musl}"
DIST_DIR="./dist"
ARTIFACT_PATH="${DIST_DIR}/rust_hltb_bot.zip"

if [ "${TARGET_TRIPLE}" = "x86_64-unknown-linux-musl" ]; then
  export AR_x86_64_unknown_linux_musl=ar
fi

mkdir -p "${DIST_DIR}"
rm -f "${ARTIFACT_PATH}"

cargo build --release --target "${TARGET_TRIPLE}"
zip -j "${ARTIFACT_PATH}" "./target/${TARGET_TRIPLE}/release/bootstrap"
