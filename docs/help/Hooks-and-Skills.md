# Hooks and skills

Connecting an agent gives it Tisty's tools, but not the habit of using them: when it starts it does not know where it left off, and when it finishes it forgets to note down. For that I built two more pieces, both optional and each with its own yes. The **hooks** help your agent at three moments: when a session starts, when a turn ends and before it leaves something outside Tisty. **Tisty's skills** teach your agent to work with Tisty: pick up where it left off, tidy up, and leave work ready for the next one.

Neither is required. Without them, your agent uses the same tools when you ask, and Tisty still asks it whether it left its task up to date. And if you would rather work without an agent, nothing on this page is needed: your tasks and documents work just the same, by hand.

How to connect an agent is in [[Agents and MCP|Agents-and-MCP]]; a session call by call, in [[How an agent works|How-an-Agent-Works]].

## In short

| | Hooks | Tisty's skills |
| :-- | :-- | :-- |
| **What they are** | Commands your agent runs by itself when a session starts, before and after some tools and when a turn ends | Three folders with a `SKILL.md` that your agent reads when what you ask fits them |
| **What they do** | Bring the agent its open tasks when it starts, and hand the turn back once if it edited files without writing anything in Tisty, and ask it once before it leaves a text file outside a repository | Teach it to pick up where it left off, to tidy what you keep without deleting anything, and to write for the next agent |
| **Which agents** | Claude Code, Cursor, Gemini CLI and GitHub Copilot | Claude Code, Cursor, Codex, GitHub Copilot and Gemini CLI |
| **Where you turn them on** | **Settings › Agents › Connected**, under each agent's row | **Settings › Agents › Connected**, under each agent's row or on the **Tisty's skills** row, at the end |
| **Both at once** | **Hooks and skills for your agents**, above the tabs | The same box |
| **The yes** | One per agent, when you connect it or on its switch, naming the file that will be written | One for the whole computer, when you connect or on its switch, naming the folders that will be written |
| **How they come out** | The same switch: it takes them all out of that agent. Another one, in **Door**, quiets them in all your agents | The same switch, or **Turn off on this computer**, on their card. They are never deleted: they stay in Tisty |
| **With the Tisty plugin** | In Claude Code, the plugin brings them | In Claude Code, the plugin brings them |

Both count only on the computer where you turn them on: on another computer you decide there.

## The hooks

### What they do

- **When a session starts**, the agent gets the last three open tasks that same agent wrote on this computer, newest first, with how many steps each has ticked and the next one, what moved since and what is pressing. It is about 2,000 characters, and it arrives even if the agent never calls Tisty.
- **When a turn edited files and wrote nothing in Tisty**, it hands the turn back once: if that was work on a task, leave it in one `tick`. If the agent lets it pass, the hook stays quiet until it calls Tisty again, so a turn outside any task costs one extra turn at most.
- **Before it leaves something outside**, if it is about to create a text file outside a repository, or to propose a task card of its own in the Claude app, it asks it once whether that belongs in Tisty. [Before it leaves something outside](#before-it-leaves-something-outside) tells it.

What they look at and what stays is in [[Privacy]].

Work that edits no file, like planning or research, the hook at the end does not see: there the question Tisty asks in its own answers remains, and it reaches any agent.

### Before it leaves something outside

An agent with Tisty in front of it sometimes leaves the work for another one in a `.md`, where nobody will look for it. This hook asks it **first**, only once, and lets it through if it insists.

| What it is about to leave | What Tisty names |
| :-- | :-- |
| Something to do: work to hand over, a plan, what goes on another day | A task: `propose`, or `note` and `tick` with `state` on the one there is |
| Something to read again: an analysis, a contract, a decision | A document: `brain` and then `append_doc`, `edit_doc` or `write_doc` |
| A task card of its own | A task: `propose` in its list, or `note` |

**It looks at new text files**: `.md`, `.markdown`, `.txt`, `.rst`, `.adoc`, `.org`, and those named `HANDOFF`, `NOTES`, `TODO`, `PLAN`, `PROMPT` or `BRIEF`, with no extension. Editing one that is already there does not ask.

**It does not ask** about code, nor about what goes in a repository or a project (a folder with `.git`, `package.json`, `Cargo.toml` and the like), a notes vault (`.obsidian`, `.logseq`), a temporary or settings folder, `node_modules`, nor a `README`, a `LICENSE`, an `AGENTS.md` or a `requirements.txt`. Nor about reminders, the turn's list of steps or subagents.

| Agent | New file | Terminal | Card |
| :-- | :-- | :-- | :-- |
| **Claude Code** | Yes | Yes | Yes, in the Claude app |
| **Cursor** | Yes | Yes | — |
| **Gemini CLI** | Yes | No | — |
| **GitHub Copilot** | Yes, except with `apply_patch` | No | — |

In Cursor, the question comes from its own hooks: with only Claude Code's, which Cursor runs too, it stays quiet. Of a command it looks only at where it writes what it says itself (`echo … > note.md`, `cat > note.md`, `tee`, `Set-Content`): not at what a program it runs would write. Codex and Claude Desktop have no hooks, and an agent in the cloud does not run the ones on your computer.

**Once per session and folder**, with the ones inside, and three folders a session at most; cards, once. If the turn already got that question, it is not also handed back when it ends.

**Example.** Marisol asks Claude Code for a summary of what is left “so Codex can carry on”. The agent is about to create `C:\Users\Marisol\Documents\handover.md`, and the hook answers:

```json
{ "hookSpecificOutput": { "hookEventName": "PreToolUse", "permissionDecision": "deny",
  "permissionDecisionReason": "This would leave a text file outside any repository. If someone will read it later (the person, another agent, you on another day), it belongs in Tisty: …" } }
```

So it leaves it on a task, with its steps, and gives Marisol the title:

```json
propose { "title": "Finish the payments migration", "list": "Work",
  "steps": ["Test the retries", "Tell the provider"] }
```

Had Marisol asked for exactly that file, the agent makes the same call again and it is created, without asking Marisol anything.

### Where they are written

| Agent | File |
| :-- | :-- |
| **Claude Code** | `~/.claude/settings.json` |
| **Cursor** | `~/.cursor/hooks.json` |
| **Gemini CLI** | `~/.gemini/settings.json` |
| **GitHub Copilot** (its command line and the Copilot sessions in VS Code) | a file of its own, `~/.copilot/hooks/tisty.json` |

Tisty writes only its own entries and leaves the rest of the file as it is, with the same two backups as the connection, `.before-tisty`, the first version it ever saw, and `.before-tisty-last`, the one from just before. If the file could not be read, the row says **Could not be read** and Tisty does not touch it.

### Adding them

1. Open **Settings › Agents › Connected**.
2. Under the row of a connected agent, like **Claude Code**, the **Tisty's hooks** row has a switch. Turn it on.
3. Tisty tells you what it will do before touching anything: «Tisty will add its hooks to `~/.claude/settings.json`. When a session starts, the agent gets the tasks it left open and what moved…», with **Add them** and **Not now**. In Claude Code it adds a line: «For Claude Code, Tisty first reads its plugin's records (installed_plugins.json and settings.json) so as not to write hooks the plugin already brings.».
4. With **Add them**, the switch stays on, and below it «Added. It runs them from its next session.».

**If you connect the agent now, the hooks come with it**: [When you connect an agent](#when-you-connect-an-agent) tells it.

**If you added them before 1.50**, nothing is added on its own. Under the row it says «They lack the newest one, which asks before it leaves something outside a repository: turn them off and on to have it.». Turning them on again writes them all as they come: a time or a filter you had changed goes back to Tisty's.

There is no need to restart the agent for the hooks: it picks them up in its next session.

### When they run another Tisty

Hooks keep the path of the Tisty that put them there. If you move Tisty or switch installers, they keep calling the old one, and the agent does not always say so. The row does:

| The row says | What happens |
| :-- | :-- |
| Switch on | They call this Tisty, or one whose path Tisty cannot check, such as plain `tisty` |
| **Set up your own way** | «It runs Tisty from hooks written your own way. Tisty leaves them as they are and adds none beside them.». They are the ones that call Tisty from a script of yours or in another way Tisty cannot read: they are not fixed, not removed, and the row does not offer to turn them on |
| **They run a Tisty that is gone** | «Its hooks point at `C:\…\tisty.exe`, which is no longer there». **Fix** points them here |
| **They run another Tisty** | That other one is still on this computer. **Fix** points them here, if you want |
| **Could not be read** | Tisty could not read their file, so it does not touch it |
| Switch off | That file holds no Tisty hooks |
| **The Tisty plugin brings them** | The plugin brings them, in its own file |

**Fix** shows you the file and waits for your **Yes, fix**. It changes only the program they call: the matchers, the times and a wrapper of yours stay as they were. The ones that are gone show in the window's notice and in **Fix all** too, as [When an agent can no longer find Tisty](Agents-and-MCP#when-an-agent-can-no-longer-find-tisty) tells.

**Example.** Jaime was trying Tisty from a downloads folder and deleted it on installing it for real. Jaime connected Cursor again, but never its hooks: Cursor's row says **Connected** and, below it, **They run a Tisty that is gone**. Jaime presses **Fix**, reads `~/.cursor/hooks.json` under **It writes in:**, presses **Yes, fix** and opens Cursor again. The longer time set on one of the hooks stays as it was.

### Removing them or turning them off

- Turning the row's switch off takes out only Tisty's entries: «Removed. Its next session runs without them.». If there are others of your own way too: «Tisty's own are removed. The ones written your own way stay.».
- **Remind it when a turn starts and ends**, in **Settings › Agents › Door**, turns the three off for every agent without taking them out of any file. It is handy for an afternoon when you do not want turns handed back.
- **Stop it**, on the same **Door**, closes the door: while it is closed, the hooks stay quiet.

### When the Tisty plugin already brings them

The Tisty plugin for Claude Code brings its own hooks, in its own file, from version 1.4.0. It brings the first two: the one before it leaves something outside does not come with it yet, and then Claude Code runs without that one. When Tisty knows it is installed, whatever its version, Claude Code's hooks row says **The Tisty plugin brings them**, with no switch, so they do not run twice.

If you already had Tisty's and then install the plugin, the row warns you: «The Tisty plugin brings them too, so they run twice. Turn these off to keep the plugin's.», with their switch.

The plugin's record is one of your agent's files, so Tisty reads it only with a yes from you. With the one for [Tisty's skills](#tistys-skills) or the one for the [[Agents|AGENTS-Folder]] folder, it knows when you open **Settings › Agents**; for the skills it also looks when the app opens and when something changes in your data, even with them off. With neither, the row offers to turn them on even with the plugin there, and Tisty reads it right when you press **Yes, connect** with its hooks, **Add them**, **Add hooks**, **Fix**, **Fix all** or **Yes, fix**, before writing Claude Code's hooks. If the plugin already brings them, it writes nothing: «The Tisty plugin already brings Claude Code's hooks, so those in its settings were left as they were.». If the record cannot be read, it does not write either, and tells you which file it was.

If Tisty knows the plugin brings them and the ones left from before point at a Tisty that is gone, **Fix** removes them, because with the plugin's they would run twice, and it tells you first: «The Tisty plugin already brings its hooks, so these are removed. Tisty first reads its records (installed_plugins.json and settings.json) to make sure.». Those records are read with your click, right before removing them. If the plugin no longer brings them, none is removed: «The Tisty plugin no longer brings Claude Code's hooks, so these were not removed. Look again to see how they are now.».

## Tisty's skills

A skill is a folder with a `SKILL.md`: your agent always reads its description, and the rest only when what you ask fits it. Tisty brings three, inside the app:

| Skill | What it teaches your agent | Ask your agent |
| :-- | :-- | :-- |
| `tisty` | Picking up where it left off, working tasks, documents and ideas, and leaving the task ready for the next one | “Pick up where we left off” · “Leave this ready for the next agent” |
| `consolidate` | Tidying what is kept: it looks, proposes each fix numbered and makes only those you pick. It never deletes | “Tidy up my Tisty” |
| `writing-for-the-next-agent` | Writing a page, a task's state or an agent file so the next one understands it without the conversation. Adapted from mattpocock/skills, under the MIT licence, which travels with it | “Make this page clear for the next agent” |

The phrases are the ones their card shows, under **How to use them**: «Write it to your agent in your own words; there is no need to call them by name.». They need not be exact: “let's carry on with yesterday's” or “where were we?” also lead to the `tisty` skill.

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
2. **See** opens their card. At the top, «Tisty comes with three skills that teach your agents to work with it. They ship inside the app: no agent writes them, and nothing can change them through the MCP.». Below, **Where they go on this computer**, with one row per folder, and folded, **How to use them**, with what you write to your agent and what each one is for, and **More about them**.
3. Read the yes, at the foot: «To turn them on, Tisty writes only one folder per skill in the folders above, and reads the files already in those skill folders so as never to write over one of yours. In Claude Code it also reads its plugin's records (installed_plugins.json and settings.json) so as not to copy them twice. It reads no rules, memory or other files of your agents, and needs no Agents folder.».
4. Press **Turn on here**. Each folder turns **In sync**, and the row in Settings says **On**.

Under each connected agent that reads them there is another **Tisty's skills** row, with a switch that counts for this whole computer: turning it on opens the same card, and turning it off turns them off in all your agents. And if you have connected agents without them, the box [Hooks and skills for your agents](#both-at-once) turns them on together with the hooks, with the same yes written out first.

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

The row under each agent sums it up:

| The row says | What happens |
| :-- | :-- |
| Switch on, with nothing else | All three are up to date in that agent's folder |
| Switch off | You did not turn them on here. Turning it on opens the card |
| **To bring up to date** | This Tisty's version is still to be written. It shows in the box on top too, with **Fix** |
| **Not all up to date** | A copy is not Tisty's, you changed it or it cannot be read, or you already have a skill with that name in Agents; below it says what happens. Tisty does not touch it, not with **Fix** either |
| **Not here** | «Tisty found no folder of this agent to put them in.», or «You already have skills with those names in Agents, so Tisty's stay out.» |
| **The Tisty plugin brings them** | That agent has them through the plugin |

If you already have a skill of your own with the same name in the **Agents** folder, like your own `consolidate`, the card says so instead of its line: «You already have a skill named consolidate in Agents, so Tisty's is left out.». Yours wins, in every agent, once this computer gave its yes to Agents; without it, the folder guards it: a skill that is not Tisty's is left as it is.

### They update by themselves

When you update Tisty, the next time you open it, it writes each skill's new version into the folders that are **In sync**. A copy you changed, or a folder that is not Tisty's, is left alone. The card says it: «They update when you update Tisty, and never write over a skill of yours with the same name.».

### Turning them off

The switch on the **Tisty's skills** row, under any agent, or **Turn off on this computer**, on the card, which first tells you what it does: «Turning them off takes Tisty's untouched copies out of your agents' folders, with a backup, and the skills stay in Tisty, ready to turn on again. A copy you changed stays in this card.».

- **Turning off is not deleting.** The skills stay inside Tisty, and **Turn on here** puts them back.
- **A copy you changed** is not taken out, and its whole skill stays with it in that folder even if only one file changed: it stays listed on the card, for you to take away if you like.
- **Nothing deletes them.** There is no **Delete** on the card or in the Agents folder, and no agent can take them out through the MCP.

### In the Agents folder

If you use the [[Agents|AGENTS-Folder]] folder, **Agents › Global** shows a fixed row, **Tisty's skills**, with the same standing as in Settings, only if this computer has some agent, if the skills are on, or if copies of them remain. A click opens the same card.

- **On or Off.** The row says so, in green or in gray, and opens the same card, where they are turned on or off, with the yes and the folders. If you had hidden it before, it shows again.
- **It has no menu.** No **Delete**, no **Pause**, no **New document**: it is not a document.
- **Bring does not bring them.** If you turned the skills on before opening Agents, the first-time list, **Bring another file** and **Look again** show their copies as **Tisty's own**, and do not bring them in: they never become documents of yours.
- **Pausing Agents does not reach them.** Pausing an agent, or stopping sync for the whole folder, holds your documents. Tisty's skills follow their own switch: on, they reach and update in every agent on this computer. The card reminds you: «Only on this computer: on another one, you decide there. Pausing Agents does not reach them: they have their own switch.».
- **The two yeses stand apart.** Stopping sync for Agents does not turn the skills off, and turning the skills on brings nothing into Agents.

### With the Tisty plugin

- **Claude Code.** The Tisty plugin brings the same three skills. When it is installed for your user and you did not turn it off, Tisty does not copy them to `~/.claude/skills`, and the card says so: «Claude Code already has them through the Tisty plugin, so Tisty does not copy them there.». Codex, Copilot, Gemini and Cursor keep getting them.
- **The plugin comes later.** The copies Tisty had left in `~/.claude/skills` show up on the card under «Copies left over: Tisty no longer writes them there. Removing one keeps a backup.», each path with **Take the file away**. They are never taken away on their own.
- **Cursor.** Its plugin leaves no record Tisty can read, so the card always warns: «If you installed the Tisty plugin in Cursor, Cursor will list each skill twice: Tisty cannot tell.». If it happens to you, keep only one of the two: the plugin in Cursor or Tisty's skills.

## Both at once

If you already have connected agents and one of them lacks the hooks or the skills, **Settings › Agents** offers them together, above the tabs. It takes in the ones that did not go through [When you connect an agent](#when-you-connect-an-agent), since that one already offered them:

> **Hooks and skills for your agents**
>
> Optional. With hooks, an agent starts each session knowing what it left open. With Tisty's skills, it learns to work with Tisty.
>
> Hooks for Claude Code, Cursor
> Tisty's skills on this whole computer: Claude Code, Cursor, Codex · Copilot · Gemini
>
> **Add hooks and turn on skills** · **Not now**

1. **Add hooks and turn on skills** writes nothing yet: it opens two ticks, one with the files the hooks go in and one with the folders the skills go in and what Tisty reads to write them.
2. Untick what you do not want. The button changes with what you leave: **Add hooks** or **Turn on skills**.
3. Press it again. Tisty turns the skills on, adds the hooks and tells you what was done, agent by agent: **Claude Code** · hooks, or «the Tisty plugin already brings its hooks» when the plugin does.
4. Quit each one completely and open it again.

If they all have the hooks already, the box offers only the skills, and the other way round. **Not now**, or unticking a part, means it is not offered again; what failed is, and the usual rows stay. **Overview** recalls it in one line, **Your agents can use Tisty's hooks and skills · Optional**, which does not count as pending. It waits if an agent can no longer find Tisty: that goes first. And it does not offer the hooks while **Remind it when a turn starts and ends** is off.

## When you connect an agent

When you connect an agent from **Settings › Agents › Connected**, its hooks and Tisty's skills come with it, with a single yes, and stay on. **Connect**, or **Connect anyway**, shows you first what it will write:

> **Connect Claude Code to Tisty**
>
> It writes in:
> `~/.claude.json`
> `~/.claude/settings.json`
>
> ☑ **With its hooks: Claude Code**
> Tisty will add its hooks to `~/.claude/settings.json`. When a session starts…
>
> ☑ **Tisty's skills on this whole computer: Claude Code**
> `~/.claude/skills`
> To turn them on, Tisty writes only one folder per skill in the folders above…
>
> **Yes, connect** · **Not now**

- **Yes, connect** writes what you left ticked. What you untick stays off, with its switch under the agent's row.
- **Connect the 3 found** shows the same for all of them, with each one's files. The ones inside WSL too.
- **If the hooks cannot be added**, because their file could not be read or the Tisty plugin already brings them, the agent is connected all the same and Tisty tells you, with no hooks twice.
- **If the skills cannot be turned on**, the agent is connected all the same and Tisty tells you why.
- The skills count for the whole computer: when they are already on, they are not asked again. Codex, which has no hooks, shows only the skills; an agent with neither, like Claude Desktop, is connected with one click. **Fix** adds nothing new.
- With **Remind it when a turn starts and ends** off, they are added and stay quiet.

After that, under its row are the two switches, **Tisty's skills** and **Tisty's hooks**, on.

**If you already had the agent connected** before 1.50, nothing turns on by itself. For those there is [Hooks and skills for your agents](#both-at-once), above the tabs, or the switches under the agent. It is the same yes.

## A case from end to end

Yisset uses Codex, which she connected to Tisty months ago. Today she updates Tisty to 1.50 and installs Claude Code on the same computer.

### 1. She connects Claude Code

She opens **Settings › Agents**. Above the tabs, **Hooks and skills for your agents** offers to turn the skills on, because Codex does not have them; she leaves it for after connecting. In **Connected**, Claude Code shows **Not connected**. Yisset presses **Connect**, and Tisty shows her what it will write: `~/.claude.json`, `~/.claude/settings.json` and the skills' folders.

### 2. With its hooks and the skills

She reads what the hooks will do and where the skills go, in Claude Code's folder, `~/.claude/skills`, and in the shared one, `~/.agents/skills`, for Codex. She leaves both ticks on and presses **Yes, connect**.

### 3. They stay on

The row says **Connected** and, below it, both switches are on: **Tisty's skills**, which count for Codex too, and **Tisty's hooks**.

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

Yisset goes back to Codex. Codex has no hooks, so nothing reaches it when it starts, and since it was connected before 1.50 it never saw the box for connecting; but it reads the `tisty` skill, which Yisset turned on in step 3 and which landed in `~/.agents/skills`. “Pick up where we left off”, alone, would take it to the last thing **it** left, so Yisset names the task: “Pick up the monthly report export”.

```json
> brain {"query": "export monthly report"}
{ "matches": [ { "title": "Export the monthly report to CSV", "list": "Work", "status": "open" } ] }
> read {"task": "…Export the monthly report to CSV…"}
```

It reads Claude Code's state and note, knows not to try the spreadsheet library again, and carries on with the button. It would also have got there through what moved, which `catch_up` tells it since the last time it looked.

### 10. Weeks later, the plugin

Yisset installs the Tisty plugin in Claude Code. The next time she opens **Settings › Agents**:

- Claude Code's hooks row says «The Tisty plugin brings them too, so they run twice. Turn these off to keep the plugin's.». She turns them off, and the row stays at **The Tisty plugin brings them**;
- the skills card says «Claude Code already has them through the Tisty plugin, so Tisty does not copy them there.», and lists under **Copies left over** the files Tisty had left in `~/.claude/skills`. She presses **Take the file away** on each one, and their backups stay. Codex's shared folder stays **In sync**.

## Cases worth knowing

- **On another computer.** The hooks and the skills are decided on each computer: on the work computer, the **Tisty's skills** row starts **Off** and the hooks off, even if at home they are on. A record copied from another computer does not count as a yes to the skills.
- **Before any yes**, the skills' or Agents', the card opens nothing inside your agents' folders: it only looks at which ones exist and whether the copies Tisty wrote before are still there. It does not read the plugin's record either, unless you press to add or fix Claude Code's hooks, and only then. With Agents' yes alone, it reads that record and those copies, to tell whether they changed.
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

- **The hooks** write only their entries in the file named. With the door open, Tisty reads that file, and the connection's, when its window opens and when Settings opens, to see whether they point at this Tisty, writing nothing until you click. In Claude Code, when you add or fix them, also when fixing is removing them because the plugin brings them, it also reads its plugin's record; without the skills' yes or Agents', only then. Their marks for each session stay in this computer's cache and do not sync. What they hand the agent when it starts, the titles of a few tasks, travels wherever that agent travels.
- **The skills** write only one folder per skill. They read the files in those folders, so as not to write over one of yours, and in Claude Code its plugin's record, only to tell whether it already brings them. Nothing else of your agents is opened.
- **The skills' yes** is kept in `data/agents/` inside Tisty's folder, on this computer, with the backups of what Tisty took away. It does not travel with sync, backups or parcels. The hooks keep no separate yes: they are added when you press **Add them** or **Add hooks**, after reading which file they go in, they come out with **Remove**, and Tisty knows whether they are there by reading that file.

All of it, file by file, is in [[Privacy]].

## Where next

- [[Agents and MCP|Agents-and-MCP]]: connecting an agent, the door, the ready-made requests and what it costs.
- [[How an agent works|How-an-Agent-Works]]: a whole session, call by call.
- [[What an agent may do|MCP-Permissions]]: its word on your tasks and **To confirm**.
- [[Agents: your agents' rules, skills and memory|AGENTS-Folder]]: your own rules and skills, written once for all your agents.
- [[Tools reference|MCP-Tools]]: every tool the skills use.
