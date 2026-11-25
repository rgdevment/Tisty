# Agents: your agents' rules

This sheet is a page of the guide. It tells the Agents folder: what bringing is and what syncing is, its three states in the sidebar, how to keep your AI agents' rules, skills and memory in Tisty, how they reach the files each one reads, what to do when a file changes outside or on another computer, how to bring a file in later or take away the one of a rule you take out, how to stop syncing or hide it, and how to approve what your agent proposes.

I use more than one AI agent, and each one reads its rules from a different file, in a different folder: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md`, `GEMINI.md`. Changing one rule everywhere meant opening four files, and sooner or later one fell behind. The Agents folder gathers those files in Tisty, as documents, so you can read and arrange them, and if you want, Tisty writes underneath the file each agent reads.

> [!TIP]
> **In short:** **Bring** copies your agents' files into Tisty, to read and arrange them, and writes nothing into them. **Sync** makes what you change in Tisty reach those files from then on, and has Tisty warn you if someone changes them outside. Before touching a file it keeps what it held, it never writes over a change made outside, and your agent does not write your rules: it proposes the change, and only you approve it.

## What it is and when it helps

Agents is a folder in your Library, at the top, with a robot icon. It sits in your sidebar from day one, in grey, until you bring something in. Each document inside it is one of an agent's files: a **rule**, a **skill** or its **memory**. It works with Claude Code, Codex, GitHub Copilot CLI, Gemini CLI and Cursor.

It helps when:

- you want to read and arrange the rules you already have, as documents, with headings and search: **Bring** is enough for that;
- you use several agents and want the same skill in all of them: for that, **Sync**, with the skill in **Global**;
- you want to be able to go back: while syncing, Tisty keeps the original file and the last 20 before each overwrite;
- you work on more than one computer: each one has its own folder, reads and writes only its own, and what belongs to another computer you read but do not change from this one.

It does not handle each repository's rules: only the files in your user folder, such as `~/.claude/CLAUDE.md`.

## Bring or sync

- **Bring** copies the files you tick into Tisty, as documents. Your agents' files stay as they are: Tisty writes nothing into them and asks you to decide nothing when they change. What you edit in those documents stays in Tisty.
- **Sync** makes what you change in Agents reach your agents' files from then on, with a backup before each overwrite, and has Tisty warn you on a document's card if its file changed outside.
- **Stop syncing** stops it: no agent file is touched and no document is deleted.

**An example.** Jaime only wants to read his `CLAUDE.md` and his skills calmly: he chooses **Bring**, and Claude Code keeps reading its files as always. Yisset uses Claude Code and Codex and wants one rule for both: she chooses **Bring and sync**, writes the rule in Claude's document and in Codex's, inside her computer's folder, and Tisty writes it to her `CLAUDE.md` and her `AGENTS.md`. A skill for both, on the other hand, she writes once in **Agents → Global**.

## Three states in the sidebar

Agents carries no word beside it: its colour tells you how it stands.

1. **Nothing brought yet**: the robot and the name in grey. A click opens the first-time box. Its right-click offers **Hide from the sidebar**.
2. **Brought, not synced**: the grey robot and the name like any folder; when you hover over it, how many documents it holds, like any folder. It opens like any folder, and at the top it shows one quiet line: «Not synced with your agents · Sync», where **Sync** is a link. Its right-click offers **Sync**, **Look again** and **Hide from the sidebar**, nothing else.
3. **Syncing**: the **coral** robot and the name like any folder; when you hover over it, how many documents it holds. It opens like any folder. Its right-click offers **Stop syncing**, **Look again** and **Hide from the sidebar**, which first asks you to stop syncing.

Coral is Agents' own: purple belongs to diagrams. What waits for your decision, such as a file changed outside or a proposal from your agent, does not show in the sidebar: you see it on each document's card and in **Settings › Agents › Agents folder**. The right-click on Agents itself has no other folder action; its subfolders, such as **Agents › Home (this computer) › Claude**, have a menu for agents only: **New document**, **Bring another file**, **Pause sync with Claude** while Agents syncs, **Hide from the sidebar** and **Delete**. **Global** and your computer's folder have no pause, since they belong to no agent, and a folder inside **Claude** has the same menu as **Claude**. Another computer's folder offers only **Hide from the sidebar**.

**Agents stays on the first level**, with its name and its robot: it cannot be put inside another folder, renamed, given another icon or archived, and while it holds anything, it cannot be deleted either, not from the window, not by an agent, not from another computer. Inside it is kept as `AGENTS`, and that is how earlier versions of Tisty and your agents through MCP find it. If you try, for example by dragging it into another folder, Tisty tells you «Agents stays at the top, with its name and its robot, so it keeps guarding your agents' rules. Its color and its place among your folders are yours to change».

## Getting started

1. In the sidebar, click **Agents**, in grey. **Your agents, in one place** opens, with an X at the top to close it, and what it found in your agents' folders: **only their names and sizes**. It has not opened any file yet.
2. At the top, one line tells you the two words: «Tisty looks for the rules, skills and memory of your agents and brings them here as documents, to read and arrange. Bring only copies them; Bring and sync also makes what you change in Agents reach your agents by itself.».
3. Open an agent's row, such as **Claude · 2 rules · 1 skill · 1 memory**, and tick what you want to bring. Each rule is one row, and so is each skill, whole: the row **pr · Skill · 3 files** brings the `pr` skill with all its files, with a single box. Rules and skills come ticked; **Agent memory**, unticked. Settings never show, a credentials file inside a skill stays out, and hidden files, those starting with a dot, do not even show.
4. Choose one of the two buttons:
   - **Bring** copies what you ticked and nothing else: «7 documents came into Agents. Your agents' files stay as they are until you choose Sync.». Agents is left brought, not synced.
   - **Bring and sync**, the highlighted one, also syncs: «7 documents came into Agents, and Tisty keeps them in sync from now on.». Agents is left with its coral robot.

The X or `Esc` close the box without reading or bringing anything, and Agents stays grey; come back whenever you like with another click. The box has no button to hide it: that is on the right-click, as **Stopping sync or hiding Agents**, further down, tells.

The list brings only what your agent really reads. A skill is a folder with its `SKILL.md` at the root, such as `~/.claude/skills/pr/`, found as deep as its agent looks: right inside for Claude, Copilot and Gemini, up to six folders down for Codex, and in any category folder for Cursor. From `~/.claude/rules` the `.md` files come in, and from `~/.copilot/instructions`, the `.instructions.md` ones. Nothing starting with a dot shows, such as Codex's `skills/.system/`, and neither does Claude Code's `skills/synced/`, the copy of the skills in your claude.ai account, which Claude Code may overwrite or delete. Each agent's row counts rules, skills and memories, not files: with nothing to bring, it says **Claude · nothing to bring**. An agent it found with no file yet has a line of its own: «Gemini: no rules, skills or memory yet. It gets what you write in Global or in its folder.».

If you have no agent files yet, the box says «I found no agent files on this computer. You can still sync: Agents fills up as you write your rules.». **Bring** stays greyed out, and **Bring and sync** still makes Agents with **Global** and this computer's folder, with one subfolder for each agent it found, empty, ready for your first rule, and says «Nothing new came in. The Agents folder syncs on this computer now: what you write there reaches your agents.». What your agent makes later you bring with **Look again** or **Bring another file**, as told further down.

**Try it:** click **Agents**, in grey in your sidebar, to see which agents it finds on this computer. If you only wanted to look, the X leaves it as it was. The same is in **Settings › Agents › Agents folder**, on the **Agents** card: before anything is brought it says which agents it found and offers **Bring**, **Bring and sync** and **Hide**; brought, **Sync**, **Hide** and, once this computer has given permission, **Bring another file**; while syncing, **Bring another file**, **Stop syncing**, **Hide** and one switch per agent, such as **Sync Claude**. Under the switch of an agent with no files it says it has no rules, skills or memory yet.

Afterwards, in the sidebar, Agents has **Global**, which keeps what is for all your computers (rules per agent, skills and hooks), and one folder for each of your computers: this one's carries its name with «(this computer)» and the others go last, faint. What you bring goes to this computer's folder, and each agent Tisty found here has its subfolder in it, even an empty one. A loose agent folder at the root, such as **Codex**, shows only if you moved or made something there: it is for all your computers. **Global** has a blue globe and each agent a symbol of its own in its label's colour: an orange chat bubble for Claude, a teal terminal for Codex, indigo braces for Copilot, a green compass for Gemini and a brown pointer for Cursor. A subfolder you delete does not come back. Each document shows with its name and an icon: a book for a rule, a piece for a skill and a brain for memory. Hover over one and you see which file it is.

![The sidebar with the Agents folder open and syncing, with the robot in coral: inside, Global with its blue globe and Claude with its orange chat bubble; in Claude, the rule Rules, picked, with a book icon, and below it the skills, each with a puzzle-piece icon and how many files it holds](agents-sidebar.png)

## A rule: in Tisty and in `CLAUDE.md`

Your `CLAUDE.md` comes into Agents as a document. At the top it carries its properties; the `agent:` line says who it is for, what it is and where it goes: `level/kind/name`, plus one more mark, `on-` and your computer's code, which Tisty puts there. The folder and the line must agree; if they do not, no computer writes it and the card offers **Go by its folder**.

```markdown title="The document in Tisty, in Agents / Home / Claude"
---
agent: [claude/rule/CLAUDE.md, on-dev_ab12cd34]
title: claude · CLAUDE.md
---
# How I work

- Answer in English.
- Branches shaped type/name.
```

In the sidebar it is called **How I work**, after its first heading. And this is what Tisty writes to the file, byte for byte, without the properties:

```markdown title="~/.claude/CLAUDE.md"
# How I work

- Answer in English.
- Branches shaped type/name.
```

To change a rule, open the document, press **Edit** on its card, write and press **Done**. If Agents syncs, a moment later the file says the same.

### One document is one file, and Global is for skills

One document is one file, word for word: what the document says is what the file says, and nothing is joined with anything else. **Global** keeps what is for all your computers: rules per agent, skills and hooks. An agent's rule placed in Global is written on each computer when you press **Apply here** on it.

To add rules to an agent, make another file that agent reads: Claude reads every file in `rules/`, and Copilot every one in `instructions/`. Codex and Gemini read a single rules file.

A rule whose level is `global` (`agent: global/rule/…`) from 1.49 or 1.50 stays there for you to read, copy or delete, but it is no longer written to any agent: copy it into an agent's folder, which may also be inside Global.

> [!IMPORTANT]
> Bring in each agent's main file. If `~/.codex/AGENTS.md` already exists and you never brought it, Tisty cannot tell which part is yours and does not write over it: it asks **It exists on both sides**, and there **Bring it into Agents** brings it in with its own words. **Bring another file**, further down, tells how.

### A new rule, without writing the `agent:` line

Right-click an agent's folder on your computer, and **New document**: the document already starts as a rule of that agent, with its `agent:` line in place.

- In an agent's folder, its main file while no document has it: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md` or, for Gemini, `tisty-rules.md`. Then a file in its rules folder: `rules/rule.md` for Claude, `instructions/rule.instructions.md` for Copilot.
- Codex and Gemini read a single rules file: once a document holds it, **New document** in their folder opens that same one, so you add what is new there.
- In **Global**, in **Cursor**, which has no rules file, in Agents itself and in a subfolder of yours, **New document** makes a plain document, with no `agent:` line.

**An example.** Marisol installs Gemini CLI, and **Agents › Home (this computer) › Gemini** shows up in her sidebar, empty. Right-click **Gemini** and **New document**; on its card she presses **Edit**, writes «Answer in Spanish.» and presses **Done**. Since Agents syncs, Tisty writes it to `~/.gemini/tisty-rules.md` and adds the line `@tisty-rules.md` to her `GEMINI.md`, so Gemini reads it.

## A skill with its `SKILL.md`

A skill is a folder with a `SKILL.md` and, sometimes, scripts. Each file is a document with the same skill folder in its `agent:` line. The `SKILL.md` also carries its own properties: the card takes its `name` as the name, puts its `description` under it and the rest in its facts box, so the text starts where what your agent reads starts:

```markdown title="The SKILL.md of the pr skill"
---
agent: [claude/skill/pr/SKILL.md, on-dev_ab12cd34]
title: claude · pr/SKILL.md
---
---
name: pr
description: Opens a PR with the title and body in the repository's format.
---

# Open a PR

1. Check that the branch is up to date.
2. Run scripts/open-pr.sh.
```

A script is kept inside a code block, and Tisty writes it as is, without the block:

````markdown title="The skill's script"
---
agent: [claude/skill/pr/scripts/open-pr.sh, on-dev_ab12cd34]
title: claude · pr/scripts/open-pr.sh
---
```sh
#!/bin/sh
gh pr create --fill
```
````

Tisty writes the skill to `~/.claude/skills/pr/`. In the sidebar, the skill is **one single row**, **pr**, with **3 files** on the right: a click opens it and shows `SKILL.md`, `scripts/open-pr.sh` and the rest. A **Global** skill is written for all: to `~/.claude/skills/`, to `~/.cursor/skills/` and one single copy to `~/.agents/skills/`, which Codex, Copilot and Gemini read.

**Tisty's skills.** Tisty brings three skills of its own, `tisty`, `consolidate` and `next-agent`, which are not Agents documents: they live inside the app and are turned on with their own yes, in **Settings › Agents › Connected**, even if you never open this folder. In **Agents › Global** you see them as a fixed row, **Tisty's skills**, which opens the same card; it shows only if this computer has some agent, if the skills are on, or if copies of them remain. **Bring** does not bring them: their copies show as **Tisty's own**. A skill of yours with the same name leaves Tisty's out, pausing an agent does not reach them and the row says **On** or **Off**, and opens the card. They are never deleted. [Step by step with your AI agent](tisty:agent) tells it.

## Gemini and the `@tisty-rules.md` line

Gemini writes its own `GEMINI.md` when you ask it to remember something, so that file is Gemini's and Tisty never writes over it. Your rules go to a separate file, `~/.gemini/tisty-rules.md`, and Tisty adds **one single line** at the end of `GEMINI.md` so Gemini reads it:

```markdown title="~/.gemini/GEMINI.md"
## Gemini Added Memories
- Mario's dog is called Rina.
@tisty-rules.md
```

It touches nothing else in that file, and if the line is already there it does not add it again.

**If you delete that line**, Tisty takes it as your decision and does not put it back; it only counts if Tisty had put it there. Gemini stays **Paused**, and Tisty stops writing `tisty-rules.md`. The card of each rule Gemini reads tells you: «Gemini does not read this rule: the line @tisty-rules.md is missing from GEMINI.md, and Tisty does not put it back by itself.». **Settings › Agents › Agents folder**, under **Sync Gemini**, says so too, and so does the box **Tisty's line is missing** when you go into Agents. **Put the line back**, in any of the three, adds the line again at the end of `GEMINI.md`, and Gemini gets your rules again.

## The card of an agent document

A document in Agents does not look like one more document: it opens for reading, on a single **card**, with no column beside it. At the top, the folder path, such as «Agents / Home (this computer) / Claude», and then:

- **the labels**: **Rule**, **Skill** or **Agent memory**; **Global** or the agent; and the standing, such as **In sync**;
- **the name**, **How I work**; on a skill, its `description` sits under it, in grey;
- **the file's facts box**: **Read by**, **Written to** with its button to copy the path, one row per file on a Global skill, **Size**, **Backups** with **See**, **Format**, **Last written**, **Original kept**, **Syncs on this computer**, **Shared** when the document has a live shared link and, in small rows, the file's other properties, such as `disable-model-invocation`;
- **the agent's text**, just as you wrote it, with compact headings and no separate sheet. Tisty imposes no sections on it and never turns its first line into a title, and the file stays the same, byte for byte;
- **the notices and proposals**, inside the same card, under the facts box: a file changed outside, a document that comes from another computer, two documents that go to the same file, words waiting for your approval or the change an agent proposes to you;
- **at the foot, the buttons**, held at the bottom while the whole card scrolls: **Edit**, **Backups**, **History**, **Copy link** and **Pause sync with Claude**; on a Global skill, one button per agent, such as **Pause Claude**, and its backups open with **See**, on each file's row. While you edit, the text moves right to leave room for the editor's bar, as in any document. Another computer's document has no **Edit**, **Apply here**, **Delete** or pause: the card says «It belongs to another of your computers: on this one it is only read.».

Codex reads its `AGENTS.md` only up to 32 KiB. When the file nears that cap, **Size** says so in amber, «close to the 32 KiB Codex reads», and past it, in red: time to move the long part into a skill.

![The card of the rule Rules with no column beside it: at the top the path Agents / Claude; the labels Rule, Claude and In sync; the name in large type; the facts box with Read by Claude, Written to with the path of CLAUDE.md and its copy button, Size, Backups with See, Format Markdown · LF, Last written, Original kept and Syncs on this computer Yes; below, the rule's text with compact headings and no separate sheet; at the foot, the buttons Edit, Backups, History, Copy link and Pause sync with Claude; in the sidebar, Agents open, with Global, Claude, the rule and its skills](agents-card.png)

## A change made outside: bring it in or overwrite

You can keep editing your agents' files in your editor, or let the agent change them. While syncing, Tisty never writes over those changes; brought but not synced, it neither writes those files nor asks you to decide anything.

**An example.** You open `~/.claude/CLAUDE.md` in your editor and add «- Run the tests before every commit.». The next time Tisty looks, the card of **How I work** says **Changed outside**:

- **See the file** shows what it holds now.
- **Bring the change in** writes it into the document. What it said before stays in its history: **History**, at the foot of the card, shows it and lets you go back.
- **Overwrite with Tisty** asks first and writes the file again from the document. What the file held is kept as a backup.

Until you choose, Tisty does not write that file. If you changed both the file and the document, the card says **Changed on both sides**, and you choose which to keep.

## Restoring a backup

Before writing over a file, Tisty keeps what it held. **Backup #1** is the original, the file as it was before Tisty touched it, and it is never removed; it also keeps the last 20, so a file can have up to 21. They live on this computer and do not travel with sync.

**Backups** and **History** keep different things. A backup is your agent's *file*, as it was on disk before Tisty wrote over it. The history is the earlier versions of the *document* in Tisty, like any other's, and [this guide's cover](tisty:guide) tells it, under **Going back to what it said before**. The comparison with what is **Approved** reads from that history: if you empty it, the card asks you to read the whole text before approving it.

1. At the card's foot, press **Backups**, or **See** on the **Backups** row of its facts box, beside «3 backups».
2. Pick one, such as **Backup #2 · 1.1 kB**. **See** shows it whole.
3. **Restore** asks «Put this backup back in `~/.claude/CLAUDE.md`? What the file holds now is kept as a backup.» Say yes.
4. The file says what the backup says again, and the card turns to **Changed outside**: your document did not change. **Bring the change in** makes it match the backup; **Overwrite with Tisty**, if you changed your mind.

That way you never lose either side.

## Pausing an agent

If one day you want Tisty to leave Claude's files alone, for example while you try other rules by hand:

- in **Settings › Agents › Agents folder**, on the **Agents** card, turn off the **Sync Claude** switch;
- or at the foot of a document's card, **Pause sync with Claude**; on a Global skill, the **Pause Claude** button;
- or in the sidebar, right-click **Agents › Home (this computer) › Claude** and choose **Pause sync with Claude**. While paused, the same menu offers **Resume sync with Claude**. This choice shows only while Agents syncs: when it does not sync, there is nothing to pause.

To stop them all at once, **Stop syncing**, as the next section tells.

Paused, Tisty neither opens nor writes those files, and you keep editing your documents. **Resume sync with Claude** writes what you changed, except a file that changed outside meanwhile, which stays marked for you to decide. A pause belongs to this computer only.

## Stopping sync or hiding Agents

**Stopping sync.** Right-click **Agents** and choose **Stop syncing**, or the same button on its card in **Settings › Agents › Agents folder**. No agent file is touched and no document is deleted: your agents keep reading what Tisty wrote last time, and your documents stay in Agents, to read and edit. Agents goes back to the grey robot, and when you open it, it says «Not synced with your agents · Sync» at the top.

**Syncing again.** The **Sync** link in that line, **Sync** on its right-click or on its card. Since this computer had already let you read your agents' files, it syncs at once, without the first-time box. It writes what you changed in the meantime, except a file that changed outside, which stays marked for you to decide. A file you had only brought that is still as it was is taken as it stands.

**An example.** Lucas wants to try some new rules by hand in his `CLAUDE.md` for a week. He chooses **Stop syncing**, edits the file in his editor and keeps arranging his documents in Agents. At the end of the week he presses **Sync**: the card of **How I work** says **Changed outside**, and he chooses **Bring the change in** to keep what he tried, or **Overwrite with Tisty** to go back to what his documents say.

**Taking it out of the sidebar.** Right-click **Agents** and choose **Hide from the sidebar**, or use **Hide** on its card in **Settings › Agents › Agents folder**, in any of its three states. Agents is never hidden while syncing, writing files you do not see: if it syncs, it first warns «To hide Agents, first stop syncing. Your agent files are not touched.», with **Stop syncing and hide** and **Cancel**. **Cancel** leaves it as it was, in sight and syncing. The same goes for its subfolders, such as **Global**: hiding one while Agents syncs goes through the same warning and stops syncing the whole of Agents. It leaves the sidebar on this computer without a trace, and you find it in **Settings › Look and window**, under **Hidden folders**, with **Show again**, which puts it back in the sidebar in the state it had. **Sync**, on the card, syncs it and shows it again too: while syncing, it always shows.

Hidden, Agents does not sync: it writes none of your agents' files, and nothing from those files waits for your decision. Your documents are still there, and if an agent proposes a change, the proposal stays on its document's card.

**If you come from the earlier version**, where the folder showed in capitals and, while it did not write your agents' files, looked dimmed, in grey, with a word beside it, you have nothing to do. If it showed in its colour and wrote those files, you find it syncing, with its coral robot; if it looked dimmed and held documents, brought but not synced. If you had hidden it while it wrote files, you now find it in the sidebar again, syncing: from there you can stop syncing it and hide it.

Nothing that comes through MCP brings, syncs, stops syncing, hides or shows Agents: you do that, in Tisty's window.

## Bring another file

The first time does not have to be the only one. If you install a new agent or write by hand a rules file that did not exist before, you bring it in later. The list is the same as the first time: a skill is a single row that comes in whole, and what is hidden or what the agent itself maintains does not show.

**An example.** You write a new rule for Claude Code in your editor:

```markdown title="~/.claude/rules/tests.md"
# Tests

- Run the tests before every commit.
```

1. In the sidebar, right-click a subfolder of Agents, such as **Agents › Home (this computer) › Claude**, and choose **Bring another file**. With Agents or one of its subfolders open, it is also on the **+** button beside **Library** and in the command palette, under **Create**; and always in **Settings › Agents › Agents folder**, on the **Agents** card. It does not change whether Agents syncs: brought but not synced, your agent's file stays as it is.
2. The box says «These are the agent files on this computer that are not in Agents yet. Choose what to bring:». This time **nothing comes ticked**, and what already has a document says **already in Agents**. **Not now** closes it without bringing anything.
3. Tick `rules/tests.md` and press **Bring**. **Tests** shows up in **Agents › Home (this computer) › Claude**, and from then on you read it and change it in Tisty.

**Look again.** If you installed an agent or your agent made new files, right-click **Agents** itself, brought or syncing, and choose **Look again**. Tisty looks for your agents on this computer again, makes the subfolder of any that is missing and opens **Bring another file** with **only what is new already ticked**; memory, as always, unticked. If nothing is left to bring, not even an unticked memory, it says «Nothing new found. This is what each agent has:» and one line per agent, such as «Claude · already in Agents». On a computer that has not given permission yet, it opens the first-time box instead, with its names and sizes, and opens no file and makes no folder before your yes.

**Bring touches nothing**, not even the record of what Tisty wrote: a file you only brought is never offered for taking away, even if you later take its document out of Agents.

**A file you never brought.** If `~/.codex/AGENTS.md` already held your Codex rules and the first time you left it unticked, and then you write a Codex document that goes to that same file, Tisty cannot tell which part is yours and does not write over it: it asks **It exists on both sides**, with «Tisty has never brought this file. Bring it into Agents and it stays as it is, as a document of this computer.». **Bring it into Agents** brings it in as a document, with its words just as they are, and what it held before stays in its backups. A document is one file: Tisty adds nothing to it.

`AGENTS.override.md` is the file Codex reads instead of `AGENTS.md` when it exists, and for Tisty it is another file and another document: if you make it later, bring it with **Bring another file**, and it sits beside `AGENTS.md`.

## Deleting a rule or a skill

When you delete from Agents a rule or a skill Tisty wrote into an agent's folder, with **Delete** in its menu, Tisty asks right there whether to take it away from your agents too. It is one box: on top, the usual (what goes with the document, its languages and its pages, and that it cannot be undone); below it, in bold, «Take it away from Claude, Codex and Copilot too?», with the agents that have it:

- **Take away too** deletes the document and takes its files out of your agents' folders, with a backup of each on this computer.
- **Only from Tisty** deletes the document and leaves the files where they are: your agents keep reading them.
- **Cancel** deletes nothing.

**An example.** Yisset no longer wants her Global skill **voice**, which Tisty put in `~/.claude/skills/voice/` and in `~/.agents/skills/voice/`, the one Codex and Copilot read. She chooses **Delete**, and the box says «Delete «voice»? The document and what it holds go for good. This cannot be undone.» and «Take it away from Claude, Codex and Copilot too?». With **Take away too**, both files go, each with its backup, and none of her agents reads that skill again.

The box says apart what does not go, and nothing goes without you seeing it:

- **A copy changed outside Tisty**: «Changed outside Tisty in Codex and Copilot: that copy stays, and you see it after, on its own.». After deleting, Tisty asks about it with **Take the file away too?**, which the section below tells.
- **The copy a paused agent reads**: «Paused: Copilot. Its copy stays, and Tisty asks again once it syncs.».

**What you leave with Only from Tisty stays yours.** The **Agents** card in **Settings › Agents › Agents folder** lists it apart, like «2 files you left with your agents when you deleted them from Agents.», with **Review**, in case you change your mind; if you deleted the last document in Agents and the folder went, the list is still there. Tisty does not ask about those files by itself and never writes over them: if a new rule or skill lands on the same file again, it asks you first. That list is this computer's only.

## Taking a rule and its file out

When you move a rule out of Agents, move it to another computer's folder, leave its text with no words or go back to a version without its card, Tisty stops writing it. Its file is still on disk and your agent keeps reading it, so about a second later Tisty asks **Take the file away too?**, with the file's path, for example `~/.claude/rules/tests.md`, and «If you take it away, a backup stays on this computer.»:

- **Take the file away** keeps a backup on this computer and deletes it: Claude stops reading that rule.
- **Leave it** leaves it where it is, and Tisty neither writes it nor asks again. If one day you want it back, it shows in **Bring another file**.

If you edited it by hand after Tisty wrote it, the box tells you before you choose. It never asks about your agent's memory, about a paused agent, or while Agents does not sync: then Tisty wrote no file.

**Deleting a subfolder.** Right-click a subfolder, such as **Agents › Home (this computer) › Claude**, and choose **Delete**. Tisty asks first, and its documents go to **Unfiled**: as they leave Agents, Tisty stops writing them and asks you about each file they wrote, just as when you take a rule out, always with its backup. The folder does not come back by itself.

**An example.** Marisol kept her testing rules in a folder of her own, **Agents › Home (this computer) › Claude › Tests**, and no longer wants them. She right-clicks **Tests** and chooses **Delete**; a second later, Tisty asks **Take the file away too?** about `~/.claude/rules/tests.md`. She chooses **Take the file away**: the backup stays on this computer and Claude stops reading those rules.

**Emptying Agents.** If you take all its documents out with Tisty open, it first asks about each file it wrote, and then the folder goes. Agents stays in the sidebar in grey, as at the start: a click on it opens the first-time box again, with your rules and skills already ticked.

## A second computer

Your documents in Agents travel with your sync to your other computers, but **each computer has its own folder and writes only its own**. What is in another computer's folder you read, move, copy as text and take out; you do not edit it, apply it or delete it from this one.

**An example.** On the home computer you chose **Bring and sync**: your `CLAUDE.md`, **How I work**, sits in **Agents › Home (this computer) › Claude**. On the work computer:

1. In the sidebar, Agents arrives brought, not synced: the grey robot and, last and faint, the **Home** folder with its documents, which you can already read. At the top it says «Not synced with your agents · Sync».
2. Press **Sync**. Since this computer never let you read its agents' files, the first-time box opens: tick what you want to bring from this computer and press **Bring and sync**. This computer's folder is made, with a subfolder for each agent it found, and Agents is left with its coral robot.
3. Open **How I work**, the one in **Home**. The card says «It belongs to another of your computers: on this one it is only read.», with no **Edit** or **Apply here**: on the work computer it is written to no file.
4. If you want that same rule on both computers, move it to the root: right-click the document, **Move to** and **Agents**. On the computer where you move it, it counts as applied; on the other, the card says «This document comes from another computer. Apply it to the agents on this one?». **Apply here** writes it to that computer's files, with its backup first, and **Not on this computer** leaves it out, with **Apply here** at hand if you change your mind.

From then on, what you already apply updates by itself when you change it on the other computer. What you write or bring in on a computer that syncs is applied on that same computer with no click. Moving a document to another computer's folder stops applying it on this one, and Tisty asks about its file. A document with an agent's proposal waiting does not move to another computer until you approve or discard it. And an agent's memory belongs to the computer that brought it: on the others it is only read.

**Two documents to the same file.** If the document in your folder and one at the root write the same file, or the same skill, nothing changes on this computer until you choose: the card says «On this computer «X» already goes to the same file. Which one rules here?», on both documents and whichever came first, with one **Use** button for each that says its title and whose it is. A skill is chosen whole, and it is chosen per file: if you choose for Claude, a Global skill keeps working in the other agents. The one that does not rule stays in its folder, and you can change your mind with its **Use** button. If the one that ruled goes away, Tisty does not write the other by itself: it asks again.

**Approving is not applying.** If on the work computer you approve a proposal, press **Approve this text** or go back to an earlier version of a root document that waits, you change its words on all your computers, but there it keeps waiting. The card says «You approved its words on this computer, but applying it here is a separate choice. Apply it to the agents on this one?», with the same two buttons. Only **Apply here** writes its files on that computer.

**Two Agents folders.** If home and work both made Agents before they synced, two show up for a moment once they meet. Tisty joins them by itself, into the oldest: it puts their subfolders together by name (Global, each computer's…), moves the documents over and deletes the one left empty. It touches none of your agents' files, offers none for taking away and loses no approval. If the second Agents holds something an agent put there, Tisty leaves it as it is until you move that document to a folder of yours yourself.

**If you come from 1.50.1.** What each computer brought or wrote moves by itself to its own folder once all your computers have 1.50.2 or later, and no agent file changes by a byte. A `CLAUDE.md` that 1.50 had joined with a Global rule becomes one document that says exactly what the file says. While another computer is still on an earlier version, nothing moves. A Global rule from that time stays there for you to read, copy or delete.

Two more notices you may see on a card:

- **«Its text does not have your approval yet, so it reaches no agent until you approve it.»**: an amber notice, not a label. The document's text changed without going through your hands, for example through an imported parcel. Before the **Approve this text** button, the notice tells you whose hand wrote it, such as «Written on another computer.» or «Changed outside Tisty.», and what changed: the compared lines, in red with `−` what was **Approved** and in green with `+` what it says **Now**. If it was never approved, it says «Never approved: read all of it before approving it.»; if the words last approved are not on this computer, that there is nothing to compare them with and that you should read all of it. The button asks «Approve what this document says now? Read it first: it reaches your agents' files on each computer where Agents syncs and you applied it.», and if the text changed between being shown and your click, it approves nothing. If that document was already written to a file, the file stays **On hold**, as it was.
- **Written by an earlier Tisty**: an earlier version of Tisty wrote the file and today it does not say what your documents say. **See the file** and, if you keep the document, **Overwrite with Tisty**.

## What your agent can and cannot do

In the Brainsty network, which the page **Brainsty: everything joined** of this guide tells, your Agents documents come hidden, so they do not cover what is yours: they show, small and in a faint coral, when you turn **Show AGENTS** on in its filters, and Tisty never marks them as alike to anything.

Your agent **reads** your documents in Agents like any other, whether it syncs or not: you ask «which of my rules do you read?» and it answers with their titles. But it **does not write them on its own**: when it changes one, its change stays apart, as a proposal, and reaches your agents only if you approve it. While syncing, a rule in Agents reaches all your agents, on each computer where Agents syncs and you applied it, and nobody but you should decide what it says. A document in another of your computers' folders it reads as data, not as a rule of its own for this computer, and it cannot propose on it; on this computer's documents it proposes as always.

Any other tool that names those documents or the folder `AGENTS` is refused: it cannot move, archive or mark them, propose on a memory, or move, rename or archive Agents. Nor can it add or change an `agent:` line in any document, in any folder, not even with `edit_doc`, or make a new document with that line. If it tries, Tisty refuses it, touching nothing, and tells it how to propose the change or to tell you what it would change.

## A proposal from your agent

**An example.** You ask your agent «add to my rules that it runs the tests before every commit».

1. The agent edits **How I work** with `edit_doc`, like any document. Tisty keeps its change apart, byte for byte, and answers that it was proposed: the document and `~/.claude/CLAUDE.md` stay as they were. Its guide asks it to tell you just that, «I proposed it», and not to call it done.
2. Open **How I work**. If Tisty was open, the proposal shows by itself within a second or two, even when the window is not in front.
3. The card gains the label **Proposal** and a purple box: «An agent (Claude Code) proposes a change», and the compared lines, in red with `−` what is **Approved** and in green with `+` what is **Proposed**.
4. **Approve the change** asks «Approve this change? It is written to the document, and reaches your agents' files on each computer where Agents syncs and you applied it.». With your yes, the document says what was proposed and Tisty writes it to `CLAUDE.md`, with its backup first. **Discard** removes it and the document stays as you approved it.

Each button acts only on what the card showed you: if the agent wrote again while you were looking, Tisty asks you to look at it again. If it writes again, its change goes on top of its own proposal, so there is one per document. And while a proposal waits, what you already approved keeps reaching your agents.

## More in the help

Each standing, each agent with its files and variables, what is never read and what to do when something does not add up are in [the Agents folder](https://www.tisty.app/docs/agents-folder/), in the help. What Tisty keeps of all this, and what travels, in [privacy](https://www.tisty.app/docs/privacy/). And how to connect your agent to Tisty, in [Step by step with your AI agent](tisty:agent).
