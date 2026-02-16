# Hexorder Coordination

## Active Cycle

_No active cycle. Run `/hex-cooldown` to start the next cycle._

### Current Bets

_No active bets._

_Bets are set at the betting table during cool-down. Run `/hex-cooldown` to start the protocol._

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

## Integration Branch

> Each cycle uses an integration branch for multi-pitch work. See `docs/guides/git.md` → Integration
> branch and Merging sections.

| Cycle | Branch | Pitches Merged | Status |
| ----- | ------ | -------------- | ------ |

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

Delivery Order values: `1`, `2`, `3`... (pitches with the same number can build in parallel). Status
values: `planned` | `in-progress` | `done`

## Known Blockers

- Bevy 0.18 and bevy_egui 0.39 API patterns are documented in `docs/guides/bevy.md` and
  `docs/guides/bevy-egui.md`.
