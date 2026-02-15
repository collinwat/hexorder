---
name: hex-wiki
description:
    Read, create, and update pages in the GitHub Wiki. Use when accessing wiki content, writing new
    pages, or pushing changes. Other skills depend on this for wiki operations. Also use when the
    user invokes /hex-wiki.
---

# Wiki

Manage the GitHub Wiki — read, create, edit, and publish pages.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name              | Value                    | Description                                       |
| ----------------- | ------------------------ | ------------------------------------------------- |
| `project_root`    | repository root          | Base directory; all paths are relative to this    |
| `wiki_dir`        | `.wiki`                  | GitHub Wiki local clone (gitignored by main repo) |
| `wiki_home`       | `{{ wiki_dir }}/Home.md` | Wiki landing page with links to all pages         |
| `wiki_clone_task` | `mise run wiki:clone`    | Task to clone the wiki repo                       |

## Ensure `{{ wiki_dir }}/` Exists

If `{{ wiki_dir }}/` is missing, clone it:

```bash
{{ wiki_clone_task }}
```

To pull the latest content:

```bash
git -C {{ wiki_dir }} pull
```

## Reading Pages

- `{{ wiki_home }}` — landing page with links to all wiki pages
- Read any page directly: `{{ wiki_dir }}/<Page-Name>.md`

## Creating or Editing Pages

1. Write the page file at `{{ wiki_dir }}/<Page-Name>.md`
2. Update `{{ wiki_home }}` to include a link to the new page
3. Update any relevant index pages (e.g., `{{ wiki_dir }}/Research-Index.md` for research content)

## Committing and Pushing

```bash
cd {{ wiki_dir }}
git add <files>
git commit -m "<descriptive message>"
git pull --rebase
git push
cd ..
```

If `git pull --rebase` surfaces conflicts, stop and resolve them with the user before pushing.
