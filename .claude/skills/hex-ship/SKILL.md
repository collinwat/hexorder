---
name: hex-ship
description:
    Run the ship gate audit and verify a cycle is ready to close. Use when a cycle's work is
    complete and ready to ship, when running the constitution audit before tagging a release, or
    when the user invokes /hex-ship.
---

# Ship

Run the constitution audit that gates every release. Both automated and manual checks must pass
before the cycle ships. See CLAUDE.md → Ship Gate for the full list.

## Automated Checks

Run the full audit:

```bash
mise check:audit
```

This covers:

1. `cargo test` — all tests pass
2. `cargo clippy --all-targets` — zero warnings
3. `cargo build` — clean compilation
4. No `unwrap()` in production code — `mise check:unwrap`
5. No cross-plugin internal imports — `mise check:boundary`
6. Formatting, typos, TOML, dependency audit

If any check fails, fix the issue and re-run before proceeding.

## Manual Checks

Walk through these with the user. Each requires human judgment:

1. **No `unsafe` without documented justification** — search for `unsafe` in `src/`; if found,
   verify justification exists in the relevant plugin log
2. **All public types derive `Debug`** — spot-check public structs and enums
3. **Contracts spec-code parity** — every type in `src/contracts/` has a matching spec in
   `docs/contracts/`, and vice versa
4. **Brand palette compliance** — the `editor_ui_colors_match_brand_palette` architecture test
   passes; any new color literals in `src/editor_ui/` are in the approved palette
5. **No stray ideas** — all TODOs, deferred items, and "coming soon" placeholders have corresponding
   GitHub Issues: `gh issue list --search "<keywords>"`

## Gate Decision

Present the results to the user:

- **All pass** → proceed with the cycle ship merge (`docs/guides/git.md` → Cycle ship merge)
- **Any fail** → circuit breaker fires. Work does not ship. The problem must be re-shaped and
  re-pitched.

## After the Gate Passes

Follow steps 17-20 in `docs/guides/git.md` → Cycle ship merge:

1. Issue cleanup — close completed issues for the milestone
2. Triage new items — review `status:triage` issues
3. Run cool-down protocol — retrospective (`/hex-retro`), shaping (`/hex-pitch`), betting
