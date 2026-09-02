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
  is no cloud service. Tisty makes one request of its own — a daily GET for a
  version manifest, carrying nothing but the version you are running — and two
  more only if you press the button that offers you an update. It is written out
  in full in [PRIVACY.md](PRIVACY.md), down to the headers.
- **An update is verified before it is installed.** What the button fetches is
  checked against a public key compiled into the copy already on your machine.
  A build signed with any other key is refused, so a release that is not the
  maintainer's cannot arrive this way, whatever it claims to be. Two narrower
  guards sit in front of that one: the address an update is fetched from must be
  where this project's releases live, not merely whatever the feed named, and
  the download is given a deadline, so a server that answers slowly forever is
  eventually hung up on.
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
- **The signature says who built it, not which version it is.** minisign signs
  the bytes of an installer and nothing else, so an attacker who controlled the
  update feed could serve an *older* Tisty — genuinely signed, genuinely ours —
  and an installed copy would accept it, losing whatever the newer one had
  fixed. Serving something that is not ours stays impossible. Tisty narrows the
  window by refusing anything not newer than what is running, but the feed
  states the version, so a lie about the version is a lie Tisty cannot check.
  This is inherent to the update format, and worth knowing rather than hiding.
- **The updater's signing key is a single point of failure.** One private key
  signs every release, it is held by the maintainer alone, and the format offers
  no way to rotate it: the public half is already compiled into every copy out
  there. If it leaked, a build signed with it would be accepted by installed
  copies; if it were lost, those copies would go on working and simply stop
  being able to update themselves. Either way the fix is a new release installed
  by hand, and it would be announced on the releases page.
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

## An assistant, if you admit one

Tisty speaks MCP so an assistant already running on your machine can file work
for
you. That door is shut until you open it, and what follows is what holds it.

**Admitting one is an event in your log, not a line in a settings file.**
Nothing
arriving over the wire can register an agent; the log decides who may write, so
an
assistant cannot grant itself a voice by editing the file it can reach.

**It writes under a device of its own.** Your undo never reaches what it filed,
and
throwing it out leaves everything it wrote in place. What it files is tagged,
and it
may only file into a list that already exists — it cannot make one.

**It cannot reach what you hid.** A task you folded away is neither returned nor
counted; a count of one would say the thing exists. What it reads comes back
without
the absolute paths of your disk.

**A body it did not read is a body it cannot replace.** An assistant may write a
whole document again, not only add to it — but only by sending back the print
that document read at when it last looked. If you have written in it since, the
write is refused and it has to read again. It can be wrong about a document; it
cannot quietly take what you wrote while it was thinking. What it wrote over is
kept beside the documents either way.

**Attachments are the one thing that leaves the machine.** They reach the shared
folder, and from there whatever cloud client you run. So the file an assistant
may
copy is judged by its bytes rather than its name: pictures, PDFs, plain text and
office documents pass; a private key, a PKCS#12 bundle or a file shaped like an
environment file is refused even when renamed to `.png`. Files outside
Downloads,
Documents, Pictures, Desktop and the temporary folder are refused before
anything is
read, and the path is canonicalised first, so traversal does not reach past it.

**What this does not protect against.** An assistant under prompt injection is
still
an assistant with your permission: it can file nonsense, write documents you did
not ask for, write over one you already had with a version of its own, and put a
screenshot holding a password into a task, because a
screenshot is a picture and Tisty cannot read what is in it. Anything it reads
travels wherever that assistant travels — that is between it and you, and it is
why the door stays shut until you open it, and why closing it is one click.

## Supported versions

Until the first stable release, only the latest published version receives
fixes.
