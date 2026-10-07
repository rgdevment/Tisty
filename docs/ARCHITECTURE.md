# Architecture

How Tisty stores, merges and reads its data. This is a reference for how the
system behaves, not a record of why it was designed this way.

## The two layers

```text
<data>/store/                    the truth. Syncs. Survives everything.
├── dev_a3f1/
│   ├── 000001.tisty             closed segment, never written again
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

One JSON object per line. Files only ever grow at the end, and a closed segment
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
| `via` | the client an assistant spoke through — `claude-code`, `codex` — as the MCP session named itself; sealed by `tisty mcp` on every event it writes, read by nothing that decides |
| `opt` | a reader that cannot make sense of this operation skips it instead of refusing the store; absent means refuse |
| `op`, `id`, `d` | the operation, the entity it affects, its payload |

Only mark `opt` on an operation that **adds**. A reader forgives it solely
when the name is one it has never heard of — a known operation that fails to
parse is corruption and stops the read regardless — but nothing can stop a
writer marking something that changes what already exists, and a reader would
then drop it and diverge in silence.

`person.signed` sits at the edge of that rule and stays inside it: skipping it
leaves the alias unshown, and nothing else, because every document carries the
name it was written under in its own `doc.add`. `doc.signed`, which re-signs a
document already written, is **not** marked — dropping it would show the old
name on one machine and the new one on the next.

Schema 16 took the mark off four that had carried it, because each turns out to
decide the fate of a file rather than add something to look at. `attach.kept`
and `attach.let_go` are what the holders of a body are read from, and letting go
of a local copy asks exactly that: a reader that walks past the release believes
another machine still holds the body and lets go of the last one there is.
`doc.said` is what says a body holds nothing, which is the only thing that tells
a document somebody emptied from one the folder truncated — walk past it and the
old body is written back over the emptying, and travels. `device.key` is a
machine's own word for what it signs with, so walking past it is being unable to
check a single signature and falling back to taking a history on trust.

The mark is read only where the operation's **name** is one the reader has never
met: a known operation written above the version it knows is refused whatever the
mark says. So taking the mark off reaches exactly the builds that do not know the
name — every release so far for `attach.kept`, `attach.let_go` and `device.key`,
which no tag contains, and anything older than `doc.said` for that one. A build
that does know the name is stopped by the fence instead, which is why the fence
has to move with the mark.

Some payload fields carry more than their name says:

| Field | On | Meaning |
|---|---|---|
| `k` | `device.join` | `agent` or `machine`. Absent is not a claim of either: an event written before the field existed must not demote an agent |
| `p` | `device.join` | the machine's public signing key, 32 bytes in hex. Only a machine's own word for its own key counts, and the first one published stands: a later join naming another key is refused, not believed. Anything that is not a key is dropped and the join lands without it |
| `d`, `of`, `p` | `device.host` | the machine an agent device is hosted on. The join is written as the agent and said nothing about where; this says it, marked `opt`, and only the agent's own word or its host's counts — `d` an agent, `of` a machine. Written by the host with `p`, the agent's key, it is what lets a machine that confirmed the host take the agent without asking. The window writes it as the machine, so it settles like `doc.said`: `undo` walks past it |
| `d`, `name`, `os` | `device.named` | what the computer calls itself and the system it runs, so a machine waiting to be confirmed is found by a name the person knows. Marked `opt`, and only the machine's own word counts; the window writes it when it takes its seat and the name changed. Its `name` is not `n`, which every event already uses for its sequence |
| `source` | `task.add` | what the task was written from, so the same thing is not filed twice |
| `filled` | `task.done` | closed in bulk by the backfill, so its stamp is the hour of the marking rather than its own |
| `read_as` | `task.update` | `story` or `trace`, the layer the person converted the task to; `null` reads it by what it holds again — what undo writes, and what `tisty set --read-as auto` asks for; the window only moves between the two |
| `open_to_agents` | `task.update` | the person let an assistant fill this task in. Only their own hand sets it: written by an assistant, the field is dropped and the rest of the patch lands |
| `tags` | `doc.said` | the tags read out of the body. Absent is not «none»: it is a build that did not read them, and treating the two alike would have an older machine wipe the tags of every document it saved |
| `by` | `doc.said` | the alias the body was saved under, sealed at the writing rather than worked out afterwards from whoever happens to be signing now. Absent is a hand that did not sign, not the reader's own |
| `print` | `doc.said` | the SHA-256 of the body as written, which is how the log answers for a body: one arriving from the folder is taken in when its print is one a log wrote down |

`active.tisty` is closed as `NNNNNN.tisty` every 5.000 events. Closed segments
are numbered from one without gaps.

A machine that holds a signing key writes a `.sig` beside each of its own
segments — `active.sig`, `NNNNNN.sig`. Inside is a tip and a signature over it:
the tip is a SHA-256 chain folded line by line across this machine's whole
history, and what is signed is the machine, the segment's name and that tip
together, so one signature never answers for another segment, nor for the same
segment on another machine. It is written before the segment it covers is
renamed and carried after it is copied, never the other way round. A `.sig`
that is missing or does not verify is not read as tampering on its own: the
chain is folded again from the last one that does. This is not the seal a
parcel carries: that one is an HMAC over a manifest, under the store's key.

The key itself lives in `<config>/private/`, beside the store's own secret, and
is never carried anywhere. What it owes is decided by the copy in the folder,
not by this machine's log: a history whose events were written at
`SIGNED_FROM` (schema 16) or later owes a signature, and so does one that
carries its machine's own `device.key`, whatever version its lines claim. Such a
history arriving with no signature at all is disowned — none of it comes in —
while one written below the fence, before that machine had a key, never could
have carried one and still comes home. A machine that gets a key late answers
for the past it wrote before: on the first write after the key exists, every
closed segment of its own that nothing answers for gets a `.sig` of its own,
over the chain that segment really closes. A signature already in place is left
exactly as it stands, whether it answers or not, because writing another over
it would bless bytes that history never covered.

What it does not catch, and is not meant to: a segment rolled back **whole** to
an earlier state together with the signature that answered for it then. Both
agree, and nothing outside them says which of the two is the later one. What
answers for that is the closed segment that follows, whose chain runs through
the rolled-back one — so only the active segment, the one nothing follows yet,
is open to it.

A key is a claim until somebody here answers for it. The first folder a machine
reaches is taken up whole — it has nothing confirmed yet, so there is nobody to
ask — and after that a machine appearing in that folder that nobody here
answered for is left where it is: its history does not come in, the round says
so, and the window shows its key to compare and confirm (`tisty sync --confirm
<machine>` does the same from the command line). There is no telling a machine
of yours coming back from a directory somebody planted without a person looking,
so the person looks. What that machine wrote waits in the folder and arrives
whole once its key is answered for. A machine that was in the folder when it was
taken up but still in the cloud is taken up when it comes down: the names a
joining round meant to take up are kept aside in `adopting` until each has
arrived, and written before the round reads anything, so a first round cut short
does not leave half the folder waiting on a person. A machine that appears later
is not on that list. What the person answered for is kept in
`<data>/.keys-confirmed`, apart from the log: the log decides what a machine
claims, and only a person decides what is believed. A machine that never said
what it signs with is not held this way — there is nothing to answer for.

A machine this store already held from before signing existed is the one
exception, taken once and on narrow terms (`store::before`): its first key is
believed without asking when the copy held here has lines from before the fence
and the folder's copy, read whole and in order however its segments were renamed
since, begins with it byte for byte, the key is the first one said after it, and
the whole of it verifies under that key. That is exactly what the folder could
already do to it before signing, for one moment, and then the door closes; a key
planted first leaves the real machine showing as one that changed its key, which
nobody can miss. It is kept with `carried` beside it in `.keys-confirmed`, so
what nobody compared can be told from what somebody did. A signed history that
has not yet said what it signs with waits like any other, except in the first
folder a machine takes up whole. The memo that says a machine has been to a
folder before names that machine, so a reinstall that kept the cache meets the
folder as the new machine it is.

An agent is the other exception. It is minted on the computer it runs on, which
holds its key, so at that moment the computer also writes `device.host` in its
**own** signed history with `p`, the agent's key (`agent::register`); an agent
minted before this is spoken for once, the next time the window opens, and
`agent_vouched` in the local configuration keeps it from being said or looked
for again. A machine that already answered for that computer takes the agent on
its word (`store::Ledger::vouched`, `hosted` in the round): the line must be the
host's own, the host a machine and neither an agent nor removed, the key the one
the agent says, and the agent's history must join as an agent under that very
key, which the round then verifies before anything comes in — a whole machine is
never seated this way, and an agent's word about itself or another counts for
nothing. The first such line stands. It is kept with `host:` and the host's name
in `.keys-confirmed`. What this trusts is what the person already trusted:
whoever holds that computer's key could mint an agent there anyway. An agent
whose host's line arrives in the same round can wait that one round. Without it
every agent waited on every machine, and meanwhile the bodies it wrote reached
the folder ahead of any line a trusted history held for them, so the person was
asked to settle documents that were only waiting.

### What a power cut leaves behind

A line is written whole or not at all, so one that will not parse at the very
end of the segment still being written is the half of an event a cut took. It is
set aside as `active.torn` rather than read: refusing it would take every whole
event before it down as well. Only ever the last line, only ever this machine's
own active segment — a closed one has its count to answer for, and another
machine's history is not ours to mend.

Three things it must get right, and each of them was once wrong:

- The file is read as **bytes**. Half of an accented letter is not text, and a
  reader that gives up there leaves the very store this exists to save unopenable.
- A tail that parses is a whole event whose **newline** the cut took, and it is
  given the newline back. Left without it, the next append lands on the same
  line and takes both events down for good — and mending never sees it again,
  because the file now ends properly.
- Mending happens **behind the lock** every writer takes. It rewrites the
  segment by renaming a fresh file over the old one, and a rename under a live
  writer drops whatever they appended to the inode that just went away.

Damage anywhere but the tail is a different accident. It is not mended and not
guessed at: the read fails and says so, which is better than handing back half a
history as though it were whole.

### An assistant is a second writer, never a second decider

`tisty mcp` speaks JSON-RPC over stdin and stdout so an assistant on this
machine can file work. It is a subcommand of the same binary, so it resolves
the same paths, takes the same lock, and duplicates no logic.

An agent writes under a `device_id` of its own, minted only when the person
turns one on — from the Agents tab or `tisty agent --on`, which asks them on
the terminal first. Nothing arriving over the wire can register one. Its own
directory is what keeps `undo` apart: the person's undo never reaches what the
agent filed.

It can propose a task, move the day of a task it filed itself, say that a task
it filed is done, add to a journal, read one whole task or the fields of it that
it names, search, attach a file to a task or into a document, write documents,
add to them and change a passage of one, hang one off another as a page and
write the line that names it where that page belongs, list what is written and
file it into folders, and read the names of the lists. There is no tool for completing,
dropping, deleting, undoing, editing a task the person wrote, or making a list.
A new body whole it can hand over, but only against the print it read.

`restore_doc` is the one thing that goes backwards, and it reaches documents
only. Changing a passage and replacing a body already keep what they replaced
beside the documents, so putting that body back costs nothing and needs nothing
kept that was not kept anyway. What it replaces is kept in turn, so calling it
twice lands where it started.

Not every write keeps one, though — adding to the end does not, and neither
does the person saving from the window — so the body kept beside a document is
not always one step back. The one window write that does keep one is putting a
document back, which is what makes that reversible in turn. What is kept
alongside it is the print the document read at when that body was set aside,
and going back is refused unless the
document still reads exactly that way. Without it the tool would have been at
its most destructive in the case it exists for: an agent that wrote, watched
the person work all afternoon, and then decided to undo itself.

`even_if_more` goes back anyway, and exists because the refusal would otherwise
be a dead end for the one who has read the document and knows the writing since
is its own. It is not a way out of the guarantee so much as a way of saying it
out loud: the body it writes over is kept in turn, so the call that went too far
is undone by the same call without the flag.

`reschedule` writes over something already filed, and reaches only tasks whose
`created_by` is a device the person turned on as an agent: a day the person set
is refused with the reason, so is a task they wrote, and so is one an agent has
already said is done. The two days are checked against each other as well: a
deadline that falls before the day the work is planned for, while neither has
gone by, is a pair nobody meant, so the call is refused with both days named
rather than written. A deadline already past is a different thing and is taken as
it is, and the same check runs on `propose`, on one task and on every task of a
batch alike. Everything else an
agent knows it must add rather than change: a journal note, a new task, a new
document.

**Who wrote it is the client, not the device.** One agent device per machine
is the door; the name a person reads is the hand that came through it. The
MCP session introduces itself (`clientInfo` in the greeting, or the same under
`_meta` in the newer revision), the server keeps the name for the session —
one process serves one client — and seals it as `via` on every event it
writes; a client that never introduced itself is named by nothing. What it
called itself is kept as said; naming it for people is one table in the core
(`agent::client_id`, `agent::client_named`): `codex-mcp-client` is the
`codex` of the person's settings and reads «Codex». The window says «by
Claude Code», «by Codex», and «by an assistant» for what was written before
clients were named; the tree-and-number nickname of a device names a device,
never a hand — Sync says it of machines, and Settings says it once of this
machine's agent, as the signature its events carry. `via` is a label and
nothing more: no rule reads it, so a forged or missing one can only
mislabel, never open a door. Where an agent lives is said once by
`device.host`, and the detail says «from this machine» or names the other.

Filling a task in is the one thing an agent does on a task it did not file, and
only where the person let it. `say_done`, `say_not_doing`, `describe`, `plan`,
`tick`, `untick`, `reword_step`, `unplan` and `reword_note` reach a task an
assistant filed, or one the person opened
with `open_to_agents` — a verb of the window, on an open task of their own,
never part of an edit and never the terminal's. Opened, the task stays theirs:
its day, its title, its list and its closing are as out of reach as before,
`say_not_doing` leaves the same mark as `say_done` saying the task should be
dropped rather than closed — confirming it drops, and nothing is dropped until
the person does — `tick` marks a step, `untick` takes back only a tick an
assistant gave — the step remembers whose hand ticked it, and a tick the person
gave is theirs — `describe` writes the description anew, and `plan` adds steps
under whatever is there. Mending reaches what is still open: `reword_step` and
`unplan` rewrite or take off a step that is not ticked, whoever wrote it, and
`reword_note` rewrites a note but never empties one, since an empty note is a
note taken out. `rename` is the one mend that needs the task filed by an
assistant: a title the person wrote stays theirs, opened or not. A part has its
own door — opening the whole opens none of its parts. The core judges it again
at replay — `TaskResolve`, `TaskDescribe`, `StepAdd`, `StepDone` and
`TaskLogEdit` from an assistant on a task nobody opened to them are let go, and
so is a `StepUndone` on a step whose tick was not an assistant's, a `StepText` or
`StepRemove` on a ticked step, and a `TaskLogEdit` that empties a note, whatever
the server that wrote them believed — so a fill-in written on one machine before
the person shut the door on another projects the same everywhere. Shutting it
keeps what was filled in; it is not an unsaying.

A task the person closed is history to an agent. It comes back from `read`
and `find` with `closed` set to the moment it ended and, from `read`, a
`notice` that says so in words, and every tool that writes on a task —
`note`, `reschedule`, `say_done`, `remind`, `attach` — is refused on it with
the same answer: it reads as it ended, nothing on it changes, and work that
came back is a new task whose description says how the last one ended. A
closed task that left nothing written — the person's trace, in the archive's
own three layers — is served the same way, with the same notice: hiding it
was considered and turned down, because an agent that cannot see a thing
happened proposes it again, and a notice costs less than a duplicate. The
source says it too: `find` by `source` and a second `propose` from one answer
«closed since» with the moment, which is what an agent tells the person who
asks whether it filed something — it did, and it is done. A second filing from
that source takes `again`, for the one case where the person wants the work
done once more, and never while the earlier task is still open.

`say_done` is the narrower of the fill-ins, and deliberately so: it adds a
mark beside the task and the account that holds it up, and changes nothing
else. It is refused while a step of the task is unticked, because a mark beside
an unticked checklist reads as work nobody did: the agent ticks what it did,
and a step that no longer applies goes in a note for the person to take off.
The task stays open. Whether it closes is the person's, who may take the mark off
instead. A second `say_done` on a mark nobody has looked at is refused rather
than stacked, and so is everything else that would speak over it: no bell, no new
day, no step added under a mark that says the work is over. What the agent has
learnt since goes in a `note`, which is the one thing the mark leaves open.

### A door an agent can afford to walk through

A tool that hands back more than was asked is a tool that is used once and then
worked around. Three rules keep the door cheap.

*Read the shape before the body.* `outline_doc` answers with the headings, the
lines each one spans and what it costs, the length, the print, and for a book a
row per page — a few hundred tokens for a document that costs tens of thousands
whole. `read_doc` then takes a `section`,
a run of lines, or a budget of `chars` with a cursor; a body past 12 000
characters comes back as its outline anyway, with `whole` set to false, rather
than filling the window with text nobody asked for. A run of lines is held to
that same budget and says where to carry on, because naming one wide enough to
cover the document was the way round it, and the way round was the one anybody
reaching for the whole text would find first. Carrying on from a cursor
needs the print the previous part came with, so two halves of a document that
moved in between are refused rather than joined.

*Write without reading.* `edit_doc` names its passage either by what it says or
by where it sits — a section number, a line range — and `append_doc` takes
`under`, a heading to add beneath. With an outline and a print, one part of a
long document is changed without the body ever crossing the wire. What
`edit_doc` names by place is guarded by the print, because there is no text to
recognise it by.

*Answer with what changed, not with what was sent.* No writing tool echoes the
body back unasked: the answer is the title, the length, how much the document
`grew` by, and the new print. The size is there to be read — an edit that
changes it by far more than what was sent did not do what was asked — and
`echo` adds the few lines around the change, which is cheaper than reading the
document again to see it landed. A refusal for a stale print brings the
outline, not the document.

*Cut only what can be cut.* `edit_doc` naming a place, and `append_doc` naming a
heading, both work by keeping the head and the tail of the document around what
they write. The two are cut from the same text, so together they can never be
longer than it was; when they are, the document is not written. That invariant
is cheap and it is checked, because the day it did not hold the answer was a
document with a second copy of itself pasted behind the edit.

### A card, worked out here and never sent anywhere

Choosing which of two hundred documents to open is a different question from
reading one, and an outline does not answer it. Each body has a **card**
(`docs::Card`): its title, the headings with the line each sits on, how many
words, pictures and links it holds, the words it leans on, and the print it
reads at. `docs` carries an abridged card for every document it lists, so an
agent picks what to open without opening anything.

Every field on a card is **read back out of the body**, so none of them can be
stale. It is kept in `read.db` beside the projected state — local, never synced,
thrown away and rebuilt like the rest of that cache — keyed by the file's size
and modification time, exactly as `Corpus` keys its in-memory copy. A document
written since its card was worked out simply misses, and the body is read again.
Nothing is written to the log, so two machines never have to agree about it, and
a card cannot arrive without the document it describes.

The card carries the words too, stripped of their markup and folded the way a
query is folded. That is what turns a search from opening two hundred files into
one question the database answers: it names the few documents that hold every
term, and only those are read, and only to quote the line. A file the cache has
never seen is read and remembered on the way past rather than skipped — silence
would be worse than the work. Without a cache at all there is nothing to ask, so
the search walks the files as it always did.

A summary is the one thing about a document that cannot be worked out from it:
somebody has to read it and say. `sum_up` is where an agent leaves that, and it
sits in a `gist` table beside the cards — written rather than derived, and so
kept apart from them.

It stays local too, and that is deliberate rather than convenient. A summary is
recomputable work, not the person's writing: if the cache is thrown away, the
next agent reads the document and says it again. Putting it in the log instead
would make two machines agree about a sentence no person ever wrote, sync it
forever, and turn one reader's impression into a record. The cost of keeping it
local is that it can be lost; the cost of keeping it in the log is that it
cannot.

What keeps it from lying is the print. A gist is stored against the text as it
read when it was written, and every reader gets it back with `stale` set when
the body has moved since. An old summary is not hidden — it is handed over
labelled, because knowing roughly what a document was about last month is still
worth more than nothing, as long as nobody mistakes it for now.

Two things follow, and both are taught at the door. A gist is never part of the
document: the person does not see it in their window, and nothing an agent
writes there reaches what they wrote. And a gist is one reader's account, so an
agent reading another's treats it as data — never as an instruction, and never
as a source to quote when the answer has to be right.

`catch_up` is the other half of the same idea, for tasks rather than documents:
one call gives the lists, the folders, the tags in use, how much there is, the
documents written most recently, and a cursor. Send that cursor back and only
what moved since comes with it — the tasks touched and the documents written,
read straight off the log. It replaces the three blind calls a session used to
open with. `propose` takes a `tasks` array for the same reason: eight tasks in
one call instead of eight calls, each judged on its own so a bad draft does not
take the good ones with it. What the door costs an agent is mostly the
conversation it has to send again every time, not the writing.

`find` carries the same idea: a `doc` argument searches inside one document and
answers with line numbers and the section each sits in, and `tag`, `list`,
`by_agent`, `said_done` and a `from`/`to` range sift by what a task *is* rather
than what it says, so
«everything an agent filed for next week» is one call instead of a search and a
read of each hit. `said_done` is how an agent tells what it has already spoken
for from what it has not, so it does not say the same thing twice. Fields
that say nothing — a null, an empty list — are left out of every answer.

**The person's command line is frozen, and being retired by stages.** The
binary is not: `tisty mcp` is the door, the window carries it as a sidecar, and
`agent`, `doctor`, `sync`, `export`, `leave` and `demo` are maintenance that
stays. What goes is the terminal as a second window — the task and document
commands — because every rule was being written three times: core, window,
terminal, and the third copy is the one nobody uses. Until they go, no feature
reaches them and no new command joins them; what has to hold in the terminal
for safety — a rule the window keeps, like which task may be erased — still
does, and is written once, in the core, so that the terminal only obeys it.
That is how `tisty rm` came to refuse a story and `tisty set --read-as` to
exist the same day the policy was written: the erase rule moved to the core,
and the terminal had to be able to convert to obey it. The check below is the
binary's own and stays. When the commands are gone, it guards what is left:
nothing an assistant could reach for.

**The terminal is the person's, and the binary checks that it is.** An
assistant with a shell could type `tisty done 3` the day its server is not
connected, or `tisty agent --on` and mint itself the device the paragraphs
above say only the person mints. So every subcommand but `mcp` looks at who is
at the keyboard before the store opens, and is refused when a coding assistant
is: by a mark in the environment (`CLAUDECODE` and the like), by one up the
process tree (`claude`, `codex`, `gemini`, `opencode`…), or by an editor up the
tree with no terminal attached at all — the editor's own integrated terminal
has one, its agent's shell has none. The refusal is written for the assistant
that reads it, because it is the one instruction that reaches it without the
server: go through the server, or tell the person it is down. It is noted in
the log, so the person can see it happened. A store the person did not choose —
`TISTY_DATA`, a `TISTY_PROFILE` sandbox — is not guarded, which is what lets
the tests and `demo` run under an assistant. The check is a heuristic and says
so: the same user in the same shell cannot be told apart with certainty, and
an environment variable does not cross from WSL into a Windows binary reached
through interop. On Windows the tree is walked by `sysinfo` rather than by
hand, because reading it there takes a call this workspace forbids itself.
Where the assistant's client can turn the command down before it runs — a
hook, a rule — that is the layer that does not depend on the server; the
binary is the floor beneath it.

`tisty agent --on` is guarded once more, and this time by no list of names:
past the check above it asks the person, on the terminal itself, and a shell
with no terminal is turned away — a piped answer never reaches the prompt, and
the shell most assistants drive is a pipe. One driving a pseudo-terminal has a
terminal to answer from, and is back in the hands of the heuristic above. A
store the person did not choose asks nobody, which is what lets the tests turn
an agent on — read the way `Paths::resolve` reads it, so a profile name it
throws away does not open the person's own store to the check's exemption.

**An update never cuts the door.** A client starts `tisty mcp` once and does
not start it again when it dies, so an update that closed it left every
assistant without Tisty until the person reconnected it by hand. The Windows
installer no longer closes a running `tisty.exe`: it moves it aside under a
name of its own, which Windows allows for a running binary, and removes those
left from earlier updates once nothing runs them. Only a move that fails falls
back to closing it. An auto-update never runs the old uninstaller, which still
closes everything: uninstalling means the program goes. The door, for its part,
keeps what its binary looked like when it started, and before each message it
checks whether a new one was laid in its place, by an installer or by a bundle
swap on macOS. When one was, it starts that one, greets it with the client's
own `initialize` or `server/discover` — or a `ping` when the client never
greeted — waits for the answer to that same id, drops it, and from then on
passes every line through. The client keeps talking to the same process and is
served by the new version, so each update adds one link to that session's
chain until the client closes it. A new binary that will not start or answer
within ten seconds is left alone and the old door serves on.

**No assistant ever deletes, and that is the design rather than an omission.** The
MCP has no tool that writes any of the deletions — not a task, not a list, not a
folder, not a document. Finishing is the person's, and so is unmaking; an agent
that cannot delete cannot be talked into deleting. The terminal is the person,
not an assistant: `tisty rm` and `tisty list delete` do delete, under the
machine's own device. **Documents are narrower still — only the window deletes
one**, so a
mistyped command cannot lose a document.

### What it said before is kept first

`edit_doc` and a whole-body `write_doc` both copy the old text into
`data/originals` **before** the new text is written, inside the same lock. If
that copy cannot be made the write is refused and the document is left alone.
The order matters: keeping the copy afterwards means a failure there leaves the
person with no way back, which for a product whose promise is that nothing is
destroyed is the one irreversible loss the door could allow.

The absence of a tool is a locked door, not a law, so the rule is written into the
log as well. `Op::destroys` names the six operations that take something away for
good — the four deletions, removing a device, and retiring an attachment — and
`State::apply` drops any of them written by a device that ever joined as an agent,
beside the tombstone check, over every log, on every machine. `store::ledger` reads
the raw log rather than a projection, so it applies the same rule itself; otherwise
an assistant's `DeviceRemove` would still lock a machine out of syncing while the
projection said nothing was wrong.

That the rule lives here and not at the tool gate matters because the gate only
guards one binary: a shared folder takes lines from anywhere, and one future tool
or one CLI subcommand written without this in mind would otherwise be honoured
everywhere the folder reaches. The same replay keeps the rest of what an
assistant's hand may do to a task that exists, as a list of what is let in
rather than of what is kept out: a journal line on any task; a bell on any
open task, since `remind` only ever adds one; the rest of a patch on what an
assistant filed; a mark, a description and steps on what an assistant filed or
the person opened to them; and nothing else — not a close, a drop, a hide, a
move, a mark taken off, a step taken back — whatever the server that wrote it
believed. The trail (`story.rs`) keeps the same list, so it never tells of a
step the task did not take.

It is a defence against an honest binary that was wrong, not against a forged
log. One thing is checked: an event claims the device whose directory holds it
— a device writes only there and sync copies directories whole, so on
everything Tisty ever wrote the two agree — and one that claims another is let
go at reading. Beyond that, nothing signs an event, and a hand that can append
to the store's files can also delete them: that hand is the person's own, and
the terminal gate below is the floor it stands on, not a lock.

Judged at replay, the door is judged in the order the merged log sorts, by
stamp, and a clock behind another machine's would stamp a fill-in before the
opening that let it in — let go everywhere, the machine that wrote it
included, after the agent was told it landed. So every fill-in is written
under the agent's lock, judged against the whole log as read then and stamped
after the newest event in it (`Store::append_batch_unless`): what was read as
open lands after what opened it, whatever the clocks say. The lock is one
device's, so the person's window can still write in the same instant; what the
re-check closes is the gap between reading and writing, not the other hand.
An opening that has not synced over yet is simply not there, and the server
refuses rather than writes. Filling in is judged by the one door: what an
agent filed — any agent, this one or one since retired — or what the person
opened; `attended_by_agents` in the core, and nothing else in the server.

The trade it takes: this changes how an already-written log projects, so a machine
on an older build still honours what this one drops.
That is the rule working — the old build is unprotected, not wrong — and it is
worth being plain that it is the same shape of divergence refused for compaction
below. The difference is what sets it off: compaction would fork on ordinary
traffic, this forks only on an event no honest binary has ever written.

Two supports make it hold. `assistants` only ever grows: `agents` does not — taking
an assistant off the list clears it — and a rule built on that one would hand an
assistant's old deletions back on the next replay. And a device only says what kind
it is *itself*: `DeviceJoin` is honoured for the kind claim only when the payload
names the device that wrote it. Without that, one line could mark the person's own
machine an assistant, permanently and irreversibly, and quietly disarm its deletions.

The danger is the other side of it: **one path carries every deletion.**
`doc_drop` gathers the parent's file and its pages' files from the state *before*
it commits, then removes them from disk one at a time *after*. A file that will
not go is not a refusal — the log has already spoken, so the removal warns, the
rest of the run carries on, and `take_out_the_shed` sweeps what stayed at the next
opening. Only what the projection actually shed is removed: if the log refuses
the deletion, no file is touched and the window says so. What it cannot repair
announces itself — `doctor` and the keeping panel count both halves, the files on
disk the log does not name and the documents the log names with no file.

A file kept with a task goes on its journal; one kept in a document is added at
the end of it, as the same markdown the window writes when you drop a file in.
The two carry different ceilings, and the agent gets the ceiling of the place it
writes to: a task copies what the person set, up to 50 MB, and a document copies
up to 750 MB, which is fixed. That is the window's rule, not a second one — a
recording or a deck of slides is a document's to hold, not a task's.

Whether the document can take it is asked **before** the copy, not after: whether
the line still fits under the reader's ceiling, and whether the document already
carries as many files as one is read with. Three quarters of a gigabyte is a
slow way to find out there was no room, and a copy made for a line that is never
written is a copy nothing names afterwards.

**A file is copied in 64 kB at a time**, hashed as it goes and written to a
`.part` beside the shelves, then renamed into place — and one already kept is
compared to it side by side rather than both being read in. A ceiling of 750 MB
paid in memory would be three quarters of a gigabyte to read and another to
compare against, and a failed allocation ends a process rather than a request.
What is written is either renamed into its shelf or taken away: nothing half
copied stays.

It reaches a document that exists in two ways, and neither is a rewrite.
**Adding** puts text after the last line, leaving every byte that was there.
**Editing** replaces one passage with another, and the passage has to be named
as it is written, character for character, matching exactly one place — no
match or two matches writes nothing and says which, because anything else is a
guess, and a guess here writes over what somebody wrote. What an edit replaced
is copied to `originals/` first, and the window does read that directory back:
the bar that says something wrote here offers to put the document back to what
is kept beside it. The body it writes over is kept in turn, so going back twice
lands where it started, and it asks once — twice when the document has been
written in since that copy was set aside, because going back then takes that
writing with it.

Handing over the whole body is not shut, it is gated. The person may have the
document open in the window while the agent writes, so a whole-body write is
refused unless it carries the print the body was read at: against a named
passage a stale agent fails to match and stops, and against a whole body a stale
print stops it just the same. Without that gate the last writer would simply win
and the other's work would be gone. The window is held to the same rule — it is
refused a save over a body it did not read — and there the refusal can be put to
somebody, which a tool call cannot.

A body lives outside the log, and what either writes there is only the note that
answers for it — a `doc.said` with the body's print, its title, size and tags.
The window's watch compares a print of the documents themselves and tells the
window to read the open one again, which it does unless there are unsaved
changes in it.

**And unsaved changes are where the window used to write over what arrived.** It
keeps a print of every document it read or wrote; a save whose document no longer
matches that print is refused, and the person is told that something wrote here
while they had it open. Then it is theirs to say which one stands: keep mine
anyway, or read it again and lose what was typed. Refusing is the only honest
answer a whole-body write can give — it cannot tell which half of the file is the
part that arrived.

**Every writer of a body passes one lock**, kept beside the documents. Reading a
body and writing it back is two steps, and a lock only the agent took would
leave the window free to write between them — the agent would then save what it
read before, over what the person just kept. `write` takes the lock; `append`
and `edit` hold it across both steps and write through the unlocked path inside.
Waiting is two seconds, and a writer that waits longer is told the document is
being written rather than made to queue.

Syncing holds it too, per document rather than per round, so a long round never
keeps the editor from saving. It is the one writer that carries on **unheld**
if two seconds are not enough: a round that skipped a body comes back for it,
and if an agent wrote in the meantime the next round reads that as both sides
moving and weaves. Refusing there would trade a settled disagreement for a
stalled one.

What this settles only halfway: a person typing in a document with **unsaved
changes** while an agent edits it. The window does not pull the rug — it leaves
the text being typed alone, and holds back the redraw while they type — so they
write on for a while against a body that is no longer on disk. The save is not
lost work, though: it is refused with `documentMoved` and the person is asked
which stands, theirs or what arrived. What stays open is narrower and is the
choice itself — keeping theirs writes over the agent's body, and the copy in
`originals/` holds what came before the agent, not what the agent wrote, so
that one is the version nothing keeps.

Across two machines it behaves like any other edit, and the weave was measured
against it rather than assumed: if only one side grew, the round copies it and
asks nothing. If both grew, both added at the same place — the end — which the
weave will not settle on its own, so it refuses and the person is asked once,
with «keep both» offered first. Taking both yields the document with both
entries in order, losing neither. That is friction, not loss, and it is the same
road every other refusal takes. Two edits land as ordinary block edits: apart,
they weave; on the same block, one question.

Folders are the one place it may tidy: it can make one, give it an icon from the
closed catalogue, and move documents between them. It cannot rename, empty or
delete a folder, so the worst it can do is put a paper in the wrong drawer — and
a document it wrote is one nobody else has to find for it. A document it cannot
list is a document it wrote into the dark, so `docs` answers with everything
kept, the folder each one sits in, and whether it was put away. Reading one that
was put away is allowed, and the answer says so: a summary that treats an
archived paper as current is worse than no summary.

A folder name is forty characters at most, and that is the core's number, not
the agent's — `FOLDER_NAME_AT_MOST` in `model/folder.rs`. The window refuses the
same name the agent is refused, and the field stops accepting at the same count;
a test reads the constant out of the Rust and pins the window to it, the way
`DEEPEST` is pinned. Nothing shortens a name that is already stored: the limit
is on writing, so a folder named before the limit existed keeps the name it has.

A step is a hundred characters at most — `STEP_AT_MOST` in `model/task.rs` — by
the same recipe: the window, the terminal and the agent's `propose` and `plan`
refuse a longer one, the field stops at the same count and counts down over its
last twenty, and a test pins the window to the Rust. A step that needs more is a
task of its own, with a description and a journal; the context of a sentence
goes in the journal, naming the step. `tick` and `untick` name a step by its
text and take any length, so a step written before the limit can still be
ticked, and nothing already stored is shortened.

It may file into a list, but only one that already exists — a name that matches
nothing is refused with the names that do, so the agent cannot quietly invent a
place. Without a list it lands in the inbox for the person to place, and either
way it is tagged. What it reads carries the list and the priority back, because
those are the person's decisions and an agent that cannot see them keeps
proposing against them. It takes files only from where a download lands, never
from Tisty's own directories, since attachments reach the shared folder. And
the log says who may write, not the settings file an agent could edit
itself.

`stdout` carries MCP messages and nothing else, which the core already
guaranteed: it prints nothing, ever.

The protocol is the 2026-07-28 revision, which dropped the `initialize`
handshake: every request carries its own version in `_meta`, and
`server/discover` answers with the versions, tools and instructions on offer.
A client still speaking 2025-11-25 or earlier gets the old handshake instead,
so an older assistant is not locked out.

Filing the same thing twice is the failure a person notices first, so it is
settled where it cannot race: what an agent files may be stamped with the
`source` it came from, and the check for that stamp happens inside the same
lock that appends. Sixteen processes told the same thing write one task and
refuse fifteen.

What comes back is trimmed on the way out. A task the person hid is not
returned, and not counted either — a total of one would say the thing exists.
Journal lines lose the absolute paths they name, so what an assistant reads
never carries the shape of the disk it was read from.

### Connecting one means writing in somebody else's house

The Agents tab lists the assistants installed on this machine and offers to
write Tisty into the settings of each. None of them is found by asking the
PATH: an assistant is known by the file it keeps — `~/.claude.json`,
`~/.codex/config.toml`, `~/.gemini/config/mcp_config.json`, VS Code's
`mcp.json`, Claude Desktop's `claude_desktop_config.json` — because a command
can be installed and absent from the PATH the window inherited, and on the
machine this was written on, Codex is exactly that.

Those files are theirs, so only the `tisty` entry is written and the rest is
left byte for byte. The JSON is edited in place rather than parsed and written
back out, which is what keeps a 70 KB `.claude.json` in its own order and VS
Code's comments in its `mcp.json`; the TOML has its `[mcp_servers.tisty]` table
replaced as text, since re-serialising it would hand back a file with somebody
else's quoting rewritten. What cannot be edited that way — a server defined
inline, a key that is not an object — is refused with the lines to paste rather
than guessed at. What was there is copied to `<file>.before-tisty` first, and
the write itself is atomic.

Claude Desktop is one row and not two: Cowork reads that same file and Desktop
bridges it into its sandbox, so connecting the one connects both. On Windows the
file has two homes — the Store package's own `%APPDATA%`, under
`Packages\Claude_*\LocalCache\Roaming`, and `%APPDATA%\Claude` for the plain
installer — and only looking in both finds it.

What gets written is `command::calling()`: the CLI beside the window where there
is one, and the bare name otherwise. Packaged for the Store there is not — the
window is `Tisty.exe`, Windows answers `tisty.exe` with it, and the CLI is off
in `cli\` — so `beside()` refuses a command that is the running executable, and
the name alone reaches the execution alias the manifest declares, which survives
the version-stamped package directory changing under every update.

Two things it does not do. It never says an assistant is running: what it knows
is what a settings file says, and that is a claim it can keep. And the other
program may write its own file back — Claude Desktop rewrites its settings as it
closes — so the row asks for it to be closed and opened again, and an entry lost
that way comes back as «Connect» on the next look rather than as a lie.

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

- a closed segment is missing from the sequence,
- a closed segment is present but empty,
- a closed segment holds a different count of events than its `.count` declares,
- any line fails to parse,
- an event declares a schema version this build does not know.

Syncing holds the same line from the other end: a device's history that arrives
**shorter than the one already held** is left where it is. Contiguity alone does
not catch that — the gap is not *between* closed segments but *before* `active`,
and nothing in the folder says how many closed ones there should be. So the
counts are compared instead. A cloud client may well deliver a rotated `active`
before the closed segment that carries what it dropped; that ordering must not
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

A task is deletable once it is closed and **reads as a trace**. A story or a
routine is only hidden, never erased: to erase a story the person converts it to
a trace first, and a trace kept as a story stops being erasable. The conversion
is the deliberate step, and what the task holds plays no part in it — what the
person converted is what they said it is. The rule is `State::erasable()`, in
the core — `Task::erasable()` reads the task alone, and the state adds that a
task other turns hang from counts as a routine's root, however bare — and the
window and `tisty rm` both obey it at the moment they commit, judged again
under the lock, since erasing has no undo and an agent or the other window may
have written since they last looked. Erasing and hiding are one task at a time:
a sweep of the whole layer was built and taken out, because a trace is let go
of by looking at it, not by a count. One thing is judged at replay,
deterministically, the way an assistant's deletion is: a delete that reaches a
task the person kept as a story is let go, on every machine alike, because that
word outlives a delete written elsewhere while the task still read as a trace.
Two deletions are not
the person's and stay outside the rule: reopening a routine's turn deletes the
untouched turn born from it, and undoing a capture deletes what it captured.
The tombstone travels, and it keeps what the task was written from: an
assistant reading the same message again is told it was let go, and files it
again only when the person asks for it back with `again`. What the log already
recorded about the task stays where it was written.

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

How the next date is worked out depends on how it was written. Naming a day or an
hour fixes it to the calendar — the bin goes out on Tuesday whether or not it
went last week, and «every day at ten» is due at ten whether or not yesterday's
was ticked — and naming only an interval, with no day and no hour to hang it on,
counts from the doing, which is what a habit means. Either way the next one lands
past today **and** past the day it was finished: coming back from a fortnight
away does not owe you a fortnight of
bins at once, and finishing today's does not hand you another one for today. A
time of day is kept as asked — finishing the 09:00 pill at 08:04 does not move
it to 08:04 for ever — and months and years count off the calendar even when
said as an interval, or the rent would drift a few days later every month.

Nothing is ever created ahead of time. There is no timer and no scheduler: a
task can only come from you writing one or from finishing a repeat. Skip a day
and there is still exactly one, waiting, overdue — never two.

### Naming the next turn without closing anything

The week ahead has to say when a routine falls next, and it must do that without
writing anything: the strip is a reading, not a decision. `recurring` walks the
cadence forward and reports the turns that land inside the window.

**The walk starts at the series anchor, never at today.** A cadence is a phase as
much as an interval: «every Monday» is Mondays, and a routine you did not tick
on time is still Mondays. Restarting the walk at midnight today looks harmless and
quietly reprojects the series onto whatever weekday you happen to be reading
it — a Monday routine read on a Thursday would announce itself for Thursday, while
ticking it produced the Monday, so two parts of the program answered the same
question differently. Worse for a monthly cadence: the first step from today
lands a month away, past the window, and the routine vanishes from the strip
although its real turn was three days off.

The walk itself is `Cadence::beyond`, which is the same one `Repeat::next` uses
when a turn is kept. One definition, so the strip and the tick cannot disagree.

**Only the last few turns cross the bridge.** A routine's series grows by one
every time it is kept, and the strip draws five beads from it. Sending the whole
chain would make every snapshot — and there is one on every tick, every focus,
every midnight — cost as much as the routine has been honoured, which is exactly
backwards. The counters are computed over the whole chain first, then the turns
are trimmed to what the strip draws.

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
which is reconstruction, not memory. A cadence counted from the doing is asked
the same: it is measured against no calendar, but a daily closed two days late
still left two days with nothing on them, and the person is the one who knows
whether they happened.

**`covering` checks the claimed dates against `owed_since` itself**, rather than
trusting whoever called it. A date the cadence never had would write a turn that
never existed, and a mistyped year would drag the live turn years out and take
the routine with it — so the window and the command line cannot widen it between
them, and neither can a future caller.

## A task with parts

A list is where work lives and never ends; a tag cuts across. Neither says what
a run of separate tasks is *for* until it is finished. A task can hold parts
for that: each part is a whole task, with its steps and its journal, and
`part_of` names the task it belongs to. One level only — a part holds no parts,
and a task that holds parts is never one — and one whole per part; what cuts
across is still a tag.

`part_of` is an optional field on `task.add` and `task.move` (`null` lets a
part go), with no new op and no schema bump: a build that knows nothing of it
reads past the field and shows the part as a task of its own, which is all a
part is. Everything that judges a part is judged at replay, in `parts.rs`, the
way a folder's move is, so two machines that crossed a move agree on where the
part landed: the later move wins, a whole that is not there or is itself a
part is refused, and something that repeats can neither hold parts nor be one
— a routine never ends, and a whole is there to end. The window and the
terminal refuse a `repeat` on a whole or on a part before it is written
(`State::repeat_refused`), so undo never has to put one back; one that arrives
anyway, from a build that knew no parts, lets the whole and its parts go apart,
and undoing it hangs them again.

Nothing cascades in the projection, because an older build would not cascade
and the machines would part ways. What a whole does to its parts is written as
events of their own: finishing a whole with parts still open drops those parts
and closes it in one transaction (`State::completing`, the one every window and
the terminal already call), so one undo takes all of it back; a whole that is
deleted anyway leaves its parts standing on their own; and a task that holds
parts is not erasable (`Stays::Parts`), the same rule as the head of a routine.
Nothing ever closes a whole by itself.

The window learns which tasks are wholes from the snapshot itself: `wholes`
maps each one to its title and its open and closed parts, counted once over
the store, so a part names its whole with `⌂` even when the whole is not in
the view, and a whole shows `▣ closed/total` and folds its parts beneath it
when both are. The detail fetches a whole's parts on its own (`parts_of`).
Hanging, adding a part and turning a step into one are judged before they are
written (`answers/parts.rs`), so the person is told why rather than seeing a
move that replay would refuse quietly; turning a step writes the new task and
removes the step in one transaction.

An assistant may write a task as a part only of a task it filed or that the
person opened to it; anywhere else the task is kept and the place let go. It
never moves the person's tasks under anything, which the door already refuses.

## Reminders

A reminder is an hour, kept on the task as a list of moments. Nothing about it is
scheduled anywhere: `herald::owed` reads the store when the program is running
and reports what has come due since it last looked, which is why a reminder rings
on the machine that holds the store and nowhere else. There is no server, so
there is nobody to ring for you while the computer is off.

That reading window is what makes **a moment already gone a refusal rather than
a value**. `owed` floors its window at twelve hours before now, so an hour
written into the past falls behind the floor and never rings. Storing it anyway
would leave the person — or the assistant that filed it — believing an
appointment is covered when nothing will ever sound. Every door refuses it: the
window, `set --remind`, and the `remind` tool.

**Two moments are the same moment when their instants match**, not when their
whole specs do. A reminder written before a journey, or merged in from another
machine, carries that machine's zone name; comparing the specs whole would let
the same instant in twice and ring the person twice for one appointment.

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
not from how long it lived: one that closed in an hour carrying a log somebody
took the trouble to write is a story, and one that closed after a month with
nothing written is a trace. A line of three words is not substance; the weight a
log carries climbs with what it says, and a plan or a pile of links cannot
make a story on their own.

Unless the person converted it. `read_as` names the layer they chose — story or
trace, never routine — and wins over the weight until they convert it back:
without a pin, the weight decides; the pin freezes; what was not converted goes
on changing layer with what is written. It is the one thing about a layer that
is stored, and it is stored as the word, not as a number: a manual weight was
considered and turned down, because it would have kept a difference against a
base that keeps moving — one eight-word note, or one word taken out, or the
other machine writing, would have undone the conversion without anyone asking.
Search orders by `heft()`, the weight clipped to the chosen side of the
threshold, so a conversion counts there; `told` on the cover and in the series
keeps reading `weight()`, because it says «left something written» and must
keep saying the truth. A conversion is a chapter in the trail, unlike hiding,
because it explains why a task sits where it sits. Reopening keeps a story pin,
as it keeps the agent's mark — a finish taken back by mistake must not unkeep
it — and lets a trace pin go: work starts again, and is judged again by what it
writes. A patch an assistant wrote lands without the pin, and its chapter is
not written: converting is the person's.

`Reading` is derived, but the `Volume` it reads from is **not** one of the three
views above: it is counted on write and travels in the read cache. Changing how
it is counted therefore means bumping the cache schema, or a store written by an
older build keeps answering with weights nobody computes any more.

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

## Nothing shrinks, and that is the design

The log only grows. Deleting grows it too: a deletion removes the entity from
the projection, keeps every event ever written about it, and appends one more.
The tombstone it leaves is what makes deleting safe — a late event about that id
is dropped, so a machine offline for six months cannot resurrect a task by
arriving with an old edit. On one real store, 29% of the log described entities
that no longer existed. This is not waste; it is the price of the tombstone.

**The measured rate.** A line is 225 bytes on average (p50 200, p90 310). One
user action is 1.09 events. A developer's machine with three writers and a
275-event test day in it averaged 40 events a day; ordinary personal use is
nearer 10 to 30. That is **1 to 3 MB a year**, a closed segment of 5,000 events
every six to eighteen months, and **about 130 MB after forty years** at the
faster rate. On that same store the log was 158 KB while the attachments were
65 MB: **the log is 0.2% of the data directory, and attachments are four hundred
times its size.**

**Reading does not degrade with history.** Replaying is linear at about 6 µs an
event — measured end to end through the CLI, 0.4 s at 50,000 events and 1.3 s at
200,000, against a 0.1 s floor for starting the process at all. A warm read
skips the replay entirely, and what is left scales with what you *hold* rather
than with what you did: the same 200,000 events project to 40,000 tasks, and
loading those from the cache costs 0.13 s however long it took to write them.

### Why the log is not compacted

Three things in this repo refuse it, and they refuse it for the sync's sake
rather than out of taste.

**Lineage is a byte comparison.** `one_grew_from_the_other` concatenates a
device directory's segments and asks whether one side `starts_with` the other.
Rewriting a single line — dropping an event, re-serialising one, reordering a
field — turns `Grew::Yes` into `Grew::No`, and an ordinary reconnection becomes
`Kin::Clash`: the four-answer question you are asked when you meet a stranger's
store.

**A gap refuses the whole store.** `contiguous` requires closed segments
numbered 1..N. Truncating from the front does not make the store smaller, it
makes it unreadable — every device in it, not only the truncated one.

**And a peer undoes it.** Syncing compares, per device, how many distinct events
the shared folder holds against how many this machine holds. A compacted
directory holds fewer, so the longer copy is brought back over it. Compaction
does not survive contact with a single peer that is switched on, never mind one
that has been away for months. Surviving would mean every machine compacting
identically at the same point, which is coordination — and there is no server
and no clock anyone trusts to do it with.

Snapshots, checkpoints, per-device watermarks, tombstone horizons and a separate
archive store all founder on one of those three, and a tombstone horizon adds
its own: it is a clock rule in a system whose premise is that no clock can be
trusted, and its failure is a deleted task coming back.

**The one shape that would work, and is not built.** A store rewrite the person
performs deliberately — build a fresh store from the log keeping the tombstones,
give it a new identity, and have every other machine adopt it. The machinery
exists (`stitch`, `take_over`, `forebears`, and the four-answer question is the
adoption step). It is not built because 130 MB over forty years does not justify
asking every machine you own to adopt a new store, and a machine that never
adopts is left clashing.

### The sets that only grow

`shed`, `retired`, `tombstones`, `assistants` and `forebears` never shrink
(`dropped` and `devices` do, as machines come and go). In bytes they are nothing
— a shed entry is 15 bytes, a retirement 55 — and pruning them would mean either
a new operation that destroys, or a projection that is no longer deterministic.

Their cost was never memory. It was that every launch walked the whole of `shed`
attempting a `remove_file` per entry, twice over when syncing, and that the
first attachment ever retired made every later launch read every document body
to find out what still names it. `tidy::Already` is a machine-local mark in the
cache — never in the log, never in the settings file an agent can edit — of what
this machine has already taken out. A name is written off only once it is
genuinely absent — from the shared folder too, and never while that folder is
merely unreachable, or an unmounted drive would be mistaken for a tidy one. So
sweeping is O(new) while everything owed can be taken out, and stays O(all) for
what cannot: a retirement something still names, or a file that will not go.
A store with nothing retired never opens a document at all. The mark is a second
copy of two of these sets, rewritten whole on each change, and that is the price
of not walking them.

### Maintenance is the person's, and it is about attachments

The Maintenance tab reviews and reports; nothing there runs on its own. It is
grouped by what it costs to be wrong rather than by what each thing is:
attachments nothing names go to the bin with thirty days to change your mind;
documents on disk the log does not name are offered **taking in** before letting
go, because adopting an orphan is the recoverable half and deleting it is final;
and a document the log names with no file is not offered for forgetting while a
machine has been quiet, since that deletion reaches every machine. The state
both buttons judge against is re-read first, because another process — an agent
through the MCP — writes the same store, and a file it has just written is an
orphan for the instant between the file and its event. Taking in also refuses a
name the log has already shed: the file is on its way out, and adopting it would
only have the next sweep take it again.

Letting a stray file go deletes a file **without writing an event**, so
`Op::destroys` cannot see it and the rule above does not protect it. Like
deleting a document, it lives in the window and nowhere else: no subcommand, no
tool.

## What a search means

One engine, three doors: the window, `tisty find` and the agent's `find` all cut
the query the same way, because a person who finds something in the window and
not in the terminal has found a bug, not a document.

A query is **words, not a string**. Each word has to turn up somewhere in the
same task or the same document, in any order; a phrase in quotes stays in its
order. Accents are folded away on both sides, so `analisis` finds *Análisis* and
`ANÁLISIS` finds it too — Spanish is the language this is written in, and a
search that demands the tilde is a search that fails half the time. Twelve words
is the ceiling: past that a query would scan the store a dozen times over for
nothing.

Where a word lands decides the order, not whether it counts. A task whose title
or tags hold every word ranks above one that only mentions them in a
description, a step or the journal — but a word in the title and the next one in
the journal is still a match, because that is how someone remembers a task.

Documents are searched over their **stripped** text: markdown syntax is taken
out before matching, so nobody has to type the punctuation. Bodies are held
folded in memory between searches, keyed by size and modification time, and the
whole cache is capped — past the ceiling the document is read from disk instead
of cached, which is slower and never wrong. The line shown back is the one
holding the most of what was asked, never a line of pure punctuation.

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
tisty sync --confirm <machine>     # answer for the key a machine says it signs with
```

Tisty always works in its own local directory. Syncing **leaves a copy** in that
folder and **brings home the copies others left**. Whatever keeps the folder
alive — the Google Drive, OneDrive or iCloud client you already run, a mounted
NAS, an external drive you plug in once a week — is not Tisty's business.

The window offers the four it can recognise — Google Drive, OneDrive, iCloud,
Dropbox — by the folder each one actually keeps on this machine, so nobody has
to go hunting for a path. Of Drive it takes **My Drive** and never the
per-machine backup section: two computers pointed there would never meet, and a
Drive holding only that section is not offered at all. What it cannot place is
said out loud rather than guessed at, because a folder nothing syncs looks
exactly like one that works, right up until the other machine never answers.

That folder is **not** the data directory, and pointing a cloud client at
`AppData` is still the wrong thing to do. The store stays on your disk; only
copies travel.

### The folder says what shape it is in

```text
<shared>/tisty.toml              shape = 1, and the folders that shape has
├── store/                       every machine's history, one directory each
├── attachments/
└── docs/
```

`tisty.toml` is **read before anything else there and written after everything
else**, so finding it is finding a round that got to the end. A number this build
does not know means a later one arranged that folder around something this one
cannot see, and the round stops in both directions rather than write over it —
the same refusal a newer schema in the log already earns.

Its absence is never read as «there is nothing here». A folder that has never
said its shape is one from before this file existed, and is adopted and stamped.
A folder that said it and has gone quiet is one where a round did not finish or
a hand went through, and nothing is read from it until it says so again. The
difference is remembered in `<data>/.shape-seen`, which never travels.

**What sits beside a segment travels whether this build has a name for it or
not.** Only the lock and a mend left behind stay on the machine that made them.
A later build may write a sibling this one cannot read, and leaving it where it
was loses it as surely as deleting it.

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

**What that check costs, and where the answer is kept.** Vouching for an
attachment means reading all of it — half a gigabyte takes about six seconds, and
that is the same cold or warm. Heavy files are exactly the ones the shared folder
keeps and the local store does not, so the cost lands on every video and every
recording. The answer is written beside the cache, keyed by path, size and date,
so it is paid once in the life of a file rather than once per launch; a file that
changes gets a new key and is read again. Losing that file costs a re-read and
nothing else.

**A body a cloud left up there is a hole, not a file that lied.** What a keeper
leaves in place of a body has one module answering for it, `holes.rs`. iCloud
takes the name away and stands a `.{name}.icloud` sidecar beside it, which on a
Mac `brctl download` is asked to bring down; Windows keeps the entry with its
logical size and marks it in the file's attributes, and reading it is what
fetches it. So a placeholder whose size matches is not hashed to vouch for it —
that read would bring down the very file the setting leaves in the cloud — and a
body that could not be read answers neither way: nothing is written down, and it
is reported as held away rather than torn. Sidecars never travel and never go
into a backup.

Since Sonoma, iCloud Drive leaves no sidecar: the file stays where it was and
`stat` reports it whole, with only the `SF_DATALESS` flag saying its bytes are
in the cloud — Dropbox and OneDrive through File Provider do the same — and
reading it waits for the download. A round never makes that wait. Before a
history, a document body or an attachment is read from the shared folder,
`holes::still_away` and `a_hole` ask whether it is there, by metadata alone; what
is not stays out of the turn, is reported in `Moved::coming` — on its way, not
unreadable — and is asked for with `holes::ask_for`, which reads one byte of it
on a thread of its own so the cloud starts bringing it down. The next turn finds
it here. A body that is here while its history is not yet is on its way in the
same sense, and is reported the same way (see the documents' merge, below). A
store of thousands of files arriving on a new machine therefore comes in over
several turns that each finish, instead of one turn that waits for every
download in a row.

A turn says how far it got. `Reached::Log` and `Reached::Papers` mark the
history and the documents arriving, and `Reached::Along` counts each stage — the
machines' histories, then the bodies, then the attachments — as `done` of
`whole`, never counting back; the history is whole before a body or an
attachment is opened, so the window can draw what it has while the rest comes
in. The window hears it as `bringing`, at most every quarter second, and
`brought` when the turn ends however it ends — carrying the refusal when the
turn failed, so the foot of the sidebar keeps saying why until a turn gets
through: after the welcome has already let the person in, and on a machine
whose rounds stop because another writes with a newer Tisty. A turn that
commits what landed while it runs marks the session behind first, so the
history it brought is projected rather than stamped over. Nothing in the window
waits on it:
the welcome lets the person in as soon as the first `bringing` arrives — the
folder has passed every check that needed them by then — a list with nothing yet
says the history is on its way instead of saying it is empty, and a turn that
runs past three seconds shows its stage and count at the foot of the sidebar,
saying so when the cloud is slow rather than dropping the warning. Each
attachment that lands is `Reached::Kept`, and the window writes `AttachKept` for
it in handfuls as the turn goes, so a turn cut short still says this machine
holds what it already brought.

That is the only road by which a print gets written, so an attachment no machine
ever brought home has none: one kept only in the folder because of its size, or
one that came in before every attachment's print was written down. It opens,
and nothing vouches for it. The maintenance panel's **Attachments with no
print** lists what a task or a document points at, lies here or in the folder,
and has no `attach.kept` (`unvouched.rs`), and writes them down on request: each
is read whole and its print must match its name (`attach::vouched`) before
anything is written, so a file that only looks right is named and left alone. A
copy found only in the folder is written down and let go in the same breath —
`attach.kept` then `attach.let_go` — so its print is known without this machine
claiming a copy it does not hold, which another machine's *free up* would take
as leave to drop its own. It is a button rather than part of the turn because
reading means downloading: a cloud drive brings down whatever it keeps online
only.

Nothing waits on that read while holding a lock, or every other command touching
an attachment would queue behind it. The same holds for the folder and the
store as a whole: `joining`, `sync_kin` and `sync_state` take what they need
under the session lock and read after letting it go, and `snapshot` reprojects
a store that moved off the main thread and outside the lock, the way
`catching_up` does, so a first turn that brings thousands of segments never
leaves the window waiting to draw. And nothing in the window may assume the
answer is there yet: a player that asks once and gives up mounts with no source
and stays mute until the document is opened again, which is the bug this rule
exists to prevent.

**A preview that fills itself in must not be keyed on what it is waiting for.**
The editor rebuilds a preview when its key changes, and the one being replaced
leaves the page while whatever it started keeps running. Key a player on the url
it is waiting for and the arrival rebuilds it: the copy taken out of the page is
handed the source anyway and plays where nobody can see it — or pause it. One per
video, all at once, out of step with the one on screen. So the url stays out of
the key, and anything a preview leaves running is called off when the editor
destroys it.

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
second time. Identical means the same length and then the same bytes, compared
whole, 16 KiB at a time, and not the same timestamp. A length that differs
answers at once without opening either file, which is what keeps the common case
cheap. The date is deliberately no part of that answer: a date can be equal while
the content is not, and the other way round.

Comparing only the tail would leave a blind spot: two files of the same length
that differ **before** their last bytes would read as identical, and a segment
rewritten in the middle is exactly what the round must not take for the one it
already holds. The price is reading both files through whenever their lengths
agree, which on a projected drive — which is what a cloud folder is — means
bringing the body down. Within a round each machine's segments are compared once
(`Alike`), and bringing and handing on both read that one answer.

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
folder's, and the last this machine carried to that folder. If one side moved,
it is copied without asking. A clock would be worse than useless — a laptop
waking up is an hour out, and that has already cost us a real bug.

That third print belongs to the folder it was taken against, so pointing Tisty
at another one drops it, bodies and all. `carried.json` names that folder
(`Carried::up_to`) by volume and inode first and by path second, so a drive that
comes back under another letter is still the folder it was. A version the new
folder never held is not what the two of you came from, and leaning on it would
copy one side over the other without ever comparing them. The price is paid once
and in questions: what both sides already hold alike stays quiet, and only what
differs is asked about.

If both moved, the two versions are **merged block by block** before anyone is
asked. The unit is the block — text between blank lines — which buys atomicity
for free: a table and an ordered list carry no blank line inside, so each is one
whole block and cannot be spliced half from each side. Fenced code keeps its
blanks, because there a blank line is content. Only overlapping edits are a
question; two adjacent ones are simply both taken.

A body reaches the folder only after the log that answers for it. A merge is
written here alone, and so is a body changed with another editor; the window or
the CLI writes its print down and pushes, and only then does the body leave.
What the cloud does with the two is its own affair: a provider uploads and
brings down files in whichever order it likes, so the other machine can meet a
body before the history that vouches for it. What the folder
holds is taken in if it matches the newest print or the last one another machine
wrote down — clocks decide which is newest, and a laptop can be an hour out — but
a body that is only another machine's last word is set aside here before it
replaces anything. A round that only pushes takes nothing in.

That holds only because every path that rewrites a body writes its print down:
saving, settling, converting and weaving a document, a file kept in one from the
CLI or an assistant, a parcel whose references were rewritten on the way in. A
body here that the log does not answer for — edited outside Tisty — is
`unanswered` and held back rather than sent. One the folder holds that no log
answers for is not taken in and not written over either. When only the other
machine changed it, its history is most likely still on the way, so it waits,
reported in `Moved::coming`, for up to an hour (`awaited::LANDING`) from when
this machine first saw it, and comes in on the round its print arrives. The hour
is counted on this machine's clock and kept in `awaited` in the data folder, by
document rather than by body, with the first and the last time it waited.
Rewriting the body does not start the wait over; a body locked here, or held by
a machine waiting to be confirmed, is timed all the same, so unlocking it or
removing that machine puts it to the person at once. The wait is forgotten once
a body comes in, or a day after it last waited, so a body planted again soon
after is asked about at once, and only a round that takes in and finishes
forgets anything. Whoever writes the folder can delay the question by an hour
at a time, and only by putting a body the log answers for in between: nothing
unanswered ever comes in. Past that hour, or at once when it was also edited
here or is locked, it is left undecided and the person chooses mine, theirs or
both. Settling «mine» is refused when the folder moved since the person was
asked, the same as weaving, and the window says the document changed again
while they decided rather than calling it locked.
Keeping both leaves no copy when what arrived is the text here already, or a
copy kept beside it before, so being asked again never piles up twins.

The other body that waits is one whose print a machine still waiting to
be confirmed wrote down, and only when nothing here changed it. That history is
in the folder unread, so its answer is on the way rather than missing: the
round reads every print it gave (`store::introduced::prints_in`), from one copy
of it, and only if that copy is signed whole under the key known here for that
machine, or failing one, the key it says. Its own signature cannot prove the
machine is ours, which is the confirmation still missing, so a history planted
in the folder can make a document wait and never more: nothing comes in, the
notice names the computer, and removing it puts the document to the person
again, with that computer no longer listed as waiting. The document stays as it
is, the round reports it in `waiting` and the window says so with the way out —
confirm the machine and it settles on the next round, or remove it in
Maintenance. One edited here as well, or locked, is never held: it is put to the
person, so their own changes do not wait on someone else's. The history is read
only when some document needs it.

The engine refuses rather than guess, and every refusal lands on the same tested
road: the merge returns nothing, the document is left undecided, and **the
person decides**, with «keep both» offered first because it is the only answer
that loses nothing. It refuses when the two sides rewrote the same block
differently, when the comparison would cost more than sixteen million cells, when
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
name become one once the other side has come in, the one holding more keeping
the name and taking the other's tasks, and the guide each side brought stays
once (`doubled.rs`) — a fresh install's example lists are the usual twin, and
the person asked for them gone; and ordering keys were minted independently, so
lists interleave. An install that finds what an earlier one left, or a folder
that already holds a history, plants no example lists to begin with.

Twins that came before that rule, or by any other road, are put together by
hand: the maintenance panel's **Repeated lists** names each repeated name, how
many lists share it and how many tasks they hold (`doubled::repeated`), and
joins them with the same `doubled` a merge runs. The choice of which list keeps
the name reads the store as this machine sees it, so it is best done once the
other machines have been brought in; done on two machines before they meet, a
task can lose its list and land in the inbox, never more.

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

**What a document says about itself is read from the body, never asked for
separately.** Saving one writes a note with what is read out of what you typed:
the title, the size, the tags, and the print of the body. A tag is a hash with a
letter or a digit against it — `# Heading` carries a space and stays a heading,
and a colour or the fragment of a web address sits behind something that is not
a separator.
Fenced blocks and runs between backticks are stepped over, so pasted code is not
mistaken for labelling. Accents come off and case is folded, so `#camion` and
`#camión` are one word however anybody typed them that day, and a word pasted
from a Mac — where an accent is a letter of its own — folds to the same place.

**What is read has to be narrower than what is kept**, and the two rules live
apart on purpose. `Tag::worth_reading` in `model/tag.rs` is what a reader may
take from writing nobody meant as labelling: two characters at least, one of
them a letter, so `#1234` stays the ticket somebody wrote down and `#1` the
item they numbered. `Tag::new` stays as forgiving as it always was, because it
is also what deserialises the log — a line that fails to parse stops the whole
store from opening, and a tag saved before this rule still has to come back.
The capture bar (`tisty-nl`) reads the same way, and so does the editor, which
paints only what will be saved. What was recorded before is corrected by the
maintenance panel's **Tags in documents**: it re-reads every body and writes
the difference.

Two limits are the point rather than the detail. **Sixty-four tags is where a
body stops tagging**: a stylesheet pasted outside a fence reads every colour as
one, and the log is append-only, so a line of four thousand would be written
once and kept forever, on every machine. And **the note is only written when
what it says changed** (`Said::news_for`): a save that leaves the body as it was
writes nothing, and one that changes it writes a note, because the print it
carries is what lets another machine take that body in.

**A deletion is carried out, not inferred.** Deleting a document names its file
in the log, and every machine that reads that event removes its own copy — the
same treatment a retired attachment gets. What is left over from before this
existed, or from a copy that stopped halfway, is not deleted on a guess: `tisty
doctor` and the maintenance panel **count** the document files on disk that the
log does not know about, and leave them where they are.

## A page is part of a document, not a document beside it

A document may hold pages. A page is an ordinary document file — same name, same
ceiling, same way out — with one field saying which document it belongs to, and
that field is the whole difference between the two.

**There is one level, and the core is what enforces it.** A page holds no pages:
`DocAdd` naming a page as its parent keeps the deeper one as a document, and
`DocMove` refuses both a document that is its own parent and one that already
holds pages. The window and the agent refuse the same thing first, with a message
that says why, but neither is where the rule lives — an event that arrives from
another machine has passed no window.

**A page goes where its document goes.** It is born in its document's folder,
follows it when the document is filed elsewhere, is put away and brought back
with it, and is deleted with it. Filing a page into a folder of its own does
nothing: the folder of a page is the folder of the document it belongs to, so
there is nothing to keep in step later. Undoing a move puts the page back under
the document it was under, since the move recorded which parent it had.

**Coming out is deliberate.** «Make it a document of its own» sends a null
parent, and the page becomes a document standing in the folder it was already
showing in. Nothing is copied and no text changes: only the field goes. That is
why the tree can offer it as one menu entry and the agent as one call — the
event is the same move that files a document.

**A page sits where its document names it.** The body of a document may name
another document — `![Title](tisty:doc/its-name)`, the same reference that has
always drawn a card — and when what it names is one of its own pages, the window
draws the way into that page instead. The order those references are written in
is the order the pages are in: saving a body works out the sequence and moves
only the pages that have to move, so a body saved on every keystroke writes
nothing to the log until the text really says something different.

**A body says nothing about the pages it does not name**, and they are left
exactly where they were. That is not tidiness, it is what makes a book written
before any of this survive: such a book names none of its pages, and a rule that
sent the unnamed ones to the end would turn it inside out the first time a single
page ever named itself — the newest chapter would become the first. Instead the
document lists them as loose, with the one action that puts one in the text, and
the book turns into a named one at whatever pace its owner chooses. Reading it
follows that same rule and not a softer one: `State::pages_read` deals the named
pages back out, in the order the text names them, into the places named pages
already held — the very run `pages_told` is about to write — so what a reader is
shown is never an order the next save undoes. Both doors read through it, and the
window's `inTextOrder` is the same run in TypeScript. The agent's
door has two moves of its own: `page_doc` naming where a line goes, and `order`
dealing the lines a set of pages already have back out in the order asked for.
The window's own hang writes the line too, and writes it where the window already
writes: into the editor that holds the book. Dropping a document on another in
the tree puts the card at the end of what the person is reading, so their own
save carries it, with the page's live title, and nothing is written behind the
text they have open. A book that is not open gets nothing, and the page waits in
the loose half of the index with the one button that puts it in.

**That is the second thing this taught us, and it cost a rewrite.** The first
attempt had the window append to the parent's *file* through the same core call
the assistant uses. It looked like the symmetry the work was after and it was the
wrong symmetry: the assistant writes to a document nobody is holding, while the
window would have written underneath its own editor. The editor's next save then
read the file as changed by somebody else and offered to overwrite it, so the
person's own drag could silently take the line back out; the page the editor had
just been given looked loose for the thirty seconds until the herald noticed; and
the one entry point that makes a page from inside the editor named it twice, once
on disk with no label and once in the buffer with the right one. None of that is
reachable when the card goes in through the editor, because then there is one
writer, not two.

**A card is a place, a link is a mention, and they are not the same thing.** Both
are references — `refs::extract` sees both, which is what keeps a file nothing
else points at from being swept away — but only `![Title](tisty:doc/id)` gives a
page its place in the book. It was not always so: for a while any reference to a
page counted as naming it, and a sentence saying "as I wrote in [March]" made the
chapter sit wherever that sentence fell. Three things went wrong with it at once,
all found on a real store of three hundred documents. Taking a page out and
putting it back reported an order the body did not have, because the card was
already there and nothing rewrote the body, so nothing re-settled the log.
Reordering a book whose prose mentioned one of its chapters moved that chapter's
card past the sentence, after which every further call resolved the page to the
sentence and refused, blaming a paragraph that had nothing to do with it — the
book was stuck. And hanging a page a sentence already mentioned said it had
written the line and wrote nothing.

The rule that fixes all three is the one a person would guess: the card is the
chapter, the link is a cross-reference. Which also means a question about
references is asked of `extract` and never of `papers` — what points at a
document before it is archived, and which ids a parcel must rewrite on the way
in, both want every mention.

**What counts as code is decided once, for everybody.** A line naming a page
inside a fenced block is not a way in, it is an example of one, and for a while
the parts of Tisty disagreed about which fences existed: `refs` knew backticks
only, the placing had its own count that knew tildes but not how wide a fence
was, and the editor used a CommonMark parser that knew both. Making the reading
and the placing agree was not enough on its own, because what they agreed on was
something the person's own screen contradicted — a card shown inside a `~~~`
example counted as a chapter, so putting the chapters in order rewrote the
example. One tracker answers for all of them now: `docs::Fencing`, which opens on
three or more of one marker and closes on the same, at least as long, and which
already read the headings and the tags. The reading order walks past whatever it
calls code, so the order, the line a page is placed on and what the editor draws
cannot come apart; `paging.ts` follows the same rule — marker, width, and the
quote or bullet that may sit in front of it — for the index the window draws.

**What keeps a file alive is read more widely, and that asymmetry is the point.**
`refs::extract` is what the sweep asks before deleting an attachment nothing
names any more, and what the export asks before leaving one behind. It was left
reading tilde fences, because the two mistakes are not the same size: counting a
file named only in an example costs a stale file nobody looks at, and missing one
costs the file. Only the reading order, which decides where a chapter sits and
can be undone, is strict.

**Putting pages in order keeps a copy beside the document, and says so.** It
rewrites the body, which is the same thing `write_doc` and `edit_doc` do, and
the person has one step back per document. Hanging a page does not: it appends,
because the line it adds is not a change to anything they wrote, and spending
their one undo to add it would be taking something of theirs to pay for
something of ours.

That leaves one source of truth for where a chapter belongs, which is where the
person put it in the text. Cutting the reference and pasting it higher up moves
the page, in the tree, in the export and in print, without a second panel that
orders pages and can disagree with what is written. Deleting the reference is not
deleting the page: text is text, and a document is deleted where documents are
deleted.

The keys themselves are fractional, so a page moved out of ten is one event, not
ten. A run already in order asks for nothing; the longest rising run keeps its
keys and only what breaks the order is given a new one.

Folders count documents, not pages: a folder holding one document of forty pages
says one. The pages are shown under the document, in the tree and in `tisty
doc`, which is where the person went looking for them.

**Refusing to hang a page somewhere is not a reason to unhang it.** Two machines
can disagree — one moves a page under a document the other has just deleted — and
the move arrives naming a parent that is no longer there. The page keeps the
document it had. A rejected move that emptied `page_of` would leave the person
with a loose document nobody asked for, which is worse than the move not
happening.

**The schema is a fence, and it moved to 16.** Every event carries the version
that wrote it, and a store refuses a log written above the one it knows rather
than reading half of it. The version rises when the same event would project
differently — `doc.archive`, which used to write a mark on every page of the
document and now covers them instead, is what raised it to 15 — and it rises
again when an operation stops being one a reader may walk past, which is what
raised it to 16. Without the second rule the mark would come off at a version an
older reader still accepts, and a reader that does not know the name has nowhere
to go: it cannot parse the line, the mark is no longer there to excuse it, and it
refuses the whole store as corruption. Moving the fence is what turns that into
«you are behind», and what gives the folder's own guard something to read so it
can turn the round away before a byte is copied. Two machines must both
update before they sync again; the one left behind says so, and says that what
is written there stays there until it does. The parcel carries its own version,
raised to 2, but only when what it holds needs it — a parcel with nothing new to
say keeps the old shape, and an older Tisty still opens it.

**A page answers for itself, and covering is not marking.** Archiving a document
no longer writes a mark on each of its pages: `held_away` derives what the
archive holds from the document's own mark, the document above a page and the
folder above both, so bringing the document back wakes exactly what it covered
and leaves apart whatever the person had already put apart. What a page carries
in `archived` is a mark of its own, and nothing else may write one on its
behalf: hanging a page under an archived document covers it, and taking one out
from under a document leaves the archive around it rather than walking it out.

**Cascades cost the read cache its shortcut.** The cache rewrites one row per
event, and the operations that reach a document's pages — delete, archive,
unarchive, and a move that changes the folder or the document a page belongs
to — cannot be told a row at a time, so they throw the cache away and it is
rebuilt from the log.

**Putting a whole folder away is not one of them.** The mark lives on the folder
and no document row changes — what the archive holds is worked out on read, by
walking up from the document — so one row is rewritten and the cache stays true.
Deleting that folder *is* on the list already, and it is the one case where rows
do change, because the mark is written down into what the folder lets go of.

A move that carries nothing but an order is the exception, and it has to be, or
saving a body would cost a rebuild every time the text moved a page. It is safe
because that move touches one row and no other: the projection does walk every
page of the document to keep folders in step, but no reachable event makes that
walk change anything on an order — a page is born in its parent's folder and
`DocMove` refuses to file a page anywhere else. If that ever stops being true,
the exception has to go with it: the cache would keep a stale folder on a page
and no fingerprint would say so. Without that, a delete would leave its pages
alive in the cache, invisible in every view because their document is gone, and
their files would be carried back into the shared folder on the next round.

**Two ways out are one way out.** A copy of a document copies its pages, the way
they name each other rewritten to the copies, and a document taken out as
Markdown writes its pages beside it, numbered in reading order, with the
references pointing at those files rather than at names only Tisty knows. What
holds a book in forty parts and hands out the cover is not an export. A page that
cannot be read from disk is left out and counted: the export says how many went
missing rather than handing over a book quietly short of a chapter.

**Where the order is settled, and where it is not.** The body is a file that
syncs like a file; the order is in the log. A round of syncing reports which
bodies it brought, and every caller of it settles those before going on — the
window on sync and on opening, `tisty sync` too. That last one is not
housekeeping: a body the sync *merged* holds an order no machine ever wrote, so
if the machine that merged it never settles, no machine ever does. Settling
writes `DocMove` events, so *reading* a document is not a pure read — it is a
fixed point, and a body that already agrees with the log produces nothing.

What is still not settled on the way in: a round that fails partway loses the
list of what it brought, and a body the sync could not place at all — a symlink
where a file was expected — leaves the log ahead of the file. Both cases end the
same way as before, when the document is next opened or saved.

**Settling re-keys the whole run, not the part the text names.** A body may name
only some of a book's pages — hanging one writes its card, but a person editing
the text can take a card out, and a book written before any of this names none
of them, so a book can still hold pages the text never mentions. Those pages keep
the *places* they held, but the keys are dealt across every page at once.
Re-keying only the named ones would hand out a key an unnamed sibling already
holds, and a duplicate key
is decided by whichever id sorts first — a place nobody chose, and no later
settle repairs it, because the text never names that page again.

**Compaction is not atomic across machines, and is left that way on purpose.**
Re-basing a run is N independent moves, not one operation. Two machines
re-basing the same run inside one sync window can therefore produce duplicate
keys. `afresh` deals a whole run positionally, so identical runs give identical
keys and only a genuine disagreement collides; `pages_of` breaks a tie by id, so
every machine replaying the same log still reads the same order; and the next
save settles it. Making it truly atomic means replaying batches as a unit, which
would have an old binary and a new one project the same log differently, with no
version to catch it — a quiet incompatibility traded for a loud one, to fix
something that has never fired.

**Unhanging is its own undo.** Hanging a document under another takes the
parent's folder and a key on the scale of its sibling pages. A bare
`page_of: null` would leave both behind, and the document would reappear at an
arbitrary spot in a folder it never chose. So `undo::unhung` reads the last hang
out of the log and returns its inverse — the folder and the place it held — and
falls back to the end of the parent's folder in the three cases the inverse cannot
serve: there was no hang, the folder it names is gone, or the last hang moved the
page straight from one document to another — inverting *that* would re-hang it
under the earlier one instead of setting it free. Deleting has no such inverse
and never will: it is permanent by design, which is why it is the one thing
asked about first.

**The schema was raised to 8 for this.** A machine still on 1.0.x rejects the whole
event rather than reading a page as a loose document and filing it somewhere the
person never put it.

What is not settled: two machines whose clocks disagree by more than the time a
round takes can order the page's own creation before its document's. Every
machine still agrees — the log is replayed in one order — but the page is read as
a document of its own, standing in the folder it was written into. Nothing is
lost and nothing hides; the tie to the document is what goes.

## Putting a whole folder away

A folder can go into the archive with everything under it — subfolders,
documents, pages — and comes back the same, which is the whole point: nothing is
moved, so nothing has to be put back.

**The state is derived, not cascaded.** The folder carries its own `archived`,
and a document is in the archive when its own mark says so *or* when any folder
above it does. Marking every document instead would have been shorter and would
have thrown away the one thing worth keeping: which of them somebody had archived
by hand before the folder was shelved. Those come back archived when the folder
returns, and only those.

The cost of deriving is that the mark can be separated from what it applies to,
and there it would be lost. Three places close that, all in the projection, where
an event arriving from another machine lands too:

- **Deleting the folder** writes the mark down into the documents and subfolders
  it lets go of, before it lets go. Both machines replay the same log in the same
  order, so both land on the same answer.
- **Packing** writes what the archive holds into every document, not only its own
  mark, so a parcel opened by a build that knows nothing of shelved folders still
  restores it closed. A separate `by_folder` flag says which of those are only
  closed by the folder, so bringing it back opens the right ones.
- **Naming a folder that already exists** — what merging two stores and restoring
  a parcel both do — keeps its archive instead of rebuilding it open.

**What the archive holds is read, exported and packed as always, and written by
nobody** — no window command, no agent tool. Two writes reach it anyway, and both
predate this: settling a rift, because leaving a conflict with no way out is
worse, and recording what a body already said when it arrives from the other
machine.

**Only the folder that was shelved has a door.** A document inside it does not
unarchive on its own; it would leave a hole in the shape the folder is keeping.
For the same reason it cannot be deleted from in there, and deleting has no
inverse.

**The schema was raised to 12 for this.** `folder.archive` changes what already
exists, so it does not carry the skip-me mark: a build that ignored it would show
the folder open and write inside it. An older machine stops syncing until it
updates, which it is told to do the moment it meets the newer store.

**It was raised to 13 for `task.resolve` and `task.unresolve`.** The same
reasoning: a mark an older build walked past would leave that build showing a
task nobody has spoken for, and the person deciding without the account. Neither
carries the skip-me mark. What the two of them say sits beside the task rather
than inside it, so a build that does read them loses nothing if it meets them out
of order; and because the account travels as a plain `task.log`, even a build
that refuses the store entirely is refusing something whose reasons it would have
understood.

## What the editor may write into a document

A document is a Markdown file, so the question of what the editor is allowed to
write is the question of what another reader will still understand. The answer is
plain Markdown and nothing invented: headings, lists, emphasis, links, tables
**with the alignment their columns were given**, fenced code carrying its
language, and a quote that opens with `[!WARNING]` — a callout here, an alert on
GitHub, and a plain quote to anything that knows neither. The one exception is an
icon, which Markdown has no way to say at all: that goes as a small piece of HTML
with its name inside, so a reader that cannot draw it still reads the word.

A widget is not an exception either: it is fenced code whose language is
`widget`, so any other reader shows the HTML as code. Tisty draws it — and an
attached `.html` the same way — in a sealed frame served over `widget://` under
its own policy: no network, no reach into the window, and a link opened only on
a click inside it.

A widget is a fragment: it is wrapped in the kit that makes plain HTML look like
Tisty and measured by the box that holds it. An attached page that is a whole
document — it opens with `<!doctype` or `<html` — is served as itself instead,
because nesting one document inside another breaks it, and many such pages are
bundles that unpack their own scripts and frames from `blob:` and `data:` at
load. Its policy lets it make and use those, and run what it unpacks, but still
names no address: nothing it does reaches the network. Its frames may only be
`blob:` it made, never `data:`, since an engine is not bound to carry the page's
policy into a `data:` frame. It is measured from the
window by what flows in its body, leaving out what is fixed to the screen, since
a page may fill the whole viewport or swap its document while it unpacks.

Text alignment used to be the second exception and no longer is. It wrote
`<p style="text-align: center">` into the file to say something no Markdown
syntax says, and a document full of that is a document that has stopped being
Markdown. Column alignment reads like the same feature and is not: `|:---:|` is
the table's own syntax, and every reader honours it.

**When the editor meets syntax it is known not to keep**, it does not guess and
it does not quietly drop it. `frail` names what it found — front matter,
footnotes, reference links, blocks of HTML — the document opens read-only, and
the person is offered a conversion they can refuse. Losing formatting loudly is
recoverable; losing it silently is not.

`frail` is a list of shapes, though, not a proof. It and `docs::survives` — the
same question asked on the agent's side — are two implementations of one idea and
they do not agree everywhere: an HTML entity, or an inline `<kbd>`, is refused to
an agent and edited by the window. Where they disagree, the honest reading is that
the window keeps less than the list implies, not that the file is safe.

## Taking a document out

A document is a Markdown file, and the whole point is that it survives without
us. Five ways out, and the differences are not cosmetic.

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

**Export everything as Markdown** is the same trade repeated across the whole
store, with the folder tree standing up on disk: `Personal/House/Minutes/`. Two
folders that spell the same once their names are made safe for a filesystem —
`Casa` and `Casa?` — are told apart rather than poured into one directory, and
two documents with the same title inside a folder are numbered rather than
overwritten.

**Export for Tisty** is the one that does not lose anything, because it is not
Markdown: a zip named `.tistyx` carrying the bodies byte for byte, the
attachments they name, a `README.txt` telling whoever it lands on what the file
is and that the documents inside are readable without Tisty, and a manifest with
what Markdown cannot hold — folders
with their order, icon and colour, which document each page hangs from, what is
archived, what is locked, and the alias the writing was signed with. **Not one
line of the log travels inside it.** What lands in the store that takes it in is
born there: new ids, new events, the references between documents rewritten to
the names of their new home. It is how a document moves between installations,
and it is not a backup — bringing the same parcel in twice makes a second copy
of everything, because nothing in it says «you already have this».

A parcel of everything can be **locked with a number**, and that is what tells a
move from a hand-over. Locked, it is not a zip at all: the zip is sealed in
64 KiB blocks with XChaCha20-Poly1305, under a key scrypt grinds out of the
number, and each block's nonce carries its own count and a byte that is only set
on the last one — so a file cut short cannot read as a whole one. Whoever opens
it with the number gets what is inside as their own writing, because only the
machine it was packed for knows it; what was already a guest where it came from
stays one, because the manifest says so. Without the number the parcel says it
is locked and nothing lands. A single document is never locked: it is always
somebody else's to keep.

Every store is given two things when it is made: a name of its own, which travels
inside every parcel it writes, and a secret that never leaves the machine. The
manifest carries a seal — an HMAC of itself under that secret — so a store can
tell a parcel it really wrote from one that merely wears its name. Anybody handed
a parcel knows the name written in it; without the secret they cannot forge the
seal, and what they send lands as a stranger's however it is addressed. A parcel
with no seal at all is a stranger's by definition.

The secret belongs to the machine, not to the shared folder: two machines that
sync carry the same name but not the same secret, so an open parcel written on
one lands on the other as somebody else's. What carries writing between machines
of your own is the number, not the name — which is the whole reason the number
exists.

The store that receives it decides what is a guest by the identity of the store
that sent it, written in the manifest, and never by the name inside: two people
who happen to share an alias do not inherit each other's writing, and what comes
from somebody who never signed lands without an author rather than under the
name of whoever opened it.

Who *edited* it does not travel at all. The author is the document's, and it is
the same wherever the document goes; the hand that last wrote is this store's
own reading, sealed in `doc.said` as it was written. Changing an alias rewrites
neither one: the log keeps what it kept, «signed before as» keeps the name the
document was born under, and re-signing what is mine is a deliberate act with
its own event. What the reading does is resolve — an alias this store signed
with before reads as the alias it signs with now, because it is the same hand,
while somebody else's stays theirs however this machine signs today.

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

One zip of `store/`, `docs/`, `originals/` and `attachments/` (`backup::CARRIED`),
never the configuration — a shared `device_id` would put two machines in one
file — and never `<config>/private/`, so neither the signing key nor the secret
that seals parcels leaves in one. What goes in, and what a zip may put back, is
one list of name shapes rather than a list of exclusions, so a file nobody
thought to name stays out. Where attachments are held only in the shared folder,
the bodies and attachments the folder alone keeps are packed in as well. A copy
is capped at 8 GB and 200,000 files; past the size, the window says both
numbers side by side and the button stays disabled.

Restoring is **a photograph**: back to that moment, and what came after is lost
on purpose. The machine **takes a new device id** so its directory starts empty
and can never shrink what other machines already hold. A copy of another store
is refused (`OtherStore`) unless this one is still empty.

Nothing of yours is touched until the whole backup has been unpacked beside it
and read back, and the swap moves every old folder aside before a single new one
steps in. A zip that turns out to be corrupt, truncated, somebody else's, or not
a backup at all costs you nothing — and half a restore is the one outcome worth
less than either whole.

**A shared folder is not a backup, so the backup stays.** Both are offered
whether or not a folder is chosen; the buttons wait only while something else is
being written, and making a copy also waits on its size. The folder holds the
same history, not an earlier one, and it may live in somebody else's account.
Restoring stops sharing — otherwise the folder would bring back what the copy
went back on — and Settings names the folder it was, so choosing it again asks
which side wins. It is a local decision with global consequences, and the other
machines never hear about it.

The honest limit: with syncing you get **redundancy, not a way back in time**.
Delete a task and the deletion travels. Going back for everyone would have to be
an event of its own — a `store.rewind` the projection honours — which is written
down as an idea and not built.

## Schema 17: the segment that seals itself

Designed, not built: this is what the 1.25 writes, and the one break with
everything before it. It is written here before the cloud's code so that the
format is shaped by the contract below rather than the other way round.

**Why the `.sig` has to go.** A segment and its `.sig` are two files that have
to arrive together, and nothing promises that. A folder client copies them
when it likes; an API promises that one file lands whole or not at all, and
nothing about two. A round cut between them leaves a segment with no signature
or a signature over the wrong bytes, and a machine marked as signing is then
read as disowned when it was only half carried. Worse, renaming `active.tisty`
to `000002.tisty` on one side while the other still holds the old `active`
pair chains two copies that never followed each other.

**The seal.** Every write appends, in the same append and the same `fsync`, one
more line after its events:

```json
{"v":17,"op":"seal","seg":3,"at":40960,"tip":"…","n":212,"inst":"…","sig":"…"}
```

- `seg` is the segment's number, not its file name, so renaming `active.tisty`
  to `000003.tisty` does not touch what was signed.
- `at` is how many bytes came before the seal, `tip` the SHA-256 chain folded
  across this machine's whole history up to there, `n` how many events it
  holds. The last seal of a segment that rotates says `"closed":true`.
- `inst` is the installation that wrote it: `machine::here()`, the digest of
  the computer's own identifier that #153 already keeps in `config.inst`. It
  sits inside what is signed, so two installations writing as one machine — a
  configuration copied to another computer, two packages of one build — are
  told apart by the very lines they write, and it can never be added later.
- `sig` signs everything above it with the machine's key.

A segment and its signature are now one object. The seal is always the last
line, which also makes the newest schema of a history readable from its tail.
Reading follows from that:

- Bytes after the last seal are on their way, never tampering: the history is
  `Unreadable` this round, and whoever wrote them seals them again on opening
  its store.
- An `active.tisty` whose seal names a segment at or below the last closed one
  is a leftover of a rotation carried halfway, and is skipped.
- Closed segments are uploaded only if they do not exist; the live one is a
  single object rewritten whole each round, against its revision where the
  provider has one. The size at which a segment rotates becomes the writer's to
  choose, about 1 MiB through an API, because the reader counts from the seal.
- `.count` goes with `.sig`: the count lives in the seal.

**What else the 17 requires**, because each changes bytes that are signed or
that other machines read:

- A `device.join` written at 17 carries `p`: from then on no machine is
  without a key.
- Every document body written at 17 has its print in `doc.said`, so a body that
  arrives before the log that answers for it waits instead of coming in. Today
  it waits an hour and is then put to the person; under 17 every body has a
  print, so the wait can only end in its history arriving or the question.

**Migration.** `SCHEMA_VERSION` becomes 17 with `SEALED_FROM = 17`. The first
write at 17 rotates the active v16 segment the old way, with its `.sig` and
`.count`, and carries the same hash chain on with seals in line. Segments
without a seal are still read by their `.sig`. Once a machine has sealed, a
later segment of its own without seals is disowned. A build that only knows 16
stops at the first line with `"v":17`, as it already does for any newer schema.

### Changing a machine's key without asking again

A key is meant to outlive the computer's software, not to last forever. Until
the 17, changing it meant reinstalling: the machine came back under a new name
and every other machine had to confirm it again with its twenty digits. The 17
lets a machine move to a new key **on the word of the old one**.

- The machine makes a new key and writes `device.rotate { d, p }`, `p` being
  the new key, in a batch sealed by the **old** key. That is the last seal the
  old key ever makes.
- The next seal is made by the new key. A seal by the old key after its
  rotation, or a seal by the new key before the rotation that names it, is
  disowned: there is one switch, in one direction, at one place in the chain.
- A machine that had confirmed the old key takes the new one without asking,
  because the old key, which the person answered for, is what vouches for it.
  It is kept in `.keys-confirmed` with `rotated:` and the old key's code, so
  what the person compared can be told from what followed from it, the same
  way `carried` and `host:` are kept.
- A machine that had **not** confirmed the old key gains nothing: it still
  waits for the person, now with the new key's code.
- Agents keep their own road: a host vouches for its agent's new key exactly
  as it vouched for the first.

What it is for: a planned change, such as moving the key to the system's
keychain, or retiring a key that may have been seen while it is still in this
machine's hands. What it is not for: a key already stolen. Whoever holds the
old key can rotate it too, so recovering from a theft is still removing the
machine and confirming a fresh one, which the person does by looking.

## The cloud: one carrier, chosen once

Designed, not built. Syncing through an API — Google Drive first, then OneDrive
and Dropbox — reaches computers without the provider's client and, later, a
phone. Nothing about it is required: Tisty works whole without it, and taking
it away gives back a complete application.

**The pattern is the one notices already use.** `herald::Channel` is a trait
with sibling implementations (`Screen`, `Chime`) and `Heralds` picks them once;
nothing else asks which ones exist. Carrying gets the same shape with a
cardinality of one:

| Level | Trait | Implementations |
| --- | --- | --- |
| Notice (several at once) | `herald::Channel` | `Screen`, `Chime`, later a phone |
| Carry (exactly one) | `Carrier`, picked by `chosen()` | none, `Folder`, `Cloud` |
| Speak to a provider | `Remote`, inside `Cloud` | Drive, OneDrive, Dropbox |

`chosen()` is the only `match` on the way of syncing. Today the folder is asked
about in some thirty places; they move behind `Carrier` first, with no change
in behaviour, and only then does a second carrier exist. A rule in `rules.sh`
keeps `Sync::Folder` and `Sync::Cloud` from being named anywhere else.

**The cloud is the same folder, reached through an API.** The same
`tisty.toml`, `store/`, `docs/`, `attachments/` and `.store-id`: one format,
one engine, and the folder as the reference every test is checked against.
The configuration says `Sync::Cloud { provider, account }`; a build without
that variant reads it as `Unknown` and leaves it exactly as it found it.
Folder and cloud are never mixed over the same store, in any provider: moving
between them is the only road, and it is cheap because the format is the same.

**A hybrid mirror.** History, documents and `tisty.toml` go through a local
mirror of the remote tree, so the round keeps reading files as it does today: a
quiet round is one request for changes and no content at all. Closed segments
never change and every machine writes only its own directory, so the mirror is
cheap. Attachments go straight to the provider instead: mirroring them would
copy a large file onto the same disk and then let the original go on the
strength of a copy that never left it. The mirror lives outside the cache a
restore empties, outside a backup's walk and outside what Maintenance weighs.

**Nothing installed that was not checked.** The carrier downloads once into the
mirror, checks what it downloaded, and installs exactly those bytes. The round
never checks one read and copies from another.

**The behaviour lives once, in `Cloud`.** The mirror, the rounds, deferral,
budgets, caps, moving in stages, reconnecting, Maintenance and `tisty doctor`
are written and tested once. A `Remote` only translates verbs and declares its
numbers, so the three providers behave the same and the same suite runs against
all three, changing nothing but the numbers.

- **Class 1**, which every provider has: `list`, `fetch` from an offset, `put`
  with an `Expect` (`Absent`, a revision, or anything), `delete`, and `hash_of`,
  the provider's own fingerprint computed locally — `content_hash`,
  `quickXorHash`, MD5 — so a landing is checked without downloading it.
- **Class 2**, which defaults to class 1: `about`, `changes` since a cursor, and
  `append`.
- **Class 3**, optional: hearing changes pushed, and lending a link.
- **`limits()`**: the daily budget of requests and bytes, the shortest polling
  interval, the chunk size and the provider's own cap per file. `Cloud`
  enforces them; the `Remote` only states them.

**A refusal says which one it is.** A `Hitch` is the provider's limit (with how
long to wait), this installation's own budget spent, the person's storage full,
authorization lost (reconnect) or a different account than the one chosen. Each
has its own place in Maintenance, because each asks the person for something
different. A provider's 403 or 429 is waited out, with `Retry-After` where it
is given.

**Deferred is not failed.** `Moved` gains what was left for later and when it
is tried again, apart from `Trouble`. A round that stops at a budget has
written everything locally, lost nothing and asks nothing of the person.

**Budgets, because a quota is shared.** In Google the quota belongs to the
project, so every person spends from the same pool, and nobody's round may
spend everyone else's. Each installation keeps its own daily budget of
requests and bytes, counted locally and shown by `tisty doctor`; reaching it
stops the round with a notice, and the local store carries on. The caps a
person sees — per attachment and per store — are the same in all three
providers. Their numbers are still to be decided (C-1 to C-3); the mechanism is
not.

**Moving in stages.** Leaving a carrier is `backs_up`, which also downloads the
attachments this machine does not have; entering one is `adopt`. Both take a
budget, keep their progress locally per file, and accept «carry on from here»,
so a move larger than a day's budget goes over several days and survives a cut.
The new destination does not replace the old until it holds everything: copy
first, then switch. In the cloud `adopt` is a single creation of the root
`tisty.toml`, written only if absent; if another machine adopted first, its
file is read instead.

**What the person asks for goes first.** Opening an attachment that lives only
in the cloud (`held`) goes ahead of a move and of polling.

**Drive works by id.** It allows two files with one name and has no
create-if-absent, so its `Remote` keeps a map from name to id and emulates
`Expect::Absent`, for example by reserving ids before creating. Whether that
holds, and whether `drive.file` is shared between the desktop and phone
clients of one project, is checked against Google itself before the format is
closed.

## Where things live

| | Location | Synced |
|---|---|---|
| Events | `<data>/store/<device>/` | yes |
| Attachments | `<data>/attachments/` | yes |
| Documents | `<data>/docs/` | yes, by three prints and no clock |
| Attachment ledger | `<data>/attachments.jsonl` | **no** — local and rebuilt on demand |
| Carried prints | `<data>/carried.json` | **no** — what this machine last carried, and to which folder |
| Merge bases | `<data>/carried/` | **no** — the body each print stands for |
| Highest name given out | `<data>/docs/.spent-<device>` | **no** — so a name is never reused |
| What a write replaced | `<data>/originals/`, its print in `<data>/originals-at/` | **no**, but `originals/` is in a backup |
| Keys answered for | `<data>/.keys-confirmed` | **no** — what the person at this machine confirmed |
| Folder shapes met | `<data>/.shape-seen` | **no** |
| Retired attachments | `<data>/bin/` | **no** — thirty days of grace |
| Settings and device id | `<config>/config.toml` | **no** |
| Signing key and store secret | `<config>/private/` | **no**, and never in a backup |
| The program itself | `%LOCALAPPDATA%\Programs\Tisty` and friends | **no** |
| Read cache | `<cache>/read.db` | **no** |
| Last listing | `<cache>/selection.json` | **no** |

The **guide** is the one thing Tisty writes into your store on its own. It ships
inside the program, in the language you chose, and the welcome copies it in: a
document under `<data>/docs/` and its images under
`<data>/attachments/`, indistinguishable afterwards from anything you wrote and
deletable the same way. Nothing is downloaded: the words and the images travel
inside the program.

`<data>`, `<config>` and `<cache>` are the platform's own directories, except
on Windows, where all three live under `%USERPROFILE%\.tisty`. The Store
version is packaged, and a packaged app's `AppData` is a private copy Windows
deletes when the app is uninstalled; the profile root is not virtualized, so
every install — Store, loose, the command line and the assistant's door —
shares one real store that outlives any of them. A name that starts with a dot
hides nothing on Windows, so the window sets that home aside on every start
(`paths::home_set_aside`): cut off from what it would inherit and kept to this
account first, as the key folder is, and hidden last, so a hidden home is one
already kept and every start after the first only looks. A failure is said and
tried again on the next start, and never keeps the window from opening.
`TISTY_DATA`, `TISTY_CONFIG` and `TISTY_CACHE` override them, and exist for
tests.

Only the window moves the store out of `%LOCALAPPDATA%\tisty`, once, before it
resolves its paths and never while an override or a profile is set; the
command line and the assistant's door use whichever root exists, so nothing
races at sign-in and nothing slow sits in front of an assistant's handshake.
The window holds every device's write lock while it copies, so an older Tisty
still writing makes it stop and say so instead of losing what it writes, and a
move that fails closes the window with a message rather than open an empty
store. Only `data` and `config` move, as one view: the real folder with the
Store app's private copy laid over it file by file, which is exactly what the
packaged app read — so a Store app updated in place, a loose install put in
before the Store app is removed, and the Store app put in over a loose install
all arrive with what the person last saw. Everything goes into a `.part` folder
renamed into place. A start that has a store to move (`paths::home_moves`) opens
a small `moving` window first and copies on a thread of its own, telling the
window the bytes carried against the bytes weighed, so a large store never looks
like a window that will not open; only once the move has settled does the
session open, and the main and quick windows, which loaded with nothing to
answer them, start over. The old folders are left as they were, with a `MOVED.txt`,
except for one line appended to each active segment: an event at a version no
build will ever reach, so an older Tisty refuses that store instead of writing
where nobody reads. That line is never copied along and never written twice.
Since every install now shares the settings, leaving one sweeps its cache but
keeps `config.toml`.

What it costs: a Store app on 1.23 removed before a 1.24 is put in takes its
data with it, as it always did — install the new one first. Using a loose
install and the Store app side by side before moving keeps the Store app's copy
of any file both touched; the loose one's stays in the old folder. And a profile
that roams carries `.tisty` with it, which suits a person's own machines but not
a domain that roams one profile across several.

The **configuration** never syncs, and that is what matters: if two machines
shared a device id they would write to the same file and every guarantee above
would stop holding. It is also why the configuration stays out of a backup.

A setting this build has no name for is kept exactly as it was read and written
back with the rest (`Config::rest`, flattened), so one run of an older build
never erases what a newer one wrote. The way of syncing is `config::Sync` —
`Local`, `Folder(path)`, or `Unknown`, which holds a way a later build knows
untouched: refusing it would stop this build opening at all. A way this build
does know and cannot make sense of is still refused out loud.

The device id itself does travel, and has to — it is the name of the directory
and the `by` field of every event, which is what tells the writers apart. What
must never travel is the file that says *«this machine is that id»*. That file
lives in the local config directory, never a roaming one: a Windows domain
profile copies `%APPDATA%` to a company server at logoff, and it would take the
device id and the `private/` folder with it.

Since the Windows store moved to `%USERPROFILE%\.tisty`, a roaming profile or
a home copied to a new computer can still carry that file along, so the
configuration also keeps `inst`: a digest of what the operating system calls
this computer — `MachineGuid` on Windows, `IOPlatformUUID` on macOS,
`/etc/machine-id` on Linux — never the identifier itself (`machine.rs`). It is
read again every time the configuration loads. When it no longer matches, the
configuration was last used on another computer, and this one stops speaking as
that machine. A computer it has woken on before takes back the device id and
the agent it had there, kept in `homes`, so a profile roaming between two
desks does not mint a name at every logon. One it has never seen takes a device
id derived from the new `inst` and the old id, so that the window and the
terminal waking at once agree on it, joins like any new machine and waits to be
confirmed; its agent stays off until the person turns it on, because minting
one is theirs. Saving the result is best effort: a configuration that cannot be
written is worked out the same way next time.

`inst` is not the computer's name, and the two stay apart. The name
(`called.rs`) is for people: the person can change it, and it travels to the
other machines in `device.named` so a machine waiting to be confirmed is found by
it. `inst` is for this computer alone: it never leaves the configuration, never
changes when the computer is renamed, and is a digest nobody reads. Two
computers can share a name; only a cloned disk, below, makes them share an
`inst`.

What it cannot catch: a configuration written before `inst` existed takes the
computer it wakes on as its own, so a copy made before this check shipped goes
unnoticed; and a Windows or Linux disk cloned without sysprep or a fresh
`machine-id` carries its identifier with it, so both computers answer the same.
A Mac's identifier belongs to the hardware and does not travel.
