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
  telemetry, no remote server, no phone-home. There is no cloud service to
  breach because there is no cloud service.
- **Synchronisation is yours.** Tisty writes files; you choose whether Git or
  a folder in a cloud you already trust moves them. There is no Tisty-operated
  backend at any tier.
- **No plugin system.** Nothing loads third-party code into the process.
- **`unsafe` is forbidden** at the workspace level, not merely discouraged.

## What it deliberately does not protect

Being explicit here matters more than sounding reassuring.

- **Data is stored in plain text.** Tasks, notes and documents sit readable on
  disk, protected only by your operating system's file permissions. At-rest
  encryption was evaluated and rejected: it breaks three-way merges of
  documents, which is the single place where synchronisation most needs them.
- **Git history does not forget.** If you sync through Git and a credential
  ends up in a task, deleting the task does not remove it from history — and
  once pushed to a remote, it cannot be purged without rewriting that history
  everywhere. Prevention is the only real defence: keep anything sensitive in
  the `private/` folder, which never leaves the machine under any transport.
- **Anyone with access to your user account has access to your data.** Tisty
  adds no second authentication layer on top of your operating system's.

## Supported versions

Until the first stable release, only the latest published version receives
fixes.
