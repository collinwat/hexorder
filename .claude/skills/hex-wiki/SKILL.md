---
name: hex-wiki
description:
    Read, create, and update pages in the GitHub Wiki. The wiki is a separate git repo cloned
    locally at .wiki/ (gitignored). Use this skill whenever you need to access wiki content, write a
    new wiki page, or push changes to the wiki. Other skills (e.g., research) depend on this skill
    for wiki operations.
---

# Wiki

The GitHub Wiki lives at `.wiki/`, a separately-managed git repo (gitignored by the main repo).

## Ensure `.wiki/` Exists

If `.wiki/` is missing, clone it:

```bash
mise run wiki:clone
```

To pull the latest content:

```bash
git -C .wiki pull
```

## Reading Pages

- `.wiki/Home.md` — landing page with links to all wiki pages
- Read any page directly: `.wiki/<Page-Name>.md`

## Creating or Editing Pages

1. Write the page file at `.wiki/<Page-Name>.md`
2. Update `.wiki/Home.md` to include a link to the new page
3. Update any relevant index pages (e.g., `.wiki/Research-Index.md` for research content)

## Committing and Pushing

```bash
cd .wiki
git add <files>
git commit -m "<descriptive message>"
git pull --rebase
git push
cd ..
```

If `git pull --rebase` surfaces conflicts, stop and resolve them with the user before pushing.
