# The repository

Each rule ends with its check, or says it is *not mechanically checked*.

## What belongs in the repository

- **The repository holds the current state of the project, and nothing else.** How the work got here, where it stands, what was tried and what is planned next live in `private/`, never here: status, handoffs, decision records, roadmaps, plans, research notes, and the task markers `TODO`, `FIXME`, `XXX` and `HACK`. Check: `git grep -nwE 'TODO|FIXME|XXX|HACK' -- ':!.claude/rules/repository.md'` finds nothing.
- **`private/` is one-way**: gitignored and its own repository, it holds the owner's own material and the project's tracking. It may cite `docs/`; no published file cites it (`CLAUDE.md` and the agent guidance of `.claude/`, the agents' way in, are the exception), and a claim it shows refuted gets an inline *do not use* warning where it is published instead. `git mv` into it leaves the file tracked: untrack it too. Check: `grep -rln 'private/' README.md docs data tools crates apps scripts` and `git ls-files private` find nothing.
- **Nothing committed names the owner's instruments or other projects**, in a file or a commit message, because the repository is meant to be published and the owner's material is not. Check: `git diff --cached | grep -iFf private/forbidden-names.txt` finds nothing.
- **No secret, credential, `.env` file or personal data,** such as a path into a home directory. *Not mechanically checked.*
- **No file committed by accident:** build output, logs, temporary captures. `.gitignore` names what the stack writes. *Not mechanically checked.*
- **A change updates every document that describes what it changes, in the same change:** the rules file of a rule, `docs/FOUNDATIONS.md` for the design and its reasons, the skill of a procedure, `README.md` for setup and usage, `CLAUDE.md` for how a session works, and `CREDITS.md` for what is reused. Where a document and the code disagree, the code is right and the document is the bug. *Not mechanically checked.*

## External work

- **Credit every external work the project reuses in `CREDITS.md`, in the same change that brings it in:** a library shipped in a build, code or an algorithm ported from another project, data taken from someone else's work, an asset. *Not mechanically checked.*
- **Each entry says** what is reused, where it lives in the repository, where it comes from and under which terms. When a license asks for its notice to be kept, the notice is copied verbatim. *Not mechanically checked.*
- **The works the design comes from are named,** with their locators, in `docs/FOUNDATIONS.md` > Sources, and every number taken from one carries its locator in `data/params/`. The corpus itself stays in `data/references/`, which git ignores: third-party copyright, kept for study and never redistributed. Check: `git ls-files data/references` finds nothing.
