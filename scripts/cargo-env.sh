#!/usr/bin/env bash
# Shared cargo environment for all worktrees.
# Sourced by mise.toml [env]._.source — sets CARGO_TARGET_DIR to a single
# directory under the project root so dependency artifacts are compiled once.
# See .wiki/Incremental-Build-Performance.md → Strategy 3.

PROJECT_ROOT="$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")"
export CARGO_TARGET_DIR="$PROJECT_ROOT/.cargo/targets"
