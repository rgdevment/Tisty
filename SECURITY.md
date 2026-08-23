# Security policy

Tisty stores tasks, notes and documents — material that is often personal and
sometimes confidential. This document explains what the design protects, what
it deliberately does not, and how to report a problem. For what is collected
and where it goes, see [PRIVACY.md](PRIVACY.md) — the short answer is nothing
and nowhere.

## Reporting a vulnerability

**Do not open a public issue.** Use either private channel:

- **Email** <github@apirest.cl>, subject `[SECURITY] short description`.
- **GitHub Security Advisory** — the *Security* tab, *Report a vulnerability*.

Useful in a report: what an attacker could do and to whom, steps to reproduce,
the affected version, and your operating system. A proof of concept helps but
is not required.

You will get an acknowledgement within 48 hours. Fixes for anything serious
are aimed at days, not weeks. You will be credited in the release notes unless
you prefer otherwise.

Good-faith security research will never be met with legal threats, and a
report will never be dismissed for being inconvenient.

## What the design protects

- **The data never leaves your machine unless you tell it to.** No account, no
  telemetry, no remote server. There is no cloud service to breach because there
  is no cloud service. Tisty makes exactly one request of its own — a daily GET
  for a version manifest, which you can turn off, and which carries nothing but
  the version you are running. It is written out in full in
  [PRIVACY.md](PRIVACY.md), down to the headers.
- **Synchronisation is yours.** Tisty copies files into a folder you name, and
  whatever already keeps that folder in step between your machines is what moves
  them. There is no Tisty-operated backend at any tier, and no transport of ours
  to attack: the code that syncs reads and writes local paths.
- **No plugin system.** Nothing loads third-party code into the process.
- **`unsafe` is forbidden** at the workspace level, not merely discouraged.
- **The window is fenced by a content policy.** Everything it runs is shipped
  inside the program: no script from anywhere else, no page from anywhere else,
  and the only address it may talk to is the local channel to the Rust side.
  Two openings are worth naming because they are real. Scripts may compile
  WebAssembly, which is what draws the PDF you see before exporting it; and a
  frame may show a `blob:`, which is that PDF, built in memory on your machine
  and never fetched. Neither lets remote code in: a blob has no origin to be
  loaded from, and there is no path by which one arrives.

## What it deliberately does not protect

Being explicit here matters more than sounding reassuring.

- **Data is stored in plain text.** Tasks, notes and documents sit readable on
  disk, protected only by your operating system's file permissions. At-rest
  encryption was evaluated and rejected: it would put every reader of your data
  behind a key Tisty would then have to manage, and the promise here is that
  your files stay readable with tools you already have.
- **An append-only log does not forget.** If a credential ends up in a task,
  deleting the task removes it from the current state but not from the history
  underneath — and if it already travelled to the shared folder, it is on the
  other machines too. Prevention is the only real defence: keep anything
  sensitive in the `private/` folder, which never leaves the machine.
- **Whoever holds the shared folder holds your data.** Syncing through a
  provider's folder means that provider stores your tasks, under their terms.
  That is the trade you make when you turn it on, and Tisty states it rather
  than hiding it.
- **Anyone with access to your user account has access to your data.** Tisty
  adds no second authentication layer on top of your operating system's. On
  Unix it does narrow the permissions it controls — its directories to `0700`
  and its files to `0600`, owner only — but that stops another local account,
  not you, and not anything running as you.
- **Whoever can write to the shared folder can write to your history.** The
  transport reads what it finds there. It cannot tell a genuine event from a
  forged one, and no signature would help while the folder is shared with
  whoever holds it.

  What it does check is narrower and worth stating: it never writes through a
  symbolic link, into the folder or into any directory inside it; an attachment
  must hold the bytes its own name vouches for; one that was retired is not
  carried back in; a document body past the reader's ceiling is refused rather
  than swapped in for one that opens; a store carrying a different name stops
  everything until you decide; and a machine that was removed is refused
  outright — that removal is absorbing, so a removed identifier is never valid
  again and a machine that returns comes back as a new one.

## Supported versions

Until the first stable release, only the latest published version receives
fixes.
