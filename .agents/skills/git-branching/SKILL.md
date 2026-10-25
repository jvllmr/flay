---
name: git-branching
description: Load this skill before any Git branch creation, commit, or push in this repository. Sets the default branch workflow: never commit on main, use a task branch.
---

# Git branching

`main` is push-protected. Commits on `main` cannot be pushed directly, so
never commit there unless the user explicitly asks for it.

## Rules

1. Before starting any change, base new work on the latest `main`:
   `git checkout main && git pull`, then create a task branch:
   `git checkout -b <type>/<short-slug>` (e.g. `feat/bundle-namespaces`,
   `fix/treeshake-loc-provider`, `chore/update-deps`).
2. Default branch for new work is the current task branch, not `main`. If the
   working tree is on `main`, create the task branch before committing.
3. Commit only on the task branch. If asked to work on `main` anyway, ask the
   user to confirm in plain words first.
4. Never push to `main`. Deliver via a PR from the task branch.
5. Updating an existing PR: use its existing head branch, keep its history.
6. Before any branch switch or checkout of files, check `git status` for
   unsaved work. Preserve it.

## Commands

- Check state: `git status --short && git branch --show-current`
- New task branch off latest main: `git checkout main && git pull && git checkout -b <type>/<short-slug>`
