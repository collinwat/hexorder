---
name: triage
description:
    Survey and group open GitHub Issues to identify clusters worth shaping into pitches. Use during
    cool-down to review the landscape of raw ideas, feature requests, bugs, and deferred items. Also
    use ad-hoc when exploring what problems exist before committing to shape anything. Also use when
    the user invokes /triage.
---

# Triage

Survey the full pool of open issues, identify themes and clusters, and present them for review. The
output feeds into `/pitch` when the user is ready to shape.

## Gather Issues

Pull issues from all relevant sources:

```bash
gh issue list --state open --label "status:triage"      # unprocessed raw ideas
gh issue list --state open --label "status:deferred"     # deferred from prior cycles
gh issue list --state open --label "type:feature"        # feature requests
gh issue list --state open --label "type:bug"            # bugs
gh issue list --state open --label "type:tech-debt"      # tech debt
gh issue list --state open --label "type:research"       # open research questions
```

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
- Add labels to untriaged issues: `gh issue edit <number> --add-label "type:<type>"`
- Remove `status:triage` after processing: `gh issue edit <number> --remove-label "status:triage"`

## Output

The result is a short list of clusters the user considers worth shaping. For each, run `/pitch` to
shape it into a formal pitch for the betting table.
