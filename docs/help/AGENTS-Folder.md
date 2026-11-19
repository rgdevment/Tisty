# AGENTS: your agents' rules, skills and memory

I use more than one AI agent, and each one reads its rules from a different file, in a different folder: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md`, `GEMINI.md`. Changing one rule everywhere meant opening four files, and sooner or later one fell behind. AGENTS gathers those files in Tisty, as documents, and Tisty writes underneath the file each agent reads.

> [!TIP]
> **In short:** the **AGENTS** folder keeps the rules, skills and memory of Claude Code, Codex, GitHub Copilot CLI, Gemini CLI and Cursor as documents. What you write in **Global** reaches all of them; what you write in one assistant's folder reaches only that one. Before writing over a file, Tisty keeps what it held, a change made outside is never written over: it stays marked until you decide. And an agent does not write your rules: it proposes the change, and only you approve it.

![AGENTS the first time, in English: the box Your agents, in one place, with its three lines on privacy, sync and backups, and below it the list This is what I found. Choose what to bring, with Claude · 7 files open: CLAUDE.md and two files in rules ticked as Rule, the skill pr/SKILL.md ticked as Skill, a project memory unticked as Agent memory, and a .env inside a skill greyed out with credentials, left out; further down Codex · 2 files and Gemini · 1 file folded, and the buttons Not now and Bring and sync](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-01-first-run.png)

## What it is and when it helps

AGENTS is a folder in your Library, at the top, with a robot icon. Each document inside it is one of an agent's files: a rule, a skill or its memory. You edit it in Tisty like any document, with its search and its earlier versions, and Tisty writes the file that agent reads, byte for byte. Beside it, the **Agent file** panel tells you who reads it, where it is written and how many backups it has.

It helps when:

- **You use several agents and want the same rules in all of them.** You write «Answer in English» once in **Global**, and it reaches Claude Code's `CLAUDE.md`, Codex's `AGENTS.md`, Copilot's `copilot-instructions.md` and Gemini's rules.
- **Your rules grew and you can no longer find them.** In Tisty they read as documents: each with its name and its icon in the sidebar, with headings and search, instead of a flat file in a hidden folder.
- **You work on more than one computer.** You write the rule once, and each computer chooses whether to apply it to its agents.
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
6. Press **Bring and sync**. If you ticked nothing, the button reads **Allow and sync**: it allows this computer without bringing any file, and if AGENTS does not exist yet, it makes it empty, with its **Global**.
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

The box says «I found no agent files on this computer. You can still allow it: AGENTS fills up as you write your rules.», and the button stays at **Allow and sync**. Pressing it allows this computer and still creates the **AGENTS** folder, with its **Global** subfolder. The sidebar says **empty** beside AGENTS, and going into it does not open the box again.

From there you have two ways:

- **Write your first rule.** Make a document inside AGENTS with its `agent:` line, as in [Global or one assistant](#global-or-one-assistant), and Tisty creates the file as soon as you save it.
- **Bring what shows up later.** Once your agent makes its files, for example after you use it for the first time, bring them with [Bring another file](#bring-another-file).

## Bring another file

The first time does not have to be the only one. If you install a new assistant, make a rules file that did not exist before, or left something unticked, you bring it later:

- **From the sidebar:** right-click AGENTS, or any folder inside it, and choose **Bring another file**.
- **From Settings › Assistants:** on the **AGENTS** card, the **Bring another file** button.

**Bring another file** opens, with «These are the agent files on this computer that are not in AGENTS yet. Choose what to bring:» and the same list as the first time, with one difference: **nothing comes ticked**, not even the rules. Tick what you want and press **Bring**: the button stays off until you tick something. What Tisty already writes from a document says **already in AGENTS**, and if nothing is left to bring, the box says «Every agent file on this computer is already in AGENTS.».

**An example.** One day you write a new rule for Claude Code by hand in your editor, `~/.claude/rules/tests.md`:

```markdown title="~/.claude/rules/tests.md"
# Tests

- Run the tests before every commit.
```

1. Right-click **AGENTS › Claude** and choose **Bring another file**.
2. Open Claude's row: `CLAUDE.md` shows dimmed, with **already in AGENTS**, and `rules/tests.md`, unticked, as a **Rule**.
3. Tick it and press **Bring**. The box says «1 document came into AGENTS, and Tisty keeps it in sync from now on.», and **Done** closes it.
4. **Tests** shows up in **AGENTS › Claude**, with a book icon. From now on you change it in Tisty, like any rule.

![The Bring another file box in English, without the first time's three lines: at the top These are the agent files on this computer that are not in AGENTS yet. Choose what to bring:, and below Claude's row open, with CLAUDE.md dimmed and already in AGENTS, and rules/tests.md ticked as a Rule; at the bottom, the buttons Not now and Bring](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-10-bring-another.png)

If this computer has not allowed AGENTS yet, **Bring another file** opens the first-time box, with its three lines and what it found already ticked.

### A file you never brought

**An example.** You have `~/.codex/AGENTS.md` with your Codex rules, and the first time you left it unticked. Later you write a rule in **Global**, which also goes to that file. Tisty cannot tell which part is yours, so it does not write over it: it asks **It exists on both sides**, when you go into AGENTS or through **Review** in Settings › Assistants. Since it never brought that file, the box adds «Tisty has never brought this file. Bring it into AGENTS and it keeps what it says, with Tisty's part added on top.» and the button **Bring it into AGENTS**.

With **Bring it into AGENTS**, the file comes in as a document in **AGENTS › Codex**, with its own words, and Tisty writes it again with your Global rule on top. What it held before stays in its [backups](#backups):

```markdown title="~/.codex/AGENTS.md, after Bring it into AGENTS"
# For all my agents

- Before deleting anything, ask.

# Codex rules

- Use pnpm, never npm.
```

**Bring another file** does the same: that `AGENTS.md` shows in its list, unticked, and once brought it ends up the same, with Global on top and its backup.

### Codex's `AGENTS.override.md`

Codex reads `~/.codex/AGENTS.override.md` instead of `AGENTS.md` when it exists, so your Global rules go to that file while it exists. If you make it after the first time and have rules in Global, Tisty asks **It exists on both sides** with **Bring it into AGENTS**, as with any file it never brought. With no rules in Global it asks nothing: bring it with **Bring another file**. Once you bring it:

- `AGENTS.override.md` holds your Global rule on top and what you wrote below, and from then on Codex gets Global there;
- `AGENTS.md` keeps only your Codex rules, without Global. Codex does not read it while the override exists.

## How the folder looks

After bringing, your Library has an **AGENTS** folder with one subfolder per assistant and a **Global** one. In the sidebar, each document shows with its name and an icon that says what it is:

```text
AGENTS                           in sync
├── Global
├── Claude
│   ├── [book]     How I work
│   ├── [book]     Code style
│   ├── [piece]    pr                3 files
│   └── [brain]    MEMORY.md
├── Codex
│   └── [book]     Codex rules
└── Gemini
    └── [brain]    GEMINI.md
```

- **The icon**: a book for a **rule**, a puzzle piece for a **skill** and a brain for **agent memory**.
- **The name**: the document's first heading, such as «How I work» in a `CLAUDE.md` that starts with `# How I work`. A skill takes the `name` in its `SKILL.md`. A script, or a file with no headings, is named after the file: `MEMORY.md`, `extract.py`.
- **The path**: hover over a row and it shows, with the assistant in front: `claude · CLAUDE.md`, `claude · rules/style.md`.
- **A skill is one single row**, with how many files it holds on the right, such as **pr** and **3 files**. A click opens it and shows its files by their path inside the skill: `SKILL.md`, `scripts/open-pr.sh`. So a 200-file skill takes one line until you open it. A skill of a single file shows like any other document.

The subfolders are there so you find things: **what decides where a document goes is its `agent:` line**, not the folder it sits in. You can move a document between subfolders or make your own inside AGENTS, and it keeps being written to the same file.

Beside the AGENTS folder's name, the sidebar tells you how everything stands:

| Beside AGENTS | What it means |
| :-- | :-- |
| **in sync** | Every file says what its document says |
| **2 changed** | Two files wait for you to decide: they changed outside or on both sides, were written by an earlier Tisty, [no longer belong to any rule](#taking-a-rule-and-its-file-out), or [Gemini's line](#gemini-paused) is missing |
| **3 waiting** | Three documents came from [another of your computers](#on-several-computers) and wait for you to choose whether to apply them on this one |
| **1 to approve** | A document changed without going through your hands, and reaches no agent until you [approve it](#to-approve) |
| **1 with a proposal** | An agent [proposes a change](#a-proposal-from-your-agent) to a document, and waits for you to approve or discard it |
| **off** | This computer is not allowed, or you paused the whole folder |
| **empty** | The folder holds no documents, and nothing waits for you to decide |

If more than one applies, the most pressing shows: **empty** when there are no documents and nothing to decide, and if not, first **off**, then **changed**, then **waiting**, then **to approve** and last **with a proposal**.

![The sidebar in English with the AGENTS folder open, its purple robot icon and the words in sync on the right; inside, the subfolders Global, Claude, Codex and Gemini. In Claude, the rule How I work with a book icon, the rule Code style, the skill pr with a puzzle-piece icon and 3 files on the right, open, showing SKILL.md, scripts/open-pr.sh and template.md, and the memory MEMORY.md with a brain icon; the pointer over How I work shows claude · CLAUDE.md](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-02-sidebar.png)

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

- `~/.claude/CLAUDE.md`: the Global rule, then what your document of that file says, **How I work** in the example.
- `~/.codex/AGENTS.md`: the Global rule, then your document of `AGENTS.md`, if you have one.
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
> **Bring each assistant's main file in.** If `~/.codex/AGENTS.md` already exists and you never brought it in, Tisty cannot tell which part is yours: once you write a Global rule, the card marks that file **Changed on both sides**, and **Overwrite with Tisty** would replace the whole file with your Global rules. Use **Bring it into AGENTS** instead: it brings the file in with its own words and writes Global in front, with the original as backup #1. [A file you never brought](#a-file-you-never-brought) tells how.

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
- **Codex.** Codex reads `AGENTS.override.md` instead of `AGENTS.md` when the first one exists, so your Global rules go to that file while it exists. If you make it after the first time, [bring it in](#codexs-agentsoverridemd) so Global goes there without writing over yours. Its skills go to `~/.agents/skills`, which `CODEX_HOME` does not move.
- **GitHub Copilot CLI.** It brings in and writes `copilot-instructions.md` and each file in `instructions/`, such as `rust.instructions.md`.
- **Gemini CLI.** Gemini writes its own `GEMINI.md` when you ask it to remember something, so that file is Gemini's: Tisty can bring it in as memory and never writes over it. Your rules go to a separate file, `~/.gemini/tisty-rules.md`, and Tisty adds **one single line** at the end of `GEMINI.md`, `@tisty-rules.md`, so Gemini reads it. It touches nothing else in that file, and if the line is already there it does not add it again. If you later delete it, Tisty does not put it back: Gemini stays [paused](#gemini-paused) until you decide.
- **Cursor.** Skills only, its own or Global ones.

**The variables that move a folder** are read from the environment Tisty started with. If you use `CLAUDE_CONFIG_DIR` to keep Claude Code somewhere else, set it as a user variable and open Tisty again, so it sees it.

## The card of an agent document

A document in AGENTS opens **for reading**, with a card at the top in place of its properties, and the **Agent file** panel beside it:

![An agent document open in English. At the top, the card of How I work: the labels Rule, Claude and In sync in green; below, the name How I work in large type and the path CLAUDE.md small and grey; the row Claude reads it at ~/.claude/CLAUDE.md with Written as Markdown · together with one other document · file changed 5 minutes ago and the link 3 backups on the right, and the buttons Edit, Backups and Pause sync with Claude. On the right, the Agent file panel with Read by Claude, Written to ~/.claude/CLAUDE.md, Format Markdown · LF, Last written, Backups 3 backups, Original kept Yes, Syncs on this computer Yes, and the buttons History and Copy link](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-03-card.png)

- **The labels**: the kind (**Rule**, **Skill** or **Agent memory**), the level (**Global** or the assistant) and the standing.
- **The name**, the same as in the sidebar, and under it, in small type, **the path** of the file inside the assistant's folder, such as `CLAUDE.md` or `pr/SKILL.md`.
- **One row per file it is written to**: «Claude reads it at» and the full path. Below, **Written as Markdown** or **Written as is**, whether it is put together «together with one other document» or «together with 2 other documents», when the file changed, and the link to its backups. A Global document has one row per assistant, each with its own **Pause Claude**, **Pause Codex**… link.
- **The buttons**: **Edit** opens the document for writing and **Done** closes it again; **Backups**, when the file has any; and **Pause sync with Claude**, when the document is written to a single file.

A memory document has no **Edit**: the agent writes it, and the card says «The agent writes this file itself: Tisty shows it and brings it in, and never writes over it.».

### The Agent file panel

In an AGENTS document, the column beside the page does not show the paper size, the outline or **Share**: an agent's file is neither printed nor shared. In their place sits **Agent file**:

| Row | What it says | Example |
| :-- | :-- | :-- |
| **Read by** | Which assistants on this computer read the file | Claude, Codex |
| **Written to** | The file's path. A Global document has one row per assistant: **Claude file**, **Codex file** | `~/.claude/CLAUDE.md` |
| **Format** | Markdown, or the script's type; how its lines end, and the marks it keeps | Markdown · LF, or PY · CRLF · BOM |
| **Last written** | When the file last changed | 5 minutes ago |
| **Backups** | How many this computer keeps | 3 backups, or None yet |
| **Original kept** | Whether backup #1 is there, the file as it was before Tisty touched it | Yes, before Tisty's first write |
| **Syncs on this computer** | Whether Tisty keeps it up to date here | Yes; Yes, except Codex (paused); Tisty only reads it; No, AGENTS is off; No: it waits for «Apply here»; No: its text waits for your approval. With a proposal waiting, «an agent's change waits for your approval» is added |

Below, **History**, once the document has earlier versions, opens its [history](Documents#earlier-versions), and **Copy link**, its [link](Documents#a-documents-link). If you already shared that document before, the sharing panel is still there.

### Each standing

| Standing | What it means | What to do |
| :-- | :-- | :-- |
| **In sync** | The file says what the document says | Nothing |
| **About to be written** | You changed the document and Tisty writes it in a moment | Nothing |
| **Changed outside** | Someone changed the file outside Tisty, or deleted it | Choose: [bring or overwrite](#a-change-made-outside-bring-it-in-or-overwrite) |
| **Changed on both sides** | Both the file and the document changed since the last write | Choose which to keep |
| **Written by an earlier Tisty** | An earlier version of Tisty wrote the file without noting which documents it came from, and today it does not say what they say | **See the file**, then **Overwrite with Tisty** if you keep the document |
| **On hold** | One of the documents that make up this file [changed without your approval](#to-approve), or left AGENTS by an agent's hand, so the file stays as it was. The card names that document | Approve that document, or put it back and take it out yourself |
| **Paused** | You paused sync for that assistant, or for the whole folder. For Gemini, also when the line `@tisty-rules.md` is missing from `GEMINI.md` | **Resume** whenever you like; for Gemini, [**Put the line back**](#gemini-paused) |
| **Not allowed on this computer** | This computer has not allowed AGENTS | **Allow on this computer**, in Settings › Assistants |
| **Nothing to write** | The document is empty, and Tisty never empties an agent's file | Write something, or take the document out of AGENTS |
| **The file is a link** | The file, or a folder on its way, is a symbolic link; writing it would replace the link | Swap the link for the real file |
| **The code block is broken** | The block that holds a script is no longer whole | Mend the block in the document |
| **Cannot be read** | The file exists, but Tisty could not read it | Check its permissions |

Two more notices can take the place of the rows:

- **«No assistant on this computer reads this document yet.»** The assistant is not on this computer, or the name is not a file that assistant reads.
- **«Another document in AGENTS already writes this file, so this one is left out.»** Two documents carry the same `agent:` line: only one is written.

And if the document is not applied on this computer yet, an amber notice says why, with the button that settles it: [On several computers](#on-several-computers) tells it.

## A change made outside: bring it in or overwrite

You can keep editing your agents' files in your editor, or let the agent change them. Tisty never writes over those changes.

**An example.** You open `~/.claude/CLAUDE.md` in your editor and add «- Run the tests before every commit.». The next time Tisty looks (on start, when you come back to the window, or when you open the document), the sidebar says **1 changed** beside AGENTS, and the card of **How I work**, the document of that `CLAUDE.md`, says **Changed outside**, with a notice:

![The card in English of How I work, with the path CLAUDE.md under the name, the label Changed outside in amber and the notice The file changed outside Tisty, ~/.claude/CLAUDE.md no longer says what this document says, and the buttons Bring the change in, Overwrite with Tisty and See the file](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-04-outside.png)

- **See the file** shows what the file holds now.
- **Bring the change in** writes into the document what the file says. What the document said before stays in its [earlier versions](Documents#earlier-versions). If someone changed the document after the card was drawn, Tisty does not write over it: it asks you to read it again and choose again.
- **Overwrite with Tisty** asks «Write this document over `~/.claude/CLAUDE.md`? What the file holds now is kept as a backup.» and, if you say yes, writes the file again from the document.

Until you choose, Tisty does not write that file, even if you change the document.

**Bring the change in** only shows when the change can be traced to this document: the document belongs to that same assistant, not to Global, and you are not editing it. If `CLAUDE.md` is put together from a Global rule and your Claude document, and the change falls in the Global part, only **Overwrite with Tisty** is left: copy into Global by hand what you want to keep, then overwrite.

**If the file was deleted**, the card says **Changed outside** without **See the file**: **Overwrite with Tisty** makes it again.

### Changed on both sides

If both the file and the document changed, the card says **Changed on both sides**. When the document is the only one that makes up that file, **Keep the file's version** shows, which asks «Make this document read as … does now? What it says now stays in its history.»; otherwise only **Overwrite with Tisty**.

The same can happen the first time, when you already had a document in AGENTS for a file that changed in between. Then **It exists on both sides** opens, with the lines that differ: in red, with `−`, what is **In Tisty**; in green, with `+`, what is **In the file**. You choose **Keep Tisty's**, **Keep the file** or **Decide later**, and what you do not keep is still kept: the file in its [backups](#backups), the document in its earlier versions. If the file is made of several documents, you can only keep Tisty's or decide later. If Tisty never brought that file, the box adds **Bring it into AGENTS**, as [A file you never brought](#a-file-you-never-brought) tells. From **Settings › Assistants**, «1 file says something different in Tisty and in the agent.» and **Review** bring you back to that box.

![The box in English It exists on both sides for ~/.codex/AGENTS.md: the path at the top, the compared lines below, in red with a minus sign the one that is only In Tisty and in green with a plus sign the one that is only In the file, and the buttons Decide later, Keep the file and Keep Tisty's](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-07-both.png)

## Backups

**Before writing over an agent's file, Tisty keeps what it held.** Backups live on this computer, in `data/agents/` inside Tisty's folder, never in your agents' folders, and they do not travel with sync or parcels.

- **Backup #1 is the original**: the file as it was before Tisty first wrote over it. That one is never removed.
- **Then, the last 20.** When the 21st comes, the oldest one after the original goes.
- **Taking a file away leaves one too.** If you [take away the file](#taking-a-rule-and-its-file-out) of a rule you took out, its backups stay on this computer, and if a rule writes that same path again, they show in its **Backups**.

To see them, press **Backups** on the card, or the **3 backups** link on the file's row:

![The Backups box in English for ~/.claude/CLAUDE.md, with the line What ~/.claude/CLAUDE.md held before Tisty wrote over it. The last 20 are kept., and three rows, Backup #3 · 1.2 kB, Backup #2 · 1.1 kB and Backup #1 · 0.9 kB, each with See and Restore](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-05-backups.png)

- **See** shows the whole backup.
- **Restore** asks «Put this backup back in …? What the file holds now is kept as a backup.» and writes it into the file.

**A restored backup does not change your document.** The file says again what the backup said, and the card turns to **Changed outside**: you choose **Bring the change in**, so the document says the same, or **Overwrite with Tisty**, if you changed your mind. That way you never lose either side.

The **document's** earlier versions, on the other hand, are those of any document: they are in its [history](Documents#earlier-versions).

## Pausing, turning off and emptying

**Pausing one assistant.** On the card, **Pause sync with Claude**, or the **Pause Claude** link on its row. Also in **Settings › Assistants**, on the **AGENTS** card, with the **Sync Claude** switch. Paused, Tisty neither opens nor writes that assistant's files: they stay as the last write left them, and you keep editing your documents. **Resume sync with Claude** writes what you changed, except a file that changed outside in the meantime, which stays marked for you to decide.

**Turning the whole folder off.** On the same card, **Sync the AGENTS folder**. Off, the sidebar says **off** beside AGENTS, and each document's card says «Sync is off for the whole AGENTS folder.».

![The AGENTS card in Settings › Assistants, in English: the line Your agents' rules, skills and memory as Tisty documents, the button Bring another file, and below it the switch Sync the AGENTS folder on and Sync Claude, Sync Codex and Sync Gemini, with Codex off and the note Off: its files stay as they are, and nothing in Tisty is deleted](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-06-settings.png)

A pause and the permission belong to this computer.

### Gemini paused

Tisty puts the line `@tisty-rules.md` in your `GEMINI.md` once. If you later delete it, or delete `GEMINI.md` altogether, Tisty takes that as your decision and does not put it back: Gemini stays **Paused**, stops getting your rules, and Tisty stops writing `tisty-rules.md`. You see it in three places:

- **On the card** of each rule Gemini reads: the **Paused** label and «Gemini does not read this rule: the line @tisty-rules.md is missing from GEMINI.md, and Tisty does not put it back by itself.», with the **Put the line back** button.
- **In Settings › Assistants**, under **Sync Gemini**: «Paused: the line @tisty-rules.md is missing from GEMINI.md.», with **Put the line back**.
- **When you go into AGENTS**, the box **Tisty's line is missing**: «The line @tisty-rules.md is no longer in GEMINI.md, so Gemini does not read your rules. Tisty does not put it back by itself: Gemini stays paused until you do.», with **Leave it paused** and **Put the line back**.

**Put the line back** adds the line again at the end of `GEMINI.md`, touching nothing else, and Tisty writes your rules to `tisty-rules.md` again. Until you do, the sidebar counts Gemini under **changed** beside AGENTS.

### Taking a rule and its file out

Tisty stops writing a document when you take it out of AGENTS: when you delete it, archive it or move it out, when you change its `agent:` line to another file's, and when you go back to an [earlier version](Documents#earlier-versions) that had no card, or another one.

- **If the file was put together from several documents**, like `CLAUDE.md` with a Global rule, Tisty writes it again without that part, keeping a backup first.
- **If it was the only one**, no rule writes that file any more, but it is still on disk and your agent keeps reading it. So about a second later, Tisty asks what to do with it.

**An example.** You had the rule **Tests**, from `~/.claude/rules/tests.md`, and no longer want it. You delete it, and **Take the file away too?** opens: «No rule in AGENTS writes this Claude file any more, but it is still on disk and Claude keeps reading it.», with the path below and «If you take it away, a backup stays on this computer.».

![The box Take the file away too? in English, with the line No rule in AGENTS writes this Claude file any more, but it is still on disk and Claude keeps reading it., the path ~/.claude/rules/tests.md in grey, the line If you take it away, a backup stays on this computer., and the buttons Leave it and Take the file away](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-11-take-away.png)

- **Take the file away** keeps a backup and deletes it: Claude stops reading that rule. If it was the last file in a skill's folder, that folder goes with it; the assistant's own folders, such as `rules`, stay.
- **Leave it** leaves it where it is, and Tisty stops answering for it: it neither writes it nor asks again. If one day you want it back in AGENTS, it shows in [Bring another file](#bring-another-file).

If the file changed after Tisty wrote it, for example because you edited it by hand, the box adds «It changed after Tisty last wrote it.». The backup keeps the file as it is now, with that change.

If you take away `tisty-rules.md`, Gemini's rules file, Tisty also takes the line `@tisty-rules.md` out of `GEMINI.md`, and nothing else.

Until you answer, the sidebar counts that file under **changed**, and the **AGENTS** card in **Settings › Assistants** says «1 file no longer belongs to any rule in AGENTS.», with **Review** to come back to the question.

Tisty does not ask, and leaves the file as it is:

- about agent memory, which the agent writes itself: neither each Claude Code project's memory nor Gemini's `GEMINI.md`;
- about a file of an assistant you paused or turned off;
- about a file whose document [waits for your approval](#to-approve);
- about a file that no longer exists.

Your other computers do the same: when the deleted rule reaches them through sync, each one asks about its own file the next time Tisty looks, and the backup stays on that computer.

### Emptying AGENTS

If you take every document out of AGENTS while Tisty is open, it first asks about the files its last rules wrote, one by one, as in [Taking a rule and its file out](#taking-a-rule-and-its-file-out). Once you answer the last one, the AGENTS folder and its subfolders leave Tisty, and the files you chose to leave stay as they were.

Only an AGENTS that held documents and that you emptied with the window open goes. An AGENTS empty some other way stays: one made without finding any files, or one that was already empty when Tisty opened.

If you later want AGENTS again, **Create the AGENTS folder**, in Settings › Assistants, opens the first-time box again, with the rules and skills it finds already ticked, the ones you left included, and memory unticked, as always.

### Hiding it from the sidebar

If you would rather not see it all the time, right-click AGENTS and choose **Hide from the sidebar**. It only leaves the sidebar on this computer: it keeps syncing, and you find it under **Hidden**, at the bottom of the sidebar, or with **Show hidden**.

Even hidden, if something waits for your decision, its row under **Hidden** says so, such as **1 changed** or **2 waiting**. If everything is in sync, it says nothing.

### What stops it being AGENTS

Tisty knows the folder because it sits at the top, is named **AGENTS** and carries its robot icon. If you rename it, change its icon, move it inside another folder or archive it, Tisty stops writing the files, and leaves them as they were, without asking about them.

## On several computers

If you use Tisty on more than one computer, say the one at home and the one at work, your AGENTS documents travel with your sync like any other. But **each computer chooses what it applies to its agents**: a document that comes from another computer is written to no file until you say so, on that computer and from Tisty's window.

### A second computer, step by step

**An example.** On the home computer you brought in your `CLAUDE.md` and wrote the rule «For all my agents» in **AGENTS › Global**. There they applied by themselves: you brought them and wrote them, on that computer. Now you install Tisty on the work computer and sync:

1. In **Settings › Assistants**, the **AGENTS** card says «This computer has not allowed it yet.». Press **Allow on this computer**.
2. The same first-run box opens, with this computer's files. Those that already have a document say **already in AGENTS**, and if everything it found is already there, the box says «Everything I found is already in AGENTS. Allow this computer to keep it in sync:». Tick what you want to bring from this computer, such as its projects' memory, and press **Bring and sync** or **Allow and sync**.
3. The sidebar says **2 waiting** beside AGENTS: the two documents that come from home.
4. Open **For all my agents**. The card has no file rows yet, and an amber notice says «This document comes from another computer. Apply it to the agents on this one?», with two buttons:
   - **Apply here** writes it to the files of this computer's assistants, with its backup first, as always. If a file on this computer already said something else, Tisty does not write over it: the card stays at **Changed on both sides** and you choose.
   - **Not on this computer** leaves it out here. The card says «You chose not to apply it on this computer.», and **Apply here** stays at hand in case you change your mind.
5. Do the same with **How I work**, your `CLAUDE.md`. The sidebar is back to **in sync**.

![The card in English of For all my agents on the work computer: the labels Rule and Global, the name For all my agents with the path for-all.md under it, no file rows, and the amber notice This document comes from another computer. Apply it to the agents on this one? with the buttons Apply here and Not on this computer; in the sidebar, beside AGENTS, 2 waiting](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-08-another-computer.png)

What you choose holds for that document on that computer, and it stays:

- **A document you already apply updates by itself.** If tomorrow you change «For all my agents» at home, work writes it as soon as it arrives, without asking again.
- **What you write or bring in on one computer is applied on that computer with no click.** Only what comes from another one waits.
- **Waiting counts only what this computer would read.** A Codex document does not wait on a computer without Codex.
- **Once applied, the card offers no way back.** For one assistant on this computer to stop receiving it, pause that assistant here; for no computer to receive it, take it out of AGENTS.

### Memory belongs to the computer that brought it

An agent's memory speaks of what happened on that computer, so it is never written on another. On your other computers, its card says «Another computer's agent memory: on this one it is only read.», with no buttons. Each computer can bring its own memory of the same project: it shows unticked in the first-run box, even if another computer already brought its own.

### To approve

**Only words you wrote or approved reach an agent's file.** What you write in Tisty's window, what you bring in the first time and what you bring with **Bring the change in** already counts as approved. A text that changed without going through your hands does not: for example, if someone edited the document's file inside Tisty's data folder, or an imported parcel brought it.

Then the sidebar says **1 to approve** beside AGENTS, and the card warns «Its text does not have your approval yet, so it reaches no agent until you approve it.», with **Approve this text**. Read it first: the button asks «Approve what this document says now? Read it first: it is written to your agents' files.», and if the document changed between your reading and your yes, Tisty approves nothing and asks you to read it again.

If that document was already written to a file, the file does not change meanwhile: its card says **On hold** and names the document holding it, for example «This file stays as it was: claude · rules/style.md changed or left AGENTS without your approval. Approve it, or put it back and take it out yourself.».

A document an agent put in AGENTS, and not you, is not written either: its card says «An agent put it here, so no agent reads it until you move it yourself.». Move it yourself to another AGENTS subfolder you made (reordering it is not enough), and it applies. If an agent made or renamed the folder it sits in, the same goes for everything inside it: move it to a folder of yours.

### Written by an earlier Tisty

An earlier version of Tisty wrote the files without noting which documents each one came from. The first time the new version looks at them, it compares: if the file says what its documents say, it takes it over without a word. If it says something else, the card stays at **Written by an earlier Tisty**, with the notice «An earlier Tisty wrote this file without noting which documents it came from, so it is left as it is until you choose». **See the file** shows you what it holds, and **Overwrite with Tisty** writes it from your documents, with its backup first.

## What is never read

- **Your assistants' settings**: `settings.json`, `settings.local.json`, `config.toml`, `config.json`, `hooks.json`, `mcp.json`, `mcp-config.json`, `.claude.json` and `keybindings.json`. Tisty only looks where rules, skills and memory go, never in an assistant's main folder, and inside a skill a file with one of those names does come in, because it belongs to the skill.
- **Credentials**, by name and without opening them: `.credentials.json`, `credentials.json`, `auth.json`, `.env` and its variants, `.netrc`, and `.pem`, `.key`, `.p12`, `.pfx`, `id_rsa` or `id_ed25519` keys.
- **A file that looks like it holds a secret**, such as a token or a private key, even under an innocent name. The check runs on this computer and looks for known shapes, so it may not recognise them all: keep secrets out of your agents' rules.
- **Claude Code's conversations**: of its `projects` folder, only each project's `memory` folder is looked at.
- **Symbolic links** and files larger than 512 kB.

And an AGENTS document is only written where each assistant keeps its rules, skills and memory, never to its settings or credentials.

**Agent memory** comes unticked the first time, and once in Tisty it is read only: the agent keeps writing it, and when it changes, the card says **Changed outside** and **Bring the change in** brings Tisty up to date. It has no **Edit**, **Overwrite with Tisty** or **Restore**, and on your other computers it is [only read](#memory-belongs-to-the-computer-that-brought-it).

What Tisty keeps of all this, and what travels, is in [Privacy](Privacy#your-assistants-in-agents).

## With your agent, through MCP

An agent you let in through [[MCP|Assistants-and-MCP]] **reads** your AGENTS documents like any other, and its guide tells it what they are. **It does not write them: it proposes the change, and only you approve it.** For example, you ask Claude Code «Which of my rules do you read?».

**1. It reads its guide.** `rules` with the topic «documents» brings, among others, the section on AGENTS:

```json
{ "name": "rules", "arguments": { "topic": "documents" } }
```

```text
## Assistants' files in AGENTS

A document whose properties hold an `agent:` line is an assistant's own file kept in the folder
AGENTS — a rule file such as `CLAUDE.md`, a skill, a script — and Tisty writes its text, byte for
byte, to the file that assistant reads, on each computer where the person applied the document.
`read` gives the text the person approved. You change one only by proposing: `write_doc` with
`doc` and `print`, `edit_doc` or `append_doc` keep your text byte for byte, apart, and nothing
reaches any file until the person approves it on the document's card: say so, never call it done. …
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

**3. It looks at its card and reads it.** `brain` with `doc` gives its outline and place; `read` with `doc`, the text you approved, whole, `agent:` line included:

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

### A proposal from your agent

You ask it «add to my rules that it runs the tests before every commit». The agent edits the document like any other, but in AGENTS its change is not written: **it stays apart, as a proposal**, byte for byte. Nothing changes in the document or in your agents' files until you approve it.

**1. The agent proposes.** It uses `edit_doc`, `append_doc` or `write_doc` with `doc`, `print` and the whole text:

```json
{ "name": "edit_doc", "arguments": { "doc": "k4r5j4qr-0001", "old": "- Branches shaped type/name.", "new": "- Branches shaped type/name.\n- Run the tests before every commit." } }
```

```text
Proposed that change to "claude · CLAUDE.md" [k4r5j4qr-0001]. It waits beside the approved text for
the person, who approves or discards it on the document's card in Tisty; no assistant's file
changes until they approve it. Your next write to this document goes on top of this proposal, at
this `print`.
```

Its guide asks it to tell you just that, «I proposed it», and never to call it done.

**2. You see it in Tisty.** The sidebar says **1 with a proposal** beside AGENTS. Open the document: the card gains the label **Proposal** and, below, a purple box:

- «An agent (Claude Code) proposes a change», with when it proposed it;
- «Nothing of it reaches your agents' files until you approve it. What it changes in the approved text:»;
- the compared lines: in red, with `−`, what is **Approved** and goes; in green, with `+`, what is **Proposed** and comes;
- the buttons **Approve the change** and **Discard**.

![The card in English of How I work with the labels Rule, Claude, In sync and Proposal, and below it the purple box An agent (Claude Code) proposes a change · 2 minutes ago, the line Nothing of it reaches your agents' files until you approve it, the compared lines with the header − Approved and + Proposed, the new line + - Run the tests before every commit. in green, and the buttons Approve the change and Discard; in the sidebar, beside AGENTS, 1 with a proposal](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-09-proposal.png)

If Tisty is open while the agent proposes, the proposal shows by itself within a second or two, even when the window is not in front.

**3. You approve or discard.**

- **Approve the change** asks «Approve this change? It is written to the document and to your agents' files.». With your yes, the document says what was proposed, with your approval, and on each computer where that document applies, Tisty writes it to `~/.claude/CLAUDE.md` like any change of yours, with its backup first. What it said before stays in its history.
- **Discard** asks «Discard this proposal? The document stays as you approved it.». The proposal goes and nothing else changes.

Each button acts only on what the card showed you. If the agent wrote again while you were looking, Tisty does not choose for you: «The proposal changed since the card showed it. Look at it again before choosing.». While you edit the document, **Approve the change** does not show: press **Done** first.

**Good to know:**

- **One proposal per document.** If the agent writes again, its change goes on top of its own proposal, and `read` hands it back apart from the approved text, in `proposal`. If it writes the approved text back, it withdraws it.
- **Approved text keeps reaching your agents** while a proposal waits: a proposal on one rule holds back nothing else. The **Agent file** panel says so: «Yes · an agent's change waits for your approval».
- **If the approved text changed since**, the card warns «It was proposed over an earlier version, so approving it may undo a change made since.». Look closely at the lines before approving.
- **If the document is to approve**, you first approve its text with **Approve this text** and then the proposal: «This document's text does not have your approval yet. Approve it first, then the proposal.».
- **`#words` in an agent document are not Tisty tags.** A rule that says «use #rust» does not show under that tag.

### What it cannot propose

**The `agent:` line does not change**, because it decides which file the document is written to:

```json
{ "name": "edit_doc", "arguments": { "doc": "k4r5j4qr-0001", "old": "claude/rule/CLAUDE.md", "new": "codex/rule/AGENTS.md" } }
```

```text
"claude · CLAUDE.md" [k4r5j4qr-0001] would get another `agent:` line, which only the person
changes, so nothing was proposed. Tell the person what you would change and why.
```

**Moving, archiving, marking or summing up** an AGENTS document, attaching a file to it or going back to an earlier version is refused, and Tisty tells it how to propose:

```text
the document "claude · CLAUDE.md" holds an assistant's own files in AGENTS, which only the person
changes, in Tisty: nothing an assistant moves, files, marks or sums up there reaches them. To
change its text, propose it: `write_doc` with `doc`, `print` and the whole `body`, `edit_doc`, or
`append_doc` at its end, and nothing else with them. The person approves or discards it on the
document's card.
```

**Putting another document or making a folder** inside AGENTS is refused too, and there Tisty asks it to tell you what it would change, for you to do it.

**A new document with an `agent:` line** is refused in any folder, so nobody can bring a rule in from outside:

```text
a body with an `agent:` line is an assistant's own file, which only the person makes, in AGENTS.
Write it without that line, or tell the person what the file should say.
```

**Agent memory** takes no proposals: the agent itself writes it, in its own file. Nor does a proposal get in that would leave a script's code block broken, or that goes past 64,000 characters or 2,000 lines, what the card can compare.

And editing the file on disk does not help either: that change stands as **Changed outside** and waits for your decision. **Why it works this way:** an AGENTS rule reaches all your agents, on all your computers; if an agent could write it alone, an instruction hidden in a page it read could end up in the rules of the others. With a proposal, you see every line before it arrives.

## Questions

**Does Tisty replace my files?** It neither moves nor converts them: it keeps writing them where your agent looks for them. Before the first write it keeps the original, backup #1, which is never removed.

**Can I still edit `CLAUDE.md` by hand?** Yes. Tisty marks the file **Changed outside** and waits for you to choose: bring your change into the document, or write over it.

**What if I close or uninstall Tisty?** Your agents keep reading their files as Tisty left them. Nothing depends on Tisty being open.

**Does AGENTS touch `settings.json`, hooks or my MCP configuration?** No: AGENTS neither reads nor writes them. Connecting an assistant is something else: there Tisty adds its MCP server and its hooks to that assistant's configuration, as [[Assistants and MCP|Assistants-and-MCP]] tells.

**Why does agent memory come unticked?** Because the agent writes it, it may talk about your projects, and once in Tisty it travels with your sync and your backups like any document. Bring it in only if you want to read it in Tisty.

**What is the `@tisty-rules.md` line in my `GEMINI.md`?** It is the only line Tisty adds to that file, so Gemini reads your rules, which go in `tisty-rules.md`. The rest of `GEMINI.md` is written by Gemini. If you delete it, Tisty does not put it back: Gemini stays [paused](#gemini-paused) until you press **Put the line back**.

**Does AGENTS see the assistants I use inside WSL?** No. On Windows, Tisty looks at your Windows user folder, and an assistant running inside WSL keeps its files in your Linux home.

**And each repository's rules?** They stay where they are. AGENTS handles your user's files, not each project's `AGENTS.md` or `CLAUDE.md`.

**Can I write a new rule from scratch?** Yes: make a document inside AGENTS and write its properties with the `agent:` line, as in [Global or one assistant](#global-or-one-assistant). Tisty writes it as soon as you save it.

**How many backups does it keep?** The original and the last 20 of each file, on this computer.

**Can I ask my agent to change my rules?** Yes, and it proposes it: its change stays apart, on the document's card, until you press **Approve the change** or **Discard**. Nothing reaches your agents before. See [A proposal from your agent](#a-proposal-from-your-agent).

**I use Tisty on two computers. Do my rules reach the other one by themselves?** They arrive as documents, with your sync, but on the other computer they wait for your **Apply here** the first time. After that they update by themselves. See [On several computers](#on-several-computers).

**Can I keep a rule on one computer only?** Yes: on the computer where you do not want it, press **Not on this computer** on its card. What each computer chooses is its own.

**I deleted a rule. What happens to its file?** Tisty asks «Take the file away too?»: **Take the file away** deletes it and keeps a backup, and **Leave it** leaves it where it is, without Tisty writing it again. See [Taking a rule and its file out](#taking-a-rule-and-its-file-out).

**The first time I left a file unticked. Can I bring it now?** Yes, with **Bring another file**, by right-clicking AGENTS or from Settings › Assistants. See [Bring another file](#bring-another-file).

**Why does AGENTS say empty?** It holds no documents: you made it without Tisty finding any files, or its documents left while Tisty was closed, for example when syncing with another computer. Write a rule with its `agent:` line, or bring a file with **Bring another file**.

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
- **On hold**: another document of the same file waits for your approval. The card names it; [approve it](#to-approve).
- **Written by an earlier Tisty**: look at the file and, if you keep the document, **Overwrite with Tisty**.
- **An amber notice about another computer or your approval**: the document is not applied on this computer yet. See [On several computers](#on-several-computers).

**My rule does not reach my other computer.** On that computer, check that AGENTS is allowed (**Allow on this computer**, in Settings › Assistants) and open the document: if it says «This document comes from another computer», press **Apply here**; if it says «You chose not to apply it on this computer.», that too. If the card says «No assistant on this computer reads this document yet.», that computer does not have the rule's assistant.

**The sidebar says to approve and I changed nothing.** A document's text changed without going through Tisty's window, for example through an imported parcel or a direct edit in the data folder. Open it, read it, and if it is right, **Approve this text**.

**I wrote a Global rule and a file stands Changed on both sides.** That file already existed and you did not bring it in, so Tisty cannot tell which part is yours. Do not overwrite it: in **Settings › Assistants**, press **Review** and, in **It exists on both sides**, **Bring it into AGENTS**. The file ends up with Global in front of your part, and the original as backup #1. See [A file you never brought](#a-file-you-never-brought).

**It asks «Take the file away too?» and I deleted nothing.** Changing a rule's `agent:` line to another file's, or going back to an earlier version that had no card, also counts as taking a rule out. If you would rather touch nothing, **Leave it** leaves it where it is.

**AGENTS is gone.** You took all its documents out with Tisty open, and when that happens the folder goes. If you want it back, **Create the AGENTS folder** in Settings › Assistants brings in what it finds again. If you did not empty it, look under **Hidden**, at the bottom of the sidebar.

**The whole folder stopped writing.** Check that AGENTS still sits at the top of the Library, with that name and its robot icon, and not archived.

**Codex does not read my rules.** If you have `~/.codex/AGENTS.override.md`, Codex reads only that file. Your Global rules go there, but a document with `agent: codex/rule/AGENTS.md` is written to `AGENTS.md`, which Codex does not read while the other one exists. If you made the override after the first time, [bring it into AGENTS](#codexs-agentsoverridemd).

**Gemini does not read my rules.** If its card says **Paused**, the line `@tisty-rules.md` is missing from `~/.gemini/GEMINI.md`: press **Put the line back**, on the card or in Settings › Assistants. Tisty does not put it back on its own, because deleting it is your decision. See [Gemini paused](#gemini-paused).

**Bring the change in does not show.** The document is a Global one, the change falls in another document's part of the same file, or you are editing it: press **Done** and look again.

**«The file changed while Tisty was writing, so it was left alone.»** Another program wrote the file at the same moment. Tisty writes over nothing: look at the card again and choose.

**I restored a backup and it says Changed outside.** That is expected: [a restored backup does not change your document](#backups). Choose **Bring the change in** to keep it.

## See also

- [Privacy](Privacy#your-assistants-in-agents): what AGENTS reads, when, and what stays on this computer.
- [[Assistants and MCP|Assistants-and-MCP]]: connecting an agent to Tisty.
- [Earlier versions](Documents#earlier-versions): a document's history.
- [Folders](Documents#folders): filtering, archiving and unarchiving.
- [Properties](https://markdown.tisty.app/properties), in the Tisty Markdown guide: the `agent:` line and its marks.
