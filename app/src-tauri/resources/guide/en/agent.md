# Step by step with your AI agent

This sheet is a page of the guide. It tells you how to work with an AI agent in Tisty: how to connect it, how to let it file work or stop it, how to keep it from forgetting to write things down, what Tisty's skills teach it, what it can do and what it cannot, and how to ask it for things.

## Connecting it

**Settings › Agents › Connected** finds the agents you have installed, such as Claude Code, Codex, GitHub Copilot, Cursor or Gemini CLI, and says how each one stands:

- **Connected**: it already opens this Tisty.
- **Not connected**: **Connect** writes Tisty into that agent's settings, and touches only its own line.
- **Points at another Tisty**: a copy that is gone. **Fix** shows you what it will write and, with your yes, points it here. The window tells you too.
- **Uses your other Tisty install**: another copy still on this computer, such as the Store's and winget's side by side. Both work, so nothing changes on its own; **Fix** brings it to this one if you want.
- **Set up your own way**: it holds a Tisty entry that Tisty cannot check, and leaves it as you wrote it.
- **Could not be read**: Tisty could not read its file, so it does not touch it.
- **May be here**: only its folder is left, so it may no longer be installed.
- **By hand**: it keeps its settings in a format Tisty does not write, and the row shows what to paste.

**Connect the 3 found**, with however many wait, connects every one waiting in one go. If yours is not there, the search under the list finds it among the agents Tisty knows.

When you connect Claude Code, Codex, Cursor, Antigravity CLI, Gemini CLI or GitHub Copilot, its hooks and Tisty's skills, which this page tells further down, come with it and stay on: Tisty shows you the files first, with a tick to leave each one out. What you untick stays off, with its switch under the agent's row. Codex runs the hooks only once you approve them there with `/hooks`, and its row reminds you.

Then quit the agent completely and open it again: it reads its settings when it starts. If you would rather do it by hand, with a plugin or inside WSL, every way is in [agents and MCP](https://www.tisty.app/docs/agents-and-mcp/).

## Letting it file work or stopping it

Tisty leaves your agent a door, and you hold it. It comes open when you install, and in **Settings › Agents › Door** you close it with **Stop it**: while it is closed, everything an agent tries is refused, and it reads and writes nothing. **Let an agent file here** opens it again. If one tries to come in while the door is closed, the window tells you ("Claude Code tried to come in") and **Let it in** opens it right there.

Nothing listens until you connect one: the agent starts Tisty when it needs it, on this computer, with no port and no account.

> [!WARNING]
> Whatever an agent reads, it takes to the service it works with, and that includes what reaches it when each session starts with the hooks, below. More in [privacy](https://www.tisty.app/docs/privacy/).

## So it does not forget to write things down

An agent deep in code forgets about Tisty. Two things help, and you can turn both off in **Settings › Agents › Door**:

- **Ask it to keep its task up to date**: if it keeps working and the task it opened has gone ten minutes without a word from it, or while it writes three other things, Tisty's next answer asks it once to tick what it did and, if it finished, to say so with the proof. It reaches every agent, though only when it calls Tisty.
- **The hooks**: in Claude Code, Codex, Cursor, Antigravity CLI, Gemini CLI and GitHub Copilot, the **Tisty's hooks** switch, under the row of the connected agent, tells you which file it will write and puts Tisty's hooks there. From its next session on, and while the door is open, every session starts with the last open tasks that same agent wrote in, what moved and what is pressing, even if the agent never calls Tisty; and if a turn edited files without writing anything in Tisty, it hands the turn back once so it ticks what it finished. It also hands it back once when a turn left every step of its task ticked without saying it done, never on a task of yours; and after a `git push`, a `gh pr create` or a `gh pr merge` it asks whether it finished a step or the task: in Claude Code at once, in the others when the turn ends. Hooks added before lack these: turn them off and on. Turning it off takes them out, and **Remind it when a turn starts and ends** turns them off without taking them out.

## So it knows how to work: Tisty's skills

Tisty brings three skills that teach your agent to work with it. There are no commands to learn: your agent uses them when you talk to it in your own words.

| Skill | What it teaches | Ask your agent |
| --- | --- | --- |
| `tisty` | Picking up where it left off, working tasks, documents and ideas, and leaving the task ready for the next one | “Pick up where we left off” · “Leave this ready for the next agent” |
| `consolidate` | Tidying what is kept: it proposes each fix and never deletes | “Tidy up my Tisty” |
| `next-agent` | Writing so the next agent understands it without the conversation | “Make this page clear for the next agent” |

You turn them on when you connect an agent or in **Settings › Agents › Connected**, with the switch under each agent or on the **Tisty in your agents** row, at the end of the list: **See** opens a card with the skills and the hooks together, and **Turn on both** turns them on at once, after showing you what it will write. **Adjust**, on the skills, opens their card, which says where they go on this computer and what Tisty reads to write them, and **Turn on here** is their yes. They have their own yes and need no Agents folder: they reach `~/.claude/skills`, for Claude Code, and `~/.agents/skills`, the folder of the Agent Skills standard that Codex, Copilot, Gemini, DeepSeek Harness and Cursor share.

- **They update by themselves** with each Tisty version, and never write over a skill of yours with the same name or a copy you changed.
- **They count only on this computer**: on another one you decide there. Pausing Agents does not reach them: they have their own switch.
- **In a WSL distro**, Settings › Agents › Connected shows «Ubuntu · WSL» with **Agents' rules and skills** and **Tisty's skills**, each with its own yes there. Tisty only writes into it, in LF, never hooks or agents' memory, and nothing comes back from it; a stopped distro waits until it runs.
- **Turn off on this computer** takes them out of your agents' folders, with a backup, without deleting them: they stay in Tisty, ready to turn on again.
- **With the Tisty plugin** in Claude Code, the plugin already brings the hooks and the skills: the hooks row says **The Tisty plugin brings them**, and Tisty does not copy its skills there.

**An example.** Yisset connects Claude Code and leaves its hooks and the skills ticked. The next day she writes “Pick up where we left off”: the agent asks for `catch_up`, opens the task it was on, «Export the monthly report to CSV», and carries on from its **State**. Before she goes, “Leave this ready for the next agent” leaves the steps ticked, the state up to date and a note with what it ruled out. That stays on the task: another day, in Codex, name the task by its title and Codex reads the state and the note right there. “Pick up where we left off”, instead, takes each agent to the last thing it left.

All of it, case by case and with the agent's calls, is in [hooks and skills](https://www.tisty.app/docs/hooks-and-skills/).

## Your own hooks

Besides Tisty's, you can write your own: a script your agent runs at a moment you choose.

1. In the sidebar, **Agents › Global › Hooks**, press **+**. Give it a name, the kind of script (`.ps1`, `.sh`, `.py` or `.js`), the moment (when a session starts, when you send a message, before or after a tool, when a turn ends), the tools and the agents. It starts as **Only this computer**.
2. Write the script on its page, like any document.
3. **Register** shows you the whole script and the files it will touch, and **Yes, register** puts it in each chosen agent. It is one yes per hook and per computer, and any change to its words asks again, on every computer.
4. **Verify** says, agent by agent, whether it is in its file and runs this Tisty. **Try it once** runs the script for real, with a sample of what the agent would send, and tells you how it ended.

The script gets what the agent sends, plus `TISTY_HOOK_EVENT` and `TISTY_HOOK_AGENT`. If it ends with 0, it lets the call through; if it ends with 2 before a tool, it stops it and the agent reads what it wrote to stderr. With the door closed, it does not run. No agent makes or changes a hook: only you, in Tisty.

## What it reads: Brainsty

Your agent does not read everything you keep: it walks it. With a single tool, `brain`, it finds tasks and documents by their words, the Archive beside the rest and first what holds those words in its title. It asks for a document's card (its sections with their lines, who cites it and its decisions in force first) and then, with `read`, reads only the section it needs, or only what you highlighted or set in bold. If it reads again something that did not change, Tisty tells it only that, without sending the text again. And it sees a whole project grouped as blocked, under way, waiting for you and not started: a part whose state starts with «Blocked:» counts as blocked. On the card it gets apart, under `guessed`, what Tisty thinks is alike and nobody linked, with the words that say so: it reads both before joining anything. It is the same network you see drawn in **Brainsty: everything joined**, a page of this guide.

When it picks a task up, the first thing it gets is its **State**, the one you see on its page, and only the notes written after it. If the steps changed or notes arrived after the state, Tisty tells it the state is old. Every call, with an excerpt of what Tisty answers, is in [how an agent works](https://www.tisty.app/docs/how-an-agent-works/).

## What it can do

- Propose tasks with their steps, their parts and their date, set a reminder on them and note in their journal. Each task is the agent's or yours, never both: what only you can do, such as trying it by hand, publishing or merging, it hands to a task of yours, one per task, which never holds its own open.
- Tick the steps it finished and, in the same call, leave the task's **State** on its page: what is done, what is left, what was decided and what was ruled out. You read it too, and the next session starts from there.
- Work any task as it works a document, yours too: its description, its page, its title, its list and its open steps.
- Write documents, add to one, correct a passage or a table, and file them into folders and pages.
- Name a task or a document by its exact title in quotes: Tisty turns it into a link, as when you do it, and hands back what is doubtful for it to decide.
- Jot ideas in Ideas when you ask, and sort them out with you: an idea it turns into a task or files in a document leaves the sheet, as it does when you do it.
- Mark a document or an idea that had its day, saying why: you delete it, put it away or take the mark off.
- Say that a task or a step is done: the task stays in **To confirm** until you finish it or send it back with **Still to do**; the steps wait for you with **Tick them** or **Not yet**, and meanwhile the list counts them apart, with a ◆: **1/3 ◆1**. In each list, its tasks sit under **The agent does it**, apart from **Yours to do**, with the first lines of its proof and the branch or pull request; with two or more ready, **Confirm the N done** confirms them at once.
- Search and read what is there, the Archive included.
- Read your Diary and add to the end of today, signed, only if you open it to agents from the Diary itself and ask it to.

Everything it writes is signed with the name of the program it spoke through: "by Claude Code", "by Codex". In **Settings › Agents › How its signature looks** you choose, for the task list and for inside a task, whether it shows its name, «by an agent» or only a small ◆, with a preview of each.

## What it cannot do

- Close, reopen or delete a task, or delete a document or an idea.
- Tick a step of yours or finish a task of yours: it only says they are done, and you take its word or not.
- Move a day you set.
- Write on its own in the Agents folder: a change to your rules or skills stays as a proposal, and reaches your agents only if you approve it on the card. **Agents: your agents' rules**, a page of this guide, tells it.
- Empty a document's history or read the earlier versions of your diary. It can list a document's versions, read one and go back to it, as you do from **See the history**; [this guide's cover](tisty:guide) tells it, under **Going back to what it said before**.
- Read a task you hid. One you dropped it sees only as your decision not to do it, with its title, the day and the reason noted, and it does not propose it again unless you ask. Where you linked either, it reads only «a task out of sight».
- Create lists or set routines: without one of your lists, what it files lands in the inbox.
- Write over what you changed while it was thinking: it is asked to read again.
- See your Diary while it reads **Only you**, or change what you wrote in it once you open it.

For closing and deleting there is no tool: not hidden, absent. The rest is refused or taken as its word, and it is told why. Every rule is in [what an agent may do](https://www.tisty.app/docs/mcp-permissions/).

## How to ask it

Talk to it as you would to a person: "note this as a task for Friday", "keep this decision in the project's document", "how did we solve this last time?". Tisty also hands it eight ready-made requests your agent shows as commands; in Claude Code they are in the `/` menu:

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

Each one takes a word after it. `/mcp__tisty__consolidate Kitchen` goes over the Kitchen folder alone: your agent shows you what it found, numbered and with the fix it would make; you answer «1 and 2» and it makes only those two. Nothing old is ever deleted: it is marked, and you delete it. A whole pass, call by call, is in [agents and MCP](https://www.tisty.app/docs/agents-and-mcp/).

With Tisty's skills on, the command is not needed: “Pick up where we left off” does what `pick_up` does, “Leave this ready for the next agent” what `leave_off` does, and “Tidy up my Tisty” what `consolidate` does.

And with `@` you hand it a document by its title.

## What is best avoided

- Asking it to delete: it marks what had its day, and deleting is yours, in one click.
- Expecting it to close a task: when it says it finished, the task waits for you in **To confirm**.
- Connecting the same agent two ways, with the plugin and from Settings: it would see every tool twice.
- Forgetting to restart the agent after connecting it: it reads its settings when it starts. The hooks, instead, run from its next session.

What each tool does is in [MCP tools](https://www.tisty.app/docs/mcp-tools/).
