---
name: kickoff
description:
    Start a new build cycle by orienting on the selected pitch, setting up the feature branch, and
    identifying the first piece to build. Use after bets are placed and the cycle is ready to begin.
    Also use when the user invokes /kickoff.
---

# Kickoff

Transition from betting to building. Orient on the pitch, set up the workspace, and identify where
to start.

## Read the Pitch

1. Find the bet pitch for the current cycle:
    ```bash
    gh issue list --milestone "<milestone>" --label "type:pitch"
    ```
2. Read the full pitch: `gh issue view <number>`
3. Note the five ingredients: Problem, Appetite, Solution, Rabbit Holes, No Gos

## Set Up the Branch

Follow the Feature Branch Setup Checklist in `docs/guides/git.md`:

1. Determine branch name: `<release>/<feature>`
2. Create branch and worktree
3. Install hooks in worktree: `lefthook install`
4. Set pre-release version in `Cargo.toml`
5. Scaffold plugin docs via `/plugin` (if new plugin)
6. Check contracts via `/contract` (if dependencies exist)
7. Claim ownership in `docs/coordination.md`
8. Initial commit

## Consume Research

Check if relevant research exists for this pitch:

1. Read `.wiki/Research-Index.md` for relevant pages
2. If research exists, read it and summarize key findings in the plugin log
3. If unknowns remain, run `/research` to investigate before building

## Identify the First Piece

Pick the first scope to build end-to-end. It must be all three:

- **Core** — central to the project concept, not a peripheral detail
- **Small** — completable end-to-end in a few days
- **Novel** — involves something new or uncertain, to surface unknowns early

Record the chosen first piece in the plugin log with rationale.

## After Kickoff

Begin the build loop (CLAUDE.md → Development Workflow). Build the first piece end-to-end — working
code and working tests — before expanding to other scopes.
