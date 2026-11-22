# How an agent works

What tired me most about working with agents was not explaining the problem: it was reminding them, answer after answer, to tick what they did and note what they ruled out. Instructions alone were not enough. So now Tisty asks the agent when it forgets, gives it a cheap way to leave the task up to date, links what it names and serves it everything you keep as an index it can walk without reading it whole.

This page follows a real agent session, step by step, with the calls it makes and an excerpt of what Tisty answers. Connecting one is in [[Agents and MCP|Agents-and-MCP]]; every tool, in [[Tools reference|MCP-Tools]]; what it may and may not do, in [[What an agent may do|MCP-Permissions]].

## What changed with Brainsty

- **It finds and walks with a single tool, `brain`.** It finds tasks and documents by their words, with the archive beside the rest; brings a document's or a task's card with its neighbours; and shows a whole project grouped as blocked, under way, waiting for you and not started.
- **It reads with a single tool, `read`.** A task or a document, whole or in parts, only what is highlighted, or an earlier version; and if nothing changed since its last read, only that.
- **It leaves the task up to date in one call.** `tick` ticks the steps and writes the task's state, a «State» section of its page that is replaced whole each time. When it picks the task up, `read` brings that state first.
- **Tisty asks it when it forgets.** In its answers, to any agent; and with the hooks, when each turn starts and ends.
- **What it names gets linked.** A task or a document named by its id or by its exact title between quotes becomes a link when saved, just as if you had made it.

## Before you start

In **Settings › Agents**:

1. On the **Connected** tab, **Connect** puts Tisty in the agent's settings. See [[Agents and MCP|Agents-and-MCP]].
2. Right there, under a connected agent (Claude Code, Cursor, Gemini CLI or GitHub Copilot), the **Hooks: when a turn starts and ends** row says **Not added**. **Add** tells you which file it will write; **Add them** puts them there. From its next session, the agent uses them.
3. On the **Door** tab, two switches, on from the start:
   - **Ask it to keep its task up to date**: the question in the answers.
   - **Remind it when a turn starts and ends**: the hooks, without taking them out of the file.

## A session, step by step

Mario works with Claude Code on the task «Payments through the new gateway», in the **Work** list, which has four parts. Today it is «Confirmation webhook»'s turn, not started yet, and the «Refunds» part was left blocked waiting on the provider.

### 1. It starts knowing where things stand

With the hooks, each session starts with the last open tasks your agents wrote on on this computer, how many steps they have and the next one, without the agent having to ask. This is how tomorrow's session starts, after the work this page tells:

```json
"Your last open tasks here, newest first:
- \"Confirmation webhook\" in Work […], last written 2026-11-18 22:50: under way, 1 of 3 steps ticked, next \"Retries\"; state: **Done:** the HMAC signature is checked…
- \"Refunds\" in Work […], last written 2026-11-18 22:50: blocked, 0 of 2 steps ticked, next \"Full refund\"; state: Blocked: the provider…
Pressing for the person: 1 today, 28 late."
```

Without hooks, or to see everything, the agent calls `catch_up`: the lists, the folders, the most used tags, what is pressing and `last`, the task it was on, with how far it got: its stage, its steps and the first line of its state. The start with hooks says the same in words. (The stage and state, here and in the start with hooks above, came after this session, and take the shape the code gives them.)

```json
{ "last": { "title": "Refunds", "list": "Work", "at": "2026-11-18 22:50",
            "stage": "blocked", "steps": [0, 2], "state": "Blocked: the provider turns partial refunds on…" },
  "pressing": [ { "title": "Water the balcony plants", "list": "Home", "standing": "today" }, … ],
  "counts": { "open": 54, "late": 28, "today": 1, "docs": 62 } }
```

### 2. It searches with `brain`

`brain` with a few words searches the open tasks, the archive and the documents at once. First what holds the words in its title; then what moved more recently. Each one comes with the line that says them: here the task matched in one of its steps.

```json
> brain {"query": "webhook signature"}
{ "matches": [ { "title": "Confirmation webhook", "list": "Work", "status": "open", "part_of": "…", "in": "step", "line": "Check the signature" } ],
  "docs": [ { "title": "Payments: decisions", "line": "The signature is checked before the body is read." } ] }
```

When nothing holds every word, it brings what holds the most and says so (`every_word: false`), instead of bringing nothing. `brain` with nothing shows your lists, folders and tags with their counts.

### 3. It looks at the whole project

With `part_of`, `brain` groups a task's open parts by how they are going. A part whose state starts with «Blocked:» counts as blocked.

```json
> brain {"part_of": "…Payments through the new gateway…"}
{ "stages": { "blocked": 1, "under_way": 0, "waiting": 0, "not_started": 3 },
  "parts": [
    { "title": "Refunds", "stage": "blocked", "steps": [0, 2],
      "state": "Blocked: the provider turns partial refunds on in the test environment only on Monday." },
    { "title": "Confirmation webhook", "stage": "not_started", "steps": [0, 3] }, … ] }
```

### 4. It reads the task, and of the document only what matters

`read` brings a task with its state first. A task that has none yet comes with one built from it:

```json
> read {"task": "…Confirmation webhook…"}
{ "state": { "built": true, "done": 0, "of": 3 },
  "steps": [ { "text": "Check the signature", "done": false }, { "text": "Retries", "done": false }, … ] }
```

A document's card says what it holds and where, without reading it: its sections with their lines, who mentions it and, in a decisions section, the ones in force first and the struck one last, marked as replaced.

```json
> brain {"doc": "…Payments: decisions…"}
{ "decisions": [
    { "line": 6, "said": "The webhook is retried three times, with a growing wait." },
    { "line": 7, "said": "The signature is checked before the body is read." },
    { "line": 5, "said": "Retry the webhook without limit", "replaced": true } ],
  "mentioned_by": [ { "title": "Gateway: integration notes", "kind": "doc" }, … ], "mentioned_by_more": 15,
  "outline": [ { "title": "Decisions", "line": 3, "to": 8 }, { "title": "Out of scope", "line": 10, "to": 14 } ] }
```

A document's or a task's card also brings what Tisty thinks is alike, apart, under `guessed`: nobody linked it, so the agent reads both before joining or merging anything.

```json
> brain {"task": "…Ask for a quote on the roof…"}
{ "guessed": [ { "id": "…", "kind": "doc", "title": "House repairs", "state": "open", "score": 0.42, "shared": ["quote", "roof"], "method": "words" } ] }
"Guessed alike by Tisty, linked by no one: \"House repairs\" […] (open) (shares quote, roof)."
```

In `shared`, a word alone is one both of them wrote. An `a≈b` pair is a word each side wrote differently, and `method` says which of the three clues tied them: `words`, shared words; `typo`, a typo or a plural, like `impresora≈imrpesora` or `vacuna≈vacunas`, and `dictionary`, a translation from Tisty's Spanish–English dictionary, like `pagar≈pay`. The sentence below says the same in words: «shares», «spelled almost the same:» or «alike by translation:». That way the agent knows «Pay the water bill» and «Pagar la cuenta del agua» are alike by language, not because someone tied them, and reads both before deciding whether they are the same task:

```json
> brain {"task": "…Pay the water bill…"}
{ "guessed": [ { "id": "…", "kind": "task", "title": "Pagar la cuenta del agua", "state": "open", "score": 0.61, "shared": ["pagar≈pay", "agua≈water"], "method": "dictionary" } ] }
"Guessed alike by Tisty, linked by no one: \"Pagar la cuenta del agua\" […] (open) (alike by translation: pagar≈pay, agua≈water)."
```

Alike pairs come from prose only: code, HTML, paths, a task's progress notes and the documents in Agents do not count. [[Alike|Brainsty-Network#alike]] tells it.

With each section's lines, the agent reads only the one it needs, with the same tool that reads a task: `read` with `doc` brings the whole document, one section (`section`), some lines (`from`, `to`) or a piece at a time (`chars`). With `marks`, it brings only what the document highlights or sets in bold, grouped by section: on a 24 KB document, about 2,300 bytes. With `versions`, in place of the text, it lists its earlier versions on this computer, and `version` reads one, under a notice that it is not what it says now; `restore_doc` with that `version` goes back to it. It never empties the history or reads your Diary's versions. [[Earlier versions|Documents#earlier-versions]] and [[MCP tools|MCP-Tools]] tell it.

**Reading again costs almost nothing.** Every read comes with a `print`. If the agent reads again with that `print` and nothing changed, Tisty answers only that, with the time of its earlier read. The answer takes this shape (this excerpt does not come from the session above, but from Tisty's code):

```json
> read {"task": "…", "print": "…"}
{ "id": "…", "status": "open", "unchanged": true, "print": "…" }
"Nothing on \"Confirmation webhook\" changed since you read it, 22:50: what you hold still stands. Without `print`, `read` brings it whole again."
```

Measured on a store of 5,000 tasks, reading a task again this way costs 336 bytes instead of 4,360, and a whole document 332 instead of 9,321. If something changed, a ticked step or a new line, it brings everything again. A section always comes with its text.

### 5. It ticks and leaves the state in one call

When it finishes a step, the agent ticks it and writes the task's state in the same call: what is done, what is left, the decisions in force and what was ruled out.

```json
> tick {"task": "…", "steps": ["Check the signature"],
        "state": "**Done:** the HMAC signature is checked before the body is read…\n**Left:** retries and logging every event.\n**Decisions in force:** three retries with a growing wait.\n**Ruled out:** retrying without limit."}
{ "ticked": 1, "left": 2, "next": "Retries", "stated": true,
  "said": "Ticked 1 step(s) on \"Confirmation webhook\". 2 still unticked; next: \"Retries\". Left its state on its page, under «State»." }
```

The task's page shows the **State** section, between the steps and the notes, with the date and the agent's name. You read it just as it does. Each write replaces it whole, so it never grows.

- If you wrote a line of your own under **State**, the agent has to keep it: a state that leaves it out is refused and ticks nothing.
- If the steps change or notes arrive afterwards, `read` says the state is old: «older than 3 later note(s)».
- The state has room for about 3,000 characters.

### 6. It notes only what changes something

The journal is for what changes something: a decision, something ruled out or a block. Tisty never asks it to «note your progress»: that is in the state already. When a note undoes an earlier one of the agent's, the new one says so with `replaces`, and the old one stays in the history, dimmed and marked as replaced, and stops coming first.

### 7. What it names gets linked

```json
> note {"task": "…", "body": "Retries go on the job queue that already exists, as «Payments: decisions» says. Reconciliation waits on this part: 01M7RJAQKHGE785QFPK3NHK1CV"}
{ "references": { "linked": [
    { "said": "Payments: decisions", "doc": "…", "title": "Payments: decisions" },
    { "said": "01M7RJAQKHGE785QFPK3NHK1CV", "task": "…", "title": "Daily reconciliation" } ] } }
```

The note ends up with two links, the same you would make with `[[` and a task's. What is doubtful is not linked: it goes back to the agent to decide. A one-word title, like «Car», can sit in a sentence without meaning to name anything:

```json
{ "references": { "maybe": [ { "said": "Car", "could_be": [ { "doc": "…", "title": "Car" } ] } ] } }
```

A task id alone in its paragraph, in a document, becomes that task's card. Headings, tables, code and lines that were already there are never touched.

### 8. A picture without words gets a question

```json
> attach {"task": "…", "path": "…\\Pictures\\IMG_2041.jpg"}
{ "ask": { "said": "What does \"rina-ce960510.jpg\" show? The words in a picture's brackets are all an agent or a screen reader knows of it, and a file's name says nothing: write it between the brackets with `reword_note`…" } }
```

A file's name does not count as a description. What the agent writes between the brackets is all another agent, or a screen reader, will know of that picture.

### 9. If it forgets, Tisty asks

The agent went on with other things: it added to a document and made two folders, without going back to the task. The third answer brings the question:

```json
> folder {"name": "Tests", "inside": "Work"}
{ "ask": { "title": "Confirmation webhook", "left": 2,
  "said": "Is \"Confirmation webhook\" up to date? You wrote 3 other things without writing on it, and 2 of 3 steps are still unticked, the next \"Retries\". Leave it in one `tick`…" } }
```

It asks once per stretch: after ten minutes without a word on the task, or at the third write elsewhere. It asks again only after the agent writes on that task.

### 10. When the turn ends

With the hooks, when a turn edited files and wrote nothing in Tisty, the turn goes back to the agent once:

```json
{ "decision": "block",
  "reason": "You edited files this turn and wrote nothing in Tisty. If this was work on a Tisty task, tick the steps you finished; if the work is done, `say_done` with the proof. A decision, something ruled out or a block goes in `note`. If it was not work on a task, end your turn without writing anything." }
```

If the agent lets it pass, the hook stays quiet until it calls Tisty again: a turn outside a task costs one extra turn at most.

At the end of the session, `brain` with `part_of` already shows «Confirmation webhook» **under way**, with 1 of 3 steps and the first line of its state.

## What you see

- **On the task's page**, the **State** section with the date and the agent, and in the notes the links to «Payments: decisions» and «Daily reconciliation».
- **In the document**, «Gateway: integration notes» shows the card of the part the agent cited.
- **In Linked from**, «Payments: decisions» now names the task.
- **In the Brainsty network**, the new lines reach the base a few seconds later, and you see them the next time you open the network.
- **Nothing closes without you.** When the agent says a task is done, it waits in **To confirm**.

## The tools that moved into `brain` and `read`

| Before | Now |
| :-- | :-- |
| `find` | `brain` with `query`, or with the fields to sift by (`list`, `tag`, `from`, `to`, `by_agent`, `said_done`, `part_of`, `scope`); `source` still tells whether something was filed already |
| `docs` | `brain` with `folder`, or with nothing to see the folders with their counts |
| `outline_doc` | `brain` with `doc`: the card, with the outline, the `print` to edit by section and who mentions it |
| `lists`, `tags` | `brain` with nothing |
| `read_doc` | `read` with `doc`, with the same parts (`section`, lines, `chars`, `as`) plus `marks`, `versions` and `version` |

An agent that still calls one of them is told where it went («`find` moved into `brain`», «`read_doc` moved into `read`») and nothing else happens. When every row of a list shares a field, like the list or the task they are parts of, it comes once, in `each`.

## Good to know

- **Agents without hooks.** Codex and Claude Desktop run no hooks: they get only the question in the answers, and only when they call Tisty.
- **Work that edits no files**, like planning or researching, is not seen by the end-of-turn hook: there the question in the answers remains.
- **GitHub Copilot** may spend a premium request on the turn that is handed back.
- **Cursor** also runs Claude Code's hooks; even if both fire, the turn goes back only once.
- **Gemini CLI** runs hooks only in the folders you trusted.
- **What is hidden stays out.** A task you hid or dropped is never named: not in a search, not as a neighbour on a card, not as an alike pair, not at the start of a session. Your Diary, while you do not open it to agents, neither.
- **Where you linked it, it reads «a task out of sight».** If a document cites or mentions one of those tasks, the agent reads it with those words in place of the link, with no title or id. If it later rewrites the document, Tisty puts your link back as it was, byte for byte: what is hidden is not lost because the agent did not see it.
- **In the Agents folder it only proposes.** It reads your rules and skills like any document, but a change of its own stays as a proposal until you approve it on the card. [[A proposal from your agent|AGENTS-Folder#a-proposal-from-your-agent]] tells it.
- **With the door closed**, the tools refuse everything and the hooks stay quiet.
- **What it costs.** A note has room for 4,000 characters and the state for about 3,000; if the links would push it past that, it stays unlinked instead of being refused. The start with hooks weighs about 2,000 characters.

## Where next

- [[Agents and MCP|Agents-and-MCP]]: connecting an agent, the ready-made requests and what it costs.
- [[Tools reference|MCP-Tools]]: the full list.
- [[The Brainsty network|Brainsty-Network]]: the same network, drawn for you.
- [[Tasks in documents|Tasks-in-Documents]]: the cards your agent leaves too.
