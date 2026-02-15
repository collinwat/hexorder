---
name: hex-ship
description:
    Run the ship gate audit and verify a cycle is ready to close. Use when a cycle's work is
    complete and ready to ship, or when running the constitution audit before tagging a release.
    Also use when the user invokes /hex-ship.
---

# Ship

Run the constitution audit that gates every release.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name                 | Value                                   | Description                                    |
| -------------------- | --------------------------------------- | ---------------------------------------------- |
| `project_root`       | repository root                         | Base directory; all paths are relative to this |
| `claude_md`          | `{{ project_root }}/CLAUDE.md`          | Ship Gate checks (automated and manual)        |
| `git_guide`          | `{{ project_root }}/docs/guides/git.md` | Cycle ship merge steps                         |
| `contracts_spec_dir` | `{{ project_root }}/docs/contracts`     | Contract specs for parity check                |
| `contracts_src_dir`  | `{{ project_root }}/src/contracts`      | Contract implementations                       |
| `src_dir`            | `{{ project_root }}/src`                | Source directory for unsafe/debug checks       |

## Automated Checks

Read `{{ claude_md }}` to extract the Ship Gate section — specifically the automated checks and what
`mise check:audit` covers. Then run the full audit:

```bash
mise check:audit
```

If any check fails, fix the issue and re-run before proceeding.

## Manual Checks

Read `{{ claude_md }}` to extract the manual checks from the Ship Gate section. Walk through each
one with the user. These require human judgment and cannot be automated.

Use `{{ contracts_spec_dir }}` and `{{ contracts_src_dir }}` for the contracts parity check, and
`{{ src_dir }}` for the unsafe and debug checks.

## Gate Decision

Present the results to the user:

- **All pass** → proceed with the cycle ship merge.
- **Any fail** → circuit breaker fires. Work does not ship. The problem must be re-shaped and
  re-pitched.

## After the Gate Passes

Read `{{ git_guide }}` to extract the Cycle ship merge steps. Follow them.
