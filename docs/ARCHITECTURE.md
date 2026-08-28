# Architecture

How Tisty stores, merges and reads its data. This is a reference for how the
system behaves, not a record of why it was designed this way.

## The two layers

```text
<data>/store/                    the truth. Syncs. Survives everything.
├── dev_a3f1/
│   ├── 000001.tisty             sealed segment, never written again
│   └── active.tisty             the only file this device appends to
└── dev_9f2c/
    └── active.tisty             another device's, never touched by this one

<cache>/read.db                  a photograph. Local, disposable, rebuilt on demand.
```

The store is a log of events. The cache is the state those events produce.
Deleting the cache costs one slow read; deleting the store loses data.

Neither lives in the operating system's Documents folder — which is a different
thing from Tisty's own `docs/` — and the path is not configurable.

## The event log

One JSON object per line. Files only ever grow at the end, and a sealed segment
is never modified again.

```jsonl
{"v":1,"ts":"2026-08-06T22:22:18.006Z","by":"dev_a3f1","op":"task.add","id":"01KZ…","d":{"title":"buy bread","order":"V"}}
```

| Field | Meaning |
|---|---|
| `v` | schema version; a newer one is refused, not guessed |
| `ts` | when it happened, UTC |
| `by` | which device wrote it |
| `n` | sequence within that device, absent when zero |
| `tx` | groups the events of one user action |
| `un` / `re` | marks a compensation or a replay of one |
| `tz` | the zone whoever wrote it was in, so an hour reads back where it happened |
| `opt` | a reader that cannot make sense of this operation skips it instead of refusing the store; absent means refuse |
| `op`, `id`, `d` | the operation, the entity it affects, its payload |

Only mark `opt` on an operation that **adds**. A reader forgives it solely when the
name is one it has never heard of — a known operation that fails to parse is
corruption and stops the read regardless — but nothing can stop a writer marking
something that changes what already exists, and a reader would then drop it and
diverge in silence.

Three payload fields carry more than their name says:

| Field | On | Meaning |
|---|---|---|
| `k` | `device.join` | `agent` or `machine`. Absent is not a claim of either: an event written before the field existed must not demote an agent |
| `source` | `task.add` | what the task was written from, so the same thing is not filed twice |
| `filled` | `task.done` | closed in bulk by the backfill, so its stamp is the hour of the marking rather than its own |

`active.tisty` is sealed as `NNNNNN.tisty` every 5.000 events. Sealed segments
are numbered from one without gaps.

### Priorities are named, not numbered

A task's priority is one of the four quadrants of the **Eisenhower matrix** —
the method credited to President Dwight D. Eisenhower, popularised by Stephen
Covey — plus a fifth value for the tasks nobody has placed yet:

```jsonl
{"v":4,"ts":"…","by":"dev_a3f1","op":"task.set","id":"01KZ…","d":{"priority":"delegate"}}
```

`do` · `decide` · `delegate` · `minor` · `unset`. **The word goes on disk, not a
number**, so the log still says what it means when you read it without Tisty,
and the default value is written out rather than hiding inside a `4`. Schema 5
renamed the fourth quadrant from `wont`, and still reads that older word.

Two of these words are older than the labels on screen. The window calls
`decide` **Schedule**, because the quadrant is for deciding *when*, not whether;
`minor` reads **Minor** rather than the `wont` it once was. The names on disk
were left alone on purpose — renaming them would rewrite history that other
machines already hold, to say the same thing. Both spellings are accepted when
you type a priority: `!schedule` and `!decide` set the same quadrant.

Quadrants are not a ladder, so a number cannot name one: schema 4 reads the
levels `1..4` an older Tisty wrote as `unset`, and refuses anything else. Those
old events stay on disk untouched — the log only ever appends — so nothing is
destroyed by the change; it stops being shown.

The order the quadrants sort in is the order they are read in, with one deliberate
exception: **`minor` sorts last of all, behind `unset`**. That order is what numbers
the tasks you address in the CLI, and what you have declared you will not do belongs
at the bottom of that list, not floating above the untriaged pile.

### Writing

Only ever to this device's own directory. Two rules hold everything else up:

**Time only moves forward.** The store remembers the last stamp it wrote and
never emits one lower or equal. A clock that steps back reuses the instant and
raises `n` instead.

**The log is written before the cache**, with `fsync`. A crash between the two
leaves the cache *behind*, which repairs itself on the next read. It can never
leave the cache *ahead*, which would mean data that exists nowhere else.

Both rules are per device directory, and more than one process may own the same
one: a running GUI and a `tisty` command are the same device. So a write takes
an exclusive lock, and **holds it for that write only** — a process that kept it
would refuse every command for as long as it stayed open. A lock found busy is
waited on briefly, since a write lasts microseconds; only a lock that stays busy
is reported as a conflict.

Releasing it means the counters can go stale, so the next write re-reads them
first. It compares the active log's size against what it last saw, and reparses
only when they differ. Without that, two processes stamp from clocks that never
saw each other's events, and `(ts, by, n)` stops being unique — which is the one
thing the merge order cannot survive.

### Reading

Every device reads every directory, its own included. Events from all of them
are concatenated and sorted by `(ts, by, n)` — the same order on every machine,
so every machine reaches the same state.

Reading refuses to continue, rather than returning a smaller history, when:

- a sealed segment is missing from the sequence,
- a sealed segment is present but empty,
- a sealed segment holds a different count of events than its `.count` declares,
- any line fails to parse,
- an event declares a schema version this build does not know.

Syncing holds the same line from the other end: a device's history that arrives
**shorter than the one already held** is left where it is. Contiguity alone does
not catch that — the gap is not *between* sealed segments but *before* `active`,
and nothing in the folder says how many sealed ones there should be. So the
counts are compared instead. A cloud client may well deliver a rotated `active`
before the sealed segment that carries what it dropped; that ordering must not
cost anyone their copy.

## How merging works

Nothing is ever edited. Facts are recorded about an entity, and the entity is
identified by a ULID that means the same thing everywhere.

```text
dev_mac      task.add    MNKMPX  "buy bread"
dev_windows  task.log    MNKMPX  "went to the corner shop"
             task.done   MNKMPX
```

Three events, two files, no coordination. Any device that reads both files
produces the same task: created, completed, with one journal entry.

Files are never merged. **The merge happens in memory, on every read.**

Fields that nobody else touched are simply kept. When two devices write the
same field without having seen each other, the later stamp wins. Collections —
journal entries, steps, tags — accumulate instead of competing, because each
element carries its own identifier.

Deleting is the exception: it leaves a tombstone, and nothing about that entity
is ever applied again. That is what stops a late event from resurrecting it.

A task only becomes deletable once it is **archived and hidden** — two
deliberate steps, refused otherwise, so nothing goes on a slip. The tombstone
travels; what the log already recorded about the task stays where it was
written.

## Repeating

A repeat is **one task per occurrence**, not one entity collecting completions.
Finishing «take out the bins every Tuesday» writes two events in one batch: the
completion, and next Tuesday's task.

```text
task.done   MNKMPX
task.add    MNKMQ2   "take out the bins"   2026-08-18
```

One batch, because undo has to take back both — otherwise every undo would
leave a copy and the series would grow on its own.

It costs a row per occurrence, and buys the thing the archive is for: it shows
you did it twelve times. Each occurrence gets its own journal, so «the lorry did
not come this week» has somewhere to live. The archive folds repetitions of the
same month into one line so the rows do not become noise.

**The parser reads both interface languages**, and each has its own vocabulary in
`tisty-nl/src/vocab.rs`. A cadence is opened by one word — `every`, `each` in
English; `cada` in Spanish — or by two, which only Spanish uses: `todos los`,
`todas las`. It can also be an adverb that is the whole cadence on its own:
`daily`, `weekly`, `monthly`, `yearly`, `annually`, and `diariamente`,
`semanalmente`, `mensualmente`, `anualmente`.

One day per repeat: «Tuesdays and Thursdays» is two tasks, not one.

How the next date is worked out depends on how it was written. Naming a day
fixes it to the calendar — the bin goes out on Tuesday whether or not it went
last week — and naming only an interval counts from the doing, which is what a
habit means. Either way the next one lands past today **and** past the day it
was finished: coming back from a fortnight away does not owe you a fortnight of
bins at once, and finishing today's does not hand you another one for today. A
time of day is kept as asked — finishing the 09:00 pill at 08:04 does not move
it to 08:04 for ever — and months and years count off the calendar even when
said as an interval, or the rent would drift a few days later every month.

Nothing is ever created ahead of time. There is no timer and no scheduler: a
task can only come from you writing one or from finishing a repeat. Skip a day
and there is still exactly one, waiting, overdue — never two.

### Marking a turn late

Because a completion carries no date of its own, a calendar cadence closed days
after it was due leaves the dates in between with nothing on them. Rather than
call them forgotten — «I did not take it» and «I took it and did not open the
laptop» leave the same trace — the window offers them back: `owed_since` returns
the dates the cadence would have touched, and `covering` turns each one you claim
into **a turn that was already closed**, chained by `after` and bare of steps,
description and reminders. It writes only `task.add` and `task.done`, so nothing
about the format changes and a machine that has not updated reads the result.

Two caps keep it honest: at most five turns, and nothing whose turn came due more
than thirty days ago. Five turns of a weekly cadence would reach back five weeks,
which is reconstruction, not memory. A cadence counted from the doing never gets
asked, because it leaves no gaps by definition.

**`covering` checks the claimed dates against `owed_since` itself**, rather than
trusting whoever called it. A date the cadence never had would write a turn that
never existed, and a mistyped year would drag the live turn years out and take
the routine with it — so the window and the command line cannot widen it between
them, and neither can a future caller.

## Reading the archive

Three views are **derived at read time and never stored**, so nothing about them
can go stale or disagree with the log:

- `story.rs` replays one task's events into typed chapters, carrying a running
  state so a moved deadline reads «from the 12th to the 19th» rather than twice.
  Reordering, hiding and reminders deliberately produce no chapter.
- `series.rs` walks the `after` chain **in both directions** from the turn asked
  about — walking down from the root loses it when two machines closed the same
  turn before syncing — and counts what was owed as turns that came due plus the
  dates the cadence skipped, which is why the tally is 26/30 and not 26/26.
- `shape.rs` buckets closings into months for the strip on the archive cover.

A task's `Reading` — story, routine or trace — comes from the substance it holds,
not from how long it lived: a task that closed in an hour with a journal entry is
a story, and one that closed after a month with nothing written is a trace.

## The read cache

SQLite in the cache directory, holding the projected state: tasks, lists and
tombstones.

Freshness is decided by a fingerprint — the name and size of every log file. If
it matches, the state is loaded from the cache. If it does not, the log is
replayed and the cache rewritten.

After a write, the cache is **updated**, not dropped: only the entity the event
touched is rewritten. An event that reaches further than its own entity —
erasing a list returns every task it held to the inbox — gives up the fast path
and invalidates instead.

Anything that goes wrong opening or reading the cache falls back to the log.
The cache can be deleted at any time.

### Checking it

```sh
tisty doctor            # replay the log and compare it against the cache
tisty doctor --repair   # discard the cache; the next read rebuilds it
```

The cache is stale, absent, in agreement, or **wrong**. Only the last one exits
non-zero. `doctor` reports and never repairs on its own, because the log wins
every disagreement and rebuilding is the only repair there is.

## Syncing

Through a folder both machines can reach. Nothing else.

```sh
tisty config set remote <folder>   # where the copies go
tisty sync                         # leave ours, take everyone else's
tisty sync --push                  # leave only
tisty sync --pull                  # take only
tisty sync --join <backup.zip>     # back this machine up, empty it, take the folder's
tisty sync --take-over <backup.zip> # back the folder up, empty it, leave ours
tisty sync --merge <backup.zip>    # back up, then hold both histories
tisty sync --again                 # send everything of ours, skipping nothing
```

Tisty always works in its own local directory. Syncing **leaves a copy** in that
folder and **brings home the copies others left**. Whatever keeps the folder
alive — the Google Drive, OneDrive or iCloud client you already run, a mounted
NAS, an external drive you plug in once a week — is not Tisty's business.

That folder is **not** the data directory, and pointing a cloud client at
`AppData` is still the wrong thing to do. The store stays on your disk; only
copies travel.

**Only machines on the list write there.** Being on it is what gives a machine a
voice; one that was removed keeps its own copy and never pushes again. You join
by adopting, not by asking — reaching those files is the authorisation — so
**removing is the only privileged act**. A machine that comes back does not
merge: it is backed up and emptied first, which is what `--join` does.

The folder is also **someone else's writing**, and is treated that way: nothing
is written through a symbolic link — not into the shared folder itself, and not
into any device or shelf directory inside it — an attachment must hold the bytes
its name vouches for, and one that was retired is not carried back in, and a
document body past the reader's ceiling is refused rather than carried in to
replace one that could be opened.

### Why there is nothing to merge

Each device directory has **exactly one writer**. Push only your own — nobody
else touches it, so your copy is authoritative. Pull only the others — you never
write them, so theirs is. The question "which one is newer?" never comes up, and
two machines cannot produce a conflicting file.

What arrives is read before it is written. A `000002` without its `000001`, a
half-downloaded segment, a conflict copy a cloud client left behind — all are
refused at the door, because reading a broken one takes down the whole store,
every device included. **That refusal is that machine's alone**: one unreadable
device directory in the folder is left out and named in the result, and
everything else — your own writing above all — still goes through.

**Files already identical are skipped**, so syncing twice over moves nothing the
second time. Identical means the same length and the same last 512 bytes, not
the same timestamp: a segment only ever grows at the end, so a tail that matches
is a file that matches. The date is deliberately no part of that answer: a date
can be equal while the content is not, and the other way round.

The blind spot is stated rather than hidden: two files of the same length that
differ **before** their tail read as identical. Reaching that from an append-only
log would take corruption, not use, and reading further would cost a full
hydration on a projected drive — which is what a cloud folder is.

**Nothing that matters is decided by a file's date.** A copy carries the date it
was made, not the date of what it came from, so a file always answers «when did
this appear here» — which is what a local cache needs to know. The other
question, «when did that machine last write», is answered from the log: the
maintenance panel takes the highest `ts` per device. Asking a copied file that
made a machine look busy the moment you pulled its history, which silenced the
warning about machines that have fallen behind.

`tisty sync --again` is the way out when something must move regardless. It sends
everything of this machine's without asking whether it is already there. It is
for a folder that lost a file, or a cloud client that missed one — not part of
the normal round.

Attachments travel too. They are named after their own sha-256, so a name that
matches is a file that matches and two machines cannot disagree about one — and
on the way in the bytes are checked against that name.

Document bodies travel by **three prints and no clock**: the local one, the
folder's, and the last this machine carried. If one side moved, it is copied
without asking. A clock would be worse than useless — a laptop waking up is an
hour out, and that has already cost us a real bug.

If both moved, the two versions are **merged block by block** before anyone is
asked. The unit is the block — text between blank lines — which buys atomicity
for free: a table and an ordered list carry no blank line inside, so each is one
whole block and cannot be spliced half from each side. Fenced code keeps its
blanks, because there a blank line is content. Only overlapping edits are a
question; two adjacent ones are simply both taken.

The engine refuses rather than guess, and every refusal lands on the same tested
road: the merge returns nothing, the document is left undecided, and **the
person decides**, with «keep both» offered first because it is the only answer
that loses nothing. It refuses when the two sides rewrote the same block
differently, when the comparison would cost more than four million cells, when
the result would hold a block more times than either side has it or fewer than
both kept, when the woven text would not split back into the very blocks it was
made of, and when the weave would place two lists next to each other — Markdown
reads those as one list, and the seam is only refused when the merge is what
created it.

A document with YAML front matter is not merged at all: the editor cannot write
it back unchanged, so merging it would only churn. It goes straight to the
question.

Line endings are normalised on the way out. That is deliberate: if each machine
kept its own, their prints would never agree and the document would sit in
conflict for ever.

### Two histories are joined only when you say so

A store carrying a different name is **refused before anything moves**. A
`.store-id` marker guards it — a machine that has never met the folder
**adopts** its name, and an empty folder is **given** one.

When both sides hold history there is no safe guess: your own second machine and
a stranger's folder are the same gesture. So the refusal is not the end of the
road, it is a question with four answers. All of them write a backup first, and
none can be undone from the app.

**Merge them.** The store ends up holding both. This works for the same reason
syncing works — merging is concatenating — and nothing collides: entities are
ULIDs, documents are named `<device>-NNNN.md`, attachments are named after their
own contents. What it costs is said plainly beforehand: two lists by the same
name stay two lists, because joining them by name is a guess, and a wrong guess
there goes unnoticed; and ordering keys were minted independently, so lists
interleave.

**Keep this machine.** The folder is backed up, emptied, and repopulated from
here. The other machine will be refused next time and will face the same
question — that consequence is stated up front, because it is not obvious.

**Take what the folder has.** This machine is backed up, emptied, and adopts the
folder. It mints a new device id, so it returns as a new participant rather than
dragging its own removal behind it. This is what a removed machine coming back
does, and what `--join` has always done.

**Adopt without loss** — offered when the folder already holds this machine's
own history. It is not a fourth stance, it is a different fact: a machine left
behind before a merge finds its history inside the folder's, and has nothing to
decide. Without it, such a machine would be refused by every other door and
cornered.

Which case applies is read from the segments, not guessed. A device directory
present on both sides is compared as the **ordered concatenation of its
segments** — one writer, append-only, so one side must be a prefix of the other.
It is compared whole rather than file by file because rotation renames what it
seals: the same history can be one file here and two there. A file of zero bytes
— what a cloud client leaves before it fills one in — proves nothing either way.
A file the reader **cannot open** is not the same thing and is not treated as
such: there the answer is that it cannot be told yet, and nothing is offered
until it can, because the two answers lead opposite ways. Where there is no
evidence the answer turns on whether a device name appears on both sides: if
none does, "strangers", since a needless seam is harmless bookkeeping; if one
does, it is refused, because an unprovable shared name is the fatal case.
«Names» here covers directories **and the ids events name**, so an id surviving
only inside someone else's `device.remove` still counts.

When the same device name exists on both sides having written **different**
things, nothing is merged. Two writers under one name is the one thing the whole
design rests on not happening.

**When two histories are merged, the folder's name is the one that survives.**
Not for comfort: `.store-id` is the only file in the shared folder without a
single writer. Minting a new name — or imposing the local one — rewrites it, and
two machines merging at once would write two different contents into one file,
which is the exact class of conflict everything else here is arranged to make
impossible. Adopting the folder's name makes that file effectively immutable:
concurrent merges write the same bytes.

A merge writes a `stores.joined` event **before** it takes the folder's name,
carrying both names and which devices came from which side. That order is not
taste: taking the name first and dying would leave a store whose name already
matches, so the question is never asked again and the trail is lost. Writing the
event first means a death leaves the names still apart, the question comes back,
and doing it twice only records an ancestor that was already recorded.

The event touches no task and no document. Like `device.join`, it projects
something other than data: a set of ancestor store names, which only grows.

**That set is written but not yet read.** Lineage is answered from the segments
alone today, so this is a statement about what the record holds, not about how
anything behaves. It is written now because it can only be written now — the
moment has to be caught as it happens — and it is what will let a machine
arriving long afterwards recognise its own store name among the ancestors, and
be told what happened instead of merely being asked to choose.

Directory names are compared without case: on Windows and macOS `DEV_A` and
`dev_a` are one directory, so a stranger's copy would land on the only original
this machine has.

Syncing runs on its own — pull when the window opens and when it regains focus,
push shortly after each change, and both on a timer. It never blocks a local
write and never interrupts to complain: an unreachable folder is retried in
silence and reported in the maintenance panel.

## What a document is named, and what happens when it goes

A document file is `<device>-NNNN.md`. The device prefix is what lets two
machines create documents at the same time without agreeing on anything, so only
the owning machine ever mints its own numbers.

**A number is never handed out twice.** Taking the highest number on disk and
adding one is not enough: deleting the last document would free its name, and a
`tisty:doc/…` reference left pointing at it would quietly resolve to whatever
took the name next. A local high-water mark, which only ever rises, is what
prevents that. It is local because only this machine mints these names.

**A body is refused above the reader's ceiling.** Writing had no limit while
reading, exporting and printing all stopped at 500 KB, so pasting enough text
produced a document that could no longer be opened, exported or carried — with
no warning until it was too late. The refusal now happens where the writing
does.

**A deletion is carried out, not inferred.** Deleting a document names its file
in the log, and every machine that reads that event removes its own copy — the
same treatment a retired attachment gets. What is left over from before this
existed, or from a copy that stopped halfway, is not deleted on a guess: `tisty
doctor` and the maintenance panel **count** the document files on disk that the
log does not know about, and leave them where they are.

## Taking a document out

A document is a Markdown file, and the whole point is that it survives without
us. Three ways out, and the differences are not cosmetic.

**Copy as Markdown** hands the text to the clipboard exactly as it is stored,
references included. Fast, and enough for prose. But an attachment reference
reads `attachments/<shelf>/<file>`, and those bytes live in the store: paste the
text into a page or a ticket and the images are not there.

**Export as Markdown** writes a folder — the document beside an `attachments/`
holding only what that document names. **No reference is rewritten**, and that
is the point: inside the store a document sits in `docs/`, one level below the
attachments it names, so the relative path only resolves because we resolve it
ourselves from the data root. Put the document *beside* its attachments and the
very same path resolves the way every other reader would expect.

That is why the export does not need a second reference format, and why the
store does not need migrating. The layout does the work.

What still does not survive the trip is a reference to **another document**
(`tisty:doc/…`), which means nothing outside Tisty. It stays as written, as a
piece of text rather than a broken file path.

**Export to PDF** is the one that leaves Markdown behind, and Tisty composes it
rather than asking the system to print. That is a deliberate cost. Printing hands
the page to the operating system, and the operating system decides: on macOS the
paper size comes from `NSPrintInfo` and not from any CSS we write, so a document
asked for in A4 came back rescaled to whatever the print dialog had selected.
Composing it ourselves is the only way the app can promise its own paper and its
own margins.

The page is built from the editor's own tree, not from the stored Markdown, so
what leaves is what you were looking at — headings, lists, tasks with their
boxes, quotes, code, tables with real cells, and the first line read as the
title. Three sizes: A4, Letter, and one endless sheet that grows with the
document and stops short of the height a reader would refuse to open.

Attachments are the part that needs care. The webview may *show* a local file
but the composer may not *fetch* one — the policy that keeps the app from
reaching the network keeps it from reaching the disk too — so their bytes travel
through a command and are embedded in the PDF itself. A file that cannot be read
leaves its name in a dashed box rather than a hole, and a picture used five
times is read once.

## Backing up by hand

One zip of `store/`, `docs/`, `originals/` and `attachments/`, never the
configuration — a shared `device_id` would put two machines in one file.

Restoring is **a photograph**: back to that moment, and what came after is lost
on purpose. The machine **takes a new device id** so its directory starts empty
and can never shrink what other machines already hold.

Nothing of yours is touched until the whole backup has been unpacked beside it
and read back, and the swap moves every old folder aside before a single new one
steps in. A zip that turns out to be corrupt, truncated, somebody else's, or not
a backup at all costs you nothing — and half a restore is the one outcome worth
less than either whole.

**Backing up and syncing are mutually exclusive**, and the buttons disable each
other. The shared folder already holds every machine's history, so a second
snapshot beside it would be a rival truth. Restoring is a local decision with
global consequences, and the other machines never hear about it.

The honest limit: with syncing you get **redundancy, not a way back in time**.
Delete a task and the deletion travels. Going back for everyone would have to be
an event of its own — a `store.rewind` the projection honours — which is written
down as an idea and not built.

## Where things live

| | Location | Synced |
|---|---|---|
| Events | `<data>/store/<device>/` | yes |
| Attachments | `<data>/attachments/` | yes |
| Documents | `<data>/docs/` | yes, by three prints and no clock |
| Attachment ledger | `<data>/attachments.jsonl` | **no** — local and rebuilt on demand |
| Carried prints | `<data>/carried.json` | **no** — what this machine last carried |
| Merge bases | `<data>/carried/` | **no** — the body each print stands for |
| Highest name given out | `<data>/docs/.spent-<device>` | **no** — so a name is never reused |
| Before a conversion | `<data>/originals/` | **no**, but it is in a backup |
| Retired attachments | `<data>/bin/` | **no** — thirty days of grace |
| Settings and device id | `<config>/config.toml` | **no** |
| The program itself | `%LOCALAPPDATA%\Programs\Tisty` and friends | **no** |
| Read cache | `<cache>/read.db` | **no** |
| Last listing | `<cache>/selection.json` | **no** |

The **guide** is the one thing Tisty writes into your store on its own. It ships
inside the program, in the language you chose, and the welcome copies it in: a
document under `<data>/docs/` and its images under
`<data>/attachments/`, indistinguishable afterwards from anything you wrote and
deletable the same way. Nothing is downloaded: the words and the images travel
inside the program.

`<data>`, `<config>` and `<cache>` are the platform's own directories.
`TISTY_DATA`, `TISTY_CONFIG` and `TISTY_CACHE` override them, and exist for
tests.

The **configuration** never syncs, and that is what matters: if two machines
shared a device id they would write to the same file and every guarantee above
would stop holding. It is also why the configuration stays out of a backup.

The device id itself does travel, and has to — it is the name of the directory
and the `by` field of every event, which is what tells the writers apart. What
must never travel is the file that says *«this machine is that id»*. That file
lives in the local config directory, never a roaming one: a Windows domain
profile copies `%APPDATA%` to a company server at logoff, and it would take the
device id and the `private/` folder with it.
