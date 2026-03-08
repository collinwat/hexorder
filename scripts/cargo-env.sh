#!/usr/bin/env bash
# Shared cargo environment for all worktrees.
# Sourced by mise.toml [env]._.source.
#
# Previously set CARGO_TARGET_DIR to share build artifacts across worktrees.
# Removed: shared target directories cause stale-artifact contamination when
# switching between worktrees with different code. Each worktree now gets its
# own target/ directory (cargo default). The extra disk and first-build cost
# is minor compared to the debugging cost of cross-worktree contamination.
