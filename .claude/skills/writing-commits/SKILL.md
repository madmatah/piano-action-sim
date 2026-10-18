---
name: writing-commits
description: Writes the commit messages of this repository, so that each one says what its change achieves and why it was made, in the project's commit style. Use when committing, amending or rewording a commit, splitting work into commits, shaping a branch's history before pushing it, or checking the commit messages of a branch.
---

# Writing commits

`CLAUDE.md` > Workflow states the rules of a commit: "Commits", "A commit scope names where the change belongs", "A commit message says why" and "A commit footer holds only what tools read". This skill applies them to each commit that will be pushed, whether you write it at once or while you rewrite the branch before pushing. A work-in-progress commit or a `fixup!` commit needs none of it, since the rewrite replaces it. Where a rule leaves a choice, the sections below say how to make it. Write from the rules, not from `git log`, whose older messages may predate them.

## Draw the commits

Each commit holds one reason to change ("Commits"), so that a reader can review it, revert it or find it with `git bisect` on its own.

- **A subject that needs an "and"** usually holds two commits. Two unrelated fixes made the same afternoon are two commits.
- **The test that pins a fix** goes in the commit of the fix, since every commit passes the gate on its own.
- **A correction to an earlier commit of the branch** is folded into it, as `.claude/skills/pull-request/SKILL.md` > Shape the history shows. The history describes the change that ships, not the path that found it.

## Choose the type

Take the one gitmoji of the commit's dominant intention from [gitmoji.md](gitmoji.md), the complete catalog. When two seem to fit:

| The change | Gitmoji | Not |
|---|---|---|
| Fixes a defect in shipped behavior | 🐛 | 🩹, a blemish that nothing depended on |
| Fixes a defect that breaks the product or the build right now | 🚑️ | 🐛 |
| Fixes a security or privacy flaw | 🔒️ | 🐛 |
| Fixes a typo | ✏️ | 📝 or 🐛 |
| Adds behavior that a user or a caller can observe | ✨ | 👔: domain logic arrives as a feature |
| Breaks what its users rely on | 💥, with its footer | ✨ or ♻️ |
| Restructures the code without changing its behavior | ♻️ | 🎨, which is formatting and layout only |
| Moves or renames files or modules, and nothing else | 🚚 | ♻️ |
| Changes what the user sees: screens, styles, layout | 💄 | 🎨 |
| Draws a module boundary or a layer, or adds a module | 🏗️ | ♻️ |
| Changes the build system or the packaging | 👷 | 🔧 |
| Changes the CI workflows | 👷 | 💚, which only repairs a red CI |
| Configures a tool, the editor, Claude Code or the repository | 🔧 | 👷 |
| Adds or updates a development script run by hand | 🔨 | 👷, once the build or CI runs it |
| Adds, removes, upgrades or pins a dependency | ➕, ➖, ⬆️ or 📌 | 🔧, and 📦️, which is compiled files or packages |
| Documents, guides and agent guidance | 📝 | 📄, which is the license only |
| Adds or extends tests | ✅ | 🧪, which is a test that fails on purpose |
| Deletes code that still had callers | 🔥 | ⚰️, which is code already unreachable |
| Undoes an earlier decision | ⏪️ | 🔥 |

Some gitmojis never reach `main`: 🚧 and 💩, since a branch is rewritten into finished commits before it is pushed; 🧪, since every commit passes the gate; 🔀, since a branch lands by rebase; and 🍻.

## Choose the scope

Take the scope as `CLAUDE.md` > Workflow, "A commit scope names where the change belongs" says. Where it leaves a doubt:

- **A document that lives in a module,** such as its guide, takes the module's scope, not `docs`.
- **A test** takes the scope of the code it pins.
- **A dependency** takes the scope of the module whose manifest declares it, or `build` for the root's.
- **A new module** takes its own name.
- **A rule changed together with the skills, agents and documents that follow it** takes `rules`: the rule owns the decision.

## Write the subject

Picture a reader scanning `git log --oneline` for the commit that changed a behavior. The subject is what they read, and it decides whether they open the commit. Tell them what the change achieves, which only you know: the diff already shows the mechanism and the files. What does not fit goes in the body. A revert gets a header of this form too, not the `Revert "…"` that `git revert` writes.

💩 The mechanism, or nothing at all:

```
🐛(billing) fix payment bug
🐛(billing) add a retry around chargeCard
✨(billing) update invoice.ts and charge.ts
🐛(billing) Fixed the timeout error.
```

❤️ The intent:

```
🐛(billing) never charge a card twice when the provider times out
🐛(importer) say which row broke instead of rejecting the whole file
✨(calendar) read recurring events that end on a date
♻️(catalog) compute a price in one place instead of three
```

## Write the body

See the body as a brain dump of your state of mind at the moment you commit. Picture someone asking you, six months from now: "Why did you do this?" You would not answer with the list of the files you edited. You would give the context that led you there, the context you will have forgotten by then. Without it, the change gets misread, criticized, or reverted by someone who could not see what it protected against. That someone is most likely you.

Answer the questions that reader would ask:

- What broke, or what constraint appeared? What did the previous code assume, and when did that stop holding?
- Which alternative was tempting, and why did you turn it down? Nothing else in the repository records a rejected approach.
- Which facts from outside the code does the change rest on: the line of a datasheet or a standard, a measured value, how an external service behaves? No reader can recover them from the code.

Then reread the body: a sentence that the diff could have told is one to cut.

Under `🐛(billing) never charge a card twice when the provider times out`:

💩 The diff, told again:

```
Add an idempotency key to chargeCard and retry on timeout. Update the
billing tests accordingly.
```

❤️ The context that led there:

```
Under load, the payment provider accepts a charge and times out before
it replies. The client took the timeout for a refusal and charged again
on its next attempt, so a customer could pay twice for one order.

An idempotency key sent with each charge lets the provider recognize
the retry and return the first result. A longer timeout would only have
made the double charge rarer, never impossible.
```

## Add the footer

Write it as `CLAUDE.md` > Workflow, "A commit footer holds only what tools read" says, and never on a guess: a breaking change only when its users will meet it, an issue only when this commit closes it.

```
💥(api) return amounts as integer cents

Clients rounded the decimal strings each in their own way, so two
screens could show different totals for the same order. An integer
number of cents leaves no rounding to the client.

BREAKING CHANGE: `amount` is an integer number of cents, no longer a
decimal string. Divide it by 100 to show it.
Closes #42
```

## Check the message

Before committing, read the message against the rules:

1. **The header** ("Commits"): one line, in the form of the project's style, with the type of [Choose the type](#choose-the-type) and the scope of [Choose the scope](#choose-the-scope).
2. **The subject** ("A commit message says why"): what the change achieves, not the files it touches; in the imperative, in lowercase, without a final period, in a header of around 70 characters.
3. **The body** ("A commit message says why"): after a blank line and wrapped at 72 columns, it says why and lists nothing that the diff shows. Without one, the change really speaks for itself.
4. **The footer** ("A commit footer holds only what tools read"): what the rule requires, nothing that it excludes, and nothing untrue.
5. **The gate:** `pnpm verify` passes at this commit. When you rewrite a branch, `.claude/skills/pull-request/SKILL.md` > Shape the history checks every commit at once.
