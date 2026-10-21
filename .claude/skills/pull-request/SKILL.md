---
name: pull-request
description: Takes a piece of work in this repository from a new branch to a merged pull request, and reviews someone else's pull request. Use when starting a piece of work, planning it, writing its code, stacking a branch on an unmerged one, preparing the commits of a branch before pushing it, opening or updating a pull request, folding review fixes, or moving a branch after its base changed or was merged.
---

# Pull request

`CLAUDE.md` > Workflow holds the rules this procedure applies. Below, `<base>` is `origin/main`, or the unmerged branch the work is stacked on. Write each file that this procedure names, such as a plan, the findings of a round or a commit message, with the Write tool, never through a shell: in a heredoc, or in `git commit -m "…"`, the backquoted text that findings, plans and messages quote runs as commands. Give git a message from a file in a directory that `mktemp -d` creates outside the worktree, with `git commit -F <file>`.

## Start a branch

1. Run `git fetch origin`, then `git worktree add --no-track -b <type>/<topic> .claude/worktrees/<name> origin/main`, with a type from `CLAUDE.md` > Workflow, "Commits".
2. `claude --worktree <name>` names its branch `worktree-<name>`: rename it with `git branch -m <type>/<topic>`.
3. Install the dependencies of a worktree you add during a session, as `CLAUDE.md` > Workflow, "Dependencies" says.

## Plan the work

Plan the work as `CLAUDE.md` > Workflow, "Plan before writing code" says, and revise the plan the same way when the code or a review shows it wrong:

1. Spawn the `planner` agent with its brief: the worktree and its base, the goal and where it comes from, the constraints and the decisions you know, and the path of `private/`, as `CLAUDE.md` > Where the work stands gives it. For a revision, add the path of the plan, what showed it wrong, such as the findings of a review, and the commits of the branch with their files, from `git log --format='%h %s' --name-only <base>..HEAD`. Write each plan it returns, without its `Answers` section, to the path of the plan that `CLAUDE.md` > Where the work stands gives, and commit it as it says.
2. Spawn the `plan-critic` agent with the worktree, its base, the goal, the constraints, the decisions and the owner's answers you know, and the path of the plan, and, for a revision, what showed the plan wrong and the commits of the branch, as the planner got them. Resume the planner with the critic's blocking and should-fix points, rather than spawning a new one that would read the code again, then resume the critic with the planner's answers. Go on until the critic says `ready`. After three rounds, ask the owner about what remains, or, when the owner does not follow the work, stop and give the plan and its open points in your final report.
3. When the owner follows the work, show him the plan and its open questions, and wait for his approval: a change he asks for goes to the planner, then to the critic. Otherwise, take the answer that the plan recommends to each open question, and name it in the pull request. A trade term in doubt is the exception: the plan recommends no word for it, and you ask the owner for his own word, without proposing one, before the code uses it, as `.claude/rules/vocabulary.md`, "The owner is the authority on the trade's vocabulary" says; never take a word by default.

## Write the code

Write the code from the plan, or give it to an agent on the model and at the effort that `.claude/rules/configuration.md` > Claude Code automation, "Each agent runs on the model and at the effort its task needs" gives an implementation, with the worktree, its base and the path of the plan. When the plan proves wrong, stop, and revise it as [Plan the work](#plan-the-work) says. If an agent was writing the code, commit or drop its work in progress, as the revised plan says, then give the rest to a new agent, with the revised plan and the commits so far: the one that stopped carries a context that only grows.

## Stack on an unmerged branch

- **A branch may start from another unmerged branch whose work it needs:** give that branch instead of `origin/main` to `git worktree add`. Until it is merged, it is `<base>`: review with `/expert-review <branch>...HEAD`, autosquash onto it, and open the pull request against it, naming it in the Proposal.
- **Whenever the base is rewritten,** by its review fixes or by its merge, move only your own commits: `git rebase --onto <new base> $(git merge-base --fork-point <branch> HEAD)`. The new base is the branch itself, or `origin/main` once it is merged; the old copies of its commits would conflict with the new ones. Once the base is merged, retarget the pull request to `main`.
- **The owner of a base keeps it until the branches built on it have moved.** Each of those branches is moved by its own owner.

## Shape the history

1. Before pushing, rewrite the branch into atomic commits, as `CLAUDE.md` > Workflow, "Commits" and "A commit message says why" ask, and write each message with the `writing-commits` skill. Interactive editors are unavailable: fold a change into its commit with `git commit --fixup=<commit>`, then `GIT_SEQUENCE_EDITOR=: git rebase -i --autosquash <base>`.
2. Check that every commit passes: `git rebase --exec 'pnpm verify' <base>`.

## Review

1. Run `/expert-review` on the branch, or `/expert-review <base>...HEAD` when it is stacked. While it reads the worktree, run nothing that moves the worktree between commits, such as `git rebase --exec`: the experts would read another revision than the one under review.
2. Settle each finding as `CLAUDE.md` > Workflow, "Review before pushing" says. Give the findings to fix to a new `fixer` agent, with its brief: the worktree and its base, the purpose of the branch, each verified finding, and the commit each fix belongs to. It commits each fix as a `fixup!` commit, or as an `amend!` commit for a commit message, as [Fold review fixes](#fold-review-fixes) shows. A finding that needs a change of design, or that a fix brought into the code it fixed, ends the round instead: revise the plan of the branch with all the findings of the round, as [Plan the work](#plan-the-work) says, give the change to a new agent, as [Write the code](#write-the-code) says, shape the history again, as [Shape the history](#shape-the-history) says, and review the whole branch again, as the first round does. Keep the cost of each round, from its report, for the decision that `CLAUDE.md` > Workflow, "Rounds of review converge, or the owner decides" leaves to the owner. Before you fold them, review the fixes alone, against the findings they answer: write those findings, as the review reported them, to a file in a directory that `mktemp -d` creates outside the worktree, and run `/expert-review <the commit before the first fix of the round>...HEAD --fixes <file>`. Remove the directory once the round's report arrives. Go on round by round as `CLAUDE.md` > Workflow, "Review before pushing" and "Rounds of review converge, or the owner decides" say, then fold them.
3. If an answer of its triage or of an expert reaches you rather than the review, the review handed back its report too soon: set that report aside, send the answer to the review with SendMessage, addressed to the name it runs under, such as `expert-review`, and act only on the report it hands back next.

## Push and open the pull request

1. Push the branch: `git push -u origin HEAD` the first time.
2. Write the body from `.github/pull_request_template.md`, which `gh pr create --body-file` does not apply: fill each section in place of its comment, and tick a checklist item only once it holds.
3. Open the pull request against `main`, or against the base it is stacked on: `gh pr create --base <branch> --title "<title>" --body-file <file>`. Without a terminal, `gh` needs both the title and the body.
4. Make sure CI (`pnpm verify` on GitHub Actions) passes on it: `gh pr checks <number> --watch`.

## Fold review fixes

1. Commit each fix with `git commit --fixup=<commit>`, naming the commit it belongs to. A fix to the message of a commit goes in an `amend!` commit, since a `fixup!` commit keeps the message of its target: in a directory that `mktemp -d` creates outside the worktree, write, with the Write tool, a file whose first line is `amend! ` followed by the header of the commit, whole, as `git log -1 --format=%s <commit>` prints it, since autosquash matches that line, then a blank line and the whole new message; run `GIT_EDITOR="cp <file>" git commit --fixup=reword:<commit>`, or `--fixup=amend:<commit>` to fold the staged changes with it, and remove the directory. Neither `-m` nor `-F` combines with these forms.
2. Run `GIT_SEQUENCE_EDITOR=: git rebase -i --autosquash <base>`. Git before 2.44 ignores `--autosquash` without `-i`, and the `-i` form works with every version.
3. Check that `git log --format=%s <base>..HEAD` lists no `fixup!`, `amend!` or `squash!` commit. If one is left, note its hash, drop it, re-apply its change with `git cherry-pick -n <hash>`, and commit it again: a `fixup!` commit as above, and an `amend!` commit as above too, with the message file built from `git log -1 --format=%B <hash>`, whose first line must name its target's whole header. Then fold again.
4. Check every commit again, as [Shape the history](#shape-the-history) does.
5. Push with `git push --force-with-lease`, then keep the body true: update every section the fixes changed.

## Merge

- **The user merges the pull request,** as `CLAUDE.md` > Workflow, "A branch reaches `main` only through a pull request whose CI passes" says.
- **After the merge,** keep the branch until the branches stacked on it have moved, as [Stack on an unmerged branch](#stack-on-an-unmerged-branch) says.

## Review someone else's pull request

From the main checkout, run `/expert-review <number>`. The skill checks the pull request out in a temporary directory outside the repository, so it never touches a session's worktree.
