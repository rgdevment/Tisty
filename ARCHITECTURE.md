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

### Reading

Every device reads every directory, its own included. Events from all of them
are concatenated and sorted by `(ts, by, n)` — the same order on every machine,
so every machine reaches the same state.

Reading refuses to continue, rather than returning a smaller history, when:

- a sealed segment is missing from the sequence,
- a sealed segment is present but empty,
- any line fails to parse,
- an event declares a schema version this build does not know.

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

## Syncing

Over Git, and only over Git.

```sh
tisty sync --setup <url>   # create the repository, write .gitattributes, point at the remote
tisty sync                 # pull --rebase · commit · push
tisty sync --status        # where it sends, and what is pending
```

`tisty` invokes the `git` binary; it is not linked as a library. The pull runs
first and its failures do not stop the local commit — syncing never blocks
writing.

Because each device writes only its own files, two devices never touch the same
file, and the rebase always applies. `.gitattributes` marks `*.tisty` as `-text`
so line endings are never rewritten.

The store is a normal Git repository. Anyone who prefers to pull and push by
hand loses nothing.

## Where things live

| | Location | Synced |
|---|---|---|
| Events | `<data>/store/<device>/` | yes |
| Documents | `<data>/docs/` | yes |
| Attachments | `<data>/attachments/sha256/` | yes |
| Settings and device id | `<config>/config.toml` | **no** |
| Read cache | `<cache>/read.db` | **no** |
| Last listing | `<cache>/selection.json` | **no** |

`<data>`, `<config>` and `<cache>` are the platform's own directories.
`TISTY_DATA`, `TISTY_CONFIG` and `TISTY_CACHE` override them, and exist for
tests.

The device id never syncs. If it did, two machines would share it, write to the
same file, and every guarantee above would stop holding.
