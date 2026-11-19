# AGENTS: your agents' rules, skills and memory

I use more than one AI agent, and each one reads its rules from a different file, in a different folder: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md`, `GEMINI.md`. Changing one rule everywhere meant opening four files, and sooner or later one fell behind. AGENTS gathers those files in Tisty, as documents, and Tisty writes underneath the file each agent reads.

> [!TIP]
> **In short:** the **AGENTS** folder keeps the rules, skills and memory of Claude Code, Codex, GitHub Copilot CLI, Gemini CLI and Cursor as documents. What you write in **Global** reaches all of them; what you write in one assistant's folder reaches only that one. Before writing over a file, Tisty keeps what it held, and a change made outside is never written over: it stays marked until you decide.

![AGENTS the first time, in English: the box Your agents, in one place, with its three lines on privacy, sync and backups, and below it the list This is what I found. Choose what to bring, with Claude · 7 files open: CLAUDE.md and two files in rules ticked as Rule, the skill pr/SKILL.md ticked as Skill, a project memory unticked as Agent memory, and a .env inside a skill greyed out with credentials, left out; further down Codex · 2 files and Gemini · 1 file folded, and the buttons Not now and Bring and sync](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-01-first-run.png)

## What it is and when it helps

AGENTS is a folder in your Library, at the top, with a robot icon. Each document inside it is one of an agent's files: a rule, a skill or its memory. You edit it in Tisty like any document, with its outline, its search and its earlier versions, and Tisty writes the file that agent reads, byte for byte.

It helps when:

- **You use several agents and want the same rules in all of them.** You write «Answer in English» once in **Global**, and it reaches Claude Code's `CLAUDE.md`, Codex's `AGENTS.md`, Copilot's `copilot-instructions.md` and Gemini's rules.
- **Your rules grew and you can no longer find them.** In Tisty they read as documents, with headings, an outline and search, instead of a flat file in a hidden folder.
- **You have skills with scripts.** Each file of the skill is a document, and a script is kept in a code block that is written as is.
- **You want to be able to go back.** Tisty keeps the original file before it first writes over it, and the last 20 before each overwrite.
- **You want to see what your agent noted about you.** The memory Claude Code writes in each project, or Gemini's `GEMINI.md`, can be brought in to read in Tisty, and Tisty never changes it.

It is not for each repository's files: AGENTS handles your user's files, the ones in your home folder, not the `AGENTS.md` or `CLAUDE.md` that lives inside a project.

## The first time

1. Open **Settings › Assistants**. On the **AGENTS** card, press **Create the AGENTS folder**.
2. **Your agents, in one place** opens. Tisty looked at which assistants you have on this computer and which files sit in their folders, **only their names and sizes**: it has not opened any of them yet.
3. At the top, three lines tell you what happens from now on:
   - 🔒 **It stays in your Tisty**: on your computers and wherever you sync. Nobody else sees it.
   - ↻ **It syncs by itself** with the assistants it found, for example «It syncs by itself with Claude, Codex, Gemini.».
   - ⧉ **Before changing an agent's file, Tisty keeps a backup on this computer.**
4. Below, **This is what I found. Choose what to bring:**, one row per assistant, such as **Claude · 7 files**. Open it with a click to see each file with its box and what it is: **Rule**, **Skill** or **Agent memory**. Hover over a file to see its full path.
5. Tick what you want to bring. Rules and skills come ticked; **agent memory comes unticked**, because once in Tisty it travels with your sync, so you tick it yourself if you want it.
6. Press **Bring and sync**. If you ticked nothing, the button reads **Allow and sync**: it allows this computer without bringing any file.
7. Tisty opens only the files you ticked, keeps them as documents and tells you how many came in: «7 documents came into AGENTS, and Tisty keeps them in sync from now on.». Whatever could not come in shows under **These did not come in:**, with the reason. **Done** closes the box.

**Not now** closes the box without reading anything. You can come back whenever you like from the same card in Settings.

### What the list says beside a file

| In the list | What it means |
| :-- | :-- |
| **Rule**, **Skill**, **Agent memory** | It can come in. Memory comes unticked |
| **already in AGENTS** | A document in AGENTS already stands for it, so it is compared instead of brought again |
| **settings, left out** | It carries a settings file's name, such as `settings.json`, `config.toml`, `hooks.json` or `mcp.json`. Inside a skill it does come in |
| **credentials, left out** | Its name is a credentials file's, such as `.credentials.json`, `auth.json`, `.env`, or a `.pem`, `.key` or `id_rsa` key |
| **Tisty's own** | A file Tisty left or a temporary one, such as `tisty.json` or one ending in `.tmp` |
| **a link, left out** | It is a symbolic link: Tisty does not follow it |
| **too big** | Larger than 512 kB, what a document can hold |
| **too many files** | Tisty had already seen 2,000 files in your assistants' folders and stopped looking |

What is greyed out cannot be ticked. Tisty also skips hidden folders (those starting with a dot) and `node_modules`, and goes no deeper than eight levels.

### If a file does not come in

After bringing, a ticked file may stay out:

- **It looks like it holds a credential, such as a key or a token, so it stays out.** Tisty checked it on this computer and found something shaped like a secret. Take the secret out of the file and try again.
- **It is not text.** A binary file cannot be a document.
- **Its name cannot be kept as an agent document.** A name with `#`, `:`, `,`, quotes, square brackets or a backslash does not fit in the line that says where the file goes.

### If you have no agent files yet

The box says «I found no agent files on this computer.» and the button stays at **Allow and sync**. Pressing it allows this computer, but the AGENTS folder is born with the first file you bring: once you have one, for example after using your agent for the first time, go back to **Settings › Assistants** and press **Create the AGENTS folder** again.

## How the folder looks

After bringing, your Library has an **AGENTS** folder with one subfolder per assistant and a **Global** one. Each document is named after its file, with the assistant in front:

```text
AGENTS                         in sync
├── Global
├── Claude
│   ├── claude · CLAUDE.md
│   ├── claude · rules/style.md
│   ├── claude · pr/SKILL.md
│   └── claude · projects/tisty/memory/MEMORY.md
├── Codex
│   └── codex · AGENTS.md
└── Gemini
    └── gemini · GEMINI.md
```

The subfolders are there so you find things: **what decides where a document goes is its `agent:` line**, not the folder it sits in. You can move a document between subfolders or make your own inside AGENTS, and it keeps being written to the same file.

Beside the AGENTS folder's name, the sidebar tells you how everything stands:

| Beside AGENTS | What it means |
| :-- | :-- |
| **in sync** | Every file says what its document says |
| **2 changed** | Two files changed outside, or on both sides, and wait for you to decide |
| **off** | This computer is not allowed, or you paused the whole folder |
| **empty** | The folder holds no documents |

![The sidebar in English with the AGENTS folder open, its purple robot icon and the words in sync on the right; inside, the subfolders Global, Claude, Codex and Gemini, and in Claude the documents claude · CLAUDE.md, claude · rules/style.md and claude · pr/SKILL.md](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-02-sidebar.png)

### The `agent:` line

Each document in AGENTS starts with its properties, and one of them says which file it is:

```markdown
---
agent: claude/rule/CLAUDE.md
title: claude · CLAUDE.md
---
# How I work

- Answer in English.
- Branches shaped type/name.
```

The value has three parts, split by `/`:

- **The level**: `global`, or one assistant: `claude`, `codex`, `copilot`, `gemini` or `cursor`.
- **The kind**: `rule`, `skill` or `memory`.
- **The name**: the file's path inside the assistant's folder, such as `CLAUDE.md`, `rules/style.md` or `pr/SKILL.md`.

Sometimes it carries more marks, which Tisty adds only when it brings a file in, to write it back exactly as it was: `crlf` (lines ending the Windows way), `bom` (an invisible mark at the start), `as-is` (mixed line endings) and `open-end` (the file does not end in a line break). For example, `agent: [copilot/rule/copilot-instructions.md, crlf, bom]`. The [properties guide](https://markdown.tisty.app/properties) explains the rest.

The page does not show that line: in its place, at the top of the document, sits its [card](#the-card-of-an-agent-document).

## Global or one assistant

**A Global document reaches every assistant Tisty finds on this computer.** One in an assistant's folder reaches only that assistant.

### A rule in Global

Write a document in **AGENTS › Global** with these properties:

```markdown
---
agent: global/rule/for-all.md
title: global · for-all.md
---
# For all my agents

- Before deleting anything, ask.
- Never push changes unless I ask.
```

That rule is not written to a file of its own: it goes **at the start of each assistant's main file**, before that assistant's own rules. If this computer has Claude Code, Codex and Gemini, Tisty writes:

- `~/.claude/CLAUDE.md`: the Global rule, then what your document `claude · CLAUDE.md` says.
- `~/.codex/AGENTS.md`: the Global rule, then your document `codex · AGENTS.md`, if you have one.
- `~/.gemini/tisty-rules.md`: the Global rule, then your document `gemini/rule/tisty-rules.md`, if you have one.

This is how Claude Code's `CLAUDE.md` ends up:

```markdown
# For all my agents

- Before deleting anything, ask.
- Never push changes unless I ask.

# How I work

- Answer in English.
- Branches shaped type/name.
```

The card of each of those documents says so: «Claude reads it at `~/.claude/CLAUDE.md`», and below it, «together with one other document». Cursor gets no rules: its own live in its settings, not in a file.

> [!IMPORTANT]
> **Leave each assistant's main file ticked the first time.** If `~/.codex/AGENTS.md` already exists and you did not bring it in, Tisty cannot tell which part is yours: once you write a Global rule, that file stands **Changed on both sides**, and from there you can only overwrite it, which replaces the whole file with your Global rules (the original stays as backup #1), or decide later. Brought in, Global is written in front of what it already said.

### A rule for Claude alone

Claude Code also reads each file in `~/.claude/rules`. For a rule only Claude should read, write a document with `agent: claude/rule/rules/tests.md`, anywhere in AGENTS, and Tisty writes `~/.claude/rules/tests.md`. Copilot does the same with its `instructions` folder: `agent: copilot/rule/instructions/rust.instructions.md`.

A name the assistant does not read is written nowhere: `agent: claude/rule/notes.md` leaves the card saying «No assistant on this computer reads this document yet.».

### A skill in Global

A skill is a folder with a `SKILL.md` and, sometimes, scripts. Each file is a document, with the same level and the same skill folder:

```text
agent: global/skill/pdf/SKILL.md
agent: global/skill/pdf/scripts/extract.py
```

A Global skill is written once where several read it at a time:

- in `~/.claude/skills/pdf/`, for Claude Code;
- in `~/.cursor/skills/pdf/`, for Cursor;
- in `~/.agents/skills/pdf/`, one copy that Codex, Copilot and Gemini all read.

The `SKILL.md` shows its own properties, its name and description, as fields under the card. The script is kept in a code block and written as is, without the block: the card says **Written as is**, and a Markdown document says **Written as Markdown**.

## Which file each assistant reads

`~` is your user folder: `C:\Users\you` on Windows and `/Users/you` on macOS. Tisty only works with an assistant whose folder exists on this computer.

| Assistant | Its folder | Moved by | Rules Tisty writes | Skills | Memory, read only |
| :-- | :-- | :-- | :-- | :-- | :-- |
| **Claude Code** | `~/.claude` | `CLAUDE_CONFIG_DIR` | `CLAUDE.md` and what goes in `rules/` | `skills/` in its folder | `projects/*/memory/`, each project's memory |
| **Codex** | `~/.codex` | `CODEX_HOME` | `AGENTS.md`, and Global in `AGENTS.override.md` when that file exists | `~/.agents/skills/` | — |
| **GitHub Copilot CLI** | `~/.copilot` | `COPILOT_HOME` | `copilot-instructions.md` and what goes in `instructions/` | `skills/` in its folder; Global in `~/.agents/skills/` | — |
| **Gemini CLI** | `~/.gemini` | `GEMINI_CLI_HOME`, the folder that holds `.gemini` | `tisty-rules.md`, and the line `@tisty-rules.md` in `GEMINI.md` | `skills/` in its folder; Global in `~/.agents/skills/` | `GEMINI.md` |
| **Cursor** | `~/.cursor` | — | None: its rules live in its settings | `skills/` in its folder | — |

What Tisty does with each one:

- **Claude Code.** It brings in `CLAUDE.md`, each file in `rules/`, each skill and, if you tick it, each project's memory: only the `memory` folder, never the saved conversations. It writes `CLAUDE.md`, `rules/` and the skills.
- **Codex.** Codex reads `AGENTS.override.md` instead of `AGENTS.md` when the first one exists, so your Global rules go to that file while it exists. Its skills go to `~/.agents/skills`, which `CODEX_HOME` does not move.
- **GitHub Copilot CLI.** It brings in and writes `copilot-instructions.md` and each file in `instructions/`, such as `rust.instructions.md`.
- **Gemini CLI.** Gemini writes its own `GEMINI.md` when you ask it to remember something, so that file is Gemini's: Tisty can bring it in as memory and never writes over it. Your rules go to a separate file, `~/.gemini/tisty-rules.md`, and Tisty adds **one single line** at the end of `GEMINI.md`, `@tisty-rules.md`, so Gemini reads it. It touches nothing else in that file, and if the line is already there it does not add it again.
- **Cursor.** Skills only, its own or Global ones.

**The variables that move a folder** are read from the environment Tisty started with. If you use `CLAUDE_CONFIG_DIR` to keep Claude Code somewhere else, set it as a user variable and open Tisty again, so it sees it.

## The card of an agent document

A document in AGENTS opens **for reading**, with a card at the top in place of its properties:

![The card in English of claude · CLAUDE.md: at the top the labels Rule, Claude and In sync in green; below, the name CLAUDE.md, the row Claude reads it at ~/.claude/CLAUDE.md with Written as Markdown · together with one other document · file changed 5 minutes ago, the link 3 backups on the right, and the buttons Edit, Backups and Pause sync with Claude](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-03-card.png)

- **The labels**: the kind (**Rule**, **Skill** or **Agent memory**), the level (**Global** or the assistant) and the standing.
- **The name** of the file.
- **One row per file it is written to**: «Claude reads it at» and the path. Below, **Written as Markdown** or **Written as is**, whether it is put together «together with one other document» or «together with 2 other documents», when the file changed, and the link to its backups. A Global document has one row per assistant, each with its own **Pause Claude**, **Pause Codex**… link.
- **The buttons**: **Edit** opens the document for writing and **Done** closes it again; **Backups**, when the file has any; and **Pause sync with Claude**, when the document is written to a single file.

A memory document has no **Edit**: the agent writes it, and the card says «The agent writes this file itself: Tisty shows it and brings it in, and never writes over it.».

### Each standing

| Standing | What it means | What to do |
| :-- | :-- | :-- |
| **In sync** | The file says what the document says | Nothing |
| **About to be written** | You changed the document and Tisty writes it in a moment | Nothing |
| **Changed outside** | Someone changed the file outside Tisty, or deleted it | Choose: [bring or overwrite](#a-change-made-outside-bring-it-in-or-overwrite) |
| **Changed on both sides** | Both the file and the document changed since the last write | Choose which to keep |
| **Paused** | You paused sync for that assistant, or for the whole folder | **Resume** whenever you like |
| **Not allowed on this computer** | This computer has not allowed AGENTS | **Allow on this computer**, in Settings › Assistants |
| **Nothing to write** | The document is empty, and Tisty never empties an agent's file | Write something, or take the document out of AGENTS |
| **The file is a link** | The file, or a folder on its way, is a symbolic link; writing it would replace the link | Swap the link for the real file |
| **The code block is broken** | The block that holds a script is no longer whole | Mend the block in the document |
| **Cannot be read** | The file exists, but Tisty could not read it | Check its permissions |

Two more notices can take the place of the rows:

- **«No assistant on this computer reads this document yet.»** The assistant is not on this computer, or the name is not a file that assistant reads.
- **«Another document in AGENTS already writes this file, so this one is left out.»** Two documents carry the same `agent:` line: only one is written.

## A change made outside: bring it in or overwrite

You can keep editing your agents' files in your editor, or let the agent change them. Tisty never writes over those changes.

**An example.** You open `~/.claude/CLAUDE.md` in your editor and add «- Run the tests before every commit.». The next time Tisty looks (on start, when you come back to the window, or when you open the document), the sidebar says **1 changed** beside AGENTS, and the card of `claude · CLAUDE.md` says **Changed outside**, with a notice:

![The card in English of claude · CLAUDE.md with the label Changed outside in amber and the notice The file changed outside Tisty, ~/.claude/CLAUDE.md no longer says what this document says, and the buttons Bring the change in, Overwrite with Tisty and See the file](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-04-outside.png)

- **See the file** shows what the file holds now.
- **Bring the change in** writes into the document what the file says. What the document said before stays in its [earlier versions](Documents#earlier-versions).
- **Overwrite with Tisty** asks «Write this document over `~/.claude/CLAUDE.md`? What the file holds now is kept as a backup.» and, if you say yes, writes the file again from the document.

Until you choose, Tisty does not write that file, even if you change the document.

**Bring the change in** only shows when the change can be traced to this document: the document belongs to that same assistant, not to Global, and you are not editing it. If `CLAUDE.md` is put together from a Global rule and your Claude document, and the change falls in the Global part, only **Overwrite with Tisty** is left: copy into Global by hand what you want to keep, then overwrite.

**If the file was deleted**, the card says **Changed outside** without **See the file**: **Overwrite with Tisty** makes it again.

### Changed on both sides

If both the file and the document changed, the card says **Changed on both sides**. When the document is the only one that makes up that file, **Keep the file's version** shows, which asks «Make this document read as … does now? What it says now stays in its history.»; otherwise only **Overwrite with Tisty**.

The same can happen the first time, when you already had a document in AGENTS for a file that changed in between. Then **It exists on both sides** opens, with the lines that differ: in red, with `−`, what is **In Tisty**; in green, with `+`, what is **In the file**. You choose **Keep Tisty's**, **Keep the file** or **Decide later**, and what you do not keep is still kept: the file in its [backups](#backups), the document in its earlier versions. If the file is made of several documents, you can only keep Tisty's or decide later. From **Settings › Assistants**, «1 file says something different in Tisty and in the agent.» and **Review** bring you back to that box.

![The box in English It exists on both sides for ~/.codex/AGENTS.md: the path at the top, the compared lines below, in red with a minus sign the one that is only In Tisty and in green with a plus sign the one that is only In the file, and the buttons Decide later, Keep the file and Keep Tisty's](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-07-both.png)

## Backups

**Before writing over an agent's file, Tisty keeps what it held.** Backups live on this computer, in `data/agents/` inside Tisty's folder, never in your agents' folders, and they do not travel with sync or parcels.

- **Backup #1 is the original**: the file as it was before Tisty first wrote over it. That one is never removed.
- **Then, the last 20.** When the 21st comes, the oldest one after the original goes.

To see them, press **Backups** on the card, or the **3 backups** link on the file's row:

![The Backups box in English for ~/.claude/CLAUDE.md, with the line What ~/.claude/CLAUDE.md held before Tisty wrote over it. The last 20 are kept., and three rows, Backup #3 · 1.2 kB, Backup #2 · 1.1 kB and Backup #1 · 0.9 kB, each with See and Restore](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-05-backups.png)

- **See** shows the whole backup.
- **Restore** asks «Put this backup back in …? What the file holds now is kept as a backup.» and writes it into the file.

**A restored backup does not change your document.** The file says again what the backup said, and the card turns to **Changed outside**: you choose **Bring the change in**, so the document says the same, or **Overwrite with Tisty**, if you changed your mind. That way you never lose either side.

The **document's** earlier versions, on the other hand, are those of any document: they are in its [history](Documents#earlier-versions).

## Pausing, turning off and emptying

**Pausing one assistant.** On the card, **Pause sync with Claude**, or the **Pause Claude** link on its row. Also in **Settings › Assistants**, on the **AGENTS** card, with the **Sync Claude** switch. Paused, Tisty neither opens nor writes that assistant's files: they stay as the last write left them, and you keep editing your documents. **Resume sync with Claude** writes what you changed, except a file that changed outside in the meantime, which stays marked for you to decide.

**Turning the whole folder off.** On the same card, **Sync the AGENTS folder**. Off, the sidebar says **off** beside AGENTS, and each document's card says «Sync is off for the whole AGENTS folder.».

![The AGENTS card in Settings › Assistants, in English: the line Your agents' rules, skills and memory as Tisty documents, the switch Sync the AGENTS folder on and below it Sync Claude, Sync Codex and Sync Gemini, with Codex off and the note Off: its files stay as they are, and nothing in Tisty is deleted](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-06-settings.png)

A pause and the permission belong to this computer.

**Taking a document out.** If you move a document out of AGENTS, archive it or delete it, Tisty stops writing it:

- if it was the only one making up its file, the file stays as it was;
- if the file was put together from several, like `CLAUDE.md` with a Global rule, Tisty writes it again without that part, keeping a backup first.

Tisty never empties or deletes an agent's file.

**Emptying AGENTS.** If you take every document out of AGENTS, the folder and its subfolders leave Tisty, and your agents' files stay as they were.

**Hiding it from the sidebar.** If you would rather not see it all the time, right-click AGENTS and choose **Hide from the sidebar**. It only leaves the sidebar on this computer: it keeps syncing, and you find it under **Hidden**, at the bottom of the sidebar, or with **Show hidden**.

**What stops it being AGENTS.** Tisty knows the folder because it sits at the top, is named **AGENTS** and carries its robot icon. If you rename it, change its icon, move it inside another folder or archive it, Tisty stops writing the files, as if you had emptied it.

## What is never read

- **Your assistants' settings**: `settings.json`, `settings.local.json`, `config.toml`, `config.json`, `hooks.json`, `mcp.json`, `mcp-config.json`, `.claude.json` and `keybindings.json`. Tisty only looks where rules, skills and memory go, never in an assistant's main folder, and inside a skill a file with one of those names does come in, because it belongs to the skill.
- **Credentials**, by name and without opening them: `.credentials.json`, `credentials.json`, `auth.json`, `.env` and its variants, `.netrc`, and `.pem`, `.key`, `.p12`, `.pfx`, `id_rsa` or `id_ed25519` keys.
- **A file that looks like it holds a secret**, such as a token or a private key, even under an innocent name. The check runs on this computer and looks for known shapes, so it may not recognise them all: keep secrets out of your agents' rules.
- **Claude Code's conversations**: of its `projects` folder, only each project's `memory` folder is looked at.
- **Symbolic links** and files larger than 512 kB.

And an AGENTS document is only written where each assistant keeps its rules, skills and memory, never to its settings or credentials.

**Agent memory** comes unticked the first time, and once in Tisty it is read only: the agent keeps writing it, and when it changes, the card says **Changed outside** and **Bring the change in** brings Tisty up to date. It has no **Edit**, **Overwrite with Tisty** or **Restore**.

What Tisty keeps of all this, and what travels, is in [Privacy](Privacy#your-assistants-in-agents).

## With your agent, through MCP

An agent you let in through [[MCP|Assistants-and-MCP]] sees your AGENTS documents like any other, and its guide tells it what they are. For example, you ask Claude Code «Which of my rules do you read?».

**1. It reads its guide.** `rules` with the topic «documents» brings, among others, the section on AGENTS:

```json
{ "name": "rules", "arguments": { "topic": "documents" } }
```

```text
## Assistants' files in AGENTS

A document whose properties hold an `agent:` line is an assistant's own file kept in the folder
AGENTS — a rule file such as `CLAUDE.md`, a skill, a script — and Tisty writes its text, byte for
byte, to the file that assistant reads, on each computer where the person turned that on. …
```

**2. It looks for the document.** `brain` with the file's words:

```json
{ "name": "brain", "arguments": { "query": "CLAUDE.md" } }
```

```text
0 task(s) and 1 document(s) match "CLAUDE.md"; showing 0 and 1.
"claude · CLAUDE.md" (document) [k4r5j4qr-0001]
```

`brain` with `folder` and a subfolder's name or path lists that assistant's documents. The AGENTS folder alone answers with how many sit in each subfolder.

**3. It looks at its card and reads it.** `brain` with `doc` gives its outline and place; `read` with `doc`, the whole text, `agent:` line included:

```json
{ "name": "read", "arguments": { "doc": "k4r5j4qr-0001" } }
```

```text
---
agent: claude/rule/CLAUDE.md
title: claude · CLAUDE.md
---
# How I work

- Answer in English.
- Branches shaped type/name.
```

The agent answers you with the title and the folder, «your document claude · CLAUDE.md, in AGENTS / Claude», never with the id. Every tool is in the [[Tools reference|MCP-Tools]].

## Questions

**Does Tisty replace my files?** It neither moves nor converts them: it keeps writing them where your agent looks for them. Before the first write it keeps the original, backup #1, which is never removed.

**Can I still edit `CLAUDE.md` by hand?** Yes. Tisty marks the file **Changed outside** and waits for you to choose: bring your change into the document, or write over it.

**What if I close or uninstall Tisty?** Your agents keep reading their files as Tisty left them. Nothing depends on Tisty being open.

**Does AGENTS touch `settings.json`, hooks or my MCP configuration?** No: AGENTS neither reads nor writes them. Connecting an assistant is something else: there Tisty adds its MCP server and its hooks to that assistant's configuration, as [[Assistants and MCP|Assistants-and-MCP]] tells.

**Why does agent memory come unticked?** Because the agent writes it, it may talk about your projects, and once in Tisty it travels with your sync and your backups like any document. Bring it in only if you want to read it in Tisty.

**What is the `@tisty-rules.md` line in my `GEMINI.md`?** It is the only line Tisty adds to that file, so Gemini reads your rules, which go in `tisty-rules.md`. The rest of `GEMINI.md` is written by Gemini.

**Does AGENTS see the assistants I use inside WSL?** No. On Windows, Tisty looks at your Windows user folder, and an assistant running inside WSL keeps its files in your Linux home.

**And each repository's rules?** They stay where they are. AGENTS handles your user's files, not each project's `AGENTS.md` or `CLAUDE.md`.

**Can I write a new rule from scratch?** Yes: make a document inside AGENTS and write its properties with the `agent:` line, as in [Global or one assistant](#global-or-one-assistant). Tisty writes it as soon as you save it.

**How many backups does it keep?** The original and the last 20 of each file, on this computer.

## When something does not add up

**An assistant is missing from the list.** Tisty only sees an assistant whose folder exists: `~/.claude`, `~/.codex`, `~/.copilot`, `~/.gemini` or `~/.cursor`. If you moved it with a variable such as `CODEX_HOME`, set it as a user variable and open Tisty again. If you never used that assistant on this computer, open it once so it makes its folder.

**A file is greyed out.** The reason sits beside it: see [what the list says](#what-the-list-says-beside-a-file).

**My change in Tisty does not reach the file.** Look at the standing on the document's card:

- **Paused**: resume that assistant, or the folder, on the card or in Settings › Assistants.
- **Not allowed on this computer**: press **Allow on this computer** in Settings › Assistants.
- **Changed outside** or **Changed on both sides**: Tisty waits for your decision before writing.
- **Nothing to write**, **The file is a link** or **The code block is broken**: the table of [each standing](#each-standing) says what to do.
- **«No assistant on this computer reads this document yet.»**: check the `agent:` line. The level must be an assistant on this computer, and the name a file that assistant reads.
- **«Another document in AGENTS already writes this file»**: two documents carry the same `agent:` line. Keep one.

**I wrote a Global rule and a file stands Changed on both sides.** That file already existed and you did not bring it in, so Tisty cannot tell which part is yours. To keep what it said, press **See the file** on the card, copy your part into a new document inside AGENTS with the line `agent: codex/rule/AGENTS.md` (or that assistant's), then **Overwrite with Tisty**: the file ends up with Global in front of your part, and the original as backup #1. See [A rule in Global](#a-rule-in-global).

**The whole folder stopped writing.** Check that AGENTS still sits at the top of the Library, with that name and its robot icon, and not archived.

**Codex does not read my rules.** If you have `~/.codex/AGENTS.override.md`, Codex reads only that file. Your Global rules go there, but a `codex · AGENTS.md` document is written to `AGENTS.md`, which Codex does not read while the other one exists.

**Gemini does not read my rules.** Check that `~/.gemini/GEMINI.md` has the line `@tisty-rules.md`. If you deleted it, Tisty adds it back the next time it goes over your files: on start, or when you change an AGENTS document.

**Bring the change in does not show.** The document is a Global one, the change falls in another document's part of the same file, or you are editing it: press **Done** and look again.

**«The file changed while Tisty was writing, so it was left alone.»** Another program wrote the file at the same moment. Tisty writes over nothing: look at the card again and choose.

**I restored a backup and it says Changed outside.** That is expected: [a restored backup does not change your document](#backups). Choose **Bring the change in** to keep it.

## See also

- [Privacy](Privacy#your-assistants-in-agents): what AGENTS reads, when, and what stays on this computer.
- [[Assistants and MCP|Assistants-and-MCP]]: connecting an agent to Tisty.
- [Earlier versions](Documents#earlier-versions): a document's history.
- [Folders](Documents#folders): filtering, archiving and unarchiving.
- [Properties](https://markdown.tisty.app/properties), in the Tisty Markdown guide: the `agent:` line and its marks.
