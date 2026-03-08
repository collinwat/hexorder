---
name: hex-uat
description:
    Manage UAT test plans — create, update, delete, list, or output step-by-step test instructions.
    Use when writing UAT tests for a pitch, when reviewing UAT impact after a code change, when
    listing available tests, or when outputting test steps for the user to execute. Also use when
    the user invokes /hex-uat.
---

# UAT

Manage UAT test plans stored on the GitHub Wiki. Each pitch gets a wiki page with structured,
step-by-step acceptance tests that persist across sessions.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name             | Value                                                 | Description                                       |
| ---------------- | ----------------------------------------------------- | ------------------------------------------------- |
| `project_root`   | repository root                                       | Base directory; all paths are relative to this    |
| `uat_guide`      | `{{ project_root }}/docs/guides/uat.md`               | UAT workflow, result format, regression checklist |
| `pitch_template` | `{{ project_root }}/.github/ISSUE_TEMPLATE/pitch.yml` | Pitch template with UAT Criteria field            |
| `wiki_dir`       | `.wiki`                                               | GitHub Wiki local clone (gitignored by main repo) |
| `wiki_home`      | `{{ wiki_dir }}/Home.md`                              | Wiki landing page — UAT section links here        |
| `wiki_skill`     | `/hex-wiki`                                           | Wiki management skill (clone, commit, push)       |
| `page_prefix`    | `UAT-Pitch`                                           | Filename prefix for UAT wiki pages                |
| `page_pattern`   | `{{ page_prefix }}-<number>-<title>.md`               | Naming convention for UAT page files              |

## Which Workflow?

Determine the intent from the user's request or the current build phase:

| Intent                                     | Workflow   |
| ------------------------------------------ | ---------- |
| Write tests for a pitch                    | **Create** |
| A code change was made — assess UAT impact | **Impact** |
| Update existing test steps                 | **Update** |
| Remove a test or test page                 | **Delete** |
| Show which tests exist                     | **List**   |
| Output steps for the user to execute       | **Run**    |

If the user's intent is ambiguous, ask which workflow they need.

## Create

Write a new UAT test plan for a pitch.

### 1. Read the Pitch

Read `{{ pitch_template }}` to extract the UAT Criteria field structure. Then read the pitch issue:

```bash
gh issue view <pitch-number>
```

Extract:

- **UAT Criteria** — the SC-1, SC-2, etc. checklist items
- **Build Checklist** — the scopes that map to each criterion
- **Solution section** — the UI interactions, contract types, and runtime behavior described

### 2. Read the UAT Guide

Read `{{ uat_guide }}` to extract the result format, per-scope workflow, and regression checklist
conventions. Hold these in memory for structuring the test page.

### 3. Map Criteria to Steps

For each SC criterion from the pitch, identify:

- **What UI panels and controls are involved** — read the relevant `src/editor_ui/render_*.rs` files
  to discover actual widget names, button labels, form fields, and panel locations.
- **What setup is required** — which Design/Rules/Play mode configuration must exist before the test
  can execute.
- **What observable result confirms the criterion** — what the user sees, clicks, or verifies.

Do NOT guess UI element names. Read the source code to discover the actual labels, panel names, tool
button text, and form field identifiers. Present the discovered UI elements to the user for
confirmation before writing steps.

### 4. Write the Wiki Page

Create the page at `{{ wiki_dir }}/{{ page_pattern }}` with this structure:

```markdown
# UAT — Pitch #<number>: <title> (<version>)

<One-line description of what the tests cover.>

See [UAT Guide](../docs/guides/uat.md) for the general workflow and result recording format.

---

## SC-N: <criterion name>

**What it proves:** <one sentence restating the criterion>

### Setup (Editor)

<numbered steps with exact UI interactions>

### Setup (Play mode)

<numbered steps if play mode setup is needed>

### Test

<numbered steps describing the verification>

### Pass criteria

- [ ] <observable outcome 1>
- [ ] <observable outcome 2>

---

## Change Impact Protocol

<instructions for assessing code changes against these tests>

### Change log

| Date | Commit | Change | Affected SC | Steps updated? |
| ---- | ------ | ------ | ----------- | -------------- |
```

### 5. Update Wiki Home

Add the page to the UAT section in `{{ wiki_home }}`. If no UAT section exists, create one above the
Retrospectives section.

### 6. Publish

Use `{{ wiki_skill }}` conventions to commit and push:

```bash
cd {{ wiki_dir }}
git add <page> Home.md
git commit -m "Add UAT test plan for pitch #<number>"
git pull --rebase
git push
```

## Impact

Assess the UAT impact of a code change.

### 1. Identify the Change

Read the diff or commit message to understand what changed:

```bash
git diff HEAD~1 --stat
git log -1
```

### 2. Find Affected UAT Pages

List existing UAT pages:

```bash
ls {{ wiki_dir }}/{{ page_prefix }}-*.md
```

For each page, scan for references to the changed files, panels, or components. Read the Change
Impact Protocol section if present.

### 3. Assess Impact

For each potentially affected test:

1. **Identify affected SC criteria** — which tests exercise the changed code path?
2. **Assess appropriateness** — was the change necessary? Does it alter expected behavior or fix a
   bug?
3. **Determine step changes** — do any setup steps, test steps, or pass criteria need updating?

Present the assessment as a table:

| SC   | Impact     | Steps need update? | Details                            |
| ---- | ---------- | ------------------ | ---------------------------------- |
| SC-1 | None       | No                 | Change does not affect pathfinding |
| SC-2 | Behavioral | Yes                | Spawn form field renamed           |

### 4. Update if Needed

If steps need updating, switch to the **Update** workflow for each affected test. Log the change in
the Change log table on the wiki page.

## Update

Modify existing test steps on a UAT wiki page.

### 1. Read the Existing Page

Read the UAT page from `{{ wiki_dir }}/{{ page_prefix }}-<number>-*.md`.

### 2. Identify What Changed

Compare the current test steps against the actual UI or behavior. Read the relevant source files to
verify that widget names, panel locations, and expected behaviors still match.

### 3. Edit the Page

Update the affected steps. Preserve the page structure and all unaffected tests.

### 4. Log the Change

Append a row to the Change log table at the bottom of the page:

| Date | Commit | Change | Affected SC | Steps updated? |
| ---- | ------ | ------ | ----------- | -------------- |

### 5. Publish

Commit and push using `{{ wiki_skill }}` conventions.

## Delete

Remove a test or an entire UAT page.

### 1. Confirm Scope

Ask the user: delete a single SC test from a page, or the entire page?

### 2. Remove Content

- **Single test**: Remove the SC section from the page. Update the page heading if the test count
  changed.
- **Entire page**: Delete the file from `{{ wiki_dir }}`. Remove the link from `{{ wiki_home }}`.

### 3. Publish

Commit and push using `{{ wiki_skill }}` conventions.

## List

Show which UAT tests exist.

### 1. Find All Pages

```bash
ls {{ wiki_dir }}/{{ page_prefix }}-*.md 2>/dev/null
```

### 2. Extract Test Summaries

For each page, read the file and extract:

- Pitch number and title (from the `#` heading)
- Version
- SC criteria names and their pass/fail state (from the `- [ ]` / `- [x]` checkboxes)

### 3. Present

Display a summary table:

| Pitch                    | Version | Tests                  | Status   |
| ------------------------ | ------- | ---------------------- | -------- |
| #236 Scenario Primitives | v0.22.0 | SC-1, SC-2, SC-3, SC-4 | Untested |

## Run

Output test steps for the user to execute.

### 1. Find the Page

Locate the UAT page for the requested pitch:

```bash
ls {{ wiki_dir }}/{{ page_prefix }}-<number>-*.md
```

Read the page content.

### 2. Select Tests

Ask the user which tests to run:

- **All** — output all SC tests on the page
- **Specific** — output only the requested SC numbers (e.g., "SC-2 and SC-3")

### 3. Output Steps

For each selected test, output the full step-by-step instructions from the wiki page — setup, test,
and pass criteria. Include the "What it proves" line so the user knows the goal.

Do NOT regenerate or rephrase the steps. Output them exactly as written on the wiki page. The wiki
is the source of truth.

### 4. Collect Results

After the user reports pass/fail for each criterion, record the results as a comment on the pitch
issue following the format in `{{ uat_guide }}`:

```bash
gh issue comment <pitch-number> --body "$(cat <<'EOF'
**UAT Results** (commit <sha>):

- [x] SC-1: <criterion> — PASS
- [ ] SC-2: <criterion> — FAIL: <what happened>
EOF
)"
```

Update the pass criteria checkboxes on the wiki page to reflect the results.
