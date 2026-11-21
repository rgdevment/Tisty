# Agents: your agents' rules

This sheet is a page of the guide. It tells the Agents folder: what bringing is and what syncing is, its three states in the sidebar, how to keep your AI agents' rules, skills and memory in Tisty, how they reach the files each one reads, what to do when a file changes outside or on another computer, how to bring a file in later or take away the one of a rule you take out, how to stop syncing or hide it, and how to approve what your agent proposes.

I use more than one AI agent, and each one reads its rules from a different file, in a different folder: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md`, `GEMINI.md`. Changing one rule everywhere meant opening four files, and sooner or later one fell behind. The Agents folder gathers those files in Tisty, as documents, so you can read and arrange them, and if you want, Tisty writes underneath the file each agent reads.

> [!TIP]
> **In short:** **Bring** copies your agents' files into Tisty, to read and arrange them, and writes nothing into them. **Sync** makes what you change in Tisty reach those files from then on, and has Tisty warn you if someone changes them outside. Before touching a file it keeps what it held, it never writes over a change made outside, and your agent does not write your rules: it proposes the change, and only you approve it.

## What it is and when it helps

Agents is a folder in your Library, at the top, with a robot icon. It sits in your sidebar from day one, in grey, until you bring something in. Each document inside it is one of an agent's files: a **rule**, a **skill** or its **memory**. It works with Claude Code, Codex, GitHub Copilot CLI, Gemini CLI and Cursor.

It helps when:

- you want to read and arrange the rules you already have, as documents, with headings and search: **Bring** is enough for that;
- you use several agents and want the same rules in all of them: for that, **Sync**;
- you want to be able to go back: while syncing, Tisty keeps the original file and the last 20 before each overwrite;
- you work on more than one computer and want to choose which one syncs and which rules each one applies.

It does not handle each repository's rules: only the files in your user folder, such as `~/.claude/CLAUDE.md`.

## Bring or sync

- **Bring** copies the files you tick into Tisty, as documents. Your agents' files stay as they are: Tisty writes nothing into them and asks you to decide nothing when they change. What you edit in those documents stays in Tisty.
- **Sync** makes what you change in Agents reach your agents' files from then on, with a backup before each overwrite, and has Tisty warn you on a document's card if its file changed outside.
- **Stop syncing** stops it: no agent file is touched and no document is deleted.

**An example.** Jaime only wants to read his `CLAUDE.md` and his skills calmly: he chooses **Bring**, and Claude Code keeps reading its files as always. Yisset uses Claude Code and Codex and wants one rule for both: she chooses **Bring and sync**, writes the rule once in **Agents → Global**, and Tisty writes it to her `CLAUDE.md` and her `AGENTS.md`.

## Three states in the sidebar

Agents carries no word beside it: its colour tells you how it stands.

1. **Nothing brought yet**: the robot and the name in grey. A click opens the first-time box. Its right-click offers **Hide from the sidebar**.
2. **Brought, not synced**: the grey robot and the name like any folder; when you hover over it, how many documents it holds, like any folder. It opens like any folder, and at the top it shows one quiet line: «Not synced with your agents · Sync», where **Sync** is a link. Its right-click offers **Sync**, **Look again** and **Hide from the sidebar**, nothing else.
3. **Syncing**: the **coral** robot and the name like any folder; when you hover over it, how many documents it holds. It opens like any folder. Its right-click offers only **Stop syncing** and **Look again**: to hide it, stop syncing first.

Coral is Agents' own: purple belongs to diagrams. What waits for your decision, such as a file changed outside or a proposal from your agent, does not show in the sidebar: you see it on each document's card and in **Settings → Assistants**. The right-click on Agents itself has no other folder action; its subfolders, such as **Agents › Claude**, keep the usual menu, with **New document** and **Bring another file**.

**Agents stays on the first level**, with its name and its robot: it cannot be put inside another folder, renamed, given another icon or archived, and while it holds anything, it cannot be deleted either, not from the window, not by an agent, not from another computer. Inside it is kept as `AGENTS`, and that is how earlier versions of Tisty and your assistants through MCP find it. If you try, for example by dragging it into another folder, Tisty tells you «Agents stays at the top, with its name and its robot, so it keeps guarding your agents' rules. Its color and its place among your folders are yours to change».

## Getting started

1. In the sidebar, click **Agents**, in grey. **Your agents, in one place** opens, with an X at the top to close it, and what it found in your assistants' folders: **only their names and sizes**. It has not opened any file yet.
2. At the top, one line tells you the two words: «Tisty looks for the rules, skills and memory of your agents and brings them here as documents, to read and arrange. Bring only copies them; Bring and sync also makes what you change in Agents reach your agents by itself.».
3. Open an assistant's row, such as **Claude · 2 rules · 1 skill · 1 memory**, and tick what you want to bring. Each rule is one row, and so is each skill, whole: the row **pr · Skill · 3 files** brings the `pr` skill with all its files, with a single box. Rules and skills come ticked; **Agent memory**, unticked. Settings and credentials show greyed out and stay out, and hidden files, those starting with a dot, do not even show.
4. Choose one of the two buttons:
   - **Bring** copies what you ticked and nothing else: «7 documents came into Agents. Your agents' files stay as they are until you choose Sync.». Agents is left brought, not synced.
   - **Bring and sync**, the highlighted one, also syncs: «7 documents came into Agents, and Tisty keeps them in sync from now on.». Agents is left with its coral robot.

The X or `Esc` close the box without reading or bringing anything, and Agents stays grey; come back whenever you like with another click. The box has no button to hide it: that is on the right-click, as **Stopping sync or hiding Agents**, further down, tells.

The list brings only what your assistant really reads. A skill is a folder with its `SKILL.md` at the root, such as `~/.claude/skills/pr/`, found as deep as its assistant looks: right inside for Claude, Copilot and Gemini, up to six folders down for Codex, and in any category folder for Cursor. From `~/.claude/rules` the `.md` files come in, and from `~/.copilot/instructions`, the `.instructions.md` ones. Nothing starting with a dot shows, such as Codex's `skills/.system/`, and neither does Claude Code's `skills/synced/`, the copy of the skills in your claude.ai account, which Claude Code may overwrite or delete. Each assistant's row counts rules, skills and memories, not files: with nothing to bring, it says **Claude · nothing to bring**. An assistant it found with no file yet has a line of its own: «Gemini: no rules, skills or memory yet. It gets what you write in Global or in its folder.».

If you have no agent files yet, the box says «I found no agent files on this computer. You can still sync: Agents fills up as you write your rules.». **Bring** stays greyed out, and **Bring and sync** still makes Agents with **Global** and one subfolder for each assistant it found, empty, ready for your first rule, and says «Nothing new came in. Agents syncs on this computer now: what you write there reaches its assistants.». What your agent makes later you bring with **Look again** or **Bring another file**, as told further down.

**Try it:** click **Agents**, in grey in your sidebar, to see which assistants it finds on this computer. If you only wanted to look, the X leaves it as it was. The same is in **Settings → Assistants**, on the **Agents** card: before anything is brought it says which assistants it found and offers **Bring**, **Bring and sync** and **Hide**; brought, **Sync**, **Hide** and, once this computer has given permission, **Bring another file**; while syncing, **Bring another file**, **Stop syncing** and one switch per assistant, such as **Sync Claude**. Under the switch of an assistant with no files it says it has no rules, skills or memory yet.

Afterwards, in the sidebar, each assistant Tisty found on this computer has its subfolder, even an empty one. **Global** has a blue globe and each assistant a symbol of its own in its label's colour: an orange chat bubble for Claude, a teal terminal for Codex, indigo braces for Copilot, a green compass for Gemini and a brown pointer for Cursor. A subfolder you delete does not come back. Each document shows with its name and an icon: a book for a rule, a piece for a skill and a brain for memory. Hover over one and you see which file it is.

![The sidebar with the Agents folder open and syncing: the robot in coral and the name Agents, with no word on the right; inside, the subfolders with their symbol and colour: Global with a blue globe, Claude with an orange chat bubble, Codex with a teal terminal and Gemini with a green compass; in Claude, the rule How I work with a book icon, the skill pr with a puzzle-piece icon and 3 files, and the memory MEMORY.md with a brain icon; the pointer over How I work shows claude · CLAUDE.md](agents-sidebar.png)

## A rule: in Tisty and in `CLAUDE.md`

Your `CLAUDE.md` comes into Agents as a document. At the top it carries its properties; the `agent:` line says who it is for, what it is and where it goes: `level/kind/name`.

```markdown title="The document in Tisty, in Agents / Claude"
---
agent: claude/rule/CLAUDE.md
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

### One rule for all: Global

While syncing, a document in **Agents → Global** reaches every assistant on this computer. It goes at the start of each one's main file, before its own rules:

```markdown title="The document in Agents / Global"
---
agent: global/rule/for-all.md
title: global · for-all.md
---
# For all my agents

- Ask before deleting anything.
```

```markdown title="~/.claude/CLAUDE.md, with Global in front"
# For all my agents

- Ask before deleting anything.

# How I work

- Answer in English.
- Branches shaped type/name.
```

The same goes for Codex's `AGENTS.md`, Copilot's `copilot-instructions.md` and Gemini's rules. Cursor gets no rules: its own live in its settings.

> [!IMPORTANT]
> Bring in each assistant's main file. If `~/.codex/AGENTS.md` already exists and you never brought it, Tisty cannot tell which part is yours and does not write over it: it asks **It exists on both sides**, and there **Bring it into Agents** brings it in with its own words and writes Global in front, with its backup first. **Bring another file**, further down, tells how.

### A new rule, without writing the `agent:` line

Right-click **Global** or an assistant's folder, and **New document**: the document already starts as a rule of that place, with its `agent:` line in place.

- In **Global**, `agent: global/rule/rule.md`; the next one, `rule-2.md`.
- In an assistant's folder, its main file while no document has it: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md` or, for Gemini, `tisty-rules.md`. Then a file in its rules folder: `rules/rule.md` for Claude, `instructions/rule.instructions.md` for Copilot.
- Codex and Gemini read a single rules file: once a document holds it, **New document** in their folder opens that same one, so you add what is new there, or in **Global**.
- Two Global rules born with the same name on two computers before they synced each keep one: the older stays `rule.md` and the other becomes `rule-2.md`, and both reach your agents once their text has your approval.
- In **Cursor**, which has no rules file, in Agents itself and in a subfolder of yours, **New document** makes a plain document, with no `agent:` line.

**An example.** Marisol installs Gemini CLI, and **Agents › Gemini** shows up in her sidebar, empty. Right-click **Gemini** and **New document**; on its card she presses **Edit**, writes «Answer in Spanish.» and presses **Done**. Since Agents syncs, Tisty writes it to `~/.gemini/tisty-rules.md` and adds the line `@tisty-rules.md` to her `GEMINI.md`, so Gemini reads it.

## A skill with its `SKILL.md`

A skill is a folder with a `SKILL.md` and, sometimes, scripts. Each file is a document with the same skill folder in its `agent:` line. The `SKILL.md` also carries its own properties: the card takes its `name` as the name, puts its `description` under it and the rest in its facts box, so the text starts where what your agent reads starts:

```markdown title="The SKILL.md of the pr skill"
---
agent: claude/skill/pr/SKILL.md
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
agent: claude/skill/pr/scripts/open-pr.sh
title: claude · pr/scripts/open-pr.sh
---
```sh
#!/bin/sh
gh pr create --fill
```
````

Tisty writes the skill to `~/.claude/skills/pr/`. In the sidebar, the skill is **one single row**, **pr**, with **3 files** on the right: a click opens it and shows `SKILL.md`, `scripts/open-pr.sh` and the rest. A **Global** skill is written for all: to `~/.claude/skills/`, to `~/.cursor/skills/` and one single copy to `~/.agents/skills/`, which Codex, Copilot and Gemini read.

## Gemini and the `@tisty-rules.md` line

Gemini writes its own `GEMINI.md` when you ask it to remember something, so that file is Gemini's and Tisty never writes over it. Your rules go to a separate file, `~/.gemini/tisty-rules.md`, and Tisty adds **one single line** at the end of `GEMINI.md` so Gemini reads it:

```markdown title="~/.gemini/GEMINI.md"
## Gemini Added Memories
- Mario's dog is called Rina.
@tisty-rules.md
```

It touches nothing else in that file, and if the line is already there it does not add it again.

**If you delete that line**, Tisty takes it as your decision and does not put it back; it only counts if Tisty had put it there. Gemini stays **Paused**, and Tisty stops writing `tisty-rules.md`. The card of each rule Gemini reads tells you: «Gemini does not read this rule: the line @tisty-rules.md is missing from GEMINI.md, and Tisty does not put it back by itself.». **Settings → Assistants**, under **Sync Gemini**, says so too, and so does the box **Tisty's line is missing** when you go into Agents. **Put the line back**, in any of the three, adds the line again at the end of `GEMINI.md`, and Gemini gets your rules again.

## The card of an agent document

A document in Agents does not look like one more document: it opens for reading, on a single **card**, with no column beside it. At the top, the folder path, such as «Agents / Claude», and then:

- **the labels**: **Rule**, **Skill** or **Agent memory**; **Global** or the assistant; and the standing, such as **In sync**;
- **the name**, **How I work**; on a skill, its `description` sits under it, in grey;
- **the file's facts box**: **Read by**, **Written to** with its button to copy the path, one row per file on a Global document, **Size**, **Backups** with **See**, **Format**, **Last written**, **Original kept**, **Syncs on this computer**, **Shared** when the document has a live shared link and, in small rows, the file's other properties, such as `disable-model-invocation`;
- **the agent's text**, just as you wrote it, with compact headings and no separate sheet. Tisty imposes no sections on it and never turns its first line into a title, and the file stays the same, byte for byte;
- **the notices and proposals**, inside the same card, under the facts box: a file changed outside, a document that comes from another computer, words waiting for your approval or the change an agent proposes to you;
- **at the foot, the buttons**, held at the bottom while the whole card scrolls: **Edit**, **Backups**, **History**, **Copy link** and **Pause sync with Claude**; on a Global document, one button per assistant, such as **Pause Claude**, and its backups open with **See**, on each file's row. While you edit, the text moves right to leave room for the editor's bar, as in any document.

Codex reads its `AGENTS.md` only up to 32 KiB. When the file nears that cap, **Size** says so in amber, «close to the 32 KiB Codex reads», and past it, in red: time to move the long part into a skill.

![PLACEHOLDER — The card of How I work with no column beside it: at the top the path Agents / Claude; the labels Rule, Claude and In sync; the name in large type; the facts box with Read by Claude, Written to ~/.claude/CLAUDE.md with its copy button, Size 3.2 kB, Backups 3 backups · See, Format Markdown · LF and Syncs on this computer Yes; below, the rule's text with compact headings and no separate sheet; at the foot, the buttons Edit, Backups, History, Copy link and Pause sync with Claude](agents-card.png)

## A change made outside: bring it in or overwrite

You can keep editing your agents' files in your editor, or let the agent change them. While syncing, Tisty never writes over those changes; brought but not synced, it neither writes those files nor asks you to decide anything.

**An example.** You open `~/.claude/CLAUDE.md` in your editor and add «- Run the tests before every commit.». The next time Tisty looks, the card of **How I work** says **Changed outside**:

- **See the file** shows what it holds now.
- **Bring the change in** writes it into the document. What it said before stays in its history: **History**, at the foot of the card, shows it and lets you go back.
- **Overwrite with Tisty** asks first and writes the file again from the document. What the file held is kept as a backup.

Until you choose, Tisty does not write that file. If you changed both the file and the document, the card says **Changed on both sides**, and you choose which to keep.

![The card of How I work with the labels Rule, Claude and Changed outside in amber, the facts box with Written to ~/.claude/CLAUDE.md, and below it, inside the same card, the notice The file changed outside Tisty, and the buttons Bring the change in, Overwrite with Tisty and See the file](agents-outside.png)

## Restoring a backup

Before writing over a file, Tisty keeps what it held. **Backup #1** is the original, the file as it was before Tisty touched it, and it is never removed; it also keeps the last 20, so a file can have up to 21. They live on this computer and do not travel with sync.

**Backups** and **History** keep different things. A backup is your agent's *file*, as it was on disk before Tisty wrote over it. The history is the earlier versions of the *document* in Tisty, like any other's, and [this guide's cover](tisty:guide) tells it, under **Going back to what it said before**. The comparison with what is **Approved** reads from that history: if you empty it, the card asks you to read the whole text before approving it.

1. At the card's foot, press **Backups**, or **See** on the **Backups** row of its facts box, beside «3 backups».
2. Pick one, such as **Backup #2 · 1.1 kB**. **See** shows it whole.
3. **Restore** asks «Put this backup back in `~/.claude/CLAUDE.md`? What the file holds now is kept as a backup.» Say yes.
4. The file says what the backup says again, and the card turns to **Changed outside**: your document did not change. **Bring the change in** makes it match the backup; **Overwrite with Tisty**, if you changed your mind.

That way you never lose either side.

## Pausing an assistant

If one day you want Tisty to leave Claude's files alone, for example while you try other rules by hand:

- in **Settings → Assistants**, on the **Agents** card, turn off the **Sync Claude** switch;
- or at the foot of a document's card, **Pause sync with Claude**; on a Global document, the **Pause Claude** button.

To stop them all at once, **Stop syncing**, as the next section tells.

Paused, Tisty neither opens nor writes those files, and you keep editing your documents. **Resume sync with Claude** writes what you changed, except a file that changed outside meanwhile, which stays marked for you to decide. A pause belongs to this computer only.

## Stopping sync or hiding Agents

**Stopping sync.** Right-click **Agents** and choose **Stop syncing**, or the same button on its card in **Settings → Assistants**. No agent file is touched and no document is deleted: your agents keep reading what Tisty wrote last time, and your documents stay in Agents, to read and edit. Agents goes back to the grey robot, and when you open it, it says «Not synced with your agents · Sync» at the top.

**Syncing again.** The **Sync** link in that line, **Sync** on its right-click or on its card. Since this computer had already let you read your agents' files, it syncs at once, without the first-time box. It writes what you changed in the meantime, except a file that changed outside, which stays marked for you to decide. A file you had only brought that is still as it was is taken as it stands, and what joins it, such as a Global rule, is written over it with its backup first.

**An example.** Lucas wants to try some new rules by hand in his `CLAUDE.md` for a week. He chooses **Stop syncing**, edits the file in his editor and keeps arranging his documents in Agents. At the end of the week he presses **Sync**: the card of **How I work** says **Changed outside**, and he chooses **Bring the change in** to keep what he tried, or **Overwrite with Tisty** to go back to what his documents say.

**Taking it out of the sidebar.** Agents hides only while it does not sync: while syncing, its right-click offers **Stop syncing** and no hiding, so it never keeps writing files you do not see. The same goes for its subfolders, such as **Global**: while Agents syncs, none of them hides. With nothing brought yet or brought but not synced, right-click it and choose **Hide from the sidebar**, or use **Hide** on its card in **Settings → Assistants**. It leaves the sidebar on this computer without a trace, and you find it in **Settings → General**, under **Hidden folders**, with **Show again**, which puts it back in the sidebar in the state it had. **Sync**, on the card, syncs it and shows it again too: while syncing, it always shows.

Hidden, Agents does not sync: it writes none of your agents' files, and nothing from those files waits for your decision. Your documents are still there, and if an agent proposes a change, the proposal stays on its document's card.

**If you come from the earlier version**, where the folder showed in capitals and, while it did not write your agents' files, looked dimmed, in grey, with a word beside it, you have nothing to do. If it showed in its colour and wrote those files, you find it syncing, with its coral robot; if it looked dimmed and held documents, brought but not synced. If you had hidden it while it wrote files, you now find it in the sidebar again, syncing: from there you can stop syncing it and hide it.

Nothing that comes through MCP brings, syncs, stops syncing, hides or shows Agents: you do that, in Tisty's window.

## Bring another file

The first time does not have to be the only one. If you install a new assistant or write by hand a rules file that did not exist before, you bring it in later. The list is the same as the first time: a skill is a single row that comes in whole, and what is hidden or what the assistant itself maintains does not show.

**An example.** You write a new rule for Claude Code in your editor:

```markdown title="~/.claude/rules/tests.md"
# Tests

- Run the tests before every commit.
```

1. In the sidebar, right-click a subfolder of Agents, such as **Agents › Claude**, and choose **Bring another file**. With Agents or one of its subfolders open, it is also on the **+** button beside **Library** and in the command palette, under **Create**; and always in **Settings → Assistants**, on the **Agents** card. It does not change whether Agents syncs: brought but not synced, your agent's file stays as it is.
2. The box says «These are the agent files on this computer that are not in Agents yet. Choose what to bring:». This time **nothing comes ticked**, and what already has a document says **already in Agents**. **Not now** closes it without bringing anything.
3. Tick `rules/tests.md` and press **Bring**. **Tests** shows up in **Agents › Claude**, and from then on you read it and change it in Tisty.

![The Bring another file box, with the line These are the agent files on this computer that are not in Agents yet. Choose what to bring:, Claude's row open with CLAUDE.md dimmed and already in Agents, and rules/tests.md ticked as a Rule; at the bottom, the buttons Not now and Bring](agents-bring-another.png)

**Look again.** If you installed an assistant or your agent made new files, right-click **Agents** itself, brought or syncing, and choose **Look again**. Tisty looks for your assistants on this computer again, makes the subfolder of any that is missing and opens **Bring another file** with **only what is new already ticked**; memory, as always, unticked. If nothing is left to bring, not even an unticked memory, it says «Nothing new found. This is what each assistant has:» and one line per assistant, such as «Claude · already in Agents». On a computer that has not given permission yet, it opens the first-time box instead, with its names and sizes, and opens no file and makes no folder before your yes.

**Bring touches nothing**, not even the record of what Tisty wrote: a file you only brought is never offered for taking away, even if you later take its document out of Agents.

**A file you never brought.** If `~/.codex/AGENTS.md` already held your Codex rules and the first time you left it unticked, a Global rule does not write over it: Tisty asks **It exists on both sides**, with «Tisty has never brought this file. Bring it into Agents and it keeps what it says, with Tisty's part added on top.». **Bring it into Agents** brings it in as a document and writes Global in front of yours, with its backup first:

```markdown title="~/.codex/AGENTS.md, after Bring it into Agents"
# For all my agents

- Ask before deleting anything.

# Codex rules

- Use pnpm, never npm.
```

The same goes for `AGENTS.override.md`, the file Codex reads instead of `AGENTS.md` when it exists: if you make it later, bring it into Agents, and from then on Global goes there.

## Taking a rule and its file out

When you delete a rule, move it out of Agents or go back to a version without its card, Tisty stops writing it. If it was the only one making up its file, that file is still on disk and your agent keeps reading it, so about a second later Tisty asks:

![The box Take the file away too?, with the line No rule in Agents writes this Claude file any more, but it is still on disk and Claude keeps reading it., the path ~/.claude/rules/tests.md in grey, the line If you take it away, a backup stays on this computer., and the buttons Leave it and Take the file away](agents-take-away.png)

- **Take the file away** keeps a backup on this computer and deletes it: Claude stops reading that rule.
- **Leave it** leaves it where it is, and Tisty neither writes it nor asks again. If one day you want it back, it shows in **Bring another file**.

If you edited it by hand after Tisty wrote it, the box tells you before you choose. It never asks about your agent's memory, about a paused assistant, or while Agents does not sync: then Tisty wrote no file.

**Emptying Agents.** If you take all its documents out with Tisty open, it first asks about each file it wrote, and then the folder goes. Agents stays in the sidebar in grey, as at the start: a click on it opens the first-time box again, with your rules and skills already ticked.

## A second computer

Your documents in Agents travel with your sync to your other computers, but **each computer chooses whether it syncs and what it applies to its agents**.

**An example.** On the home computer you chose **Bring and sync**, brought in your `CLAUDE.md` and wrote the rule «For all my agents». There they applied by themselves. On the work computer:

1. In the sidebar, Agents arrives brought, not synced: the grey robot and its documents, which you can already read. At the top it says «Not synced with your agents · Sync».
2. Press **Sync**. Since this computer never let you read its agents' files, the first-time box opens: tick what you want to bring from this computer and press **Bring and sync**. Agents is left with its coral robot.
3. Open **For all my agents**. The card says «This document comes from another computer. Apply it to the agents on this one?».
4. **Apply here** writes it to this computer's files, with its backup first. **Not on this computer** leaves it out here, and **Apply here** stays at hand if you change your mind.

From then on, what you already apply updates by itself when you change it on the other computer. What you write or bring in on a computer that syncs is applied on that same computer with no click. And an agent's memory belongs to the computer that brought it: on the others it is only read.

**Approving is not applying.** If on the work computer you approve a proposal, press **Approve this text** or go back to an earlier version of a document that waits, you change its words on all your computers, but there it keeps waiting. The card says «You approved its words on this computer, but applying it here is a separate choice. Apply it to the agents on this one?», with the same two buttons. Only **Apply here** writes its files on that computer.

**Two Agents folders.** If home and work both made Agents before they synced, two show up for a moment once they meet. Tisty joins them by itself, into the oldest: it puts their subfolders together by name (Global, Claude…), moves the documents over and deletes the one left empty. It touches none of your agents' files, offers none for taking away and loses no approval, and the other computer's documents wait for their **Apply here** like any other. A document the other computer wrote right before the join goes into the subfolder of the same name by itself once it arrives. If the second Agents holds something an agent put there, Tisty leaves it as it is until you move that document to a folder of yours yourself.

![The card of For all my agents on the work computer, with the labels Rule and Global, the facts box with no Written to row and with Syncs on this computer No: it waits for «Apply here», and below it, inside the same card, the amber notice This document comes from another computer. Apply it to the agents on this one? with the buttons Apply here and Not on this computer; in the sidebar, Agents with its coral robot, with no word beside it](agents-other-computer.png)

Two more notices you may see on a card:

- **«Its text does not have your approval yet, so it reaches no agent until you approve it.»**: an amber notice, not a label. The document's text changed without going through your hands, for example through an imported parcel. Before the **Approve this text** button, the notice tells you whose hand wrote it, such as «Written on another computer.» or «Changed outside Tisty.», and what changed: the compared lines, in red with `−` what was **Approved** and in green with `+` what it says **Now**. If it was never approved, it says «Never approved: read all of it before approving it.»; if the words last approved are not on this computer, that there is nothing to compare them with and that you should read all of it. The button asks «Approve what this document says now? Read it first: it reaches your agents' files on each computer where Agents syncs and you applied it.», and if the text changed between being shown and your click, it approves nothing. If that document was already written to a file, the file stays **On hold**, as it was.
- **Written by an earlier Tisty**: an earlier version of Tisty wrote the file and today it does not say what your documents say. **See the file** and, if you keep the document, **Overwrite with Tisty**.

## What your agent can and cannot do

In the Brainsty network, which the page **Brainsty: everything joined** of this guide tells, your Agents documents come hidden, so they do not cover what is yours: they show, small and in a faint coral, when you turn **Show AGENTS** on in its filters, and Tisty never marks them as alike to anything.

Your agent **reads** your documents in Agents like any other, whether it syncs or not: you ask «which of my rules do you read?» and it answers with their titles. But it **does not write them on its own**: when it changes one, its change stays apart, as a proposal, and reaches your agents only if you approve it. While syncing, a rule in Agents reaches all your agents, on each computer where Agents syncs and you applied it, and nobody but you should decide what it says.

Any other tool that names those documents or the folder `AGENTS` is refused: it cannot move, archive or mark them, propose on a memory, or move, rename or archive Agents. Nor can it add or change an `agent:` line in any document, in any folder, not even with `edit_doc`, or make a new document with that line. If it tries, Tisty refuses it, touching nothing, and tells it how to propose the change or to tell you what it would change.

## A proposal from your agent

**An example.** You ask your agent «add to my rules that it runs the tests before every commit».

1. The agent edits **How I work** with `edit_doc`, like any document. Tisty keeps its change apart, byte for byte, and answers that it was proposed: the document and `~/.claude/CLAUDE.md` stay as they were. Its guide asks it to tell you just that, «I proposed it», and not to call it done.
2. Open **How I work**. If Tisty was open, the proposal shows by itself within a second or two, even when the window is not in front.
3. The card gains the label **Proposal** and a purple box: «An agent (Claude Code) proposes a change», and the compared lines, in red with `−` what is **Approved** and in green with `+` what is **Proposed**.
4. **Approve the change** asks «Approve this change? It is written to the document, and reaches your agents' files on each computer where Agents syncs and you applied it.». With your yes, the document says what was proposed and Tisty writes it to `CLAUDE.md`, with its backup first. **Discard** removes it and the document stays as you approved it.

![The card of How I work with the labels Rule, Claude, In sync and Proposal, and the purple box An agent (Claude Code) proposes a change, with the compared lines − Approved and + Proposed, the new line + - Run the tests before every commit. in green, and the buttons Approve the change and Discard](agents-proposal.png)

Each button acts only on what the card showed you: if the agent wrote again while you were looking, Tisty asks you to look at it again. If it writes again, its change goes on top of its own proposal, so there is one per document. And while a proposal waits, what you already approved keeps reaching your agents.

## More in the help

Each standing, each assistant with its files and variables, what is never read and what to do when something does not add up are in [the Agents folder](https://www.tisty.app/docs/agents-folder/), in the help. What Tisty keeps of all this, and what travels, in [privacy](https://www.tisty.app/docs/privacy/). And how to connect your agent to Tisty, in [Step by step with your AI agent](tisty:agent).
