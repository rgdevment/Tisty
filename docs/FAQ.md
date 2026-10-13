# Frequently asked questions

## I changed something on one machine and it is not on the other

Tisty writes to and reads from the folder you chose. **What uploads that
folder is your provider's own program** — Google Drive, OneDrive, iCloud,
Dropbox, Syncthing — not Tisty. If that program is not running, Tisty will
keep saving your work fine and will have no way to tell that nobody is picking
it up.

Go through this in order. The first three cover almost every case. If you
connected Google Drive directly instead of a folder, Tisty talks to it itself
and says what is wrong on its own card: see
[Syncing through Google Drive directly](#syncing-through-google-drive-directly).

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

### 5. Is the other machine waiting on you?

Every machine signs what it writes. One that came to the folder after this one
says which key it signs with, and nothing it writes comes in here until you
confirm that key. In **Settings → Maintenance → The machines** it shows as
**Waiting**: open its card here and on that machine, check the key is the same
character by character, and **Confirm**. From the terminal,
`tisty sync --confirm <machine>` does the same. What it wrote meanwhile stays in
the folder and arrives whole.

### 6. Check with your own eyes

In **Settings → Your data** there are two dates that do not mean the same thing:

- **My last round** — when Tisty read and wrote in the folder. This one being
  current does **not** mean anything was uploaded.
- **Something last arrived from another machine** — when something genuinely
  came in from another machine. If it has been still for days and your
  machines are on, that is where the problem is.

The **Open the folder** button shows it in your file browser. Look there for
your recent files, then check with your provider — from its website, say —
whether they are up there too. That comparison is what answers the question.

### If all of the above is fine

Try **Settings → Your data → Send again**. It copies everything of this
machine's to the folder again without asking whether it is already there,
which is exactly what you want when a cloud client skipped a file.

## Why does Tisty not warn me on its own?

Because it cannot know. With a folder, Tisty talks to no cloud: it has no
account of yours, no credentials, and no idea which provider — if any — is
behind the folder you named. It leaves the files there, and there its reach ends.

It could ask your provider's program, and it may one day. Today it does not,
so it shows you the facts it does know instead of a verdict it cannot back.

## Is anything lost meanwhile?

No. Your work is complete on the machine where you did it, and Tisty's log
only appends: nothing is overwritten or dropped. As soon as the folder starts
moving again, both machines catch up on their own, even if both were working
at once. If both wrote the same part of a document, the window shows the two
side by side and asks which stays — **This one**, **The other** or **Both**.

## Syncing through Google Drive directly

Since 1.30, Tisty can sync through your own Google Drive without Google's
desktop program. Here Tisty is the one talking to Drive, so unlike with a
folder, it knows when something is wrong and says so on the Google Drive card
in **Settings → Syncing**. Whatever the card says, what you write is safe on
this machine and goes up once the cause is gone.

### How do I connect it?

**Settings → Syncing → Your cloud service → Google Drive → Connect.** Your
browser opens on Google's own page; sign in and allow Tisty to keep its copies
there. Tisty asks only for a hidden space of its own in your Drive, so it cannot
see anything else there, and its copies do not show up among your files. Then
**Start uploading**: the first time everything you have goes
up, and if you close Tisty or lose the network it carries on where it left off.
Do the same on your other computer, with the same Google account.

It can only be connected from the window. The command line and assistants never
reach the network, and a copy of Tisty built without this project's Google
credentials does not offer the cloud at all.

### What travels, and what stays

Your history, your documents and their settings go up. Attachments stay on the
computer they were added on; on another one each comes down the first time you
open it there. **Free space** lets go of the attachments you have not opened in
a while; the next time you open one, it comes down again. Attachments are capped
at 25 MB per task and 650 MB inside a document, as with a folder.

### «Google no longer lets Tisty in»

The permission was removed — from your Google account, or with **Remove
Tisty's access on every computer** — or it expired. Press **Reconnect** and sign
in again. Removing access on one machine removes it on all of them, so each one
will ask.

### «This copy belongs to another account»

You signed in with a different Google account from the one this copy already
syncs with. Tisty stops rather than mix two accounts. Press **Change account**
and sign in with the one you used before. If you really mean to sync with a
different account, move there instead (below).

### «Your Google Drive is full»

Your Drive has no room left. Tisty sends your tasks first; the rest, including
what your other computers send, waits until you free up space in Drive or get
more. **See storage** opens Google's page for it.

### «Your cloud reached today's limit»

Every computer has its own daily allowance — 2,000,000 requests' worth and
50 GB downloaded — so no single copy can spend the quota Google grants the
project. It is per computer, not per person. When it runs out, syncing carries
on the next day by itself. **Cloud use today**, on the same card, shows how much
is left; `tisty doctor` prints it too.

### «Google asked to wait a little»

Google is slowing everyone down for a moment. Tisty waits as long as Google says
and carries on by itself. There is nothing to do.

### «The cloud is paused for now»

The project can pause syncing on every copy for a while, should its shared
quota with Google be at risk; the notice is signed, so nobody else can send it.
It lifts by itself. **Turn off** on the card does the same on this computer
only, until you turn it on again.

### Leaving Google Drive

**Move to another place…** takes your copies to a folder or back to this
machine alone. Tisty first brings home everything that lived only in Drive,
then tells your other computers where you went so each one follows when it
syncs. Nothing is deleted in Drive unless you ask; **Delete my copies in Google
Drive for good** deletes them afterwards, and since Tisty's space in Drive has
no bin, they cannot be brought back. To stop one computer without moving the
others, use **Disconnect this computer**: the copies stay in Drive.

### Where are my copies in Drive?

In a hidden space of your Drive that belongs to Tisty: they do not show up among
your files, and only Tisty opens them. To take them out, move to a folder from
Tisty. Disconnecting keeps them, and coming back later to the same account picks
them up again, also after *Disconnect from Drive* in **Google Drive › Settings ›
Manage apps**. Only **Delete hidden app data** there deletes them, as does
**Delete my copies in Google Drive for good** in Tisty.

## One of my machines stopped syncing after an update

Tisty writes a version into every event, and a machine only reads what it
understands. When one of them updates to a version that writes something new —
a page that answers for itself, say — the other stops syncing **in both
directions** until it updates too: nothing comes in, and what you write there
stays there. Nothing is lost, and nothing lands half understood; update the
machine that is behind and the two carry on where they left off. Update both
before you rely on them syncing again.

## A document opened read-only and I did not ask for that

Three different reasons, and the bar at the foot of the document tells you
which.

**It is in the archive.** Archiving a document closes it to writing — from the
window, from the terminal and from an assistant alike — because putting
something away means it is finished. Bring it back and it writes again.

A whole folder can be in the archive too, and then everything under it is:
subfolders, documents and pages, without any of them being marked one by one.
Only the folder comes back, and when it does each document gets its own state
back — one you had archived by hand before stays archived. Nothing new goes into
a folder that is away, and it only moves inside the archive until it returns.

**A page of it is in the archive, or the document that holds it is.** A page is
put away on its own without leaving the document it belongs to: it stays in
place, dimmed and with the icon of the archive, and it reads as it was left.
Putting the whole document away covers every page it holds; bringing it back
wakes each one as it was, and the page you had set apart stays apart.

**Or the writing itself is at risk.** Tisty's editor writes Markdown back out,
and a few shapes do not survive that
trip: front matter, footnotes, links written by reference, blocks of HTML, HTML
comments and entities, whatever a code fence says after its language beyond a
name, and a list item that opens on a block — a quote, a fence, a heading, a
picture or another list where its first line should be text. Rather than open
the document and quietly destroy those on your first keystroke, Tisty says so
and opens it for reading.

The bar at the foot offers to convert it — which rewrites the document into what
the editor can keep, and keeps a copy of how it was beside the documents. If the
conversion cannot get all of it, you are not stuck: **Edit it anyway** opens the
document with the warning left in view, so you decide what is worth losing.

## Do the diagrams and the formulas need the internet?

No. A code block whose language is `mermaid` draws its diagram, and one that
says `math` sets its formula, and both are drawn by code that ships inside
Tisty. Nothing is fetched, no font is loaded from anywhere, and it works with
the network unplugged — same as everything else here.

Both stay Markdown. GitHub draws ` ```mermaid ` and ` ```math ` too, so the
document reads the same anywhere else.

## Why does my table's column width survive, and where does it live?

In how long the rule under the header is drawn. Markdown has nowhere to keep a
column width, so Tisty writes it as the length of that column's dashes and reads
it back the same way — a wider column, a longer rule. Every other Markdown
reader ignores how many dashes there are and draws the table just the same.

It is written in steps rather than to the pixel, and a column narrower than 40
or wider than 1200 pixels is kept at that bound. A table nobody has resized is
written with three dashes, exactly as before, so nothing you already have
changes.

## Can an assistant write over a document I am working on?

Only over the version it read. When an assistant reads a document it is handed a
print of that exact text, and replacing the body whole requires sending that
print back. If you wrote in the document in between, the print no longer
matches: nothing is written and the assistant is told to read it again. What it
did replace is kept beside the documents either way.

The window tells you when something wrote in the document you have open, and if
you try to save over a change you had not seen, it asks which one stands. That
same bar offers the other way out: put the document back to what it said before,
out of the copy kept beside the documents. What it says now is kept in its place,
so the choice is not final. It is offered only when that copy is exactly one
write back, which is the only time it can do what it says.

## An assistant says one of its tasks is done. Did it close it?

No. Nothing an assistant does closes a task, and there is no tool that would let
it. What it can do is say so: the task gets a mark, the hour it spoke, and the
account of what it did in the journal — the command it ran and what that
answered, the commit, whatever holds the claim up. The task stays open until you
decide.

Those tasks gather under **To confirm** at the foot of the list, with a count of
how many wait on you, so you can go through them in one pass. What is due today
or already overdue does not move: it stays in its own band, marked, because a thing
with a date on it should be where you look for it.

Tick it and it closes like any other, keeping both signatures in the archive —
who resolved it and who finished it. Or open it and choose **Still to do**: the
mark comes off, the task goes back to its day, and the account stays, so next
time you can both see what was claimed. An assistant can only speak for tasks it
filed itself, never for the ones you wrote, and it cannot say the same thing
twice while a mark is still waiting on you.
