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

| Name               | Value                                                 | Description                                           |
| ------------------ | ----------------------------------------------------- | ----------------------------------------------------- |
| `project_root`     | repository root                                       | Base directory; all paths are relative to this        |
| `git_guide`        | `{{ project_root }}/docs/guides/git.md`               | Feature Branch Setup Checklist, branching conventions |
| `agent_ops`        | `{{ project_root }}/docs/guides/agent-ops.md`         | Agent roles, guard protocol, sync protocol            |
| `pitch_template`   | `{{ project_root }}/.github/ISSUE_TEMPLATE/pitch.yml` | Pitch template with labels                            |
| `tracking_label`   | `type:cycle`                                          | Label identifying cycle tracking issues               |
| `hook_config`      | `{{ project_root }}/lefthook.yml`                     | Hook commands for worktree setup                      |
| `wiki_index`       | `.wiki/Research-Index.md`                             | Research index for prior findings                     |
| `claude_md`        | `{{ project_root }}/CLAUDE.md`                        | Development workflow reference                        |
| `plugin_skill`     | `/hex-plugin`                                         | Plugin docs scaffolding                               |
| `contract_skill`   | `/hex-contract`                                       | Contract management                                   |
| `research_skill`   | `/hex-research`                                       | Research investigation                                |
| `task_list_prefix` | `hexorder`                                            | Prefix for CLAUDE_CODE_TASK_LIST_ID                   |

## Read the Pitch

1. Read `{{ pitch_template }}` to extract the label applied to pitch issues (from the `labels:`
   field). Use it to find the bet pitch for the current cycle:
    ```bash
    gh issue list --milestone "<milestone>" --label "<pitch label>"
    ```
2. Read the full pitch: `gh issue view <number>`
3. Note the five ingredients: Problem, Appetite, Solution, Rabbit Holes, No Gos
4. **Populate the Build Checklist.** If the pitch has a Build Checklist section, verify it is
   populated. If it is empty or absent, extract the numbered scopes from the Solution section and
   edit the pitch issue to add them as checklist items:
    ```bash
    gh issue edit <number> --body "$(updated body with checklist)"
    ```
    Each item should be a concrete, independently completable scope with a checkbox.

## Map Dependencies (multi-pitch cycles)

If the cycle has multiple pitches, map cross-pitch dependencies before setting up branches:

1. Read all bet pitches for the cycle
2. For each pitch, identify shared types, contracts, or features it depends on from other pitches
3. Record dependencies as issue cross-references. For each pitch, comment on it listing what it
   depends on and its delivery order. Use `delivery-order:N` labels if available.
4. Present the dependency map to the user for review before proceeding

If the cycle has only one pitch, skip this step.

## Find the Cycle Tracking Issue

Locate the cycle tracking issue for this milestone:

```bash
gh issue list --label "{{ tracking_label }}" --milestone "<milestone>" --state open --json number,title
```

If no tracking issue exists, **stop** — the cycle agent (or `/hex-bet`) should have created it.
Report the gap and suggest creating one via `/hex-bet` or spawning a cycle agent.

Record the tracking issue number for later use.

## Verify Integration Branch Exists

For multi-pitch cycles, verify that the integration branch exists:

```bash
git ls-remote --heads origin <version>
git branch --list <version>
```

If the integration branch does **not** exist (neither remotely nor locally), **stop** — the cycle
agent must create it first using the Integration Branch Setup Checklist in `{{ git_guide }}`. Report
the gap and suggest spawning a cycle agent.

If the cycle has only one pitch, skip this step — the feature branch merges directly to `main` using
the Solo-Pitch Merge workflow.

## Set Up the Feature Branch

Read `{{ git_guide }}` to extract the Feature Branch Setup Checklist. Follow each step.

The checklist covers branch creation (from the integration branch if one exists, otherwise from
`main`), worktree setup, hook installation, pre-release versioning, plugin scaffolding (via
`{{ plugin_skill }}`), contract checks (via `{{ contract_skill }}`), ownership claiming (via
`gh issue edit <n> --add-assignee @me`), and the initial commit.

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
**Tracking:** #<tracking-issue-number>
**First piece:** <chosen scope and rationale>
**Initial observations:** <anything notable from orientation — surprises, open questions, early reads on complexity>
EOF
)"
```

This is the first entry in a running thread. During the build, post follow-up comments on this same
issue for progress updates (read `{{ claude_md }}` for the Progress Updates guidance).

## Update Lifecycle Checklist

Check off the first two lifecycle items on the pitch issue:

1. **Branch created from integration branch** — verify the feature branch exists and was created
   from the integration branch (or main for solo-pitch cycles).
2. **Build started — kickoff comment posted** — verify the kickoff comment was posted above.

Read `{{ agent_ops }}` Guard Protocol to verify prerequisites before checking off each item.

Post a status comment on the tracking issue:

```bash
gh issue comment <tracking-number> --body "Pitch #<N> (<title>): build started on branch \`<branch-name>\`."
```

## Set Task List ID

For multi-pitch cycles, set the shared task list environment variable. This enables live task
visibility across all Claude Code sessions in the cycle.

```bash
export CLAUDE_CODE_TASK_LIST_ID={{ task_list_prefix }}-<version>
```

Verify the variable is set: `echo $CLAUDE_CODE_TASK_LIST_ID`

If the variable is already set (from a prior session), confirm it matches the current cycle's
version. If it does not match, update it.

## After Kickoff

Begin the build loop described in `{{ claude_md }}`. Build the first piece end-to-end — working code
and working tests — before expanding to other scopes.
