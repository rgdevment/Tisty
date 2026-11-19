# AGENTS: your agents' rules

This sheet is a page of the guide. It tells the AGENTS folder: how to keep your AI agents' rules, skills and memory in Tisty, how they reach the files each one reads, and what to do when a file changes outside, on another computer or by an agent's hand, and how to approve what your agent proposes.

I use more than one AI agent, and each one reads its rules from a different file, in a different folder: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md`, `GEMINI.md`. Changing one rule everywhere meant opening four files, and sooner or later one fell behind. AGENTS gathers those files in Tisty, as documents, and Tisty writes underneath the file each agent reads.

> [!TIP]
> **In short:** you write your rules once, in Tisty, and Tisty writes them to each agent's file. Before touching a file it keeps what it held, it never writes over a change made outside, and your agent does not write your rules: it proposes the change, and only you approve it.

## What it is and when it helps

AGENTS is a folder in your Library, at the top, with a robot icon. Each document inside it is one of an agent's files: a **rule**, a **skill** or its **memory**. It works with Claude Code, Codex, GitHub Copilot CLI, Gemini CLI and Cursor.

It helps when:

- you use several agents and want the same rules in all of them;
- your rules grew and you want to read them as documents, with headings and search;
- you want to be able to go back: Tisty keeps the original file and the last 20 before each overwrite;
- you work on more than one computer and want to choose which rules each one applies.

It does not handle each repository's rules: only the files in your user folder, such as `~/.claude/CLAUDE.md`.

## Getting started

1. Open **Settings → Assistants**. On the **AGENTS** card, press **Create the AGENTS folder**.
2. **Your agents, in one place** opens, with what it found in your assistants' folders: **only their names and sizes**. It has not opened any file yet.
3. Open an assistant's row, such as **Claude · 7 files**, and tick what you want to bring. Rules and skills come ticked; **Agent memory**, unticked. Settings and credentials show greyed out and stay out.
4. Press **Bring and sync**. Tisty opens only what you ticked and tells you how many documents came in.

**Try it:** open **Settings → Assistants** and press **Create the AGENTS folder** to see what it finds on this computer. If you only wanted to look, **Not now** closes the box without reading anything. If you already made it, the card offers no button: the folder sits at the top of your Library.

Afterwards, in the sidebar, each document shows with its name and an icon: a book for a rule, a piece for a skill and a brain for memory. Hover over one and you see which file it is.

![The sidebar with the AGENTS folder open and in sync: in Claude, the rule How I work with a book icon, the skill pr with a puzzle-piece icon and 3 files, and the memory MEMORY.md with a brain icon; the pointer over How I work shows claude · CLAUDE.md](agents-sidebar.png)

## A rule: in Tisty and in `CLAUDE.md`

Your `CLAUDE.md` comes into AGENTS as a document. At the top it carries its properties; the `agent:` line says who it is for, what it is and where it goes: `level/kind/name`.

```markdown title="The document in Tisty, in AGENTS / Claude"
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

To change a rule, open the document, press **Edit** on its card, write and press **Done**. A moment later the file says the same.

### One rule for all: Global

A document in **AGENTS → Global** reaches every assistant on this computer. It goes at the start of each one's main file, before its own rules:

```markdown title="The document in AGENTS / Global"
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
> The first time, bring in each assistant's main file. If `~/.codex/AGENTS.md` already exists and you did not bring it, Tisty cannot tell which part is yours and does not write over it: it stays at **Changed on both sides** until you choose.

## A skill with its `SKILL.md`

A skill is a folder with a `SKILL.md` and, sometimes, scripts. Each file is a document with the same skill folder in its `agent:` line. The `SKILL.md` also carries its own properties, which the card shows as fields:

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

It touches nothing else in that file, and if the line is already there it does not add it again. If you delete it, it adds it back the next time it goes over your files.

## The card and the Agent file panel

An AGENTS document opens for reading, with a **card** at the top:

- the labels: **Rule**, **Skill** or **Agent memory**; **Global** or the assistant; and the standing, such as **In sync**;
- the name, **How I work**, and under it, in small type, the path, `CLAUDE.md`;
- one row per file: «Claude reads it at `~/.claude/CLAUDE.md`», with when it changed and its backups;
- the buttons **Edit**, **Backups** and **Pause sync with Claude**.

Beside it, the **Agent file** panel takes the place of the paper size and the outline: **Read by**, **Written to**, **Format**, **Last written**, **Backups**, **Original kept** and **Syncs on this computer**, with **Copy link** and, once it has earlier versions, **History**.

![The card of How I work with the labels Rule, Claude and In sync, the name in large type and the path CLAUDE.md under it, the row Claude reads it at ~/.claude/CLAUDE.md with 3 backups, and the buttons Edit, Backups and Pause sync with Claude; on the right, the Agent file panel with Read by Claude, Format Markdown · LF and Syncs on this computer Yes](agents-card.png)

## A change made outside: bring it in or overwrite

You can keep editing your agents' files in your editor, or let the agent change them. Tisty never writes over those changes.

**An example.** You open `~/.claude/CLAUDE.md` in your editor and add «- Run the tests before every commit.». The next time Tisty looks, the sidebar says **1 changed** beside AGENTS, and the card of **How I work** says **Changed outside**:

- **See the file** shows what it holds now.
- **Bring the change in** writes it into the document. What it said before stays in its history.
- **Overwrite with Tisty** asks first and writes the file again from the document. What the file held is kept as a backup.

Until you choose, Tisty does not write that file. If you changed both the file and the document, the card says **Changed on both sides**, and you choose which to keep.

![The card of How I work with the label Changed outside in amber, the notice The file changed outside Tisty, and the buttons Bring the change in, Overwrite with Tisty and See the file](agents-outside.png)

## Restoring a backup

Before writing over a file, Tisty keeps what it held. **Backup #1** is the original, the file as it was before Tisty touched it, and it is never removed; it also keeps the last 20. They live on this computer and do not travel with sync.

1. On the card, press **Backups**, or the row's **3 backups** link.
2. Pick one, such as **Backup #2 · 1.1 kB**. **See** shows it whole.
3. **Restore** asks «Put this backup back in `~/.claude/CLAUDE.md`? What the file holds now is kept as a backup.» Say yes.
4. The file says what the backup says again, and the card turns to **Changed outside**: your document did not change. **Bring the change in** makes it match the backup; **Overwrite with Tisty**, if you changed your mind.

That way you never lose either side.

## Pausing an assistant

If one day you want Tisty to leave Claude's files alone, for example while you try other rules by hand:

- on the card, **Pause sync with Claude**; in a Global document, its row's **Pause Claude** link;
- or in **Settings → Assistants**, on the **AGENTS** card, turn off **Sync Claude**. **Sync the AGENTS folder** turns everything off.

Paused, Tisty neither opens nor writes those files, and you keep editing your documents. **Resume sync with Claude** writes what you changed, except a file that changed outside meanwhile, which stays marked for you to decide. A pause belongs to this computer only.

## A second computer

Your AGENTS documents travel with your sync to your other computers, but **each computer chooses what it applies to its agents**.

**An example.** On the home computer you brought in your `CLAUDE.md` and wrote the rule «For all my agents». There they applied by themselves. On the work computer:

1. In **Settings → Assistants**, press **Allow on this computer** and bring, if you like, this computer's files.
2. The sidebar says **2 waiting** beside AGENTS.
3. Open **For all my agents**. The card says «This document comes from another computer. Apply it to the agents on this one?».
4. **Apply here** writes it to this computer's files, with its backup first. **Not on this computer** leaves it out here, and **Apply here** stays at hand if you change your mind.

From then on, what you already apply updates by itself when you change it on the other computer. What you write or bring in on one computer is applied on that same computer with no click. And an agent's memory belongs to the computer that brought it: on the others it is only read.

![The card of For all my agents on the work computer, with the labels Rule and Global, no file rows, and the amber notice This document comes from another computer. Apply it to the agents on this one? with the buttons Apply here and Not on this computer; beside AGENTS, in the sidebar, 2 waiting](agents-other-computer.png)

Two more notices you may see on a card:

- **To approve**: the document's text changed without going through your hands, for example through an imported parcel. It reaches no agent until you read it and press **Approve this text**. If that document was already written to a file, the file stays **On hold**, as it was.
- **Written by an earlier Tisty**: an earlier version of Tisty wrote the file and today it does not say what your documents say. **See the file** and, if you keep the document, **Overwrite with Tisty**.

## What your agent can and cannot do

Your agent **reads** your AGENTS documents like any other: you ask «which of my rules do you read?» and it answers with their titles. But it **does not write them on its own**: when it changes one, its change stays apart, as a proposal, and reaches your agents only if you approve it. An AGENTS rule reaches all your agents, on all your computers, and nobody but you should decide what it says.

Nor can it move, archive or mark those documents, change their `agent:` line, propose on a memory or make a new document with an `agent:` line, in any folder. If it tries, Tisty refuses it, touching nothing, and tells it how to propose the change or to tell you what it would change.

## A proposal from your agent

**An example.** You ask your agent «add to my rules that it runs the tests before every commit».

1. The agent edits **How I work** with `edit_doc`, like any document. Tisty keeps its change apart, byte for byte, and answers that it was proposed: the document and `~/.claude/CLAUDE.md` stay as they were. Its guide asks it to tell you just that, «I proposed it», and not to call it done.
2. The sidebar says **1 with a proposal** beside AGENTS. If Tisty was open, you see it when it looks again, for example when you come back to its window.
3. Open **How I work**. The card gains the label **Proposal** and a purple box: «An agent (Claude Code) proposes a change», and the compared lines, in red with `−` what is **Approved** and in green with `+` what is **Proposed**.
4. **Approve the change** asks whether you approve it. With your yes, the document says what was proposed and Tisty writes it to `CLAUDE.md`, with its backup first. **Discard** removes it and the document stays as you approved it.

![The card of How I work with the labels Rule, Claude, In sync and Proposal, and the purple box An agent (Claude Code) proposes a change, with the compared lines − Approved and + Proposed, the new line + - Run the tests before every commit. in green, and the buttons Approve the change and Discard](agents-proposal.png)

Each button acts only on what the card showed you: if the agent wrote again while you were looking, Tisty asks you to look at it again. If it writes again, its change goes on top of its own proposal, so there is one per document. And while a proposal waits, what you already approved keeps reaching your agents.

## More in the help

Each standing, each assistant with its files and variables, what is never read and what to do when something does not add up are in [the AGENTS folder](https://www.tisty.app/docs/agents-folder/), in the help. What Tisty keeps of all this, and what travels, in [privacy](https://www.tisty.app/docs/privacy/). And how to connect your agent to Tisty, in [Step by step with your AI agent](tisty:agent).
