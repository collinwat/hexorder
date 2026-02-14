---
name: cooldown
description:
    Run the cool-down protocol between build cycles. Orchestrates the retrospective, triage,
    shaping, and betting phases that transition from one cycle to the next. Use after a cycle ships
    (after the ship gate passes and cycle ship merge completes), or when the user invokes /cooldown.
---

# Cooldown

The cool-down is the period between build cycles. It has four phases, run in order.

## Phase 1: Retrospective

Run `/retro` to reflect on the completed cycle. This surfaces learnings and captures new ideas as
GitHub Issues.

## Phase 2: Triage

Run `/triage` to survey the full pool of open issues — raw ideas, deferred items, bugs, tech debt.
Identify clusters of related issues worth shaping. Close stale or duplicate issues.

## Phase 3: Shaping

For each cluster identified during triage:

1. Run `/research` if the cluster has unknowns that need investigation first
2. Run `/pitch` to shape the cluster into a formal pitch for the betting table

Not every cluster needs a pitch. Only shape ideas where the problem is real, the appetite is clear,
and you have a solution sketch. Ideas that are not shaped are left alone — important ones will
resurface.

## Phase 4: Betting

Run `/bet` to review shaped pitches and decide what to build in the next cycle.
