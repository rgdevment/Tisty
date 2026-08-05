# Tisty

**English** · [Español](README.es.md)

A local, private, minimal task manager for macOS, Windows and Linux.

No account, no telemetry, no server. Your tasks are plain text files on your own
disk, readable with `cat` and searchable with `grep`. If Tisty disappears
tomorrow, your data is still there.

> **Early development.** The core works and the command line is usable, but
> natural language, sync and the graphical interface are not there yet. This is
> not a release.

---

## Why this exists

Most task managers treat what you finish as rubbish: you tick it and it is gone.
For a shopping list that is fine. For work it is not.

A task like *"fix the intermittent timeouts on save"* is not just a reminder. By
the time it is done it holds the ticket, the commit, and the two paragraphs
explaining that the real cause was a missing index on a table nobody was looking
at. Eight months later, when it happens again somewhere else, that is the only
place that knowledge exists — and ticking the task off is the same as deleting
it.

**Every completed task is an entry in your own knowledge base.** Not notes you
have to remember to write: the record you already produced while doing the work.

The tools that exist each solve a different problem. Some are excellent in a
terminal but have nowhere to write down what happened. Some are powerful editors
that are not task managers. Some are built for teams, with permissions and
assignees that get in the way when you work alone. And most of them charge a
growing subscription for features nobody asked for.

What was missing sat in between: **a personal task manager that is also the
record of how you solved things.** Local, private, with a first-class command
line and an interface that doesn't hurt to look at.

This is a tool I am building because I want to use it. It is deliberately
personal software — no teams, no collaboration, no growth plan. If it is useful
to you as well, all the better.

## A task doesn't end when you tick it

**A completed task is not finished — it is archived.**

It stops being a reminder of what to do and becomes the record of how something
got solved, with its ticket, its merge request, its links and the notes on what
actually happened. For most tasks that matter, **the value shows up after you
tick them.**

Three consequences run through the whole design:

- **Search is the main interface to the archive**, not a side feature.
- **Deleting is the exception.** The normal path is completing (archiving) or
  dropping.
- **Capture must stay instant.** Every field is optional and only shows up when
  used, so `tisty "book a call with Pepe tomorrow"` is still one line.

Because it also has to handle that call with Pepe, which is born and dies within
a day and leaves nothing worth keeping.

## What it looks like

```console
$ tisty "fix the intermittent timeouts on save" --prioridad 1

  ✓ fix the intermittent timeouts on save
    !1
    z8k4qm
```

```console
$ tisty ls

  hoy                                                   3 tareas

    1  ○ fix the intermittent timeouts on save
       !1 · hoy
    2  ○ validate payment notifications
       !3 · hoy
    3  ○ update the CI dependencies

$ tisty done 2
  ✓ validate payment notifications
```

You can refer to a task by its number in the last listing, by a fragment of its
title (`tisty done payment`), or by its identifier. A ULID is for scripts, not
for fingers.

## Built for people who live in a terminal

- **`--json` on every read command.** Without it, none of this would be
  scriptable.
- **stdout is data, stderr is conversation.** A pipe never carries decoration,
  and without an interactive terminal there is no colour and no escape codes.
- **Exit codes that mean something:** `0` fine · `1` error · `2` misuse ·
  `4` not found.
- **Anything the GUI will do, the terminal can do too.** What changes is how
  many keystrokes it costs, not what is possible.

## Your data

A directory of text files:

```
~/Documents/Tisty/
└── store/
    └── dev_a3f1/
        ├── 000001.jsonl      closed segment, never changes again
        └── active.jsonl      one line per event
```

```jsonl
{"v":1,"ts":"2026-08-05T08:27:49Z","by":"dev_a3f1","op":"task.add","id":"01KZ8G…","d":{"title":"fix the intermittent timeouts on save","priority":1,"tags":["backend","db"]}}
```

An append-only event log. History and undo come for free, and so does
conflict-free sync when it lands: **each machine only ever writes to its own
directory**, so merging two histories is concatenating them.

## Installing

No published binaries yet. With Rust 1.97 or newer:

```sh
git clone https://github.com/rgdevment/Tisty
cd Tisty
cargo install --path crates/tisty-cli
```

## Where it stands

| | |
|---|---|
| ✅ | Core: model, event log, storage, projection |
| ✅ | CLI: capture, list, complete, show detail |
| ⬜ | Natural language: `tisty "deploy the API tomorrow at 10"` |
| ⬜ | Journal, steps and lists from the command line |
| ⬜ | Sync over Git or through your own cloud folder |
| ⬜ | Graphical interface (Tauri) |
| ⬜ | Markdown documents |

## What it will never do

As important as the list above. Permanently out of scope: real-time
collaboration, kanban boards, Gantt charts, time tracking, productivity metrics,
databases with typed properties and formulas, and AI anywhere in the
critical path.

The natural language parser will be deterministic and local. Nothing is ever
sent to a model.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md). Open an issue before writing code for
anything beyond a fix: Tisty is deliberately minimal, and a well-written feature
can still be declined.

## Licence

[AGPL-3.0](LICENSE), and available under [commercial terms](COMMERCIAL.md) for
organisations that cannot comply with it.

See also [SECURITY.md](SECURITY.md) and [PRIVACY.md](PRIVACY.md) — the summary
of the latter is that nothing is collected and nothing is sent anywhere.
