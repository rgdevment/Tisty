# Architecture

How Tisty stores, merges and reads its data. This is a reference for how the
system behaves, not a record of why it was designed this way.

## The two layers

```
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

Neither lives in the documents folder, and the path is not configurable.

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
| `op`, `id`, `d` | the operation, the entity it affects, its payload |

`active.tisty` is sealed as `NNNNNN.tisty` every 5.000 events. Sealed segments
are numbered from one without gaps.

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
- any line fails to parse,
- an event declares a schema version this build does not know.

Syncing holds the same line from the other end: a device's history that arrives
**shorter than the one already held** is left where it is. Contiguity alone does
not catch that — the gap is not *between* sealed segments but *before* `active`,
and nothing in the folder says how many sealed ones there should be. So the
counts are compared instead. A cloud client is free to deliver a rotated `active`
before the sealed segment that carries what it dropped; that ordering must not
cost anyone their copy.

## How merging works

Nothing is ever edited. Facts are recorded about an entity, and the entity is
identified by a ULID that means the same thing everywhere.

```
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

## Repeating

A repeat is **one task per occurrence**, not one entity collecting completions.
Finishing «take out the bins every Tuesday» writes two events in one batch: the
completion, and next Tuesday's task.

```
task.done   MNKMPX
task.add    MNKMQ2   "take out the bins"   2026-08-18
```

One batch, because undo has to take back both — otherwise every undo would
leave a copy and the series would grow on its own.

It costs a row per occurrence, and buys the thing the archive is for: it shows
you did it twelve times. Each occurrence gets its own journal, so «the lorry did
not come this week» has somewhere to live. The archive folds repetitions of the
same month into one line so the rows do not become noise.

A cadence is opened by a word (`every`, `each`), by two (`todos los`), or by an
adverb that is the whole cadence (`daily`, `weekly`, `annually`). One day per
repeat: «Tuesdays and Thursdays» is two tasks, not one.

How the next date is worked out depends on how it was written. Naming a day
fixes it to the calendar — the bin goes out on Tuesday whether or not it went
last week — and naming only an interval counts from the doing, which is what a
habit means. Either way the next one lands past today **and** past the day it
was finished: a fortnight away does not come back owing a fortnight of bins, and
finishing today's does not hand you another one for today. A time of day is kept
as asked — taking the pills at 08:04 does not move them to 08:04 for ever — and
months and years count off the calendar even when said as an interval, or the
rent would walk down the month.

Nothing is ever created ahead of time. There is no timer and no scheduler: a
task can only come from you writing one or from finishing a repeat. Skip a day
and there is still exactly one, waiting, overdue — never two.

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
**removing is the only privileged act**. A machine that comes back does not merge:
it is backed up and emptied first, which is what `--join` does.

The folder is also **someone else's writing**, and is treated that way: nothing is
written through a symbolic link — not into the shared folder itself, and not into
any device or shelf directory inside it — an attachment must hold the bytes its
name vouches for, and one that was retired is not carried back in, and a document body past the reader's ceiling is refused rather than
carried in to replace one that could be opened.

### Why there is nothing to merge

Each device directory has **exactly one writer**. Push only your own — nobody
else touches it, so your copy is authoritative. Pull only the others — you never
write them, so theirs is. The question "which one is newer?" never comes up, and
two machines cannot produce a conflicting file.

What arrives is read before it is written. A `000002` without its `000001`, a
half-downloaded segment, a conflict copy a cloud client left behind — all are
refused at the door, because reading a broken one takes down the whole store,
every device included. Files already identical are skipped, so syncing twice
over moves nothing the second time.

Attachments travel too. They are named after their own sha-256, so a name that
matches is a file that matches and two machines cannot disagree about one — and
on the way in the bytes are checked against that name.

Document bodies travel by **three prints and no clock**: the local one, the
folder's, and the last this machine carried. If one side moved, it is copied
without asking; if both moved, **the person decides**, and «keep both» is offered
first because it is the only answer that loses nothing. A clock would be worse
than useless — a laptop waking up is an hour out, and that has already cost us a
real bug.

### Two histories are joined only when you say so

A store carrying a different name is **refused before anything moves**. A
`.store-id` marker guards it — a machine that has never met the folder **adopts**
its name, and an empty folder is **given** one.

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

**Adopt without loss** — offered when the folder already holds this machine's own
history. It is not a fourth mood, it is a different fact: a machine left behind
before a merge finds its history inside the folder's, and has nothing to decide.
Without it, such a machine would be refused by every other door and cornered.

Which case applies is read from the segments, not guessed. A device directory
present on both sides is compared as the **ordered concatenation of its
segments** — one writer, append-only, so one side must be a prefix of the other.
It is compared whole rather than file by file because rotation renames what it
seals: the same history can be one file here and two there. A file the reader
cannot open, or one of zero bytes — what a cloud client leaves before it fills
one in — proves nothing either way, and where there is no evidence the answer is
"strangers": a needless seam is harmless bookkeeping, a missed clash is not.

When the same device name exists on both sides having written **different**
things, nothing is merged. Two writers under one name is the one thing the whole
design rests on not happening.

**The folder's name is the one that survives.** Not for comfort: `.store-id` is
the only file in the shared folder without a single writer. Minting a new name —
or imposing the local one — rewrites it, and two machines merging at once would
write two different contents into one file, which is the exact class of conflict
everything else here is arranged to make impossible. Adopting the folder's name
makes that file effectively immutable: concurrent merges write the same bytes.

A merge leaves a `stores.joined` event behind, carrying both names and which
devices came from which side. It touches no task and no document; like
`device.join`, it is an event that projects something other than data — the set
of ancestor store names, which accumulates, so a machine that arrives much later
can still recognise its own name as one of them.

Directory names are compared without case: on Windows and macOS `DEV_A` and
`dev_a` are one directory, so a stranger's copy would land on the only original
this machine has.

Syncing runs on its own — pull when the window opens and when it regains focus,
push shortly after each change, and both on a timer. It never blocks a local
write and never interrupts to complain: an unreachable folder is retried in
silence and reported in the maintenance panel.

## Taking a document out

A document is a Markdown file, and the whole point is that it survives without
us. Two ways out, and the difference is not cosmetic.

**Copy as Markdown** hands the text to the clipboard exactly as it is stored,
references included. Fast, and enough for prose. But an attachment reference
reads `attachments/<shelf>/<file>`, and those bytes live in the store: paste the
text into a page or a ticket and the images are not there.

**Export as Markdown** writes a folder — the document beside an `attachments/`
holding only what that document names. **No reference is rewritten**, and that is
the point: inside the store a document sits in `docs/`, one level below the
attachments it names, so the relative path only resolves because we resolve it
ourselves from the data root. Put the document *beside* its attachments and the
very same path resolves the way every other reader would expect.

That is why the export does not need a second reference format, and why the store
does not need migrating. The layout does the work.

What still does not survive the trip is a reference to **another document**
(`tisty:doc/…`), which means nothing outside Tisty. It stays as written, as a
piece of text rather than a broken file path.

## Backing up by hand

One zip of `store/` and `attachments/`, never the configuration — a shared
`device_id` would put two machines in one file.

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
| Before a conversion | `<data>/originals/` | **no**, but it is in a backup |
| Retired attachments | `<data>/bin/` | **no** — thirty days of grace |
| Settings and device id | `<config>/config.toml` | **no** |
| The program itself | `%LOCALAPPDATA%\Programs\Tisty` and friends | **no** |
| Read cache | `<cache>/read.db` | **no** |
| Last listing | `<cache>/selection.json` | **no** |

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
