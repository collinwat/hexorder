---
name: hex-bet
description:
    Run the betting table to review shaped pitches and decide what to build in the next cycle. Use
    during cool-down after pitches have been shaped, or independently when revisiting cycle scope.
    Also use when the user invokes /hex-bet.
---

# Bet

Review all shaped pitches and decide what to commit to for the next build cycle.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name             | Value                                                 | Description                                    |
| ---------------- | ----------------------------------------------------- | ---------------------------------------------- |
| `project_root`   | repository root                                       | Base directory; all paths are relative to this |
| `pitch_template` | `{{ project_root }}/.github/ISSUE_TEMPLATE/pitch.yml` | Pitch issue template (labels and fields)       |
| `coordination`   | `{{ project_root }}/docs/coordination.md`             | Active cycle, bets, ownership                  |
| `claude_md`      | `{{ project_root }}/CLAUDE.md`                        | Development workflow reference                 |

## Review Pitches

1. Read `{{ pitch_template }}` to extract the label applied to pitch issues (from the `labels:`
   field). Use the discovered label in the search below.
2. List open pitches:
    ```bash
    gh issue list --label "<discovered label>" --state open
    ```
3. Read each pitch: `gh issue view <number>`
4. For each pitch, evaluate:
    - Does the problem matter right now?
    - Is the appetite right?
    - Is the solution attractive?
    - Is this the right time?
    - Are the right people/agents available?

Present the pitches to the user with a summary and recommendation.

## Place Bets

For each selected pitch:

1. Assign to a release milestone:
    ```bash
    gh issue edit <number> --milestone "<milestone>"
    ```
2. Record the bet in `{{ coordination }}`:
    - Set the new cycle name, type, and appetite under Active Cycle
    - Add the pitch to the Current Bets table with status `pending`

## Scaffold Dependencies (multi-pitch cycles)

If more than one pitch is selected, scaffold the Pitch Dependencies table in `{{ coordination }}`.
Add one row per bet pitch with Depends On, Delivery Order, and Status columns left blank (`—`, `—`,
`planned`). The kickoff phase will populate the actual dependencies after generating implementation
plans.

## Unselected Pitches

Pitches not selected are NOT queued or carried forward. They can be re-pitched in a future cycle if
the problem is still relevant. This is intentional — important ideas come back naturally.

## After Betting

The next build cycle begins with orientation (see `{{ claude_md }}` → Development Workflow).
