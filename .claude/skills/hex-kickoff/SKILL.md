---
name: hex-kickoff
description:
    Start a new build cycle by orienting on the selected pitch, setting up the feature branch, and
    identifying the first piece to build. Use after bets are placed and the cycle is ready to begin.
    Also use when the user invokes /hex-kickoff.
---

# Kickoff

Transition from betting to building. Orient on the pitch, set up the workspace, and identify where
to start.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name             | Value                                                 | Description                                           |
| ---------------- | ----------------------------------------------------- | ----------------------------------------------------- |
| `project_root`   | repository root                                       | Base directory; all paths are relative to this        |
| `git_guide`      | `{{ project_root }}/docs/guides/git.md`               | Feature Branch Setup Checklist, branching conventions |
| `pitch_template` | `{{ project_root }}/.github/ISSUE_TEMPLATE/pitch.yml` | Pitch template with labels                            |
| `coordination`   | `{{ project_root }}/docs/coordination.md`             | Active cycle, ownership                               |
| `hook_config`    | `{{ project_root }}/lefthook.yml`                     | Hook commands for worktree setup                      |
| `wiki_index`     | `.wiki/Research-Index.md`                             | Research index for prior findings                     |
| `claude_md`      | `{{ project_root }}/CLAUDE.md`                        | Development workflow reference                        |
| `plugin_skill`   | `/hex-plugin`                                         | Plugin docs scaffolding                               |
| `contract_skill` | `/hex-contract`                                       | Contract management                                   |
| `research_skill` | `/hex-research`                                       | Research investigation                                |

## Read the Pitch

1. Read `{{ pitch_template }}` to extract the label applied to pitch issues (from the `labels:`
   field). Use it to find the bet pitch for the current cycle:
    ```bash
    gh issue list --milestone "<milestone>" --label "<pitch label>"
    ```
2. Read the full pitch: `gh issue view <number>`
3. Note the five ingredients: Problem, Appetite, Solution, Rabbit Holes, No Gos

## Set Up the Branch

Read `{{ git_guide }}` to extract the Feature Branch Setup Checklist. Follow each step.

The checklist covers branch creation, worktree setup, hook installation, pre-release versioning,
plugin scaffolding (via `{{ plugin_skill }}`), contract checks (via `{{ contract_skill }}`),
ownership claiming in `{{ coordination }}`, and the initial commit.

## Consume Research

Check if relevant research exists for this pitch:

1. Read `{{ wiki_index }}` for relevant pages
2. If research exists, read it and summarize key findings in the plugin log
3. If unknowns remain, run `{{ research_skill }}` to investigate before building

## Identify the First Piece

Pick the first scope to build end-to-end. It must be all three:

- **Core** — central to the project concept, not a peripheral detail
- **Small** — completable end-to-end in a few days
- **Novel** — involves something new or uncertain, to surface unknowns early

Record the chosen first piece in the plugin log with rationale.

## Post to the Pitch Issue

Post a kickoff comment on the pitch issue to start the build narrative. This comment thread becomes
the agent's progress log for the cycle — the retro will read it later.

```bash
gh issue comment <number> --body "$(cat <<'EOF'
## Build started

**Branch:** `<branch-name>`
**First piece:** <chosen scope and rationale>
**Initial observations:** <anything notable from orientation — surprises, open questions, early reads on complexity>
EOF
)"
```

This is the first entry in a running thread. During the build, post follow-up comments on this same
issue for progress updates (read `{{ claude_md }}` for the Progress Updates guidance).

## After Kickoff

Begin the build loop described in `{{ claude_md }}`. Build the first piece end-to-end — working code
and working tests — before expanding to other scopes.
