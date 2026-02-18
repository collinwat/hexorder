# Hexorder Coordination

## Active Cycle

**Cycle 4** — The World Comes Alive | Production | Release: 0.9.0

### Current Bets

| Pitch | Title                                                           | Appetite    | Status |
| ----- | --------------------------------------------------------------- | ----------- | ------ |
| #77   | Core mechanic primitives — turn structure and combat resolution | Big Batch   | merged |
| #80   | Keyboard-first command access — shortcuts, palette              | Small Batch | merged |
| #53   | Workspace lifecycle — create, name, save, launcher              | Small Batch | merged |
| #54   | Editor Visual Polish — brand theme, fonts                       | Small Batch | merged |

### Prior Cycles

| Cycle | Name                | Type       | Result    |
| ----- | ------------------- | ---------- | --------- |
| 3     | Process Reform      | Process    | completed |
| 2     | The Foundation      | Production | completed |
| 1     | The Process Matures | Process    | completed |

## Active Plugins

Plugins are permanent modules under `src/`. Status and ownership are tracked in GitHub Issues and
the GitHub Project:

```bash
gh issue list --state open                    # all open work items
gh issue list --milestone "<milestone>"       # items for a specific release
gh project view 1 --owner collinwat           # project board
```

## Pending Contract Changes

Contract change proposals are tracked as GitHub Issues with `area:contracts` label:
`gh issue list --label "area:contracts" --state open`

Before changing a contract, create an issue describing the change, list affected plugins, and wait
for approval before implementing. See the Shared Contracts Protocol in CLAUDE.md.

### Active Changes

| Contract      | Change                                                      | Pitch | Status |
| ------------- | ----------------------------------------------------------- | ----- | ------ |
| `mechanics`   | New contract: turn structure, CRT, modifiers, combat exec   | #77   | done   |
| `persistence` | Add `AppScreen::Play`, mechanics fields to `GameSystemFile` | #77   | done   |
| `editor_ui`   | `CombatSelect` variant deferred to #107                     | #77   | done   |

## Integration Branch

> Each cycle uses an integration branch for multi-pitch work. See `docs/guides/git.md` → Integration
> branch and Merging sections.

| Cycle | Branch  | Pitches Merged     | Status   |
| ----- | ------- | ------------------ | -------- |
| 4     | `0.9.0` | #54, #53, #77, #80 | shipping |

Status values: `active` | `shipping` | `shipped`

### Prior Integration Branches

| Cycle | Branch | Version | Result |
| ----- | ------ | ------- | ------ |

> **Note**: Cycles 1 and 2 predated the integration branch model. They used the merge lock protocol
> (now retired).

## Pitch Dependencies

> After betting, map cross-pitch dependencies to determine delivery order. Populated during kickoff
> when implementation plans are generated.

| Pitch | Depends On | Delivery Order | Status |
| ----- | ---------- | -------------- | ------ |
| #77   | —          | 1              | done   |
| #80   | —          | 1              | done   |
| #54   | —          | 1              | done   |
| #53   | #54        | 2              | done   |

Delivery Order values: `1`, `2`, `3`... (pitches with the same number can build in parallel). Status
values: `planned` | `in-progress` | `done`

## Known Blockers

- Bevy 0.18 and bevy_egui 0.39 API patterns are documented in `docs/guides/bevy.md` and
  `docs/guides/bevy-egui.md`.
