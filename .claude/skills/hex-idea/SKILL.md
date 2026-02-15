---
name: hex-idea
description:
    Capture a raw idea as a GitHub Issue ad-hoc — outside the retrospective or triage workflow. Use
    when the user has a feature request, bug report, tech debt observation, or research question
    they want to record for future pitch consideration. Also use when the user invokes /hex-idea.
---

# Idea

Capture a raw idea as a GitHub Issue so it enters the pool for future triage and shaping. This is
the ad-hoc counterpart to the ideas captured during `/hex-retro` — use it anytime between cycles or
mid-build when something worth recording comes up.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name               | Value                                       | Description                                                              |
| ------------------ | ------------------------------------------- | ------------------------------------------------------------------------ |
| `project_root`     | repository root                             | Base directory; all paths are relative to this                           |
| `template_dir`     | `{{ project_root }}/.github/ISSUE_TEMPLATE` | Directory containing GitHub Issue YAML templates                         |
| `template_skip`    | `{{ template_dir }}/config.yml`             | Non-template file to ignore when reading templates                       |
| `pitch_label`      | `type:pitch`                                | Label that marks a template as a shaped proposal (excluded from capture) |
| `pitch_skill`      | `/hex-pitch`                                | Skill to redirect to when an idea is already shaped                      |
| `brainstorm_skill` | `/superpowers:brainstorm`                   | External brainstorming skill (may not be available)                      |

## 1. Discover Available Templates

Read `{{ template_dir }}` to learn what types are available and what fields each requires:

```bash
ls {{ template_dir }}
```

Read each `.yml` file (skip `{{ template_skip }}`). From each template, extract:

- **name** — the human-readable type name (from the `name:` field)
- **labels** — the labels auto-applied (from the `labels:` field)
- **fields** — each field's `id`, `label`, `description`, `type` (textarea, input, dropdown), and
  whether it has `validations.required: true`
- **dropdown options** — for dropdown fields, capture the list of allowed values

Hold this information in memory for the rest of the workflow. Do NOT hardcode template names,
fields, or dropdown options — always read them fresh from the files.

**Special case**: If a template's labels include `{{ pitch_label }}`, it is a shaped proposal — not
a raw idea. Exclude it from the type selection presented to the user but remember it exists (see
step 2).

## 2. Hear the Idea

Ask the user to describe their idea in their own words. A sentence or two is fine — the skill will
draw out the details.

## 3. Classify the Type

Based on the description and the templates discovered in step 1, suggest which template fits best.
Present all non-pitch templates as options (using the `name` and `description` from each template
file) and let the user confirm or override.

If the idea sounds like it could be a **pitch** (already shaped with a clear problem, appetite, and
solution), mention that `{{ pitch_skill }}` is the right tool and offer to switch. Raw ideas go
here; shaped proposals go to `{{ pitch_skill }}`.

## 4. Offer to Brainstorm

Ask the user: "Would you like to brainstorm this before filing, or is it ready to capture as-is?"

### If brainstorming

Check if `{{ brainstorm_skill }}` is available in the current skill list. If it is, invoke
`{{ brainstorm_skill }}` and provide the idea description and chosen template type as context. When
the brainstorm completes, resume this workflow at step 5 using the refined output.

If `{{ brainstorm_skill }}` is not available, run a focused brainstorming conversation inline. Use
the chosen template's description and required fields to guide the direction — help the user think
through what those fields need to capture. For example:

- If the template has a "steps to reproduce" field, help the user clarify the reproduction path
- If it has an "acceptance criteria" field, explore what "done" looks like
- If it has a "research question" field, help sharpen the question

Keep the brainstorm conversational — 2-4 rounds of back-and-forth is usually enough. The goal is to
produce a crisper description, not to shape a full pitch.

### If ready to capture

Skip ahead to step 5.

## 5. Gather Template Fields

Using the field definitions discovered in step 1 for the chosen template, ask the user for each
field. Use what was already discussed (from steps 2-4) to pre-fill answers — only ask for what's
still missing.

- For **required** fields: must be filled before posting
- For **optional** fields: present them but let the user skip
- For **dropdown** fields: present the allowed options from the template

## 6. Check for Duplicates

Search for existing issues that might cover the same ground:

```bash
gh issue list --search "<keywords from title and description>" --state all
```

If potential duplicates are found, present them to the user:

- If it's a true duplicate → skip filing, optionally comment on the existing issue
- If it's related but distinct → proceed, and reference the related issue in the body
- If no matches → proceed

## 7. Confirm and Post

Draft the issue and present it to the user for review. Show the title, labels, and full body.

Ask: "Ready to post this, or would you like to change anything?"

Once confirmed, create the issue. Use the labels from the template's `labels:` field, plus any area
labels selected by the user:

```bash
gh issue create --title "<title>" \
  --label "<labels from template>" --label "area:<area>" \
  --body "<body>"
```

Display the issue URL so the user can verify it.

## Quick Capture Mode

If the user provides a fully formed idea with a clear type (e.g., "bug: the camera snaps when
zooming past 10x"), skip the classification discussion and brainstorm offer. Still discover
templates (step 1), match the type, then go straight to gathering any missing template fields,
duplicate check, and confirmation.
