# Step by step with your AI agent

This sheet is a page of the guide. It tells you how to work with an AI agent in Tisty: how to connect it, how to let it file work or stop it, how to keep it from forgetting to write things down, what it can do and what it cannot, and how to ask it for things.

## Connecting it

**Settings › Assistants › Connected** finds the assistants you have installed, such as Claude Code, Codex, GitHub Copilot, Cursor or Gemini CLI, and says how each one stands:

- **Connected**: it already opens this Tisty.
- **Not connected**: **Connect** writes Tisty into that assistant's settings, and touches only its own line.
- **Points at another Tisty**: an older copy, or one that moved. **Fix** points it here.
- **May be here**: only its folder is left, so it may no longer be installed.
- **By hand**: it keeps its settings in a format Tisty does not write, and the row shows what to paste.

**Connect the ones found** connects every one waiting in one go. If yours is not there, the search under the list finds it among the assistants Tisty knows.

Then quit the assistant completely and open it again: it reads its settings when it starts. If you would rather do it by hand, with a plugin or inside WSL, every way is in [assistants and MCP](https://www.tisty.app/docs/assistants-and-mcp/).

## Letting it file work or stopping it

Tisty leaves your agent a door, and you hold it. It comes open when you install, and in **Settings › Assistants › Door** you close it with **Stop it**: while it is closed, everything an assistant tries is refused, and it reads and writes nothing. **Let an assistant file here** opens it again. If one tries to come in while the door is closed, the window tells you ("Claude Code tried to come in") and **Let it in** opens it right there.

Nothing listens until you connect one: the assistant starts Tisty when it needs it, on this computer, with no port and no account.

> [!WARNING]
> Whatever an assistant reads, it takes to the service it works with, and that includes what reaches it when each session starts with the hooks, below. More in [privacy](https://www.tisty.app/docs/privacy/).

## So it does not forget to write things down

An agent deep in code forgets about Tisty. Two things help, and you can turn both off in **Settings › Assistants › Door**:

- **Ask it to keep its task up to date**: if it keeps working and the task it opened has gone ten minutes without a word from it, or while it writes three other things, Tisty's next answer asks it once to tick what it did and, if it finished, to say so with the proof. It reaches every assistant, though only when it calls Tisty.
- **The hooks**: in Claude Code, Cursor, Gemini CLI and GitHub Copilot, **Add**, under the row of the connected assistant, tells you which file it will write and puts Tisty's hooks there. From its next session on, and while the door is open, every session starts with the last open tasks your agents wrote in, what moved and what is pressing, even if the agent never calls Tisty; and if a turn edited files without writing anything in Tisty, it hands the turn back once so it ticks what it finished. **Remove**, beside the hooks, takes them out, and **Remind it when a turn starts and ends** turns them off without taking them out.

## What it reads: Brainsty

Your agent does not read everything you keep: it walks it. With a single tool, `brain`, it finds tasks and documents by their words, the Archive beside the rest and first what holds those words in its title. It asks for a document's card (its sections with their lines, who cites it and its decisions in force first) and then, with `read`, reads only the section it needs, or only what you highlighted or set in bold. If it reads again something that did not change, Tisty tells it only that, without sending the text again. And it sees a whole project grouped as blocked, under way, waiting for you and not started: a part whose state starts with «Blocked:» counts as blocked. On the card it gets apart, under `guessed`, what Tisty thinks is alike and nobody linked, with the words that say so: it reads both before joining anything. It is the same network you see drawn in **Brainsty: everything joined**, a page of this guide.

When it picks a task up, the first thing it gets is its **State**, the one you see on its page, and only the notes written after it. If the steps changed or notes arrived after the state, Tisty tells it the state is old. Every call, with an excerpt of what Tisty answers, is in [how an agent works](https://www.tisty.app/docs/how-an-agent-works/).

## What it can do

- Propose tasks with their steps, their parts and their date, set a reminder on them and note in their journal.
- Tick the steps it finished and, in the same call, leave the task's **State** on its page: what is done, what is left, what was decided and what was ruled out. You read it too, and the next session starts from there.
- Work any task as it works a document, yours too: its description, its page, its title, its list and its open steps.
- Write documents, add to one, correct a passage or a table, and file them into folders and pages.
- Name a task or a document by its exact title in quotes: Tisty turns it into a link, as when you do it, and hands back what is doubtful for it to decide.
- Jot ideas in Ideas when you ask, and sort them out with you: an idea it turns into a task or files in a document leaves the sheet, as it does when you do it.
- Mark a document or an idea that had its day, saying why: you delete it, put it away or take the mark off.
- Say that a task or a step is done: the task stays in **To confirm** until you finish it or send it back with **Still to do**; the steps wait for you with **Tick them** or **Not yet**.
- Search and read what is there, the Archive included.
- Read your Diary and add to the end of today, signed, only if you open it to agents from the Diary itself and ask it to.

Everything it writes is signed with the name of the program it spoke through: "by Claude Code", "by Codex". In **Settings › Assistants › How its signature looks** you choose where that name shows, or whether only a small ◆ is left.

## What it cannot do

- Close, reopen or delete a task, or delete a document or an idea.
- Tick a step of yours or finish a task of yours: it only says they are done, and you take its word or not.
- Move a day you set.
- Write on its own in the Agents folder: a change to your rules or skills stays as a proposal, and reaches your agents only if you approve it on the card. **Agents: your agents' rules**, a page of this guide, tells it.
- Empty a document's history or read the earlier versions of your diary. It can list a document's versions, read one and go back to it, as you do from **See the history**; [this guide's cover](tisty:guide) tells it, under **Going back to what it said before**.
- Read a task you hid or dropped: where you linked it, it reads only «a task out of sight».
- Create lists or set routines: without one of your lists, what it files lands in the inbox.
- Write over what you changed while it was thinking: it is asked to read again.
- See your Diary while it reads **Only you**, or change what you wrote in it once you open it.

For closing and deleting there is no tool: not hidden, absent. The rest is refused or taken as its word, and it is told why. Every rule is in [what an assistant may do](https://www.tisty.app/docs/mcp-permissions/).

## How to ask it

Talk to it as you would to a person: "note this as a task for Friday", "keep this decision in the project's document", "how did we solve this last time?". Tisty also hands it eight ready-made requests your assistant shows as commands; in Claude Code they are in the `/` menu:

| Request | What it does |
| --- | --- |
| `pick_up` | Picks up where it left off: what moved and the task it was on |
| `leave_off` | Leaves the task ready to pick up: steps ticked and its state |
| `plan_task` | Plans a piece of work as a task with steps |
| `solved_before` | Searches the Archive for how something was solved before |
| `keep_decision` | Keeps a decision in the document on its subject |
| `brainstorm` | Brainstorms with you and jots the ideas you keep |
| `sort_ideas` | Sorts out the ideas waiting, changing nothing until you say |
| `consolidate` | Goes over what Tisty keeps, a folder, a list or a subject at a time, and proposes fixes: stale summaries, links that lead nowhere, doubled documents and states left behind. It makes only the ones you approve and never deletes |

Each one takes a word after it. `/mcp__tisty__consolidate Kitchen` goes over the Kitchen folder alone: your agent shows you what it found, numbered and with the fix it would make; you answer «1 and 2» and it makes only those two. Nothing old is ever deleted: it is marked, and you delete it. A whole pass, call by call, is in [assistants and MCP](https://www.tisty.app/docs/assistants-and-mcp/).

And with `@` you hand it a document by its title.

## What is best avoided

- Asking it to delete: it marks what had its day, and deleting is yours, in one click.
- Expecting it to close a task: when it says it finished, the task waits for you in **To confirm**.
- Connecting the same assistant two ways, with the plugin and from Settings: it would see every tool twice.
- Forgetting to restart the assistant after connecting it: it reads its settings when it starts. The hooks, instead, run from its next session.

What each tool does is in [MCP tools](https://www.tisty.app/docs/mcp-tools/).
