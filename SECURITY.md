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
- **Synchronisation is yours.** Tisty copies files into a folder you name, and
  whatever already keeps that folder in step between your machines is what moves
  them. There is no Tisty-operated backend at any tier, and no transport of ours
  to attack: the code that syncs reads and writes local paths.
- **No plugin system.** Nothing loads third-party code into the process.
- **`unsafe` is forbidden** at the workspace level, not merely discouraged.

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
  adds no second authentication layer on top of your operating system's, and
  on Unix the files carry the default permissions of your account, so another
  local account with a readable home directory can reach them.
- **Whoever can write to the shared folder can write to your history.** The
  transport reads what it finds there. It refuses what does not parse and it
  refuses a second store, but it cannot tell a genuine event from a forged one.

## Supported versions

Until the first stable release, only the latest published version receives
fixes.
