## Purpose

<!-- Why this pull request exists: the need or the problem, in a few sentences. -->

## Proposal

<!-- The solution in broad strokes: what it is made of, not how each line works. If the branch is stacked on another, name its pull request. -->

## How to test

<!-- Steps for the reviewer: what to run, what to try, what to look for. -->

<details>
<summary>How it was tested</summary>

<!-- What the author ran and observed: the tests added, the manual checks, captures of anything visible. -->

</details>

## Checklist

<!-- Tick an item only once it holds. -->

### Quality

- [ ] `pnpm verify` passes on every commit.
- [ ] `/expert-review` ran on the completed branch, and every finding is fixed or answered here.

### History

- [ ] Each commit is atomic, and its message follows `CLAUDE.md` > Workflow: the header of "Commits", a body that says why unless the change speaks for itself, and a footer as "A commit footer holds only what tools read" says; no `fixup!`, `amend!` or `squash!` commit is left.

### Documentation

- [ ] The documents describe the new state: `docs/FOUNDATIONS.md`, `README.md`, `CLAUDE.md`, `.claude/`, and a reading that moved is explained where the golden sheet was rewritten.
- [ ] Every external work brought in, including a library shipped in a build, is credited in `CREDITS.md`.

### Hygiene

- [ ] No secret, credential, `.env` file or personal data is committed, and nothing names the owner's instruments or other projects.
- [ ] No project-management material is committed: status, roadmap, decisions, handoffs, notes, `TODO`.
- [ ] No file is committed by accident: build output, logs, temporary captures.
