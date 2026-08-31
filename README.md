<div align="center">
  <img
    src="assets/icons/tisty-256.png"
    width="140"
    height="140"
    alt="Tisty — free open source task manager for Windows and macOS"/>

  <h1>Tisty — Free Open Source Task Manager</h1>

  <p><strong>A local-first task manager and personal archive for Windows and
  macOS.<br/>No accounts. No telemetry. No server. Your tasks, notes and
  documents stay as plain files on your own disk.</strong></p>

  <p>
    <strong>English</strong> ·
    <a href="README.es.md">Español</a>
  </p>

  <p>
    <a href="https://github.com/rgdevment/Tisty/actions/workflows/ci.yml">
      <img
        src="https://img.shields.io/github/actions/workflow/status/rgdevment/Tisty/ci.yml?style=flat-square&logo=github-actions&label=Build"
        alt="Build status"/>
    </a>
    <a href="https://sonarcloud.io/summary/overall?id=rgdevment_Tisty">
      <img
        src="https://img.shields.io/sonar/quality_gate/rgdevment_Tisty?server=https%3A%2F%2Fsonarcloud.io&style=flat-square&logo=sonarcloud&label=Quality%20Gate"
        alt="Quality gate"/>
    </a>
    <a href="https://sonarcloud.io/component_measures?id=rgdevment_Tisty&metric=coverage">
      <img
        src="https://img.shields.io/sonar/coverage/rgdevment_Tisty?server=https%3A%2F%2Fsonarcloud.io&style=flat-square&logo=sonarcloud&label=Coverage"
        alt="Coverage"/>
    </a>
    <a href="https://github.com/rgdevment/Tisty/releases">
      <img
        src="https://img.shields.io/github/v/release/rgdevment/Tisty?include_prereleases&style=flat-square&label=Latest&color=6C5CE7"
        alt="Latest release"/>
    </a>
    <img
      src="https://img.shields.io/badge/Platform-Windows%20%7C%20macOS-6C5CE7?style=flat-square"
      alt="Platform: Windows and macOS"/>
    <a href="#licence">
      <img
        src="https://img.shields.io/github/license/rgdevment/Tisty?style=flat-square&color=lightgrey"
        alt="Licence AGPL-3.0"/>
    </a>
  </p>

  <h4>Download Tisty</h4>

  <p>
    <a href="https://apps.microsoft.com/detail/9PGVWXD8X93N">
      <img
        src="https://img.shields.io/badge/Windows-Microsoft_Store-0078D4?style=for-the-badge&logo=microsoft"
        alt="Get the Tisty task manager from the Microsoft Store"/>
    </a>
    <a href="#getting-started">
      <img
        src="https://img.shields.io/badge/macOS-Homebrew-FBB040?style=for-the-badge&logo=homebrew&logoColor=black"
        alt="Install the Tisty task manager with Homebrew on macOS"/>
    </a>
  </p>

  <p>
    <sub>Prefer a direct download?
    <a href="https://github.com/rgdevment/Tisty/releases/latest">GitHub
    Releases</a> carries the signed installers — Windows (.exe) · macOS
    (.dmg)</sub>
  </p>

  <p>
    <a href="https://buymeacoffee.com/rgdevment">
      <img
        src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-%E2%98%95-FFDD00?style=flat-square&logo=buy-me-a-coffee&logoColor=black"
        alt="Buy me a coffee"/>
    </a>
  </p>
</div>

---

**Tisty** is a free, open source **task manager** for Windows and macOS, built
on one idea: a finished task is worth more than the tick that closes it. Most
**to-do apps** are a list you cross out — the moment you complete something, the
steps you took, the notes you wrote and the documents you leaned on go with it.
Tisty archives all of that instead, and search reaches every word of it.

This is not a company product. I am a developer who kept solving the same
problem twice, so I built the **personal task manager** I wanted and gave it
away. No ads, no telemetry, no accounts, no subscriptions — just a
**local-first productivity tool** that lives on your machine and nowhere else.

**Why people choose Tisty over other task managers:**

- **100% local** — your tasks, your journal and your documents never leave your
  computer. No cloud, no server, no account.
- **Truly free** — no premium tier, no feature gates, no trial. AGPL v3, and
  [commercial terms](docs/COMMERCIAL.md) only for organisations that cannot
  comply with it.
- **Your data outlives the app** — plain text and Markdown on your own disk,
  readable with `cat` and searchable with `grep`.
- **Finishing keeps everything** — completing a task moves it to the archive
  with its steps, its notes and its attachments intact, documents included.
- **Fast and native** — a Rust core inside a Tauri window: it starts quickly,
  stays small, and looks like it belongs on both systems.

> I use Tisty every day on macOS and Windows. If something feels off,
> [open an issue](https://github.com/rgdevment/Tisty/issues) — this project
> keeps improving because of real-world use.
>
> **It is for one person, by design.** No assignees, no permissions, no boards.
> If you need to run a team, Tisty will not carry that.

![A record of how you work](assets/Store/jpg/en-01-archive.jpg)

## Table of Contents

- [Why I Built This](#why-i-built-this)
- [What It Is / What It Isn't](#what-it-is--what-it-isnt)
- [What Makes It Different](#what-makes-it-different)
- [Who Is This For?](#who-is-this-for)
- [The Idea It Is Built On](#the-idea-it-is-built-on)
- [Getting Started](#getting-started)
- [What It Does](#what-it-does)
- [Your Data and Privacy](#your-data-and-privacy)
- [Two Machines, If You Have Two](#two-machines-if-you-have-two)
- [A Command Line, If You Want One](#a-command-line-if-you-want-one)
- [An Assistant, If You Use One](#an-assistant-if-you-use-one)
- [Where It Stands](#where-it-stands)
- [What It Will Never Do](#what-it-will-never-do)
- [Other Tools by the Same Author](#other-tools-by-the-same-author)
- [Standing On](#standing-on)
- [Contributing](#contributing)
- [Licence](#licence)

## Why I Built This

Organising your day is harder than a list makes it look. There are more moving
parts than fit in your head, plans shift under you, and something you thought
was small turns out not to be. Which is why finishing it feels good.

But look at what the task gathered on the way. The steps it actually took. The
notes you wrote while working it out. What you looked up, what it turned out to
connect to, the documents you leaned on. That is where the effort went.

Mine looked like *"fix the intermittent timeouts on save"*. By the time it was
done it had collected the ticket, the commit, and two paragraphs explaining that
the real cause was a missing index on a table nobody was looking at.

Eight months later the same thing happened somewhere else. I remembered solving
it. I could not remember how — and the note that held the answer had gone with
the tick.

**That is the whole reason Tisty exists.** A task is not a line you cross out.
It is a tree: the steps, the journal, the files and the documents that grew
around it while you worked. Finishing it should not prune it.

I wanted it to stay mine, too. On my disk, in files I can open without asking
anyone. So I built the thing I wanted, used it until it stopped annoying me, and
put it here in case it is the thing you wanted too.

## What It Is / What It Isn't

**It is** a personal task manager where finishing something is the beginning of
its useful life. Tasks carry a description, a journal, steps and attachments;
when you complete one it moves to the archive, and search reaches all of it,
documents included.

**It is for one person.** No assignees, no permissions, no boards. If you need
to run a team, Tisty will not carry that, and you deserve to know before you
install it rather than after.

**It is not something I sell.** Nothing is locked, nothing expires, and there is
no version of this with more in it. It is a program I wrote for myself and gave
away.

**Your data is files.** Plain text on your own disk, readable with `cat` and
searchable with `grep`. If Tisty disappeared tomorrow, everything you wrote
would still be there and still make sense.

## What Makes It Different

If you are looking for a **free, open-source, offline task manager with no
account, no subscription and no AI**, this is what Tisty is:

| | |
|---|---|
| Your tasks live | in plain files on your own disk |
| Account | none, ever |
| Subscription | none. There is no paid tier and no upgrade |
| Offline | always. There is no server to be away from |
| AI inside | **none**, and none is coming |
| Your own assistant | yes, over **MCP**, if you choose to open the door |
| Priorities | the **Eisenhower matrix**, by its name |
| Sorting | lists, tags, steps and a journal on every task |
| Documents | written and searched beside the tasks, not in another app |
| Attachments | kept with the task, as themselves |
| Finished work | an **archive that keeps what each task taught you** |
| Two machines | through a folder you already sync. No server of ours |
| Natural language | dates, deadlines and repeats, parsed on your machine |
| Source | open, auditable, yours to fork |

**The Eisenhower matrix, not numbered priorities.** A task is urgent, important,
both or neither — *do*, *decide*, *delegate*, *drop*. That names the decision
instead of hiding it behind a number, and it is the difference between a list
that sorts and a list that helps you choose. Around it: lists for where work
belongs, tags for what cuts across, steps for the parts, and a journal for what
you learn on the way.

**Finishing is where it starts.** Most task managers treat a completed task as
rubbish to hide. Here it moves to an archive read in three layers: the ones that
taught something come with their whole trail — what changed, when, and what you
wrote — the routines come with their tallies and their streaks, and the rest is
the trace. Search reaches all of it, documents included. A year in, that archive
is the part you would miss.

**Your assistant, not one of ours.** Tisty has no AI in it. The natural language
that turns "call the bank at 3" into a task is rules running on your machine, no
model and no cloud. But it speaks [MCP](https://modelcontextprotocol.io), so an
assistant you already use can file work into it — with its steps, its date and
the list it belongs in, on this machine, with no account and nothing over the
network. You open that door, and you can close it.

**Two machines, no middleman.** If you already sync a folder, Tisty travels
through it. There is no server of ours in between, nothing to sign up for, and
nothing that stops working the day a company changes its mind.

**Free is not a tier here.** There is no upgrade, no seat count, no feature held
back for later. The reason is not generosity: a program that keeps your work on
your disk and never phones home has almost nothing to charge for, and asking
would make it worse.

## Who Is This For?

Someone who works alone, or mostly alone, and whose tasks leave a trail worth
keeping. Developers, sysadmins, freelancers, researchers, students — anyone who
has ever solved the same problem twice and known it the second time.

If what you want is a list to cross out and never open again, Tisty will feel
like more than you asked for. That is a fair reason to walk away.

## The Idea It Is Built On

**A completed task is not finished. It is archived.**

It stops being a reminder of what to do and becomes the record of how something
got solved. Three things follow from that, and they shaped everything else:

- **Search is the main way into the archive**, not a side feature.
- **Deleting is the exception.** The normal ending is completing, which keeps
  it. Erasing something for good takes two deliberate steps first: it has to be
  archived, and then hidden.
- **Capture has to stay instant**, because most tasks are not like that at all.
  The call you have to make tomorrow is born and dies within a day and leaves
  nothing worth keeping — and writing it down must not cost more than one line.

## Getting Started

**Windows** — from the
[Microsoft Store](https://apps.microsoft.com/detail/9PGVWXD8X93N), which keeps
it updated for you.

**macOS** — with [Homebrew](https://brew.sh). The tap is added once and never
again. After that Tisty keeps itself up to date, so Homebrew stands aside:

```console
$ brew tap rgdevment/tap
$ brew install --cask tisty
```

Or take the disk image and the installer straight from
[Releases](https://github.com/rgdevment/Tisty/releases), on either system.

## What It Does

**The first time it opens**, Tisty asks four things — which language, where your
copies should go, whether it may wake you for a reminder, and what closing the
window ought to mean — and then writes you a guide. The guide is a document in
your own store, in a folder of its own: yours to read, edit, or throw away like
anything else you wrote. Settings brings the welcome back, opens the guide
again, and changes the language, which starts as your system's.

**Three columns at most:** what you are looking at, the list, and the task you
opened. Nothing else on screen.

**It reads what you write.** You type a sentence and Tisty takes the date out of
it, leaves the sentence readable, and shows you what it understood *before*
anything is saved — as chips you can correct with one click.

![The date reads itself out of the sentence](assets/Store/jpg/en-02-capture.jpg)

```text
"ship the release tomorrow at 10"   →  tomorrow 10:00
"file the report before friday"     →  due fri
"book the flights @travel #urgent"  →  @travel · #urgent
```

A day, a time, or both. Names, distances, plain dates. What it cannot read it
leaves alone rather than guess. A deadline is a different thing from a plan, and
three words open one: **before**, **due**, **until**.

**A task opens beside the list**, not on top of it: dates, list, tags, priority,
a description and a journal in Markdown, steps you tick one at a time, and
whatever you dropped on it. Completing it puts none of that out of reach.

**Priorities are a matrix, not a ladder.** Tisty borrows the four quadrants
of the Eisenhower matrix — the method President Dwight D. Eisenhower is
credited with, popularised by Stephen Covey in *The 7 Habits of Highly
Effective People*: sort what you have by urgent against important, and each
quadrant tells you what to do with it. **Do** what is urgent and important,
**Schedule** what matters and is not urgent, **Delegate** what is urgent and is
not yours, and leave in **Minor** whatever you could live without — when you are
sure, one button drops the lot.

Drag a task into its quadrant, or type it: `!do`, `!schedule`, `!delegate`.
`!decide` still works, because it is what the quadrant used to be called. Each
quadrant has a **+** that opens the quick capture with that quadrant already
set, and whatever nobody has placed waits in a tray that opens the way you
left it.

![Urgent against important, and what to do with each](assets/Store/jpg/en-04-priorities.jpg)

**Documents** live beside the tasks, for reference material that has no date and
never gets ticked. They are Markdown files you edit as documents — tables,
checklists, code, images — and search reads them too. A task can point at a
document; a document never creates tasks.

Text can be highlighted in a few colours, centred or aligned, and set apart as
an aside. It stays Markdown: where Markdown cannot say a thing, Tisty writes the
small piece of HTML that can, and reads it back.

The writing sits on a lit page, and its first line is both the name of the
document and its title. When the window is wide enough a column opens beside it
with what the document is, the formatting the `/` menu used to hide, and its
outline. **Tisty makes its own PDF** — A4, Letter or one endless sheet, with its
own margins and the attachments carried inside — and shows it to you before you
export it.

![Documents in Markdown, with attachments](assets/Store/jpg/en-03-documents.jpg)

**A global shortcut** opens a small field over whatever you are doing, so a task
that occurs to you mid-something does not cost you the something.

**Repeating tasks** come back one occurrence at a time, so the archive shows you
did it twelve times — and it counts what was owed rather than only what you
closed, so a routine reads 26 of 30 with four dates that have no record. Marking
one days late offers those dates back instead of calling them forgotten: tick the
ones you did and the gap closes. **Reminders** arrive as a system notification and
a short sound you can turn off. The whole window works from the keyboard.

**The archive is read in three layers.** The ones that taught something come with
their whole trail — what changed, when, and what you wrote along the way. The
routines come with their tallies, their streaks and the hour you usually keep
them. The rest is the trace: what left nothing written, listed dense and out of
the way, because it still happened and search still reaches it.

## Your Data and Privacy

![No account, no telemetry, no server](assets/Store/jpg/en-05-private.jpg)

Everything lives in one folder on your disk: an append-only log of what
happened, your documents as `.md` files, and your attachments as themselves.
Nothing is obfuscated and nothing is in a format only Tisty can read.

**Nothing is encrypted at rest**, and that is a decision rather than an
oversight — your operating system's permissions are the protection, and the
files stay readable with tools you already have. It is written out in
[PRIVACY.md](PRIVACY.md) and [SECURITY.md](SECURITY.md), including the parts
that are not reassuring.

Tisty makes **one** network request a day, unasked: it checks whether a newer
version exists. It sends nothing. If one does exist and you press the button
that offers it, two more follow — the file naming the release, and the
installer — and Tisty refuses to install anything not signed with a key
compiled into the copy you already have.

## Two Machines, If You Have Two

Point Tisty at a folder both computers already reach — whatever your Google
Drive, OneDrive or iCloud client keeps in step, a NAS, a drive you plug in on
Fridays — and it does the rest on its own.

There is nothing of mine in the middle: no account, no server, no daemon. Who
runs that folder is your business, not Tisty's. If you sync nothing, it never
opens a connection at all.

Two machines can genuinely write the same document at once, and there Tisty
merges them **block by block** — you edit the introduction on one, someone edits
the closing paragraph on the other, and both land with nothing to answer. Only a
real overlap becomes a question.

**Or back up by hand.** One zip, kept wherever you like.

And because what uploads that folder is your provider's program and not
Tisty, if you ever change something on one machine and it does not turn up
on the other, [FAQ.md](docs/FAQ.md) lists the causes worth checking, in order.

## A Command Line, If You Want One

The window is the main way in. But everything it does, the terminal does too:
the same store, the same tasks, the same natural language. It exists because I
wanted it, and it is entirely optional.

```console
$ tisty "call the bank at 3"
$ tisty ls today
$ tisty done 2
```

Settings puts it within reach of your terminal, or you can install only the
command with `brew install rgdevment/tap/tisty-cli` and never open the window.

## An Assistant, If You Use One

**Tisty is not AI and has none inside.** The natural language that turns "call
the bank at 3" into a task is rules running on your machine: no model, no
request, no cloud. That is not going to change.

But if you already use an assistant, you are probably telling it things worth
keeping — the school group says card stock on Monday, the invoice is due on the
30th. So Tisty leaves a door, and you decide whether to use it.

Settings › Agents lists the assistants already installed on this computer and
connects the one you pick: it writes a single line into that assistant's own
settings, leaves the rest of that file where it was, and keeps a copy of it as
it was before. For one it does not know, a line does it:

```console
$ <your-assistant> mcp add tisty -- tisty mcp
```

Where `<your-assistant>` is whatever yours is called. It speaks
[MCP](https://modelcontextprotocol.io) on the same machine: no
account, no token, nothing over the network. **You are the one who opens it.**
The assistant appears in Settings › Agents as a device you have to let in, and
it stays a device you can throw out; until you do let it in, everything it
tries is refused.

What it may do is deliberately small: file a task with its steps and its date,
add to the journal, write a document, add to the end of one that is already
there, correct a passage of one, file documents into folders, keep a copy of a
file you point it at — on a task or inside a document, which takes the larger
file of the two — and read what is already there. What it may not do: close
a task or delete one, delete a document or hand it a new body whole, rename or
empty a folder, reach a task you folded away, take files from outside the
folders where a download lands, or file the same thing twice. To correct a
passage it has to name it exactly as you wrote it, and if that text is not there
or is there twice, nothing is written at all. A document you archived it can
still read, and it is told that you archived it.

Whatever it reads travels wherever that assistant travels. That is between you
and it — which is precisely why this is a door you open, and not one that was
already open.

## Where It Stands

| | |
|---|---|
| ✅ | Tasks, lists, tags, steps, journal, attachments, search |
| ✅ | An archive read in three layers, with what each task left behind |
| ✅ | Natural language for dates, deadlines and repeats |
| ✅ | The window, the tray, and quick capture on a global shortcut |
| ✅ | Documents: editor, folders, and sync that merges block by block |
| ✅ | An assistant can file for you, over MCP, if you let it in |
| ✅ | Reminders, backup, and sync through a folder both machines reach |
| ✅ | A guided welcome, a guide written into your store, English and Spanish |
| ✅ | macOS: signed and notarised. Windows: signed installer |
| ◐ | Daily use, which is what turns up the bugs tests do not — what the release candidate is for |
| ◐ | The Microsoft Store listing, whose screenshots have to be retaken on Windows |

Two things are known and accepted rather than pending: you cannot reorder by
hand in the window — HTML drag and drop does not survive the native file drop
that attachments need, and attachments were the better trade — and nothing has
been tested with a real screen reader, though the keyboard path has.

## What It Will Never Do

As important as the list above. Permanently out of scope: real-time
collaboration, kanban boards, Gantt charts, time tracking, productivity metrics,
databases with typed properties and formulas, and AI anywhere in the critical
path.

The natural language stays deterministic and local, and **Tisty never sends
anything to a model** — not a task, not a word. If you open the door in the
section above, whatever your assistant reads travels wherever that assistant
travels: Tisty still sends nothing, the assistant is the one carrying, and you
are the one who let it in.

## Other Tools by the Same Author

Same idea, same terms: free, open source, no ads, no telemetry, everything
local.

- **[CopyPaste](https://github.com/rgdevment/CopyPaste)** — a clipboard manager
  for Windows, macOS and Linux.
- **[LinkUnbound](https://github.com/rgdevment/LinkUnbound)** — a browser
  selector for Windows and macOS: it asks which browser should open a link
  instead of assuming.

## Standing On

Tisty is small because other people's work does the heavy lifting.

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

How the store, the merge and the sync actually work is written down in
[ARCHITECTURE.md](docs/ARCHITECTURE.md) — a reference for the behaviour,
not a tour of the code.

Read [CONTRIBUTING.md](CONTRIBUTING.md). Open an issue before writing code for
anything beyond a fix: Tisty is deliberately minimal, and a well-written feature
can still be declined — usually because it would make the tool something other
than what it is.

## Licence

[AGPL-3.0](LICENSE), and available under
[commercial terms](docs/COMMERCIAL.md) for organisations that cannot comply
with it.

The signed builds in the app stores carry their own terms, because the stores'
do not accept the AGPL — [DISTRIBUTION.md](docs/DISTRIBUTION.md) says which
applies to what you have, and why. Nothing is withheld from the source either way.
