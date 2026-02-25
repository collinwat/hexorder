---
name: hex-continue
description:
    Orient on the current state and determine next actions. Use when starting a new session,
    resuming interrupted work, or switching context. Detects your role (orchestrator, pitch agent,
    cycle agent) from git state and guides you to the right next step. Also use when the user
    invokes /hex-continue.
---

# Continue

Orient on the current state and resume work from the last checkpoint.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name              | Value                                         | Description                                    |
| ----------------- | --------------------------------------------- | ---------------------------------------------- |
| `project_root`    | repository root                               | Base directory; all paths are relative to this |
| `git_guide`       | `{{ project_root }}/docs/guides/git.md`       | Git workflow, branching, session handoff       |
| `agent_ops`       | `{{ project_root }}/docs/guides/agent-ops.md` | Agent roles, continue/guard/sync protocols     |
| `tracking_label`  | `type:cycle`                                  | Label identifying cycle tracking issues        |
| `pitch_label`     | `type:pitch`                                  | Label identifying shaped pitches               |
| `kickoff_skill`   | `/hex-kickoff`                                | Skill to start a pitch build                   |
| `integrate_skill` | `/hex-integrate`                              | Skill to merge a ready pitch                   |
| `ship_skill`      | `/hex-ship`                                   | Skill to run the ship gate                     |
| `status_skill`    | `/hex-status`                                 | Skill to get full situational awareness        |

## 1. Detect Context

Read git state to determine the current role and scope:

```bash
git branch --show-current
git worktree list
gh issue list --label "{{ tracking_label }}" --state open --json number,title
```

Map the current branch to a role:

| Branch pattern                                   | Role                   |
| ------------------------------------------------ | ---------------------- |
| `main`                                           | **Orchestrator scope** |
| `<version>-<feature>` (e.g., `0.11.0-editor-ui`) | **Pitch agent scope**  |
| `<version>` (e.g., `0.11.0`)                     | **Cycle agent scope**  |

Proceed to the section matching the detected role.

## 2. Orchestrator Orientation (on `main`)

When on `main`, you are at the top level — not inside any agent's scope.

1. Run `{{ status_skill }}` for full situational awareness.
2. Find the active cycle milestone and tracking issue:
    ```bash
    gh api repos/:owner/:repo/milestones --jq '.[] | select(.state=="open") | {title,number,description}'
    gh issue list --label "{{ tracking_label }}" --state open
    ```
3. Check which pitches are assigned, which need kickoff, which are ready for integration:
    ```bash
    gh issue list --label "{{ pitch_label }}" --milestone "<active milestone>" --json number,title,state,assignees
    ```
4. For each pitch, read its Lifecycle section to determine status.
5. Present actionable options:
    - "Pitch #X needs kickoff — spawn a pitch-agent for it"
    - "Pitch #X is ready for integration — spawn a cycle-agent"
    - "All pitches merged — run `{{ ship_skill }}`"
    - "No active cycle — run `/hex-bet` to start one"

## 3. Pitch Agent Resume (on a feature branch)

When on a `<version>-<feature>` branch, you are a pitch agent resuming work.

1. **Identify the pitch.** Extract the pitch number from the branch name or find it:
    ```bash
    gh issue list --label "{{ pitch_label }}" --milestone "<version>" --json number,title,assignees
    ```
2. **Read the Lifecycle section.** Find the last checked item — that's the last completed phase:
    ```bash
    gh issue view <pitch-number>
    ```
3. **Read git state.** Check for uncommitted work and recent activity:
    ```bash
    git status --short
    git log --oneline -5
    git diff --stat
    ```
4. **Read agent-ops guide.** Read `{{ agent_ops }}` Continue/Resume Protocol for the full procedure.
5. **Check sync status.** Is the integration branch ahead?
    ```bash
    git fetch origin <version>
    git rev-list --count HEAD..origin/<version>
    ```
    If ahead, sync is needed before the next scope (see Sync Protocol in `{{ agent_ops }}`).
6. **Determine next action** based on lifecycle position:

    | Last checked item     | Next action                                                             |
    | --------------------- | ----------------------------------------------------------------------- |
    | _(none)_              | Run `{{ kickoff_skill }}` to start the build                            |
    | Branch created        | Post kickoff comment, check off "Build started"                         |
    | Build started         | Resume building — find next unchecked scope in Build Checklist          |
    | Scopes complete       | Run `mise check:audit` for quality gate                                 |
    | Gate passed           | Post build reflection on pitch issue                                    |
    | Reflection posted     | Verify spec criteria + deferred items → declare "Ready for integration" |
    | Ready for integration | Waiting for cycle agent to merge — nothing to do                        |

7. **Report.** Present: last completed phase, next action, blockers, sync status.

## 4. Cycle Agent Resume (on integration branch)

When on a `<version>` branch, you are a cycle agent resuming work.

1. **Find the cycle tracking issue.**
    ```bash
    gh issue list --label "{{ tracking_label }}" --state open --json number,title
    gh issue view <tracking-number>
    ```
2. **Read tracking issue sections.** Check Integration Setup, Pitch Status, Ship Readiness.
3. **Read integration branch git state.**
    ```bash
    git status --short
    git log --oneline -5
    ```
4. **Check pitch readiness.** For each pitch in the cycle, read its Lifecycle section:
    ```bash
    gh issue list --label "{{ pitch_label }}" --milestone "<version>" --json number,title
    gh issue view <pitch-number>
    ```
    Look for lifecycle item 6 ("Ready for integration") being checked.
5. **Determine next action:**

    | Condition                         | Next action                                                    |
    | --------------------------------- | -------------------------------------------------------------- |
    | Integration Setup items unchecked | Follow Integration Branch Setup Checklist in `{{ git_guide }}` |
    | Pitches are ready for integration | Run `{{ integrate_skill }}` for each ready pitch               |
    | All pitches merged                | Check Ship Readiness → run `{{ ship_skill }}`                  |

6. **Report.** Present: tracking issue state, ready pitches, next action, blockers.
