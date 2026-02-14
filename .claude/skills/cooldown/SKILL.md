---
name: cooldown
description:
    Run the cool-down protocol between build cycles. Orchestrates the retrospective, shaping, and
    betting phases that transition from one cycle to the next. Use after a cycle ships (after the
    ship gate passes and cycle ship merge completes), or when the user invokes /cooldown.
---

# Cooldown

The cool-down is the period between build cycles. It has three phases, run in order.

## Phase 1: Retrospective

Run `/retro` to reflect on the completed cycle. This surfaces learnings and captures new ideas as
GitHub Issues.

## Phase 2: Shaping

Review the issue pool for ideas worth shaping into pitches:

1. Browse raw ideas: `gh issue list --state open --label "status:triage"`
2. Browse existing features/bugs: `gh issue list --state open`
3. Check for deferred items from the last cycle: `gh issue list --label "status:deferred"`
4. Run `/research` for any ideas that need investigation before committing to an approach
5. Run `/pitch` for each idea worth bringing to the betting table

Not every idea needs a pitch. Only shape ideas where the problem is real, the appetite is clear, and
you have a solution sketch. Ideas that are not shaped are left alone — important ones will
resurface.

## Phase 3: Betting

Review all shaped pitches and decide what to build next:

1. List pitches: `gh issue list --label "type:pitch" --state open`
2. For each pitch, evaluate:
    - Does the problem matter right now?
    - Is the appetite right?
    - Is the solution attractive?
    - Are the right people/agents available?
3. Select bets and assign to a release milestone:
    ```bash
    gh issue edit <number> --milestone "<milestone>"
    ```
4. Update `docs/coordination.md`:
    - Set the new cycle name, type, and appetite under Active Cycle
    - Add selected pitches to the Current Bets table
5. Unselected pitches are NOT queued — they can be re-pitched in a future cycle if still relevant

## After Betting

The cool-down is complete. The next build cycle begins with orientation (see CLAUDE.md → Development
Workflow).
