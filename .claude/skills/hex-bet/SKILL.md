---
name: hex-bet
description:
    Run the betting table to review shaped pitches and decide what to build in the next cycle. Use
    during cool-down after pitches have been shaped, or independently when revisiting cycle scope.
    Also use when the user invokes /hex-bet.
---

# Bet

Review all shaped pitches and decide what to commit to for the next build cycle.

## Review Pitches

1. List open pitches:
    ```bash
    gh issue list --label "type:pitch" --state open
    ```
2. Read each pitch: `gh issue view <number>`
3. For each pitch, evaluate:
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
2. Record the bet in `docs/coordination.md`:
    - Set the new cycle name, type, and appetite under Active Cycle
    - Add the pitch to the Current Bets table with status `pending`

## Unselected Pitches

Pitches not selected are NOT queued or carried forward. They can be re-pitched in a future cycle if
the problem is still relevant. This is intentional — important ideas come back naturally.

## After Betting

The next build cycle begins with orientation (see CLAUDE.md → Development Workflow).
