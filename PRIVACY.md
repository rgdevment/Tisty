# Privacy policy

**Last updated:** 5 August 2026

## The short version

Tisty does not collect anything, does not transmit anything, and has no server
to transmit to. No account, no telemetry, no analytics, no crash reporting, no
phone-home.

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

Tisty itself only ever reads and writes local paths. It has no network code for
syncing, no credentials, and no idea which provider — if any — is behind the
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

**Nothing is encrypted at rest.** Your operating system's file permissions are
the only protection. This was a deliberate decision, not an oversight:
encryption breaks three-way merges of documents, which is precisely where
synchronisation needs them most. The reasoning is in
[SECURITY.md](SECURITY.md).

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
- Does not make background network calls.

If a future version ever needs to make a network request — an update check,
for instance — it will be read-only, documented in this file before it ships,
and visible as a code change in the public repository first.

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
