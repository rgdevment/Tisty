# Step by step with your AI agent

This sheet is a page of the guide. It tells you how to work with an AI agent in Tisty: how to connect it, how to let it in or keep it out, what it can do and what it cannot, and how to ask it for things.

## Connecting it

**Settings → Assistants** finds the assistants you have installed, such as Claude Code, Codex, GitHub Copilot or Cursor, and says how each one stands:

- **Connected**: it already opens this Tisty.
- **Not connected**: **Connect** writes Tisty into that assistant's settings, and touches only its own line.
- **Points at another Tisty**: an older copy, or one that moved. **Fix** points it here.

Then quit the assistant completely and open it again: it reads its settings when it starts. If you would rather do it by hand or with a plugin, every way is in [assistants and MCP](https://www.tisty.app/docs/assistants-and-mcp/).

## Letting it in or keeping it out

Tisty leaves your agent a door, and you hold it. It comes open when you install, and **Settings → Assistants** closes it with one click: while it is closed, everything an assistant tries is refused, and it reads and writes nothing. If one tries to come in while the door is closed, the window tells you ("Claude Code tried to come in") and **Let it in** opens it right there.

Nothing listens until you connect one: the assistant starts Tisty when it needs it, on this computer, with no port and no account. What is worth knowing: whatever an assistant reads, it takes to the service it works with. More in [privacy](https://www.tisty.app/docs/privacy/).

## What it can do

- Propose tasks with their steps and their date, set a reminder on them and note in their journal.
- Work any task as it works a document, yours too: its description, its page, its title, its list and its open steps.
- Write documents, add to one, correct a passage or a table, and file them into folders and pages.
- Jot ideas on Brainstorm when you ask, and sort them out with you: an idea it turns into a task or files in a document leaves the sheet, as it does when you do it.
- Mark a document or an idea that had its day, saying why: you delete it, put it away or take the mark off.
- Say that a task or a step is done: the task stays in **To confirm** until you finish it or send it back with **Still to do**; the steps wait for you with **Tick them** or **Not yet**.
- Search and read what is there, the Archive included.

## What it cannot do

- Close or delete a task, or delete a document or an idea.
- Tick a step of yours or finish a task of yours: it only says they are done, and you take its word or not.
- Move a day you set.
- Write over what you changed while it was thinking: it is asked to read again.

There is no tool for the forbidden things: not hidden, absent. Every rule is in [what an assistant may do](https://www.tisty.app/docs/mcp-permissions/).

## How to ask it

Talk to it as you would to a person: "note this as a task for Friday", "keep this decision in the project's document", "how did we solve this last time?". Tisty also hands it seven ready-made requests your assistant shows as commands; in Claude Code they are in the `/` menu:

| Request | What it does |
| --- | --- |
| `pick_up` | Picks up where it left off: what moved and the task it was on |
| `leave_off` | Leaves the task ready to pick up: steps ticked and what comes next |
| `plan_task` | Plans a piece of work as a task with steps |
| `solved_before` | Searches the Archive for how something was solved before |
| `keep_decision` | Keeps a decision in the document on its subject |
| `brainstorm` | Brainstorms with you and jots the ideas you keep |
| `sort_ideas` | Sorts out the ideas waiting, changing nothing until you say |

And with `@` you hand it a document by its title.

## What is best avoided

- Asking it to delete: it marks what had its day, and deleting is yours, in one click.
- Expecting it to close a task: when it says it finished, the task waits for you in **To confirm**.
- Connecting the same assistant two ways, with the plugin and from Settings: it would see every tool twice.

What each tool does is in [MCP tools](https://www.tisty.app/docs/mcp-tools/).
