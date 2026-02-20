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
| `cycle_template` | `{{ project_root }}/.github/ISSUE_TEMPLATE/cycle.yml` | Cycle tracking issue template                  |
| `git_guide`      | `{{ project_root }}/docs/guides/git.md`               | Integration Branch Setup Checklist             |
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
2. Record the bet by updating the milestone description with the cycle name, type, and bet summary:
    ```bash
    gh api repos/{owner}/{repo}/milestones/{number} -X PATCH \
      -f description="Cycle N — <name> | <type> | <pitches summary> | Integration branch: <version>"
    ```

## Create Cycle Tracking Issue

After placing bets, create a cycle tracking issue to coordinate the build phase:

1. Ensure the `type:cycle` label exists:
    ```bash
    gh label create "type:cycle" --description "Cycle tracking issue" --color "0E8A16" --force
    ```
2. Check for an existing tracking issue for this milestone:
    ```bash
    gh issue list --label "type:cycle" --milestone "<milestone>" --state open
    ```
    If one already exists, skip creation.
3. Read `{{ cycle_template }}` for the expected structure.
4. Create the tracking issue. Fill in the milestone, integration branch, and pitch status table:

    ```bash
    gh issue create --label "type:cycle" --milestone "<milestone>" --title "Cycle: <milestone>" \
      --body "$(cat <<'EOF'
    ## Integration Branch Setup
    - [ ] Local main synced with origin/main
    - [ ] No uncommitted changes on main
    - [ ] Integration branch created from main
    - [ ] Integration branch pushed to origin
    - [ ] Milestone description updated with branch name

    ## Pitch Status
    | Pitch | Assignee | Branch | Started | Scopes Done | Gate | Merged | Reflection |
    | ----- | -------- | ------ | ------- | ----------- | ---- | ------ | ---------- |
    | #N title | - | - | - | - | - | - | - |

    ## Ship Readiness
    - [ ] All pitches merged to integration branch
    - [ ] `mise check:audit` passes on integration branch
    - [ ] Manual ship gate checks pass
    - [ ] UAT complete
    - [ ] Ship merge to main
    - [ ] Release tagged and pushed
    EOF
    )"
    ```

    Replace the pitch status table rows with the actual pitches selected for this cycle.

5. Record the tracking issue number in the milestone description:
    ```bash
    gh api repos/{owner}/{repo}/milestones/{number} -X PATCH \
      -f description="<existing description> | Tracking: #<tracking-issue-number>"
    ```
6. For **solo-pitch cycles**, still create the tracking issue but clear the Integration Branch Setup
   section (it won't be needed since the feature branch merges directly to main).

## Scaffold Dependencies (multi-pitch cycles)

If more than one pitch is selected, post a dependencies comment on each pitch issue noting what it
depends on (`—` for none) and its delivery order. The kickoff phase will finalize dependencies after
generating implementation plans.

## Unselected Pitches

Pitches not selected are NOT queued or carried forward. They can be re-pitched in a future cycle if
the problem is still relevant. This is intentional — important ideas come back naturally.

## After Betting

The next build cycle begins with orientation (see `{{ claude_md }}` → Development Workflow).
