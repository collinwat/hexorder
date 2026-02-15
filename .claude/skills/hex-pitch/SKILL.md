---
name: hex-pitch
description:
    Shape raw ideas into formal pitches for the betting table. Use during cool-down to transform
    GitHub Issues, observations, or fresh ideas into shaped proposals. Handles pitches built from
    existing issues, pitches created from scratch, or a mix of both. Also use when the user invokes
    /hex-pitch.
---

# Pitch

For Shape Up shaping theory, see `docs/guides/shape-up.md` → Phase 1: Shaping.

## Which Workflow?

1. Ask the user: are we shaping from existing GitHub Issues, from a fresh idea, or both?
2. If from issues → **Browse & Shape** (below)
3. If from a fresh idea → **Shape from Scratch** (further below)
4. If both → Browse first, then shape incorporating the issues

## Browse & Shape (from existing issues)

1. Search for candidate issues:
    ```bash
    gh issue list --state open --label "status:triage"
    gh issue list --state open --label "type:feature"
    gh issue list --state open --label "type:bug"
    gh issue list --search "<keywords>"
    ```
2. Present candidates to the user for selection
3. Read selected issues: `gh issue view <number>`
4. Proceed to **Shape the Pitch** below, incorporating the selected issues

## Shape from Scratch

1. Discuss the problem with the user — what pain point or opportunity?
2. Proceed to **Shape the Pitch** below

## Shape the Pitch

Walk through the four shaping steps with the user:

### 1. Set Boundaries

- **Appetite**: Small Batch (1-2 weeks) or Big Batch (full cycle)?
- **Narrow the problem**: What specific problem are we solving? What's the current workaround?
- **Avoid grab-bags**: If it's a bundle of unrelated tasks, split into separate pitches

### 2. Find the Elements

- Identify the main components, interactions, and flows
- Use breadboarding (places → affordances → connections) or fat marker sketches
- Walk through the use case mentally — can the user's journey be traced?

### 3. Address Risks and Rabbit Holes

- Walk through the solution step by step
- Identify technical unknowns, unsolved design problems, interdependencies
- For each risk: patch the hole, declare out of bounds, or cut back

### 4. Write the Pitch

Create the pitch Issue using the pitch template (`.github/ISSUE_TEMPLATE/pitch.yml`):

```bash
gh issue create --template pitch.yml --title "<concise title>" \
  --label "type:pitch" --label "area:<area>"
```

Fill in the five ingredients: Problem, Appetite, Solution, Rabbit Holes, No Gos.

If shaping from existing issues, add them to the "Related raw ideas" field using `#number`
references.

## After the Pitch

The pitch is ready for the betting table. It will be reviewed during cool-down. If selected, it gets
assigned to a release milestone and enters the build cycle.
