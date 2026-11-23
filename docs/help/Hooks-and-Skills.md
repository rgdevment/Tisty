# Hooks and skills

Connecting an agent gives it Tisty's tools, but not the habit of using them: when it starts it does not know where it left off, and when it finishes it forgets to note down. For that I built two more pieces, both optional and each with its own yes. The **hooks** tell Tisty when each turn of your agent starts and ends. **Tisty's skills** teach your agent to work with Tisty: pick up where it left off, tidy up, and leave work ready for the next one.

Neither is required. Without them, your agent uses the same tools when you ask, and Tisty still asks it whether it left its task up to date. And if you would rather work without an agent, nothing on this page is needed: your tasks and documents work just the same, by hand.

How to connect an agent is in [[Agents and MCP|Agents-and-MCP]]; a session call by call, in [[How an agent works|How-an-Agent-Works]].

## In short

| | Hooks | Tisty's skills |
| :-- | :-- | :-- |
| **What they are** | Commands your agent runs by itself when a session starts, after it uses a tool and when a turn ends | Three folders with a `SKILL.md` that your agent reads when what you ask fits them |
| **What they do** | Bring the agent its open tasks when it starts, and hand the turn back once if it edited files without writing anything in Tisty | Teach it to pick up where it left off, to tidy what you keep without deleting anything, and to write for the next agent |
| **Which agents** | Claude Code, Cursor, Gemini CLI and GitHub Copilot | Claude Code, Cursor, Codex, GitHub Copilot and Gemini CLI |
| **Where you turn them on** | **Settings › Agents › Connected**, under each agent's row | **Settings › Agents › Connected**, on the **Tisty's skills** row |
| **The yes** | One per agent, naming the file that will be written | One for the whole computer, naming the folders that will be written |
| **How they come out** | **Remove**, on the same row | **Turn off on this computer**, on their card. They are never deleted: they stay in Tisty |
| **With the Tisty plugin** | In Claude Code, the plugin brings them | In Claude Code, the plugin brings them |

Both count only on the computer where you turn them on: on another computer you decide there.

## The hooks

### What they do

- **When a session starts**, the agent gets the last three open tasks that same agent wrote on this computer, newest first, with how many steps each has ticked and the next one, what moved since and what is pressing. It is about 2,000 characters, and it arrives even if the agent never calls Tisty.
- **After each edit and each call to Tisty**, the hook leaves a mark for that session in this computer's cache, without reading the conversation.
- **When a turn edited files and wrote nothing in Tisty**, it hands the turn back once: if that was work on a task, leave it in one `tick`. If the agent lets it pass, the hook stays quiet until it calls Tisty again, so a turn outside any task costs one extra turn at most.

Work that edits no file, like planning or research, the hook at the end does not see: there the question Tisty asks in its own answers remains, and it reaches any agent.

### Where they are written

| Agent | File |
| :-- | :-- |
| **Claude Code** | `~/.claude/settings.json` |
| **Cursor** | `~/.cursor/hooks.json` |
| **Gemini CLI** | `~/.gemini/settings.json` |
| **GitHub Copilot** (its command line and the Copilot sessions in VS Code) | a file of its own, `~/.copilot/hooks/tisty.json` |

Tisty writes only its own entries and leaves the rest of the file as it is, with the same `.before-tisty` backups as the connection.

### Adding them

1. Open **Settings › Agents › Connected**.
2. Under the row of a connected agent, like **Claude Code**, the **Hooks: when a turn starts and ends** row says **Not added**. Press **Add**.
3. Tisty tells you what it will do before touching anything: «Tisty will add its hooks to `~/.claude/settings.json`. When a session starts, the agent gets the tasks it left open and what moved…», with **Add them** and **Not now**.
4. With **Add them**, the row says **Added**, and below it «Added. It runs them from its next session.».

There is no need to restart the agent for the hooks: it picks them up in its next session.

### Removing them or turning them off

- **Remove**, on the same row, takes out only Tisty's entries: «Removed. Its next session runs without them.».
- **Remind it when a turn starts and ends**, in **Settings › Agents › Door**, turns them off for every agent without taking them out of any file. It is handy for an afternoon when you do not want turns handed back.
- **Stop it**, on the same **Door**, closes the door: while it is closed, the hooks stay quiet.

### When the Tisty plugin already brings them

The Tisty plugin for Claude Code brings its own hooks, in its own file, from version 1.4.0. When Tisty knows it is installed, whatever its version, Claude Code's hooks row says **The Tisty plugin brings them**, with no **Add** button, so they do not run twice.

If you already had Tisty's and then install the plugin, the row warns you: «The Tisty plugin brings them too, so they run twice. Remove these to keep the plugin's.», with **Remove**.

Tisty knows about the plugin only after a yes on this computer, the one for [Tisty's skills](#tistys-skills) or the one for the [[Agents|AGENTS-Folder]] folder: the plugin's record is one of your agent's files, and before that yes it is not read. With neither, the row offers **Add** even with the plugin there; if you add them, the warning above shows up as soon as you give one of those yeses.

## Tisty's skills

A skill is a folder with a `SKILL.md`: your agent always reads its description, and the rest only when what you ask fits it. Tisty brings three, inside the app:

| Skill | What it teaches your agent | Ask your agent |
| :-- | :-- | :-- |
| `tisty` | Picking up where it left off, working tasks, documents and ideas, and leaving the task ready for the next one | “Pick up where we left off” · “Leave this ready for the next agent” |
| `consolidate` | Tidying what is kept: it looks, proposes each fix numbered and makes only those you pick. It never deletes | “Tidy up my Tisty” |
| `writing-for-the-next-agent` | Writing a page, a task's state or an agent file so the next one understands it without the conversation. Adapted from mattpocock/skills, under the MIT licence, which travels with it | “Make this page clear for the next agent” |

The phrases are the ones their card shows, on the **Ask your agent:** line. They need not be exact: “let's carry on with yesterday's” or “where were we?” also lead to the `tisty` skill.

**Each agent picks up its own.** “Pick up where we left off” takes an agent to the last thing it left, not another: what opens the session and what `catch_up` calls `last` belong to each program. What passes from one agent to another is what is written on the task, so to carry on with what another left, name it by its title.

**They are not the ready-made requests.** Tisty also hands your agent [eight requests](Agents-and-MCP#ready-made-requests-and--mentions) used as commands, like `/mcp__tisty__pick_up` or `/mcp__tisty__consolidate Kitchen` in Claude Code. You type a request, and it reaches any agent that takes requests from a server; your agent picks a skill when you talk to it in your own words. They do the same work, and you can use both.

**They are not documents of yours.** They live inside the app, not in your store: they do not travel to your other computers, they do not show up in a search, in [[Brainsty|Brainsty-Network]], in the history or in an export, and no agent can name or change them through the MCP.

### Where they go

| Agent | Folder |
| :-- | :-- |
| **Claude Code** | `~/.claude/skills/tisty/`, `~/.claude/skills/consolidate/`, `~/.claude/skills/writing-for-the-next-agent/` |
| **Cursor** | `~/.cursor/skills/…`, with the same three folders |
| **Codex**, **GitHub Copilot** and **Gemini CLI** | `~/.agents/skills/…`, a single copy the three of them read |

`~` is your user folder: `C:\Users\you` on Windows and `/Users/you` on macOS. The card shows only the folders of the agents you have on this computer. The shared one says «Codex · Copilot · Gemini» and warns about it: «Codex, Copilot and Gemini share this folder: turning them on or off here does it for the three.».

### Turning them on

1. Open **Settings › Agents › Connected**. At the end of the list is the **Tisty's skills** row: «Three skills that teach your agents to work with Tisty. They have their own yes, on this computer.», with its standing, **Off**, and **See**.
2. **See** opens their card. At the top, «Tisty comes with three skills that teach your agents to work with it. They ship inside the app: no agent writes them, and nothing can change them through the MCP.». Below, each skill with its line and its **Ask your agent:**, and **Where they go on this computer**, with one row per folder.
3. Read the yes, at the foot: «To turn them on, Tisty writes only one folder per skill in the folders above, and reads the files already in those skill folders so as never to write over one of yours. In Claude Code it also reads its plugin's records (installed_plugins.json and settings.json) so as not to copy them twice. It reads no rules, memory or other files of your agents, and needs no Agents folder.».
4. Press **Turn on here**. Each folder turns **In sync**, and the row in Settings says **On this computer**.

At the foot of the card, «From Tisty 1.50.0» says which version yours come from. Your agent sees them from its next session.

**They need no Agents folder.** If you never opened [[Agents|AGENTS-Folder]], or keep it unsynced, the skills still reach every agent on this computer: their yes is a different, smaller one. Nor do the agents have to be connected through MCP to read them, though without a connection they have nothing to work with.

### What each folder says

| The card says | What happens |
| :-- | :-- |
| **In sync** | That folder's copy is this Tisty version's |
| **About to be written** | Once on, it is written on the next round, as when you open a newer version |
| **A skill that is not Tisty's: left as it is** | That folder already holds a skill with the same name, or a file Tisty did not write. Tisty leaves nothing of its own there: that whole skill stays out |
| **Changed outside Tisty: left as it is** | You edited or deleted a file of Tisty's copy. It stays as you left it, and no longer updates |
| **The folder is a link: left as it is** | The folder, one it sits in or a file of its own is a symbolic link or a junction. Tisty does not write through it |
| **Cannot be read: left as it is** | Tisty could not read or write a file in that folder |

Before you turn them on, a folder shows no standing.

If you already have a skill of your own with the same name in the **Agents** folder, like your own `consolidate`, the card says so instead of its line: «You already have a skill named consolidate in Agents, so Tisty's is left out.». Yours wins, in every agent, once this computer gave its yes to Agents; without it, the folder guards it: a skill that is not Tisty's is left as it is.

### They update by themselves

When you update Tisty, the next time you open it, it writes each skill's new version into the folders that are **In sync**. A copy you changed, or a folder that is not Tisty's, is left alone. The card says it: «They update when you update Tisty, and never write over a skill of yours with the same name.».

### Turning them off

**Turn off on this computer**, on the same card. First it tells you what it does: «Turning them off takes Tisty's untouched copies out of your agents' folders, with a backup, and the skills stay in Tisty, ready to turn on again. A copy you changed stays in this card.».

- **Turning off is not deleting.** The skills stay inside Tisty, and **Turn on here** puts them back.
- **A copy you changed** is not taken out, and its whole skill stays with it in that folder even if only one file changed: it stays listed on the card, for you to take away if you like.
- **Nothing deletes them.** There is no **Delete** on the card or in the Agents folder, and no agent can take them out through the MCP.

### In the Agents folder

If you use the [[Agents|AGENTS-Folder]] folder, **Agents › Global** shows a fixed row, **Tisty's skills**, with the same standing as in Settings. A click opens the same card.

- **Hide.** When you hover over the row, **Hide** appears and takes it out of Agents on this computer. The skills stay as they were, on or off. In Settings, the row then says «Hidden from Agents on this computer; they stay as they are.», with **Show in Agents**.
- **It has no menu.** No **Delete**, no **Pause**, no **New document**: it is not a document.
- **Bring does not bring them.** If you turned the skills on before opening Agents, the first-time list, **Bring another file** and **Look again** show their copies as **Tisty's own**, and do not bring them in: they never become documents of yours.
- **Pausing Agents does not reach them.** Pausing an agent, or stopping sync for the whole folder, holds your documents. Tisty's skills follow their own switch: on, they reach and update in every agent on this computer. The card reminds you: «Only on this computer: on another one, you decide there. Pausing Agents does not reach them: they have their own switch.».
- **The two yeses stand apart.** Stopping sync for Agents does not turn the skills off, and turning the skills on brings nothing into Agents.

### With the Tisty plugin

- **Claude Code.** The Tisty plugin brings the same three skills. When it is installed for your user and you did not turn it off, Tisty does not copy them to `~/.claude/skills`, and the card says so: «Claude Code already has them through the Tisty plugin, so Tisty does not copy them there.». Codex, Copilot, Gemini and Cursor keep getting them.
- **The plugin comes later.** The copies Tisty had left in `~/.claude/skills` show up on the card under «Copies left over: Tisty no longer writes them there. Removing one keeps a backup.», each path with **Take the file away**. They are never taken away on their own.
- **Cursor.** Its plugin leaves no record Tisty can read, so the card always warns: «If you installed the Tisty plugin in Cursor, Cursor will list each skill twice: Tisty cannot tell.». If it happens to you, keep only one of the two: the plugin in Cursor or Tisty's skills.

## When you connect an agent

When you connect an agent from **Settings › Agents › Connected**, with **Connect** or with **Connect the 3 found**, its row shows a box, once. It also shows when you connect with **Fix** or **Connect anyway**, if that agent had not seen it, and never for agents inside WSL:

> **Claude Code is connected. This helps it work with Tisty:**
>
> With its hooks, Claude Code tells Tisty when each turn starts and ends.
> **Hooks: when a turn starts and ends** · Not added · **Add**
>
> Tisty's skills teach Claude Code to pick up where it left off, tidy up and leave work ready for the next agent. **See Tisty's skills**
>
> **Not now**

- **Add** asks the same yes as the usual row, naming the file.
- **See Tisty's skills** opens the card, with **Turn on here**.
- **Not now** closes the box.

It is offered once per agent and computer: it counts from the moment it shows, even if you leave Settings without answering.

It changes with the agent and with what you already have:

| If… | The box |
| :-- | :-- |
| you gave one of the two yeses and the Tisty plugin already brings the hooks | says «The Tisty plugin already brings its hooks.» |
| you gave one of the two yeses and the plugin already brings the skills | says «Claude Code already has Tisty's skills through the Tisty plugin.», with no button |
| the skills are already on | says «Tisty's skills are already on this computer.», with no button |
| the agent is Codex | carries only the skills line: Codex has no hooks |
| the agent has neither hooks nor skills, like Claude Desktop | does not appear |

**If your agent was already connected** before 1.50, the box does not appear: it is offered only when you connect. Use the usual rows, the hooks row under the agent and **Tisty's skills** › **See**, at the end of the list. It is the same yes.

## A case from end to end

Yisset uses Codex, which she connected to Tisty months ago. Today she updates Tisty to 1.50 and installs Claude Code on the same computer.

### 1. She connects Claude Code

In **Settings › Agents › Connected**, Claude Code shows **Not connected**. Yisset presses **Connect**, and under the row the box appears: «Claude Code is connected. This helps it work with Tisty:».

### 2. She adds its hooks

She presses **Add** on the hooks row. Tisty shows her the file, `~/.claude/settings.json`, and what the hooks will do. With **Add them**, the row says **Added**.

### 3. She turns Tisty's skills on

She presses **See Tisty's skills**. The card shows the three skills and two folders under **Where they go on this computer**: Claude Code's, `~/.claude/skills`, and the shared one, `~/.agents/skills`, for Codex. She reads the yes and presses **Turn on here**. Both folders turn **In sync**. She closes the card and, in the box, presses **Not now**.

Since Claude Code reads its settings when it opens, she quits it completely and opens it again.

### 4. She gives it a job

Yisset asks Claude Code to plan the export of the report. The agent files it as a task with its steps:

```json
> propose {"title": "Export the monthly report to CSV", "list": "Work",
           "steps": ["Read the format accounting asks for", "Write the export",
                     "Try it with October's data", "Add the button to the reports screen"]}
```

That afternoon it reads the format, ticks the first step and leaves the state: semicolon as separator and dates as `2026-10-31`, the way accounting wants them.

### 5. “Pick up where we left off”

The next day, when the session opens and with nobody asking, the hook hands Claude Code the open tasks it wrote on this computer. Among the rest, it tells it:

```text
Your last open tasks here, newest first:
- "Export the monthly report to CSV" in Work […], last written 2026-11-20 18:05: under way, 1 of 4 steps ticked, next "Write the export"; state: **Done:** the accounting format is read…
Pressing for the person: 1 today, 3 late.
```

Yisset types “Pick up where we left off”. The phrase fits the `tisty` skill, which tells the agent to start with `catch_up` and open the task it was on:

```json
> catch_up {}
{ "last": { "title": "Export the monthly report to CSV", "list": "Work", "at": "2026-11-20 18:05",
            "stage": "under_way", "steps": [1, 4], "state": "**Done:** the accounting format is read…" },
  "counts": { "open": 31, "late": 3, "today": 1, "docs": 48, … } }
```

```json
> read {"task": "…Export the monthly report to CSV…"}
{ "steps": [ { "text": "Read the format accounting asks for", "done": true },
             { "text": "Write the export", "done": false }, … ] }
```

The state comes first, with what was decided the day before. Claude Code tells Yisset where they left off and carries on with «Write the export».

### 6. It works and ticks

Once the export is done, it tries it with October and ticks both steps and the state in a single call:

```json
> tick {"task": "…", "steps": ["Write the export", "Try it with October's data"],
        "state": "**Done:** the export writes the CSV with semicolons; October matches the system's total.\n**Left:** the button on the reports screen.\n**Decisions in force:** dates as 2026-10-31, amounts with no thousands separator."}
{ "ticked": 2, "left": 1, "next": "Add the button to the reports screen", "stated": true,
  "said": "Ticked 2 step(s) on \"Export the monthly report to CSV\". 1 still unticked; next: \"Add the button to the reports screen\". Left its state on its page, under «State»." }
```

Had it forgotten, the hook reminds it at the end of the turn, because it edited files without writing anything in Tisty:

```json
{ "decision": "block",
  "reason": "You edited files this turn and wrote nothing in Tisty. If this was work on a Tisty task, leave it in one `tick`…" }
```

### 7. “Leave this ready for the next agent”

Yisset has to go. She types “Leave this ready for the next agent”. The `tisty` skill asks the agent to leave it on the task, not in the conversation, which gets archived: the state up to date, with `tick`, and a note with where the work stands, what it ruled out, the next step and the branch or files it lives in.

```json
> note {"task": "…", "body": "Ruled out: exporting with the spreadsheet library; it turns the amounts into text and accounting cannot add them up. Next is the button, beside Print, as «Reports: decisions» says. The code is on the branch feat/monthly-csv-export."}
{ "references": { "linked": [ { "said": "Reports: decisions", "doc": "…", "title": "Reports: decisions" } ] } }
```

It ends by telling Yisset what it left and where: the ticked steps, the state, the note and the branch.

### 8. What Yisset sees in Tisty

- **In the Work list**, the task's row says **3/4 · ✎1** and «by an agent»; when she hovers over it, «Claude Code wrote on this machine».
- **On its page**, the **State** section, with the date and «Claude Code», says what is done, what is left and what was decided. Above it, on the right, **↓ Journal · 1 note** takes her to the agent's note, with its link to «Reports: decisions».
- **Nothing closed.** One step is left, and when the agent says the task is done, it waits in **To confirm** until Yisset closes it.

Had Yisset written the task herself and ticked the first step, the agent's ticks would be its word: the row would say **1/4 ◆2**, with «2 to confirm» when she hovers, until she takes them with **Tick them**. [[What an agent may do|MCP-Permissions]] tells it.

### 9. Later, another agent

Yisset goes back to Codex. Codex has no hooks, so nothing reaches it when it starts, and since it was connected before 1.50 it never saw the box; but it reads the `tisty` skill, which Yisset turned on in step 3 and which landed in `~/.agents/skills`. “Pick up where we left off”, alone, would take it to the last thing **it** left, so Yisset names the task: “Pick up the monthly report export”.

```json
> brain {"query": "export monthly report"}
{ "matches": [ { "title": "Export the monthly report to CSV", "list": "Work", "status": "open" } ] }
> read {"task": "…Export the monthly report to CSV…"}
```

It reads Claude Code's state and note, knows not to try the spreadsheet library again, and carries on with the button. It would also have got there through what moved, which `catch_up` tells it since the last time it looked.

### 10. Weeks later, the plugin

Yisset installs the Tisty plugin in Claude Code. The next time she opens **Settings › Agents**:

- Claude Code's hooks row says «The Tisty plugin brings them too, so they run twice. Remove these to keep the plugin's.». She presses **Remove**, and the row stays at **The Tisty plugin brings them**;
- the skills card says «Claude Code already has them through the Tisty plugin, so Tisty does not copy them there.», and lists under **Copies left over** the files Tisty had left in `~/.claude/skills`. She presses **Take the file away** on each one, and their backups stay. Codex's shared folder stays **In sync**.

## Cases worth knowing

- **On another computer.** The hooks and the skills are decided on each computer: on the work computer, the **Tisty's skills** row starts **Off** and the hooks **Not added**, even if at home they are on. A record copied from another computer does not count as a yes to the skills.
- **Before any yes**, the skills' or Agents', the card opens nothing inside your agents' folders: it only looks at which ones exist and whether the copies Tisty wrote before are still there. It does not read the plugin's record either. With Agents' yes alone, it reads that record and those copies, to tell whether they changed.
- **With the door closed.** The skills stay in your agents' folders, and your agent can read them, but every call to Tisty is refused and the hooks stay quiet. **Stop it** touches neither piece.
- **An agent paused in Agents.** The pause does not reach Tisty's skills: they keep reaching and updating in that agent. For none to have them, **Turn off on this computer**.
- **Codex and Claude Desktop** have no hooks. Codex gets the skills; Claude Desktop, neither, and only the question in Tisty's answers reaches it.
- **Gemini CLI** runs the hooks only in folders you trusted. Its Tisty extension brings them too, but Tisty recognises only the Claude Code plugin: if you use the extension, do not also add the hooks from Settings.
- **Cursor** also runs Claude Code's hooks. Even when both fire, Tisty hands the turn back only once.
- **GitHub Copilot** may spend a premium request on the turn handed back.
- **A development build** of Tisty carries no skills. The card says so, «This build of Tisty carries no skills: only installed versions do.», and **Turn on here** stays off.
- **A skill of yours with the same name** in an agent's folder always wins, and one in Agents too, on a computer that gave its yes to Agents. Tisty's stays out, writing over nothing.
- **If Tisty lost its record**, say after reinstalling the system, turning the skills on claims back as Tisty's the copies that are identical, byte for byte, to this release's: they update again and come out when you turn them off. If one is missing a file, has a different one or one of yours beside it, that whole skill stays yours and is left alone.

## What they read and what travels

- **The hooks** write only their entries in the file named. Their marks for each session stay in this computer's cache and do not sync. What they hand the agent when it starts, the titles of a few tasks, travels wherever that agent travels.
- **The skills** write only one folder per skill. They read the files in those folders, so as not to write over one of yours, and in Claude Code its plugin's record, only to tell whether it already brings them. Nothing else of your agents is opened.
- **The skills' yes** is kept in `data/agents/` inside Tisty's folder, on this computer, with the backups of what Tisty took away. It does not travel with sync, backups or parcels. The hooks keep no separate yes: they are added when you press **Add them**, after reading which file they go in, they come out with **Remove**, and Tisty knows whether they are there by reading that file.

All of it, file by file, is in [[Privacy]].

## Where next

- [[Agents and MCP|Agents-and-MCP]]: connecting an agent, the door, the ready-made requests and what it costs.
- [[How an agent works|How-an-Agent-Works]]: a whole session, call by call.
- [[What an agent may do|MCP-Permissions]]: its word on your tasks and **To confirm**.
- [[Agents: your agents' rules, skills and memory|AGENTS-Folder]]: your own rules and skills, written once for all your agents.
- [[Tools reference|MCP-Tools]]: every tool the skills use.
