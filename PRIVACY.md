# Privacy policy

**Last updated:** August 27, 2026

## The short version

Tisty does not collect anything and has no server to transmit to. No account,
no telemetry, no analytics, no crash reporting.

It makes **one** network request on its own: once a day it downloads a small
file to see whether a newer version exists. It sends nothing. Two more follow
only if you press the button that offers you an update, and never otherwise.
All three are described in full below.

That is not a policy promise you have to take on trust — it is a property of
the code, which is [public and auditable](https://github.com/rgdevment/Tisty).
Run a network monitor against it if you like.

## The part that is not "everything stays local"

Most privacy policies for local applications stop at "your data never leaves
your machine". Tisty's cannot, because **Tisty is designed to synchronise**,
and being vague about that would be dishonest.

Here is the accurate statement:

> **Tisty never sends your data anywhere on its own initiative. If you turn on
> synchronisation, your data goes exactly where you told it to go, and whoever
> runs that destination can technically read it.**

What you can choose, and what each implies:

| You choose | Who else can reach your data |
|---|---|
| **Local only** (the default) | Nobody. It never leaves the machine |
| **A folder your cloud client keeps in step** (Google Drive, OneDrive, iCloud, Dropbox, pCloud…) | That provider, under their terms and their jurisdiction |
| **A folder on hardware you own** (a NAS, an external drive) | Whoever can reach that hardware |
| **A backup zip you keep somewhere** | Wherever you put it |

For syncing, Tisty only ever reads and writes local paths. It has no network
code for it, no credentials, and no idea which provider — if any — is behind the
folder you named. Whatever keeps that folder in step between your machines is
software you already installed and already trust.

There is no option where Tisty hosts anything. There is no paid tier that
changes this. **We never operate a server that holds your data**, so there is
nothing on our side to breach, subpoena, sell, or lose.

Choosing a destination is choosing who you trust. Tisty makes that choice
visible instead of making it for you.

## What is stored, and how

- **Tasks, projects and events** — plain-text JSONL files. Readable with
  `cat`, searchable with `grep`, parseable with `jq`.
- **Documents** — plain Markdown files.
- **Attachments** — the files themselves, unmodified.
- **Local configuration** — including the device identifier. The *file* never
  leaves this machine, and it lives in the local application data directory,
  never a roaming profile, so a company logon script cannot carry it off. The
  identifier itself does appear in the shared folder — it names your device's
  directory and stamps every event — because that is what tells two writers
  apart. What must never be shared is the file that binds it to this machine.
- **A `private/` folder** — anything placed here never leaves the machine
  under any transport.
- **The guide** — the one thing Tisty writes for you rather than the other way
  round. Its words and its images ship inside the program and are copied into
  your documents in the language you picked. Nothing is fetched to do it, and
  once it is there it is an ordinary document: edit it, or delete it.

**Nothing is encrypted at rest.** Your operating system's file permissions are
the only protection. This was a deliberate decision, not an oversight:
encryption breaks the three-way merge of documents, which is precisely where
synchronisation needs it most — two machines editing the same document are
reconciled block by block, and that cannot be done on bytes nobody can read. The
reasoning is in [SECURITY.md](SECURITY.md).

**Nothing is obfuscated.** If Tisty disappeared tomorrow, your data would
still be readable with tools you already have. That is the point.

## One consequence worth knowing before it bites you

**An append-only log does not forget.** If a credential ends up in a task,
deleting the task removes it from the current state but not from the history
underneath — and if it already travelled to the shared folder, it is on every
machine that pulled it, plus whatever the provider keeps in its own version
history.

The `private/` folder exists for exactly this. Prevention is the only real
defence, because the remedy barely exists.

## What Tisty does not do

- Does not send data to any server.
- Does not create accounts or profiles.
- Does not use cookies, analytics, advertising, or tracking of any kind.
- Does not share anything with third parties.
- Does not use AI or machine learning on your data. The natural-language
  parser is deterministic rules running locally; nothing is sent to any model.
- Does not upload crash reports.
- Does not make background network calls, except the update check below.

An earlier version of this file promised that if Tisty ever needed a network
request it would be read-only, documented here before it shipped, and visible
as a code change first. That is what happened: the check went into the public
repository, and this section was written before the release that carries it.

## The request Tisty makes on its own

| | |
| :--- | :--- |
| **Why** | To tell you a newer Tisty exists |
| **What it asks for** | `https://raw.githubusercontent.com/rgdevment/Tisty/manifest/release-manifest.json` |
| **Method** | GET. Nothing is uploaded |
| **What it sends** | The headers a request cannot avoid, and a user agent that says `tisty/<version>` — the same thing the download itself would reveal. No identifier, no store, no task, no name |
| **How often** | At most once a day, and only while the window is open |
| **Timeout** | 5 seconds |
| **If it fails** | Nothing is said and nothing is retried until tomorrow |

The file it downloads contains version numbers and nothing else — no address, so
nothing that arrives from it can send you anywhere.

## The two requests only you can start

Pressing **Update** in *About* makes Tisty fetch a second file, which does name
an address, and then the installer at that address. Both go to
`raw.githubusercontent.com` and `github.com` over HTTPS, and both carry what any
download carries: an address to send the bytes back to, and a user agent — here
`tauri-plugin-updater/<version>`, the library doing the fetching. Nothing about
you, nothing about what you have written.

This is the one place where an address that arrived over the network is opened,
and it is why the signature matters: Tisty checks the installer against a key
compiled into the copy you already have, and refuses anything that does not
match. The check happens before a single byte is run or written to disk.

On macOS, if the application sits somewhere your account cannot write, the
system asks for an administrator password to put the new copy in place. You can
refuse; nothing is moved.

A link **you write or paste into a document** is a different matter: clicking it
opens your browser, the way any Markdown reader does. Tisty does not follow it
on its own, and never opens anything without a click. Worth knowing if a
document reached you from a shared folder someone else can write to — the words
of a link and where it goes are not obliged to agree, in Tisty or anywhere else.

**Nothing is downloaded or installed unless you ask for it.** What the daily
check brings back is a line of text you can ignore. If a newer version exists, a
line appears in *About*, and what it offers depends on how this copy was
installed: the Microsoft Store updates itself and Tisty leaves it alone, a copy
of the command line kept by Homebrew gets the command to run, and a copy that
owns the folder it lives in gets a button.

That button is the only thing that fetches an installer, and what it does is
described above. If you never press it, nothing is downloaded and the line just
sits there.

## Children's privacy

Tisty collects no personal information from anyone, of any age. There are no
accounts, no registration and no data transmission.

## Changes to this policy

Any change is committed to the public repository with a clear message, updates
the date above, and appears in the release notes. Because the project is open
source, a change in privacy behaviour would be visible as a code change before
it ever reached you.

## Contact

<github@apirest.cl>

---

*Tisty is in early development. This document describes decisions already
made; it will be expanded with concrete storage paths and behaviour as the
implementation lands, and never in the direction of collecting more.*
