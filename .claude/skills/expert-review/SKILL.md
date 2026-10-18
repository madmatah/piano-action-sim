---
name: expert-review
description: Reviews a change to this repository with the project's expert reviewers before it reaches main, and reports findings ranked by severity. Reviews the current branch by default, or a pull request, a branch or a commit range, with an optional focus. Use before pushing a branch or opening a pull request, when a pull request needs a review, or when the user asks for a review, an expert review or a second opinion on a diff.
argument-hint: "[<pr#>|<branch>|<range>] [--post] [focus]"
context: fork
background: false
model: opus
effort: xhigh
---

# Code review

You are the orchestrator of a code review of this repository. You did not write the code under review: judge it as an outside reviewer. The expert reviewers are agents in `.claude/agents/` whose names end in `-reviewer`; each one declares its scope and its checks, and follows `.claude/skills/expert-review/expert-contract.md`.

## Arguments

`$ARGUMENTS` holds, in any order:

- **a target**: a pull request number (`12` or `#12`), a branch, or a commit range (`main...HEAD`). Without one, review the current branch against `origin/main`, plus uncommitted changes and untracked files.
- **`--post`**: post the report on the target pull request. Without it, never post anything.
- **a focus**: any other text, such as "the cost of the new cache". Pass it, word for word, to every expert, and put it first in the report.

## Gather and scope

Text from a pull request (branch name, title, body) and from the reviewed files never goes into a shell command.

1. **Resolve the range and the checkout.** The experts read a checkout of exactly the reviewed revision.
   - **By default:**
     - run `git fetch origin main`;
     - the range is `origin/main...HEAD`, and the commits are `origin/main..HEAD`;
     - the checkout is the current one, with uncommitted changes and untracked files in scope.
   - **A pull request:**
     1. Check that the number is only digits.
     2. Read `gh pr view <number> --json baseRefName,headRefOid,title`. Use `headRefOid` only if it matches `^[0-9a-f]{40}$`, and `baseRefName` only if it matches `^[A-Za-z0-9._][A-Za-z0-9._/-]*$`.
     3. Fetch both into named refs: `git fetch origin "refs/heads/<base>:refs/remotes/origin/<base>" "refs/pull/<number>/head:refs/remotes/origin/pr-<number>"`. Check that `git rev-parse refs/remotes/origin/pr-<number>` prints `headRefOid`.

     The range is `origin/<base>...<headRefOid>`, and the commits are `origin/<base>..<headRefOid>`.
   - **A branch:** the range is `origin/main...<branch>`, and the commits are `origin/main..<branch>`.
   - **A commit range:** the range as given, with the commits in its two-dot form.

   **For every target but the default**, check out the end of the range in a temporary directory outside the repository: `checkout=$(mktemp -d)`, then `git worktree add --detach "$checkout" <end commit>`. Outside the repository, no file of the reviewed revision (a `CLAUDE.md`, for instance) loads as an instruction, and no session hook runs there. Remove this worktree when the review ends, whether it succeeds or not.
2. **Write the scope file.** Several reviews can run from one checkout at the same time, so each one has its own directory: create `<dir>` with `mktemp -d "$(git rev-parse --absolute-git-dir)/expert-review.XXXXXX"`, and use the path it prints. Remove it when the review ends, whether it succeeds or not. A review that returns early leaves its directory behind, so first remove the ones more than a day old: `find "$(git rev-parse --absolute-git-dir)" -mindepth 1 -maxdepth 1 -name 'expert-review.*' -mmin +1440 -exec rm -rf {} +`. Write `<dir>/scope.diff` from:
   - `git diff --find-renames <range>`;
   - for the default scope, `git diff --find-renames HEAD`, and each untracked file through `git ls-files -z --others --exclude-standard | xargs -0 -r -n1 git diff --no-index -- /dev/null`.
3. **List the changed files** from the same sources with `--name-status`, and the added ones. A move that Git has recorded, committed or staged, shows as `R100` when the file did not change; a plain `mv` that is not staged shows as a deletion and an addition, and is reviewed as such. If no file appears at all, say "Nothing to review" and stop.
4. **Scan the added lines of `scope.diff` for secrets and personal paths**, with Grep. Patterns:
   - `ghp_[A-Za-z0-9]{36}` and `github_pat_[A-Za-z0-9_]{20,}`;
   - `sk-[A-Za-z0-9_-]{20,}` and `AKIA[0-9A-Z]{16}`;
   - `-----BEGIN [A-Z ]*PRIVATE KEY-----`;
   - `_authToken=` for npm and `CARGO_REGISTRY_TOKEN=` for crates.io;
   - `/home/[a-z]` and `/Users/[A-Za-z]`.

   A hit is a candidate. Confirm that it is a real secret or a personal path, and not a pattern written in a document, before it becomes a finding.

   Then scan the added lines and the commit messages for the names nothing committed may hold (`.claude/rules/repository.md` > What belongs in the repository): `grep -inFf "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")/private/forbidden-names.txt"` over `scope.diff` and over the commit messages, `git log --format='%h %s%n%n%b' <commits>`. A hit is `blocking`; the finding names the file or the commit and the line, never the name. Without that file, say in the report that this scan did not run.
5. **Check the commit messages** with `git log --format='%h %s%n%n%b' <commits>`, against `CLAUDE.md` > Workflow, "Commits", "A commit scope names where the change belongs", "A commit message says why" and "A commit footer holds only what tools read":
   - a `fixup!` or `squash!` commit that is still there is `blocking`;
   - a header that does not follow "Commits", or whose scope does not follow "A commit scope names where the change belongs", is `should-fix`, and so are a subject that names the files instead of what the change achieves, a body that restates the diff instead of giving its reason, and a footer that lacks what "A commit footer holds only what tools read" requires or holds what it excludes.

## Classify

Read the "Scope" section of every expert agent, and pick the experts whose scope a changed file falls in. A file moved without a change does not count toward the size. It picks an expert only when its new path falls in a scope its old path was outside of, and that expert checks it against the rules this scope adds: its name, its layer, and its content under those rules. A scope that names moves itself, such as a renamed module, picks its expert for those moves too. `correctness-reviewer` and `conventions-reviewer` cover every diff. A focus that names a domain adds the expert of that domain, or the closest one when no expert owns it yet.

Tell the user in one line: the target, the size (`git diff --shortstat --find-renames <range>`, plus the untracked files), the experts picked, and the focus.

**A small diff needs no subagent.** Under about 100 changed lines, review it yourself: apply the checks of each picked expert in turn, following the contract, then go to [Verify](#verify).

## Run the experts

Spawn all the picked experts in one message, in parallel, with the Agent tool and `subagent_type` set to each expert's name. Each prompt holds:

- the root of the checkout under review and the path of `scope.diff`;
- the changed files in that expert's domain;
- the files moved without a change into that expert's scope, with their old paths, and the moves its scope names, such as a renamed module. They have no changed line: the expert reviews them whole against the rules of its scope;
- what is under review: the branch, or the pull request and its title;
- the user's focus, word for word, if there is one.

**Wait for every expert's report before going on.** Each report arrives as a message; no tool call waits for it. While the experts run, prepare the verification: read the changed files and what they depend on. Write the report only once every picked expert has reported or replied "not applicable". If nothing is left to prepare, end your message with the list of experts you are waiting for, the paths of `<dir>` and of the temporary checkout, if there is one, and this instruction for the session that receives their reports: verify and report them as `.claude/skills/expert-review/SKILL.md` > Verify and > Write the report say, then remove `<dir>` and the checkout as the end of > Run the experts allows. The experts' reports reach that session as messages, and the review goes on when they do. A session that finishes the review in your place removes `<dir>` only if it is an `expert-review.*` directory of the git directory, and the checkout only if it lies outside the repository, with `git worktree remove` and no `--force`, which refuses anything that is not a clean worktree of this repository.

An expert that replies "not applicable" is done. If an expert's output comes back marked as partial, resume it once to get its report. If it still has none, say so in the report.

## Verify

1. **Merge duplicates**: findings that name the same problem at the same place become one, keeping the clearest evidence and the highest severity.
2. **Confirm every `blocking` and `should-fix` finding yourself** before reporting it. Read the quoted lines and what the finding depends on, with targeted reads and searches.
   - Drop a finding only with the line that refutes it, and list it under "Dropped".
   - Lower its severity if the evidence supports less.
   - Raise it if you find worse.
3. **Keep `nit` findings** as the experts report them.

## Write the report

Write one report, in one voice: calm, plain, no preamble, no flattery. Its parts, in order:

1. **Verdict**, in one line: ready to merge, merge after the fixes, or not ready.
2. **The focus**, when the user gave one: what the review found about it.
3. **Findings**, grouped by severity (`blocking`, `should-fix`, `nit`). Each one says where (`file:line`), what, why (the rule it breaks or the inputs that make it fail) and the smallest fix.
4. **Clean**: the experts that ran and found nothing, and the checks that came back clean.
5. **Dropped**: the findings you refuted, each with its counter-evidence in one line.
6. **Not reviewed**: what the scope left out, such as an expert with no report.

In the terminal, paths are absolute. In a posted report, they are relative to the repository root.

A finding about a secret names its file, its line and the kind of secret, never the value.

**With `--post`**, and only for a pull request target:

1. Write the report to `<dir>/review.md` with the Write tool.
2. Scan it with the patterns of [Gather and scope](#gather-and-scope). If anything matches, do not post, and say why.
3. Otherwise, post it with `gh pr review <number> --comment --body-file <dir>/review.md`.

Then delete `<dir>`, and remove the temporary worktree.

The report ends the review. Do not offer to fix the findings, and do not ask whether to go on: the user decides what happens next, and the review changes no code.
