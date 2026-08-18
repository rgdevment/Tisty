# Frequently asked questions

## I changed something on one machine and it is not on the other

Tisty writes to and reads from the folder you chose. **What uploads that
folder is your provider's own program** — Google Drive, OneDrive, iCloud,
Dropbox, Syncthing — not Tisty. If that program is not running, Tisty will
keep saving your work fine and will have no way to tell that nobody is picking
it up.

Go through this in order. The first three cover almost every case.

### 1. Is your cloud program running?

This is the usual cause, and the easiest to miss: all it takes is for its
start-at-login to be off and for you to reboot one day.

- **macOS** — look for its icon in the menu bar, top right. If it is not
  there, it never opened.
- **Windows** — check the system tray, next to the clock; expand the arrow for
  hidden icons.

Open it and give it a few minutes.

### 2. Is it paused, or signed out?

An open client is not always syncing. Check in its own panel that it does not
say paused, that the account is still signed in, and that you have not run out
of space.

### 3. Is that folder among the ones it syncs?

Many clients let you choose which folders go up. If the one you gave Tisty
fell outside that selection, everything works except the one thing that
matters.

### 4. Is the other machine on?

Changes travel through the folder, so the other machine has to open Tisty to
pick them up. While it is off, your work waits up there.

### 5. Check with your own eyes

In **Settings → Data** there are two dates that do not mean the same thing:

- **My last round** — when Tisty read and wrote in the folder. This one being
  current does **not** mean anything was uploaded.
- **Something last arrived from another machine** — when something genuinely
  came in from another machine. If it has been still for days and your
  machines are on, that is where the problem is.

The **Open the folder** button shows it in your file browser. Look there for
your recent files, then check with your provider — from its website, say —
whether they are up there too. That comparison is what answers the question.

### If all of the above is fine

Try **Settings → Data → Send everything again**. It copies everything of this
machine's to the folder again without asking whether it is already there,
which is exactly what you want when a cloud client skipped a file.

## Why does Tisty not warn me on its own?

Because it cannot know. Tisty talks to no cloud: it has no account of yours,
no credentials, and no idea which provider — if any — is behind the folder you
named. It leaves the files there, and there its reach ends.

It could ask your provider's program, and it may one day. Today it does not,
so it shows you the facts it does know instead of a verdict it cannot back.

## Is anything lost meanwhile?

No. Your work is complete on the machine where you did it, and Tisty's log
only appends: nothing is overwritten or dropped. As soon as the folder starts
moving again, both machines catch up on their own, even if both were working
at once.
