---
paths:
  - "**/*.md"
  - "docs/**"
  - ".claude/**"
  - ".github/*.md"
---

# Documents and agent guidance

Each rule ends with its check, or says it is *not mechanically checked*.

## Present tense

- **Present tense, no history**: justification stays, narration goes. How a decision came about lives in `private/`, whose notes are history by design. *Not mechanically checked.*
- **Built, planned or hypothesis**: every component a published document describes is labeled **built** or **planned**; an unsettled decision is a **hypothesis**, with the test that will settle it. *Not mechanically checked.*

## Where a statement lives

Each statement has one home, so that it never drifts from a copy.

- **A rule about what the repository holds,** its code, tests, data, configuration and documents, lives in `.claude/rules/`, once and in full, in the file of its concern. *Not mechanically checked.*
- **Session conduct** lives in `CLAUDE.md`: the language, the project, how to work with the owner, where the work stands, the workflow, the commands, and one index line per rules file. Every session loads all of it, so it stays under 200 lines. Check: `wc -l CLAUDE.md`.
- **The design and its reasons** live in `docs/FOUNDATIONS.md`, the study it draws on in `docs/research/`. They may say what a rule protects and why, but do not restate the rule. *Not mechanically checked.*
- **A procedure** lives in a skill, under `.claude/skills/`. *Not mechanically checked.*
- **Setup and usage for people** live in `README.md`, and the credits in `CREDITS.md`. *Not mechanically checked.*

## References

- **A reference names a file, and one of its headings when it points at a part:** `docs/FOUNDATIONS.md` > Determinism, stated correctly. Inside a markdown document it links the heading instead, `[Stages](#stages)`, or the other document's, `[FOUNDATIONS > Stages](../FOUNDATIONS.md#stages)`. When a section holds several rules or items, the reference adds the bold name, in quotes. *Not mechanically checked.*
- **Never a number:** no section, rule, step or item number of a document of this repository, since a number says nothing about its target and shifts when something is inserted before it; so no heading of ours is numbered. A source is cited as its authors number it: `[V, Hirschkorn 2004 §4.3]`. *Not mechanically checked*: review takes a `§` outside a source's citation, or `rule` followed by a number, for a finding.
- **Every cited file and heading exists.** A change that renames or removes a heading updates every reference to it, in the documents, the code's comments and the messages of the tools. *Not mechanically checked.*

## Rules files

- **One file per concern,** in `.claude/rules/`: a concern that spans several directories stays in one file. *Not mechanically checked.*
- **A rules file is self-contained.** It states each rule in full, with enough of its reason to apply it, and never sends its reader to another document for a rule; it may point to `docs/FOUNDATIONS.md` for the longer argument. *Not mechanically checked.*
- **Each rule ends with its check,** a test name, a grep or a tool of the gate, or says it is *not mechanically checked*, so that "done" is something the tree shows. A rule known to have exceptions keeps them beside it until they are fixed. *Not mechanically checked.*
- **`paths` decide when a file loads.** Claude Code loads a file with `paths` when a session reads or edits a matching file, and a file without `paths` in every session. Only what every session needs goes in a file without `paths`. *Not mechanically checked.*
- **Each rules file has its line in `CLAUDE.md` > Rules,** which names its topics and when to read it, without restating its rules: a subagent in another worktree, or a compacted session, can miss a load by `paths`. Check: `ls .claude/rules` against `CLAUDE.md` > Rules.
