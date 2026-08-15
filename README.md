# Tisty

**English** · [Español](README.es.md)

A local, private, minimal task manager for macOS and Windows, with a command
line and a window that do the same things.

**What you finish is the point.** A task manager that throws away what you
solved is throwing away the only record of how you solved it.

No account, no telemetry, no server. Your tasks are plain text files on your own
disk, readable with `cat` and searchable with `grep`. If Tisty disappears
tomorrow, your data is still there.

> **Alpha.** Everything below works and is in daily use by the person who wrote
> it. What is missing is the mileage that only other people's machines provide.
> Linux is a phase of its own and has not started.

---

## Installing

macOS, with [Homebrew](https://brew.sh):

```console
$ brew install --cask rgdevment/tap/tisty      # the application
$ brew install rgdevment/tap/tisty-cli         # only the command line
```

Or download the disk image or the installer from
[Releases](https://github.com/rgdevment/Tisty/releases).

**You do not need both.** The application carries the command line inside it —
Settings puts it within reach of your terminal — so the formula is for whoever
wants the command and no window. Either way the command you type is `tisty`.

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

This is a tool I am building because I want to use it. One developer, one need,
and a solution shared in case it is yours too. It is deliberately personal
software — no teams, no collaboration, no growth plan, nothing to sell you
later. Free and open source, and it stays that way.

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

## It reads what you write

This is how a task is captured, in the window and in the terminal alike. You
type a sentence; Tisty takes the date out of it, leaves the sentence readable,
and shows you what it understood **before** anything is stored. In the window
the reading appears as chips you can correct with one click; the examples below
use the terminal because it fits on a page.

```console
$ tisty "ship the release tomorrow at 10 to production"
  ✓ ship the release to production
    tomorrow 10:00

$ tisty "book the flights @travel #urgent !high"
  ✓ book the flights
    !1 · @travel · #urgent
```

**When it happens.** A day, a time, or both. Names, distances and plain dates all
work; what it cannot read it leaves alone rather than guess.

```console
$ tisty "call the bank at 3"        →  today 15:00
$ tisty "meeting monday 15"         →  sat
$ tisty "review it next week"       →  23 aug
```

**When it is actually due.** A deadline is a different thing from a plan: the
date is when you mean to do it, the deadline is the wall behind it. Three words
open one — **before**, **due** and **until** — and they read the same.

```console
$ tisty "file the report before friday"      →  due fri
$ tisty "renew the domain due august 30"     →  due 30 aug
$ tisty "send the invoice until friday"      →  due fri
```

Mind the difference between *for* and the three above: `for friday` is a plan,
`before friday` is a limit.

**What comes back.** Naming a day makes it fixed; naming only an interval makes
it relative. The bin goes out on Tuesday whether or not you took it out last
week, but three days start counting when you actually watered the plants.

```console
$ tisty "bins out every tuesday"        →  tue · ↻ every week
$ tisty "water the plants every 3 days" →  ↻ every 3 days
```

A repeating task can carry a deadline of its own, and it belongs to **that
occurrence**: the rent is due by the 5th of every month, not once ever.

```console
$ tisty "submit the report every week before friday"
  ✓ submit the report
    due fri · ↻ every week
```

And a series can be told when to stop. **Until** ends the repetition — the last
one hands on no successor.

```console
$ tisty "take the pill every day at 9am until september 30"
  ✓ take the pill
    tomorrow 09:00 · ↻ every day until 30 sep
```

The same word does two jobs and the sentence decides which: with a cadence in
it, `until september 30` ends the series; without one, it is the deadline. Say
`before` or `due` when you mean a deadline on a task that repeats.

**And what it refuses to read matters more.** A guess that reads well is worse
than no guess at all, so these keep every word and get no date:

```console
$ tisty "review the monday report"   # «monday» names the report
$ tisty "a course of 3 months"       # a duration is not a date
$ tisty "ship it 3 days ago"         # the past is not a plan
$ tisty "set up 24/7 support"        # 24/7 is an expression, not 24 July
```

Two more rules worth knowing. **Anything in quotes is left alone**, so
`tisty '"meeting on monday"'` keeps the whole line. And when a phrase sits
mid-sentence with nothing backing it — `call the bank tomorrow about the
invoice` — the date is still applied, but **marked as a guess**: the terminal
says so and prints the command that undoes just that, and the window underlines
it so one click drops it.

Every sentence above is a test. The parser ships with a contract of 261 cases in
Spanish and English that says what it must read and, just as often, what it must
leave alone.

It reads Spanish with the same rules and its own words: `cada martes`,
`cada 3 días`, `antes del viernes`, `vence el 30 de agosto`,
`cada día a las 9 hasta el 30 de septiembre`.

## The window

Three columns at most: what you are looking at, the list, and the task you
opened. Nothing else on screen.

**Tasks**, with four slices — *today*, *upcoming*, *repeating*, *all* — and the
one you chose last is the one you come back to. Then your lists, your tags, the
archive, and a search that reaches all of it.

**A task opens beside the list**, not on top of it: title, dates, list, tags and
priority; a description and a journal in Markdown; steps you tick off one at a
time; and whatever you dropped on it. Completing it does not put any of that out
of reach — it moves to the archive, which is where search does its best work.

**Capture is one field at the top.** What Tisty understood appears underneath as
chips before it is saved, and a chip is one click away from being wrong on
purpose. A global shortcut opens a small field over whatever you are doing, so
a task that occurs to you mid-something does not cost you the something.

**Documents** live beside the tasks, for the reference material that has no date
and never gets ticked. They are Markdown files in your store, edited as
documents rather than as source: tables, checklists, code and images, and
whatever you paste from a page or a ticket keeps its shape. A task can point at
a document; a document never creates tasks. **Search reads them too** — title and
body — so a line you wrote in a document is as findable as a task.

Taking one out comes in two shapes, and they are not the same. **Copy as
Markdown** puts the text on your clipboard, references and all: paste it wherever
you like, but an image lives in your store and will not follow. **Export as
Markdown** writes a folder — the document beside an `attachments/` of its own,
holding only the files that document names. That one opens anywhere, zips, and
travels.

**Lists** get their own screen, each with an icon you pick from a set, and
**reminders** arrive as a system notification and a short sound that can be
turned off. Repeating tasks come back on their own, one occurrence at a time.

**Settings** hold your data (sync, backup, where the store lives), notices,
writing, and maintenance — including a report you can attach to a bug, which
shows you exactly what it contains before you save it.

The whole window works from the keyboard: arrows through the list, `Ctrl+Enter`
to complete, `Escape` to close a task, and a visible focus ring everywhere it
goes.

## And a command line, if you use one

Not the main way in — the window is — but everything the window does, the
terminal does too. What changes is how many keystrokes it costs, not what is
possible.

```console
$ tisty "fix the intermittent timeouts on save" --priority 1
  ✓ fix the intermittent timeouts on save
    !1

$ tisty log 1 "the retry budget was exhausted before the pool refilled"
$ tisty done 1

$ tisty search "retry budget"
  «retry budget»                                        1 task
    1  ✓ fix the intermittent timeouts on save
       ✎1
```

Refer to a task by its number in the last listing, by a fragment of its title
(`tisty done payment`), or by its identifier. Filters combine and read like the
markers you write with: `tisty ls week #security`, `tisty ls @work !1`.

For scripting: `--json` on every read command, stdout is data and stderr is
conversation, exit codes that mean something (`0` fine · `1` error · `2` misuse
· `4` not found), and `tisty export` gives the data back as JSON or as a
Markdown document you can read without Tisty.

## Your data

A directory of text files, in the place your system keeps application data — never in your documents folder:

```
<application data>/tisty/data/
└── store/
    └── dev_a3f1/
        ├── 000001.tisty      closed segment, never changes again
        ├── 000001.count      how many lines it holds, to catch a half-download
        └── active.tisty      one line per event
```

It lives in your system's local application data directory, and that is not
configurable — a task manager that lets you file its own store in a folder some
cloud client is quietly rewriting is handing you a footgun. Syncing is a
separate thing: Tisty **leaves copies** in a folder both machines can reach and
**brings home the ones others left**. Two different paths, and only one of them
is yours to choose.

Your settings never travel with it. The device identifier lives in the config
file precisely so it stays on this machine: if it went along, two computers
would share it, write to the same file, and the guarantee below would collapse.

```jsonl
{"v":1,"ts":"2026-08-05T08:27:49Z","by":"dev_a3f1","op":"task.add","id":"01KZ8G…","d":{"title":"fix the intermittent timeouts on save","priority":1,"tags":["backend","db"]}}
```

An append-only event log. History and undo come for free, and so does
conflict-free sync when it lands: **each machine only ever writes to its own
directory**, so merging two histories is concatenating them.

That is one list, not one per machine. Every device reads every directory and
replays them in order; it only writes to its own. Which is also why a synced
folder never produces one of those `active (conflicted copy).tisty` — no two
writers ever touch the same file.

## Two machines

Point Tisty at a folder both computers already reach — the one your Google
Drive, OneDrive or iCloud client keeps in step, a mounted NAS, an external drive
you plug in on Fridays — and it does the rest on its own: it pulls when the window opens,
pushes shortly after you change something, and does both on a timer. It never
blocks a write and never interrupts you; an unreachable folder is retried in
silence.

No file is ever merged and nothing asks "which one is newer?", because a device
directory has exactly one writer: yours is authoritative going up, theirs coming
down.

Point two machines that were already in use at the same folder and Tisty stops
and asks, because your own second machine and a stranger's folder are the same
gesture. You can **join the two histories**, keep this machine, or take what the
folder holds. Every one of them writes a backup first.

Whoever runs that folder is not Tisty's business, and there is nothing of ours
in the middle — no account, no server, no daemon. If you sync nothing, Tisty
never opens a socket at all.

**Or back up by hand instead.** One zip, kept wherever you like. Restoring is a
photograph: back to that moment, and what came after is lost on purpose. The two
are mutually exclusive — the shared folder already holds every machine's
history, so a snapshot beside it would only be a rival truth.


## Where it stands

| | |
|---|---|
| ✅ | Core: model, event log, storage, projection |
| ✅ | CLI: capture, list, complete, show detail, journal, steps, lists, tags |
| ✅ | Natural language: `tisty "deploy the API tomorrow at 10"` |
| ✅ | Search across tasks, undo and redo, `--json`, `export`, exit codes |
| ✅ | Window (Tauri): list and detail, Markdown, attachments, keyboard throughout |
| ✅ | Sync through a folder both machines reach, and backup by hand |
| ✅ | Tray and menu bar, with quick capture on a global shortcut |
| ✅ | Repeating tasks, one per occurrence, folded in the archive |
| ✅ | Reminders, with a system notification and a sound you can turn off |
| ✅ | An error log, and a report you can attach to an issue |
| ✅ | macOS: universal disk image, signed and notarised, and Homebrew |
| ◐ | Documents: the store and the editor are in; folders and sync are not |
| ◐ | Daily use, which is what turns up the bugs tests do not |
| ◐ | Signed builds: DMG and `.exe` ship; the Store package waits on a name |
| ⬜ | Linux, its own phase, not started |

Two things are known and accepted rather than pending: there is no way to
reorder by hand in the window — HTML drag and drop does not survive the native
file drop Tauri needs for attachments, and keeping attachments was the better
trade — and nothing has been tested with a real screen reader, though the
keyboard path has.

## What it will never do

As important as the list above. Permanently out of scope: real-time
collaboration, kanban boards, Gantt charts, time tracking, productivity metrics,
databases with typed properties and formulas, and AI anywhere in the
critical path.

The natural language parser will be deterministic and local. Nothing is ever
sent to a model.

## Other tools

Same hands, same idea: free, open source, no ads, no telemetry, everything on
your machine.

- **[CopyPaste](https://github.com/rgdevment/CopyPaste)** — a clipboard manager
  for Windows, macOS and Linux.
- **[LinkUnbound](https://github.com/rgdevment/LinkUnbound)** — a browser
  selector for Windows and macOS: it asks which browser should open a link
  instead of assuming.

## Standing on

Tisty is small because other people's work does the heavy lifting. These are the
ones it would not exist without, each under a licence that allows it:

**The core, in Rust** — [Tauri](https://tauri.app) puts a native window around
it without shipping a browser; [serde](https://serde.rs) reads and writes every
line of the log; [jiff](https://github.com/BurntSushi/jiff) does the dates and
the time zones, which is the part nobody should write twice;
[SQLite](https://sqlite.org), through
[rusqlite](https://github.com/rusqlite/rusqlite), holds the read cache;
[clap](https://github.com/clap-rs/clap) is the command line;
[ULID](https://github.com/dylanhart/ulid-rs) gives every task an identifier that
sorts by time and needs no coordination.

**The window** — [React](https://react.dev) draws it and
[Tailwind CSS](https://tailwindcss.com) styles it;
[TipTap](https://tiptap.dev) and [ProseMirror](https://prosemirror.net) are the
document editor; [markdown-it](https://github.com/markdown-it/markdown-it)
renders the prose everywhere else; [Vite](https://vite.dev) builds it and
[Vitest](https://vitest.dev) tests it.

The full list, with versions and licences, is in `Cargo.lock` and
`app/package-lock.json`.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md). Open an issue before writing code for
anything beyond a fix: Tisty is deliberately minimal, and a well-written feature
can still be declined.

## Licence

[AGPL-3.0](LICENSE), and available under [commercial terms](COMMERCIAL.md) for
organisations that cannot comply with it.

The signed builds in the app stores carry their own terms, because the stores'
do not accept the AGPL — [DISTRIBUTION.md](DISTRIBUTION.md) says which applies
to what you have, and why. Nothing is withheld from the source either way.

See also [SECURITY.md](SECURITY.md) and [PRIVACY.md](PRIVACY.md) — the summary
of the latter is that nothing is collected and nothing is sent anywhere.
