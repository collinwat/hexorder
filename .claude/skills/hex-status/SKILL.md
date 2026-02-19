---
name: hex-status
description:
    Report the current state of the project and agent workload for situational awareness. Use when
    switching between agent terminals, resuming work, or needing to orient on cycle progress and
    next steps. Also use when the user invokes /hex-status.
---

# Status

Produce a unified situational awareness report covering the current terminal, cycle progress, pitch
state, branch mapping, and recommended next steps.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name             | Value           | Description                                       |
| ---------------- | --------------- | ------------------------------------------------- |
| `project_root`   | repository root | Base directory; all paths are relative to this    |
| `pitch_label`    | `type:pitch`    | Label identifying shaped pitches in GitHub Issues |
| `kickoff_skill`  | `/hex-kickoff`  | Skill to suggest when a pitch needs to be started |
| `ship_skill`     | `/hex-ship`     | Skill to suggest when a cycle is ready to ship    |
| `cooldown_skill` | `/hex-cooldown` | Skill to suggest when cool-down should begin      |
| `commit_skill`   | `/hex-commit`   | Skill to suggest when uncommitted work is found   |

## 1. Gather Local State

Collect the state of the current working environment. Run these commands:

```bash
git rev-parse --show-toplevel
git branch --show-current
git worktree list
git status --short
git diff --stat
git log --oneline -10
```

Also check if a Claude Code task list is active (run `TaskList`). If tasks exist, record their IDs,
subjects, statuses, and owners.

## 2. Read Cycle Context

Query GitHub for cycle and pitch state using the milestone and issue tracker:

```bash
# Active milestone (current cycle)
gh api repos/:owner/:repo/milestones --jq '.[] | select(.state=="open") | {title,description,due_on,open_issues,closed_issues}'

# Pitches bet for the active cycle
gh issue list --label "{{ pitch_label }}" --milestone "<active milestone>" --json number,title,state,assignees,labels

# Pending contract changes
gh issue list --label "area:contracts" --state open --json number,title,assignees
```

From the milestone and issues, extract:

- **Active Cycle** — milestone title, description (contains integration branch and release version)
- **Current Bets** — pitch issues assigned to the milestone (number, title, state, assignees)
- **Pitch Dependencies** — cross-references between pitch issues
- **Integration Branch** — from milestone description
- **Known Blockers** — any issues labeled as blockers
- **Pending Contract Changes** — open issues with `area:contracts` label

## 3. Query Pitch State

For each pitch in the active milestone, query GitHub for its current state:

```bash
gh issue view <pitch-number> --json title,state,assignees,labels,comments --jq '{title,state,assignees: [.assignees[].login],comment_count: (.comments | length),last_comment: (.comments | last | .body // "none" | .[0:200])}'
```

For each pitch, also check for a matching feature branch:

```bash
git branch -a --list "*<pitch-number>*" --list "*<pitch-keyword>*"
```

If a feature branch exists, get its recent commit activity:

```bash
git log --oneline -5 <branch-name>
```

## 4. Present the Report

Format and display the report with these five sections. Use markdown tables and headers for
scannability.

### Section 1: This Terminal

| Field               | Source                          |
| ------------------- | ------------------------------- |
| Working directory   | `git rev-parse --show-toplevel` |
| Current branch      | `git branch --show-current`     |
| Uncommitted changes | `git status --short`            |
| Unstaged diff stat  | `git diff --stat`               |
| Recent commits      | `git log --oneline -5`          |
| Active tasks        | TaskList output (if any)        |

If no uncommitted changes and no active tasks, say so briefly.

### Section 2: Cycle Overview

Display:

- Cycle number, name, type, release version (from the active milestone)
- Integration branch and its status (from the milestone description)
- The bets table: pitch issues assigned to the milestone with current status

### Section 3: Pitch Detail

For each pitch in the active cycle, display:

- Pitch number and title
- GitHub state (open/closed)
- Assignee(s)
- Feature branch (if any) and its worktree location (if any)
- Last build journal comment (first 200 chars)
- Recent commit count on the feature branch

### Section 4: Branch Map

Display a table mapping branches to their role:

| Branch | Pitch/Role | Worktree | Last Commit |
| ------ | ---------- | -------- | ----------- |

Include: main, integration branch, and all feature branches related to the active cycle.

### Section 5: Next Steps

Analyze the gathered data and produce actionable recommendations. Check for each of these conditions
and generate a recommendation when true:

| Condition                                                         | Recommendation                                                  |
| ----------------------------------------------------------------- | --------------------------------------------------------------- |
| Uncommitted changes in current worktree                           | Commit or stash — suggest `{{ commit_skill }}`                  |
| A pitch has status `pending` and no assignee                      | Pitch is unclaimed — suggest `{{ kickoff_skill }}`              |
| A pitch has status `in-progress` with no recent commits (>3 days) | Pitch may be stalled — check the feature branch                 |
| All pitches have status `done`                                    | Cycle may be ready to ship — suggest `{{ ship_skill }}`         |
| Integration branch status is `shipped`                            | Cool-down is next — suggest `{{ cooldown_skill }}`              |
| Pending contract changes exist                                    | Contract changes need attention — list them                     |
| Known blockers exist                                              | Blockers need resolution — list them                            |
| Active tasks in task list are `in_progress`                       | Agent has unfinished work — list the tasks and suggest resuming |
| Feature branch exists with no worktree                            | Branch has no worktree — suggest creating one or cleaning up    |
| Pitch dependencies block a pitch                                  | Blocked pitch — identify what must complete first               |

Present recommendations as a numbered list with specific actions (e.g., "Pitch #120 is unclaimed —
run `{{ kickoff_skill }}` to start" rather than "there are unclaimed pitches").

If no recommendations apply, say the project is in a clean state.
