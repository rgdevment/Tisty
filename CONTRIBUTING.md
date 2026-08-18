# Contributing to Tisty

Bug reports, feature discussions and pull requests are all welcome.

## Before you write code

**Open an issue first for anything beyond a bug fix.** Tisty is deliberately
minimal and has an explicit list of things it will never do — task
collaboration, kanban boards, time tracking, databases with typed properties, AI
in the critical path. A pull request that adds one of those will be declined no
matter how well it is written, and that is a waste of your evening. Asking first
costs nothing.

## The CLA

Tisty is released under the AGPL-3.0 **and** offered under separate [commercial
terms](docs/COMMERCIAL.md) to organisations that cannot comply with it. Because of
that dual model, every contributor signs a one-time [CLA](CLA.md) before their
code can be merged. Offering commercial terms requires the right to license the
whole codebase that way, and that right has to come from each author explicitly.

**You keep the copyright on your work.** The CLA is a licence you grant, not a
transfer of ownership.

The first time you open a Pull Request, a bot asks you to sign. Reply on that
Pull Request with exactly:

```text
I have read the CLA Document and I hereby sign the CLA
```

That is it — every later Pull Request from the same account is covered, and so
is anything you sent before signing.

**Code in an issue or in a review suggestion counts too.** The bot only watches
Pull Requests, so those routes reach the codebase without passing it. If you
have not signed, keep such snippets to a description of the fix rather than the
patch itself, and a maintainer will write it.

**Please leave tool co-authorship out of your commits.** Assistants are welcome
here — this project is built with them — but the credit line is for people. If
your editor adds a trailer naming one, drop it before you push. It changes
nothing about what you are allowed to submit; section 4 of the CLA already puts
the responsibility for generated code on you, whichever tool helped write it.

**In return, the project commits that:**

- The community edition stays available under the AGPL-3.0.
- Your contribution is never removed from the open source project to make it
  exclusive to a commercial edition.
- Your authorship is preserved; history is not rewritten to erase it.
- No release already published is ever retroactively withdrawn.

If you would rather not sign, you can still use Tisty, report bugs, request
features, discuss design, package it for your distribution, write plugins, and
fork the project under the AGPL-3.0. Only merging code into this repository
requires the agreement.

## Development setup

Rust 1.97 or newer. Everything else comes from `rustup`:

```sh
cargo build
cargo nextest run --workspace   # or: cargo test
```

Before opening a pull request, run what CI runs:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check
```

CI runs the test suite on Linux, Windows and macOS. Path separators, line
endings, case sensitivity and file locking only diverge at runtime, so a green
build on one platform proves very little.

## House style

**Code, identifiers, comments, test names and error messages are in English.**
Conversations and issues can be in Spanish or English.

**Comments default to none.** Write one only for a non-obvious *why* that would
surprise a future reader: a hidden constraint, a workaround, a subtle
invariant. Never to narrate what the code already says, and never to record
history — that is what the commit log is for.

**`tisty-core` produces no terminal output.** No `println!`, no colours, no
reading `stdin`. The CLI and the GUI are both clients of the same API, and
anything the core prints leaks into the GUI as garbage.

**Anything the GUI can do, the CLI can do too.** What differs is how many
keystrokes it costs, not what is possible. A feature that only exists behind a
mouse cannot be scripted or emitted as `--json`, which is the point of the
tool.

## Tests

The tests worth writing verify design properties, not round trips. Storing a
value and reading it back almost never fails. What fails is the invariant:
that two concurrent edits to different fields both survive, that a late update
cannot resurrect a deleted entity, that replaying the log in any read order
produces the same state.

For the natural language parser, **write the cases before the parser**. It is
the one component that cannot be fixed by eye — fixing "on tuesday" quietly
breaks "by tuesday" otherwise.

## Commits

Conventional commits (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`,
`test:`). Explain *why* in the body when the reason is not obvious from the
diff; the *what* is already in the patch.

## Security

Do not open a public issue for a vulnerability. See [SECURITY.md](SECURITY.md).
