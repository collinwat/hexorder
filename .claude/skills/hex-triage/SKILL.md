---
name: hex-triage
description:
    Survey and group open GitHub Issues to identify clusters worth shaping into pitches. Use during
    cool-down to review the landscape of raw ideas, feature requests, bugs, and deferred items. Also
    use ad-hoc when exploring what problems exist before committing to shape anything. Also use when
    the user invokes /hex-triage.
---

# Triage

Survey the full pool of open issues, identify themes and clusters, and present them for review. The
output feeds into `{{ pitch_skill }}` when the user is ready to shape.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name           | Value                                       | Description                                    |
| -------------- | ------------------------------------------- | ---------------------------------------------- |
| `project_root` | repository root                             | Base directory; all paths are relative to this |
| `template_dir` | `{{ project_root }}/.github/ISSUE_TEMPLATE` | Issue templates with type and status labels    |
| `pitch_skill`  | `/hex-pitch`                                | Pitch shaping skill                            |

## Gather Issues

Read `{{ template_dir }}` to discover available issue templates and their `labels:` fields. For each
type and status label discovered, search for matching open issues:

```bash
gh issue list --state open --label "<label>"
```

Run this for every label found across all templates. Deduplicate issues that appear under multiple
labels.

For each issue, note the title, labels, and area.

## Identify Clusters

Group issues by theme, not just by label. Look for:

- **Problem clusters**: Multiple issues describing symptoms of the same underlying problem
- **Area clusters**: Several small items in the same plugin or area that could ship together
- **Dependency chains**: Issues that naturally sequence (A enables B enables C)
- **Standalone items**: Issues significant enough to shape on their own

Present the clusters to the user as a summary table:

| Cluster | Issues | Theme | Worth shaping? |
| ------- | ------ | ----- | -------------- |
|         |        |       |                |

## Review with the User

For each cluster, discuss:

- Is the underlying problem real and worth solving now?
- Is there a natural appetite (Small Batch or Big Batch)?
- Are there issues that should be closed as stale or duplicate?

## Cleanup

During review, handle housekeeping:

- Close stale issues: `gh issue close <number> --reason "not planned"`
- Close duplicates: `gh issue close <number> --comment "Duplicate of #<other>"`
- Add labels to untriaged issues: `gh issue edit <number> --add-label "<label>"`
- Remove triage status after processing: `gh issue edit <number> --remove-label "<label>"`

## Output

The result is a short list of clusters the user considers worth shaping. For each, run
`{{ pitch_skill }}` to shape it into a formal pitch for the betting table.
