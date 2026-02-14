# Hexorder Coordination

## Active Cycle

**Cycle 1 — "The Process Matures"** | Type: Process (no code) | Appetite: Small Batch

### Current Bets

| Pitch                                   | Appetite    | Status      |
| --------------------------------------- | ----------- | ----------- |
| Shape Up workflow documentation rewrite | Small Batch | in-progress |

_Bets are set at the betting table during cool-down. See CLAUDE.md → Cool-Down Protocol._

## Active Features

Features are scopes within a build cycle. Status and ownership are tracked in GitHub Issues and the
GitHub Project:

```bash
gh issue list --state open                    # all open work items
gh issue list --milestone "<milestone>"       # items for a specific release
gh project view 1 --owner collinwat           # project board
```

## Pending Contract Changes

Contract change proposals are tracked as GitHub Issues with `area:contracts` label:
`gh issue list --label "area:contracts" --state open`

Before changing a contract, create an issue describing the change, list affected features, and wait
for approval before implementing. See the Shared Contracts Protocol in CLAUDE.md.

## Merge Lock

> Only one merge to `main` at a time. See `docs/guides/git-guide.md` → Merge Lock Protocol for full
> rules.

| Branch                   | Version | Claimed By | Status  |
| ------------------------ | ------- | ---------- | ------- |
| 0.4.0/entity-unification | 0.4.0   | agent      | merging |

Status values: `merging` | `done`

Rules:

- Before merging, check this table. If any row is `merging`, wait.
- Claim your row before starting the Pre-Merge Checklist.
- Release (mark `done`) after the tag is created and verified.
- Do not clear another session's `merging` row without investigation.

## Known Blockers

- Bevy 0.18 and bevy_egui 0.39 API patterns are documented in `docs/guides/bevy-guide.md` and
  `docs/guides/bevy-egui-guide.md`.
