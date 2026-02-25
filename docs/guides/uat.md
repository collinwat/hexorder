# Hexorder — UAT Guide

User Acceptance Testing (UAT) validates that built features meet human-verifiable criteria before
integration. UAT happens on the **pitch branch**, not the integration branch.

---

## Per-Pitch UAT Workflow

Each pitch defines UAT Criteria in its issue template (between Build Checklist and Lifecycle). These
are human-verifiable acceptance criteria — things to confirm in the running app.

### When to run UAT

- **Per scope**: After completing each build checklist scope, verify the relevant UAT criteria. Post
  results in the scope completion comment on the pitch issue.
- **Before integration**: After the quality gate passes (`mise check:audit`), run a full UAT pass on
  the pitch branch. All criteria must pass before declaring "Ready for integration."

### How to run UAT

1. **Build and launch** the app on the pitch branch:
    ```bash
    cargo run --features dev
    ```
2. **Walk through each UAT criterion** from the pitch issue. For each criterion:
    - Perform the described action or navigate to the described state
    - Confirm the expected behavior
    - Note any deviations
3. **Record results** as a comment on the pitch issue:

    ```markdown
    **UAT Results** (commit <sha>):

    - [x] SC-1: <criterion> — PASS
    - [x] SC-2: <criterion> — PASS
    - [ ] SC-3: <criterion> — FAIL: <what happened instead>
    ```

4. **Fix failures** before proceeding. UAT failures block integration.

### UAT for process-only pitches

Pitches that modify only documentation, templates, or tooling (no runtime code) may not have
criteria verifiable in the running app. In this case, UAT criteria should describe verifiable
outcomes in the affected artifacts:

- Template fields render correctly in GitHub issue creation
- Documentation is internally consistent and cross-references resolve
- Tooling commands execute successfully

---

## Regression UAT

After pitches merge to the integration branch, run regression UAT to catch cross-pitch interactions.
The cycle agent owns regression UAT as part of ship readiness.

### Regression checklist

Maintain a regression checklist of the top 5 critical user flows. This is a living document — new
features add items, old items get retired when automated. The checklist should never exceed ~15
items.

Current critical flows (update as the app evolves):

1. **App launches** — editor window opens without crash
2. **Hex grid renders** — grid is visible with correct geometry
3. **Camera controls** — orbit, pan, zoom respond to input
4. **Cell interaction** — clicking cells triggers expected behavior
5. **Save/load** — round-trip persistence works without data loss

### When to run regression UAT

- After each pitch is merged to the integration branch
- Before the ship gate audit
- After any conflict resolution on the integration branch

### How to record regression results

Post results as a comment on the cycle tracking issue:

```markdown
**Regression UAT** (integration branch, commit <sha>):

- [x] App launches
- [x] Hex grid renders
- [x] Camera controls
- [x] Cell interaction
- [x] Save/load
```

---

## Updating the Regression Checklist

When a new pitch introduces a user-facing feature:

1. Add 1-2 regression items covering the core happy path
2. If the checklist exceeds 15 items, retire the least critical items (prefer items that have
   automated test coverage)
3. Update this document with the new items
