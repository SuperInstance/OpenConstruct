#!/bin/bash
# SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Running cargo test -p openshell-signal-chain"
cargo test -p openshell-signal-chain

echo "==> Running cargo publish --dry-run -p openshell-signal-chain"
if ! cargo publish --dry-run -p openshell-signal-chain; then
  echo "ERROR: dry-run failed. Fix issues before publishing."
  exit 1
fi

echo "==> Publishing to crates.io"
cargo publish -p openshell-signal-chain

echo "==> Done."