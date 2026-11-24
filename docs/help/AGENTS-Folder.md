# Agents: your agents' rules, skills and memory

I use more than one AI agent, and each one reads its rules from a different file, in a different folder: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md`, `GEMINI.md`. Changing one rule everywhere meant opening four files, and sooner or later one fell behind. The **Agents** folder gathers those files in Tisty, as documents, so you can read and arrange them: one document is one file, word for word. And if you want, Tisty writes underneath the file each agent reads.

> [!TIP]
> **In short:** the **Agents** folder keeps the rules, skills and memory of Claude Code, Codex, GitHub Copilot CLI, Gemini CLI and Cursor as documents. There are two words, and each means one thing. **Bring** copies your agents' files into Tisty, to read and arrange them, and writes nothing into them. **Sync** makes what you change in Tisty reach those files from then on, and has Tisty warn you if someone changes them outside. While it syncs, each computer writes only its own folder: the skills in **Global** reach all your agents, and what you write in an agent's folder on your computer reaches only that one. Before writing over a file Tisty keeps what it held, and a change made outside is never written over. And an agent does not write your rules: it proposes the change, and only you approve it.

![Agents the first time, in English: the box Your agents, in one place, with the X to close it at the top right, the line Tisty looks for the rules, skills and memory of your agents and brings them here as documents, to read and arrange. Bring only copies them; Bring and sync also makes what you change in Agents reach your agents by itself., its three lines with a padlock, arrows and copies (It stays in your Tisty…, If you sync, it reaches Claude, Codex, Copilot, Gemini by itself., Before changing an agent's file, Tisty keeps a backup on this computer.), and below it This is what I found. Choose what to bring:, the line Gemini: no rules, skills or memory yet. It gets what you write in Global or in its folder., and three folded rows, Claude · 2 rules · 29 skills · 49 memories, Codex · 1 rule · 4 skills and Copilot · 1 rule; at the bottom right, only two buttons, Bring and Bring and sync, the latter highlighted](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-01-first-run.png)

## What it is and when it helps

**Agents** is a folder in your Library, at the top, with a robot icon. It sits in your sidebar from day one, in grey, until you bring something in. Each document inside it is one of an agent's files: a rule, a skill or its memory. You read it and edit it in Tisty, with its search and its earlier versions, but it does not look like one more document: it opens on its own [card](#the-card-of-an-agent-document), which tells you who reads it, where it is written and how many backups it has, with the agent's text just as you wrote it. If you sync it, Tisty writes the file that agent reads, byte for byte.

It helps when:

- **You want to read and arrange the rules you already have.** With **Bring**, your `CLAUDE.md`, your skills and your Copilot rules come in as documents, with headings and search, instead of flat files in hidden folders. Your agents keep reading their files as they were.
- **You use several agents and want the same skill in all of them.** With **Sync**, you write the skill once in **Global**, and it reaches Claude Code, Cursor and the folder Codex, Copilot, Gemini and DeepSeek Harness share. Rules, on the other hand, belong to each agent: each one reads its own file, and what that file says is what your document says.
- **You work on more than one computer.** Each computer has its own folder inside Agents and writes only its own: what is in the work one does not write over the home one, and what is in another computer's folder you read from here but do not change from here.
- **You have skills with scripts.** Each file of the skill is a document, and a script is kept in a code block that is written as is.
- **You want to be able to go back.** While syncing, Tisty keeps the original file before it first writes over it, and the last 20 before each overwrite.
- **You want to see what your agent noted about you.** The memory Claude Code writes in each project, or Gemini's `GEMINI.md`, can be brought in to read in Tisty, and Tisty never changes it.

It is not for each repository's files: Agents handles your user's files, the ones in your home folder, not the `AGENTS.md` or `CLAUDE.md` that lives inside a project.

## Bring or sync

They are two different steps, and you can stay on the first one as long as you like.

| | **Bring** | **Sync** |
| :-- | :-- | :-- |
| What it does | Copies the files you tick into Tisty, as documents | From then on, what you change in Tisty reaches your agents' files |
| Your agents' files | Stay as they are: Tisty writes nothing into them | Tisty writes them with what your documents say, with a backup before each overwrite |
| A change made outside | Stays in the file alone: Tisty asks you to decide nothing | Tisty warns you on the document's card and waits for you to choose |
| Your documents | Stay in Agents, and you edit them like any document | The same |
| What it needs | That this computer lets you read the files you tick | The same, once per computer |
| How it is undone | You delete or move the documents, like any other | **Stop syncing**: no agent file is touched and no document is deleted |

**Two examples.**

- Jaime uses only Claude Code and wants to read his rules and skills calmly, without anything changing in `~/.claude`. He chooses **Bring**. His documents sit in **Agents › Home (this computer) › Claude**, and Claude Code keeps reading its `CLAUDE.md` as always. If he edits one of those documents, the change stays in Tisty and nowhere else.
- Yisset uses Claude Code and Codex, and wants both to know her branches take the form `type/name`. She chooses **Bring and sync**, writes the rule in Claude's document and the same one in Codex's, inside her computer's folder, and Tisty writes them to her `CLAUDE.md` and her `AGENTS.md`. If what she wants to share is a skill, that one she writes once in **Agents › Global**.

If one day Jaime wants his changes to reach Claude Code, he presses **Sync** and Agents starts syncing, without bringing anything again. If Yisset wants to keep everything still for a while, **Stop syncing**: her files stay as Tisty left them and her documents stay in Agents.

## Three states in the sidebar

The sidebar tells you how Agents stands by its colour, and no word appears beside it. What waits for your decision you see on each document's card and in **Settings › Agents › Agents folder**.

| State | How it looks | One click | Its right-click |
| :-- | :-- | :-- | :-- |
| **1. Nothing brought yet** | The robot and the name **Agents** in grey | Opens the first-time notice, [Your agents, in one place](#the-first-time) | **Hide from the sidebar** |
| **2. Brought, not synced** | The robot in grey and the name like any folder; when you hover over it, how many documents it holds, like any folder | Opens it like any folder; at the top, one quiet line: «Not synced with your agents · Sync» | **Sync**, [**Look again**](#look-again) and **Hide from the sidebar**, nothing else |
| **3. Syncing** | The robot in **coral** and the name like any folder; when you hover over it, how many documents it holds, like any folder | Opens it like any folder | **Stop syncing**, [**Look again**](#look-again) and **Hide from the sidebar**, which first asks you to stop syncing |

- **Coral is Agents' own**, and only while it syncs: purple belongs to diagrams, and no folder of yours can pick coral. In the Brainsty network, the documents in Agents come hidden, whether it syncs or not, so they do not cover what you write. They show when you turn **Show AGENTS** on in the filters: smaller than yours, in a faint coral and with fainter lines, and the legend names them **AGENTS**. They are never alike to anything. See [[Filtering|Brainsty-Network#filtering]].
- **The state 2 line** shows only on Agents, never on its subfolders. Its **Sync** link does the same as the one on the right-click.
- **With the sidebar folded**, Agents' robot has the same colour. Open or folded, a screen reader says the state in words: «Agents — nothing brought yet» or «Agents — Not synced with your agents».
- **The right-click** also opens with the menu key or with `Shift` + `F10`, with Agents selected.
- **The right-click on Agents itself** offers only syncing, looking again and hiding, by state: no other folder action. Its subfolders, such as **Agents › Claude**, have a menu for agents only, told in [A subfolder's right-click](#a-subfolders-right-click). Why Agents does not move or change its name is in [Agents stays at the top](#agents-stays-at-the-top).

## The first time

Agents is in your sidebar even if you never used it, in grey. That way you know it exists, and nothing of your agents' is read until you say so.

1. In the sidebar, click **Agents**. **Your agents, in one place** opens, with an X at the top to close it. Tisty looked at which agents you have on this computer and which files sit in their folders, **only their names and sizes**: it has not opened any of them yet.
2. At the top, one line tells you the two words: «Tisty looks for the rules, skills and memory of your agents and brings them here as documents, to read and arrange. Bring only copies them; Bring and sync also makes what you change in Agents reach your agents by itself.». Below, three more:
   - 🔒 «It stays in your Tisty: on your computers and wherever you sync. Nobody else sees it.»
   - ↻ If you sync, whom it reaches, with the agents it found: «If you sync, it reaches Claude, Codex, Gemini by itself.».
   - ⧉ «Before changing an agent's file, Tisty keeps a backup on this computer.»
3. Below, **This is what I found. Choose what to bring:**, one row per agent, with what it has to bring, such as **Claude · 2 rules · 1 skill · 1 memory**. Open it with a click: each rule and each memory is a row with its box, and each skill is **one single row**, whole, such as **pr · Skill · 3 files**. Hover over a row to see its full path.
4. Tick what you want to bring. A skill comes in whole with its box: its `SKILL.md`, its scripts and its references. Rules and skills come ticked; **agent memory comes unticked**, because once in Tisty it travels with your sync, so you tick it yourself if you want it.
5. Choose one of the two buttons:
   - **Bring** copies what you ticked and nothing else. Your agents' files stay as they are, and Agents is left in [state 2](#three-states-in-the-sidebar): brought, not synced. The box says, for example, «7 documents came into Agents. Your agents' files stay as they are until you choose Sync.». It stays greyed out until you tick something.
   - **Bring and sync**, the highlighted one, copies what you ticked and also syncs: from now on, what you change in Agents reaches your agents' files. Agents is left in state 3, with its coral robot, and the box says «7 documents came into Agents, and Tisty keeps them in sync from now on.».
6. Tisty opens only the files you ticked and keeps them as documents. Whatever could not come in shows under **These did not come in:**, with the reason. **Done** closes the box.

**Closing without choosing.** The X or `Esc` close the box without reading or bringing anything, and Agents stays grey in the sidebar. Come back whenever you like with another click on it, or from the **Agents** card in **Settings › Agents › Agents folder**, where **Bring** and **Bring and sync** open this same box. What comes into the list, and what does not, is in [What comes into the list](#what-comes-into-the-list).

**If you do not want to see it.** The box has no button to hide it: right-click **Agents**, in grey, and **Hide from the sidebar**. [Hiding it from the sidebar](#hiding-it-from-the-sidebar) tells it.

### What comes into the list

The list brings only what your agent really reads. The rest does not show, not even greyed out.

**Skills.** A skill is a folder with a `SKILL.md` at its root. Tisty looks for it as deep as each agent does, so a skill you keep inside a category folder, such as `~/.agents/skills/work/pr/`, shows when its agent reads it from there:

| Agent | Skills folder | Where a skill can be |
| :-- | :-- | :-- |
| Claude Code | `~/.claude/skills` | right inside, such as `skills/pr/` |
| Codex | `~/.agents/skills`, which it shares with Copilot, Gemini and DeepSeek Harness | up to six folders down, such as `skills/work/pr/` |
| DeepSeek Harness | `~/.agents/skills`, which it shares with Codex, Copilot and Gemini | up to six folders down, such as `skills/work/pr/` |
| GitHub Copilot CLI | `~/.copilot/skills` | right inside |
| Gemini CLI | `~/.gemini/skills` | right inside |
| Cursor | `~/.cursor/skills` | in any category folder |

A folder without a `SKILL.md` that its agent does not read deeper is not a skill and does not show. Tisty never looks for a skill inside another: a `SKILL.md` deeper inside `pr/` is just one more file of **pr**. Each skill is **one row**, such as **pr · Skill · 3 files**, with one box that brings it whole: each file of the skill becomes a document, and in the sidebar it shows as a single row, **pr**, with **3 files** on the right. A credentials file inside a skill, such as a `.pem` key, still stays out.

**Rules.** From a rules folder, only what the agent reads comes in:

- Claude Code: the `.md` files in `~/.claude/rules`, subfolders included.
- GitHub Copilot CLI: the `.instructions.md` files in `~/.copilot/instructions`, subfolders included. A plain `.md` there does not show, because Copilot does not read it either.
- A folder inside a rules folder that has its own `manifest.json` stays out whole: it belongs to another program, not to you.

**Agent memory.** Only its `.md` files.

**What never shows:**

- **A hidden file or folder**, whose name starts with a dot, at any depth. That leaves out, for example, Claude Code's `skills/.trash/` and Codex's `skills/.system/`, its system skills, which Codex rewrites itself.
- **Claude Code's `skills/synced/`**, in any capitalization: it is the copy of the skills in your claude.ai account, which Claude Code may overwrite or delete, so it is not yours to edit.

**An example.** Your Claude folder holds:

```text
~/.claude/
├── CLAUDE.md
├── rules/
│   ├── style.md
│   └── notes.txt
└── skills/
    ├── pr/
    │   ├── SKILL.md
    │   ├── scripts/open-pr.sh
    │   └── template.md
    ├── loose-notes/
    │   └── ideas.md
    ├── .trash/
    │   └── …
    └── synced/
        └── …
```

Claude's row reads **Claude · 2 rules · 1 skill**: `CLAUDE.md` and `rules/style.md` as **Rule**, and **pr · Skill · 3 files**. `notes.txt` does not show, because it is not a `.md`; `loose-notes`, because it has no `SKILL.md`; `.trash`, because it is hidden, and `synced`, because it is the copy Claude Code keeps of your claude.ai account.

**Each agent's row** counts rules, skills and memories, not files: a ten-file skill adds one, as in **Claude · 2 rules · 3 skills · 1 memory**. With nothing left to bring it says **Claude · nothing to bring**, and if everything is already in Agents, **Claude · already in Agents**.

**An agent with no files yet** is not left out as if it were not there: Tisty names it on a line of its own, above the list. If you installed Gemini yesterday and have not asked it to remember anything yet, you see «Gemini: no rules, skills or memory yet. It gets what you write in Global or in its folder.». **Bring another file**, [Look again](#look-again) and its switch in **Settings › Agents › Agents folder** say the same.

### What the list says beside a file

| In the list | What it means |
| :-- | :-- |
| **Rule**, **Skill**, **Agent memory** | It can come in. Memory comes unticked |
| **already in Agents** | A document in Agents already stands for it, so it is compared instead of brought again |
| **credentials, left out** | Its name is a credentials file's, such as `credentials.json`, `auth.json`, or a `.pem`, `.key` or `id_rsa` key. Those starting with a dot, such as `.env` or `.credentials.json`, do not even show |
| **Tisty's own** | A file Tisty left or a temporary one, such as `tisty.json` or one ending in `.tmp`, or the copy of one of [Tisty's skills](#tistys-skills-in-global) |
| **a link, left out** | It is a symbolic link: Tisty does not follow it |
| **too big** | Larger than 512 kB, what a document can hold |
| **too many files** | Tisty had already seen 2,000 files in your agents' folders and stopped looking |

What is greyed out cannot be ticked. Hidden files and folders, those starting with a dot, never show. Tisty also skips `node_modules`, and goes no deeper than eight levels.

### If a file does not come in

After bringing, a ticked file may stay out:

- **It looks like it holds a credential, such as a key or a token, so it stays out.** Tisty checked it on this computer and found something shaped like a secret. Take the secret out of the file and try again.
- **It is not text.** A binary file cannot be a document.
- **Its name cannot be kept as an agent document.** A name with `#`, `:`, `,`, quotes, square brackets or a backslash does not fit in the line that says where the file goes.

### If you have no agent files yet

The box says «I found no agent files on this computer. You can still sync: Agents fills up as you write your rules.». Below it, each agent it found with no files has its line, such as «Codex: no rules, skills or memory yet. It gets what you write in Global or in its folder.». **Bring** stays greyed out, since there is nothing to bring, and **Bring and sync** is still there: pressing it lets this computer sync and still creates the **Agents** folder, with **Global** and this computer's folder, with an empty subfolder for each agent it found, such as **Codex** and **Gemini**. The box says «Nothing new came in. The Agents folder syncs on this computer now: what you write there reaches your agents.», and in the sidebar Agents is left with its coral robot. Going into it does not open the box again.

From there you have two ways:

- **Write your first rule.** Right-click an agent's folder on your computer, such as **Home (this computer) › Gemini**, and **New document**: the document already starts as a rule of that agent, with its `agent:` line in place, as [A new document is already a rule](#a-new-document-is-already-a-rule) tells. Tisty writes the file as soon as you write something in it.
- **Bring what shows up later.** Once your agent makes its files, for example after you use it for the first time, bring them with [Bring another file](#bring-another-file).

## Bring another file

The first time does not have to be the only one. If you install a new agent, make a rules file that did not exist before, or left something unticked, you bring it later:

- **From the sidebar:** right-click a subfolder of Agents, such as **Agents › Home (this computer) › Claude**, and choose **Bring another file**. The right-click on Agents itself does not offer it: [Look again](#look-again) is there instead, which does the same and also ticks what is new for you. In state 1 nothing is brought yet, and a click on Agents opens the first time.
- **From the Library menu:** with Agents or one of its subfolders open, the **+** button beside the **Library** heading offers **Bring another file**, among the options to create and import.
- **From the command palette:** with Agents or one of its subfolders open, type «bring» and choose **Bring another file**, in the **Create** group.
- **From Settings › Agents › Agents folder:** on the **Agents** card, the **Bring another file** button.

**Bring another file does not change whether Agents syncs.** Brought but not synced, the new file comes in as a document and your agent's file stays as it is; while syncing, Tisty also keeps it up to date from then on. What you bring, here or the first time, goes to this computer's folder, and each agent Tisty finds here has its empty folder in it.

**Bring another file** opens, with «These are the agent files on this computer that are not in Agents yet. Choose what to bring:» and the same list as the first time, with the same rules: a skill is a single row that comes in whole, and what is hidden or what the agent itself maintains does not show. With one difference: **nothing comes ticked**, not even the rules. Tick what you want and press **Bring**: the button stays greyed out until you tick something. **Not now** closes it without bringing anything. What already has a document says **already in Agents**, and if nothing is left to bring, the box says «Every agent file on this computer is already in Agents.».

**An example.** One day you write a new rule for Claude Code by hand in your editor, `~/.claude/rules/tests.md`:

```markdown title="~/.claude/rules/tests.md"
# Tests

- Run the tests before every commit.
```

1. Right-click **Agents › Home (this computer) › Claude** and choose **Bring another file**.
2. Open Claude's row: `CLAUDE.md` shows dimmed, with **already in Agents**, and `rules/tests.md`, unticked, as a **Rule**.
3. Tick it and press **Bring**. If Agents syncs, the box says «1 document came into Agents, and Tisty keeps it in sync from now on.»; if it is brought but not synced, «1 document came into Agents. Your agents' files stay as they are until you choose Sync.». **Done** closes it.
4. **Tests** shows up in **Agents › Claude**, with a book icon. From now on you read it and change it in Tisty, like any rule.

If this computer has not let you read your agents' files yet, for example because Agents came in brought from [another computer](#on-several-computers), **Bring another file** opens the first-time box, with its three lines, what it found already ticked and its two buttons, **Bring** and **Bring and sync**.

**Bring touches nothing.** Neither your agents' files nor the record Tisty keeps of what it wrote: a file you only brought is never noted as written by Tisty, so if you later take its document out of Agents, Tisty does not offer to take it away with [Take the file away too?](#taking-a-rule-and-its-file-out). If you later choose **Sync**, a file still exactly as you brought it is taken as it stands.

### Look again

**Look again** is on the right-click of Agents itself, in state 2, brought but not synced, and in state 3, syncing. It looks again on this computer for your agents and their files, as the first time did: if you installed a new one, it makes its subfolder, and opens **Bring another file** with **only what is new already ticked**. What is already in Agents shows dimmed with **already in Agents**, and agent memory comes unticked, as always. Check the boxes and press **Bring**.

**An example.** Marisol had Agents syncing with Claude and Codex. One afternoon she installs Gemini CLI, asks it to remember her dog's name, and writes a new skill for Claude in `~/.claude/skills/report/`. Right-click **Agents** and **Look again**:

1. **Agents › Gemini** shows up in the sidebar, with its green compass, empty.
2. **Bring another file** opens. In Claude's row, **report · Skill · 2 files** comes ticked; in Gemini's, `GEMINI.md` shows as **Agent memory**, unticked.
3. Marisol leaves the memory out and presses **Bring**: «2 documents came into Agents, and Tisty keeps them in sync from now on.», one for each file of the skill.

The memory Marisol left out is still there: the next time she looks again, `GEMINI.md` shows in the list again, unticked.

**If nothing is left to bring**, not even an unticked memory, the box shows no boxes to tick: it says «Nothing new found. This is what each agent has:», and below it one line per agent, with its icon. For example, on Lucas's computer, which has Claude, Codex and Cursor:

- «Claude · already in Agents»
- «Codex · already in Agents»
- «Cursor: no rules, skills or memory yet. It gets what you write in Global or in its folder.»

**Not now** closes it.

![The Bring another file box opened by Look again, with nothing to bring: at the top the line Nothing new found. This is what each agent has:, and below three lines, each with its agent's icon: the orange chat bubble with Claude · already in Agents, the teal terminal with Codex · already in Agents and the green compass with Gemini · already in Agents; no boxes to tick; at the bottom right, Not now and Bring](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-17-look-again.png)

**On a computer that has not given permission yet**, such as a second computer where Agents came in brought, **Look again** opens the first-time box instead, with its names and sizes and its two buttons. It opens no file and makes no subfolder before your **Bring** or your **Bring and sync**.

### A file you never brought

This happens while syncing: brought but not synced, Tisty writes no file and has nothing to ask.

**An example.** You have `~/.codex/AGENTS.md` with your Codex rules, and the first time you left it unticked. Later you write, in **Agents › Home (this computer) › Codex**, a document that goes to that same file. Tisty cannot tell which part of the file is yours, so it does not write over it: it asks **It exists on both sides**, when you go into Agents or through **Review** in Settings › Agents › Agents folder. Since it never brought that file, the box adds «Tisty has never brought this file. Bring it into Agents and it stays as it is, as a document of this computer.» and the button **Bring it into Agents**.

With **Bring it into Agents**, the file comes in as a document in **Agents › Home (this computer) › Codex**, with its words exactly as they are. One document is one file, word for word: Tisty adds nothing to it. What it held before stays in its [backups](#backups).

**Bring another file** does the same: that `AGENTS.md` shows in its list, unticked, and once brought it ends up the same as the file.

### Codex's `AGENTS.override.md`

Codex reads `~/.codex/AGENTS.override.md` instead of `AGENTS.md` when it exists, and does not read both at once. For Tisty they are two files and two documents, each one with what its file says: nothing passes from one to the other. If you make the override after the first time, bring it with **Bring another file**: it lands in **Agents › Home (this computer) › Codex**, beside `AGENTS.md`. While the override exists, Codex reads only that one.

## How the folder looks

After bringing, your Library has an **Agents** folder with **Global**, one folder for each of your computers and, sometimes, a loose agent folder at the root. In the sidebar, each document shows with its name and an icon that says what it is:

```text
Agents
├── Global                      skills only: Tisty's and your own, for all agents
├── Codex                       only if you moved or made something here: it is for every computer
├── Home (this computer)
│   ├── Claude
│   │   ├── [book]     How I work
│   │   ├── [book]     Code style
│   │   ├── [piece]    pr                3 files
│   │   └── [brain]    MEMORY.md
│   └── Codex
│       └── [book]     Codex rules
└── Work                        another of your computers: faint
    └── Claude
```

- **One folder per computer.** Each computer reads and syncs only its own. What you bring (**Bring**, **Bring another file**, **Look again**) goes to this computer's folder, and each agent Tisty finds here has its subfolder in it, even an empty one. It is named as the computer is named in Sync, with «(this computer)» after it when it is the one you are on; the others go by name, last and faint. If two computers have the same name, a short code tells them apart.
- **The root is for all computers.** A loose agent folder in **Agents**, like **Codex** in the drawing, exists only if you moved something there or made something there; Tisty does not make it. With nothing inside, it is not shown. What having a document at the root means is told in [On several computers](#on-several-computers).
- **Global is for skills only.** Tisty's and your own, for all agents. [Global and your computer's folder](#global-and-your-computers-folder) tells it.
- **Each agent's icon**: **Global** has a blue globe, and each agent a symbol of its own, which does not copy its logo, in the same colour as its label on the card: an orange chat bubble for Claude, a teal terminal for Codex, indigo code braces for Copilot, a green compass for Gemini and a brown pointer for Cursor. The same icons show in the first-time list and in **Settings › Agents › Agents folder**. A subfolder you gave an icon of your own keeps it.
- **Each document's icon**: a book for a **rule**, a puzzle piece for a **skill** and a brain for **agent memory**.
- **The name**: the document's first heading, such as «How I work» in a `CLAUDE.md` that starts with `# How I work`. A skill takes the `name` in its `SKILL.md`. A script, or a file with no headings, is named after the file: `MEMORY.md`, `extract.py`.
- **The path**: hover over a row and it shows, with the agent in front: `claude · CLAUDE.md`, `claude · rules/style.md`.
- **A skill is one single row**, with how many files it holds on the right, such as **pr** and **3 files**. A click opens it and shows its files by their path inside the skill: `SKILL.md`, `scripts/open-pr.sh`. So a 200-file skill takes one line until you open it. A skill of a single file shows like any other document.

The subfolders are there so you find things, but **where a document sits does count**: the folder says which computer it belongs to (or that it belongs to all), and the `agent:` line says which file it writes. The two must agree, as [The `agent:` line](#the-agent-line) tells. Inside an agent's folder on your computer you can move a document between your own folders or make your own, and it keeps being written to the same file.

**One subfolder for each agent on this computer, even an empty one.** Tisty makes it when bringing, when you [look again](#look-again) or when a new agent shows up, so you have somewhere to write to it even before it has any file. Making a folder touches none of your agents' files. If you delete one of those subfolders, for example **Cursor** because you do not use it, Tisty does not make it again. And if two computers make it at the same time before they sync, only one remains.

**A computer with no agents** writes nothing with Agents: the agent documents your other computers have show there only as documents.

**The sidebar says nothing beside Agents**: no word, and no count of what waits. What waits for your decision is in three places:

| Where | What you see |
| :-- | :-- |
| **Each document's card** | Its standing, always, on the coloured label: **Changed outside**, **Changed on both sides**, **Written by an earlier Tisty** or **On hold**; the **Proposal** label, if an agent proposes a change to you; or a notice inside the card, when the document [comes from another computer](#on-several-computers), when [two documents go to the same file](#if-two-documents-go-to-the-same-file) or when its text [waits for your approval](#to-approve). The table of [each standing](#each-standing) says what to do with each one |
| **Settings › Agents › Agents folder**, on the **Agents** card | The files that ask for your decision: «1 file says something different in Tisty and in the agent.», or «1 file no longer belongs to any rule or skill in Agents.», each with **Review**; and under **Sync Gemini**, if [its line](#gemini-paused) is missing |
| **When you go into Agents** | If Agents syncs and a file exists on both sides, no longer belongs to any rule or skill or lost Gemini's line, Tisty asks about it as soon as you go in, one at a time |

![The sidebar with the Agents folder open and syncing, with the robot in coral: inside, Global with its blue globe and Claude with its orange chat bubble; in Claude, the rule Rules, picked, with a book icon, and below it the skills, each with a puzzle-piece icon and how many files it holds](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/en-02-sidebar.png)

### A subfolder's right-click

**Global**, your computer's folder, **Claude**, **Codex** and any folder inside Agents have a menu for agents only. On **Agents › Home (this computer) › Claude**, while Agents syncs, it offers this, and nothing else:

| Choice | What it does |
| :-- | :-- |
| **New document** | Makes a document in that folder that is born with its `agent:` line: in **Claude**, a Claude rule. In **Global** it makes a plain document, because Global writes no rules. See [A new document is already a rule](#a-new-document-is-already-a-rule) |
| **Bring another file** | Opens the list of your agents' files not yet in Agents. See [Bring another file](#bring-another-file) |
| **Pause sync with Claude** | Stops writing Claude's files on this computer, like the button of the same name on the card. While paused, it says **Resume sync with Claude**. See [Pausing, stopping sync and emptying](#pausing-stopping-sync-and-emptying) |
| **Hide from the sidebar** | Takes it out of the sidebar on this computer. If Agents syncs, it first asks you to stop syncing. See [Hiding it from the sidebar](#hiding-it-from-the-sidebar) |
| **Delete** | Asks first; its documents go to **Unfiled**, and Tisty asks you about the files they wrote, with their backup. See [Deleting a subfolder](#deleting-a-subfolder) |

- **Another computer's folder offers only Hide from the sidebar.** What is in it belongs to that computer: here it is read, moved and copied as text, but not written.
- **Global and your computer's folder have no pause**: they belong to no agent. To pause one, use that agent's subfolder.
- **The pause shows only while Agents syncs.** Brought but not synced, Tisty writes no file, and there is nothing to pause. Nor does it show for an agent not installed on this computer.
- **A folder inside a subfolder**, such as **Agents › Home (this computer) › Claude › Tests**, has the same menu as **Claude**, with Claude's pause.
- **There is no New diagram, New folder, Rename, Move to, Import or Unpack.** Those are your folders' actions; inside Agents, a folder's name is how Tisty knows whom it writes to.
- **An archived subfolder** also shows **Unarchive**, so it can come back.

**An example.** Jaime is trying new Claude rules by hand this afternoon, but wants Codex to keep getting its own. He right-clicks **Agents › Home (this computer) › Claude** and chooses **Pause sync with Claude**. Codex keeps syncing; Claude stays as the last write left it. When he is done, he right-clicks again and chooses **Resume sync with Claude**.

### The `agent:` line

Each Agents document starts with its properties, and one of them says which file it is:

```markdown
---
agent: [claude/rule/CLAUDE.md, on-dev_ab12cd34]
title: claude · CLAUDE.md
---
# How I work

- Answer in English.
- Branches take the form type/name.
```

The first value has three parts, separated by `/`:

- **The level**: `global`, or an agent: `claude`, `codex`, `copilot`, `gemini` or `cursor`.
- **The kind**: `rule`, `skill` or `memory`.
- **The name**: the file's path inside the agent's folder, such as `CLAUDE.md`, `rules/style.md` or `pr/SKILL.md`.

**A computer's document carries one more mark**: `on-` and that computer's code, as in `on-dev_ab12cd34`. Tisty puts it there, not you. A Global or root document does not carry it, because it belongs to all.

**The line and the folder must agree.** If they do not, for example because you restored an earlier version of the document or moved it with an older Tisty, no computer writes it: the card says «It sits in one computer's folder and its text says another's, so no computer writes it until you settle it.» and offers **Go by its folder**, which leaves the line as the folder says.

Sometimes it carries more marks, which Tisty puts in when bringing a file so it can write it just as it was: `crlf` (lines ending as on Windows), `bom` (an invisible mark at the start), `as-is` (mixed line endings) and `open-end` (the file does not end in a newline). For example, `agent: [copilot/rule/copilot-instructions.md, crlf, bom, on-dev_ab12cd34]`. The [properties guide](https://markdown.tisty.app/en/properties) explains the rest.

The page does not show that line: the document opens in its [card](#the-card-of-an-agent-document).

## Global and your computer's folder

**One document is one file, word for word.** Nothing is joined with anything else: what the document says is what the file your agent reads says. **Global** is for skills only, and rules belong to each agent, in your computer's folder.

### One document is one file

In **Agents › Home (this computer) › Claude**, write a document with these properties:

```markdown
---
agent: [claude/rule/CLAUDE.md, on-dev_ab12cd34]
title: claude · CLAUDE.md
---
# How I work

- Answer in English.
- Branches take the form type/name.
```

Tisty writes `~/.claude/CLAUDE.md` with those lines, and nothing more. If you change the document tomorrow, the file changes; if someone changes the file outside, the card tells you. The document's facts box confirms it: **Read by** Claude and **Written to** `~/.claude/CLAUDE.md`, with no «together with another document» anywhere.

Codex reads one rules file, `AGENTS.md`, and so does Gemini, `tisty-rules.md`. Cursor receives no rules: its own live in its settings, not in a file.

> [!IMPORTANT]
> **A Global rule from 1.49 or 1.50 stays there for you to read.** It is no longer written to any agent: you can read it, copy it into your agent's folder or delete it. If you want it to keep reaching an agent, copy its text into a document in that agent's folder.

### Adding rules to an agent

To add rules to an agent, make another file that agent reads, not more text in the same one:

- **Claude Code** reads every file in `~/.claude/rules`: write a document with `agent: claude/rule/rules/tests.md`, in your computer's **Claude** folder, and Tisty writes `~/.claude/rules/tests.md`.
- **Copilot** does the same with its `instructions` folder: `agent: copilot/rule/instructions/rust.instructions.md`.
- **Codex and Gemini** read one rules file, so what you want to add goes in that same document.

A name the agent does not read is written nowhere: `agent: claude/rule/notes.md` leaves the card saying «No agent on this computer reads this document yet.».

### A new document is already a rule

You do not have to write the `agent:` line by hand. A document you make with **New document** inside an agent's folder on your computer starts as a rule of that agent: its properties already carry its `agent:` line, and you only write the rule.

| Where you make it | What line it starts with |
| :-- | :-- |
| **Claude** | `agent: claude/rule/CLAUDE.md` while no document has it; after that, `claude/rule/rules/rule.md`, `rules/rule-2.md`… |
| **Codex** | `agent: codex/rule/AGENTS.md` while no document has it |
| **Copilot** | `agent: copilot/rule/copilot-instructions.md` while no document has it; after that, `copilot/rule/instructions/rule.instructions.md`, `rule-2.instructions.md`… |
| **Gemini** | `agent: gemini/rule/tisty-rules.md` while no document has it |
| **Global**, **Cursor**, Agents itself or a folder of your own inside Agents | None: it is a plain document |

In each case Tisty adds the computer's mark. **Codex and Gemini read one rules file.** When a document already has it, **New document** in their folder does not make another: it opens that same document, so you add what is new there. **Cursor** has no rules file of its own, so a new document in its folder also starts plain; you bring its skills with **Bring another file**.

**An example.** Marisol has Agents syncing and has just installed Gemini CLI. In the sidebar **Agents › Home (this computer) › Gemini** appears, with its green compass, empty.

1. Right-click **Gemini** and choose **New document**. The document opens on its card, with the **Rule** and **Gemini** labels, still with no text.
2. Press **Edit**, write «Answer in English.» and press **Done**.
3. Tisty writes `~/.gemini/tisty-rules.md` with that line and adds the line `@tisty-rules.md` at the end of `~/.gemini/GEMINI.md`, so Gemini reads it. The card moves to **In sync**, and in the sidebar the document is called `tisty-rules.md`, because it has no title; if you write `# Gemini rules` at the top, it is called that.

Its properties, which the card does not show, ended up like this:

```markdown
---
agent: [gemini/rule/tisty-rules.md, on-dev_ab12cd34]
title: gemini · tisty-rules.md
---
Answer in English.
```

If she presses **New document** on **Gemini** again later, that same document opens, and Marisol adds the new rule there.

### A skill in Global

A skill is a folder with a `SKILL.md` and, sometimes, scripts. Each file is a document, with the same level and the same skill folder:

```text
agent: global/skill/pdf/SKILL.md
agent: global/skill/pdf/scripts/extract.py
```

A Global skill is written once where several read it at a time:

- in `~/.claude/skills/pdf/`, for Claude Code;
- in `~/.cursor/skills/pdf/`, for Cursor;
- in `~/.agents/skills/pdf/`, one copy that Codex, Copilot, Gemini and DeepSeek Harness all read.

The `SKILL.md`'s own properties stay out of the text: its `name` is the card's name, its `description` sits under it in a quiet line, and the rest, such as `disable-model-invocation: true`, go in small rows of the facts box. The script is kept in a code block and written as is, without the block: the card's **Format** row names the script's kind, such as SH · LF, and a Markdown document says Markdown · LF.

**One agent's skill** goes in that agent's folder, and is written only to its skills folder.

### Tisty's skills in Global

Tisty brings three skills of its own, `tisty`, `consolidate` and `writing-for-the-next-agent`, that teach your agents to work with it. They are not Agents documents: they live inside the app and have their own yes, **Turn on here**, which needs no Agents folder. [[Hooks and skills|Hooks-and-Skills]] tells it whole.

Here you see them as a fixed row at the end of **Agents › Global**, **Tisty's skills**, with their standing: **Off** or **On**. The row shows only if this computer has some agent, if the skills are on, or if copies of them remain. A click opens their card, the same one as in **Settings › Agents › Connected**. What having Agents changes:

- **They are not brought in.** In the first-time list, in **Bring another file** and in **Look again**, their copies in your agents' folders show as **Tisty's own**, and do not come in as documents.
- **A skill of yours with the same name wins.** If you have your own `consolidate` in Agents, at any level, Tisty's is written to no agent, and its card says «You already have a skill named consolidate in Agents, so Tisty's is left out.».
- **Pausing does not reach them.** Pausing Claude or stopping sync for Agents holds your documents; Tisty's skills follow their own switch.
- **They hide, they are not deleted.** When you hover over the row, **Hide** appears and takes it out of Agents on this computer without turning them off. It has no menu and no **Delete**: **Show in Agents**, in Settings, brings it back.
- **The two yeses stand apart.** Stopping sync for Agents does not turn the skills off, and turning them on brings nothing into Agents.

**An example.** Lucas turned Tisty's skills on when he connected Claude Code, without ever opening Agents. A month later he clicks **Agents** and picks **Bring and sync**: in Claude's row, the `tisty` skill shows dimmed, as **Tisty's own**, and his own skills, like **pr**, with their box. Once done, **Agents › Global** shows **Tisty's skills · On this computer** at the end.

## Which file each agent reads

`~` is your user folder: `C:\Users\you` on Windows and `/Users/you` on macOS. Tisty only works with an agent whose folder exists on this computer.

| Agent | Its folder | Moved by | Rules Tisty writes | Skills | Memory, read only |
| :-- | :-- | :-- | :-- | :-- | :-- |
| **Claude Code** | `~/.claude` | `CLAUDE_CONFIG_DIR` | `CLAUDE.md` and what goes in `rules/` | `skills/` in its folder | `projects/*/memory/`, each project's memory |
| **Codex** | `~/.codex` | `CODEX_HOME` | `AGENTS.md`, and `AGENTS.override.md` if you have a document for it | `~/.agents/skills/` | — |
| **GitHub Copilot CLI** | `~/.copilot` | `COPILOT_HOME` | `copilot-instructions.md` and what goes in `instructions/` | `skills/` in its folder; Global's, in `~/.agents/skills/` | — |
| **Gemini CLI** | `~/.gemini` | `GEMINI_CLI_HOME`, the folder that holds `.gemini` | `tisty-rules.md`, and the line `@tisty-rules.md` in `GEMINI.md` | `skills/` in its folder; Global's, in `~/.agents/skills/` | `GEMINI.md` |
| **Cursor** | `~/.cursor` | — | None: its rules live in its settings | `skills/` in its folder | — |

What Tisty does with each one:

- **Claude Code.** It brings in `CLAUDE.md`, each file in `rules/`, each skill and, if you tick it, each project's memory: only the `memory` folder, never the saved conversations. It writes `CLAUDE.md`, `rules/` and the skills.
- **Codex.** Codex reads `AGENTS.override.md` instead of `AGENTS.md` when the first one exists, and each is its own document. If you make the override after the first time, [bring it in](#codexs-agentsoverridemd). Its skills go to `~/.agents/skills`, which `CODEX_HOME` does not move.
- **GitHub Copilot CLI.** It brings in and writes `copilot-instructions.md` and each file in `instructions/`, such as `rust.instructions.md`.
- **Gemini CLI.** Gemini writes its own `GEMINI.md` when you ask it to remember something, so that file is Gemini's: Tisty can bring it in as memory and never writes over it. Your rules go to a separate file, `~/.gemini/tisty-rules.md`, and Tisty adds **one single line** at the end of `GEMINI.md`, `@tisty-rules.md`, so Gemini reads it. It touches nothing else in that file, and if the line is already there it does not add it again. If you later delete it, Tisty does not put it back: Gemini stays [paused](#gemini-paused) until you decide.
- **Cursor.** Skills only, its own or Global ones.

An agent counts on each computer separately: Tisty writes an agent's files only on a computer where its folder exists.

**The variables that move a folder** are read from the environment Tisty started with. If you use `CLAUDE_CONFIG_DIR` to keep Claude Code somewhere else, set it as a user variable and open Tisty again, so it sees it.

## The card of an agent document

A document in Agents does not look like one more document. It opens **for reading**, on a single card: at the top, what it is and where it goes; in the middle, the agent's text; at the foot, what you can do. There is no column beside it: everything useful is on the card.

![The card of the rule Rules in Agents / Claude: the labels Rule, Claude and In sync; the facts box with Read by Claude, Written to with its copy button, Size, Backups with See, Format Markdown · LF, Last written, Original kept and Syncs on this computer Yes; the rule's text below it, and at the foot Edit, Backups, History, Copy link and Pause sync with Claude](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/app/en-agents-rule-light.webp)

**At the very top, the folder path**: «Agents / Home (this computer) / Claude». A click on a folder opens it.

**The heading:**

- **The labels**: the kind (**Rule**, **Skill** or **Agent memory**), the level (**Global** or the agent) and the standing, in its colour.
- **The name**, the same as in the sidebar. On a skill, its `description` sits under it, in a quiet line.

**The file's facts box**, the only place they show:

| Row | What it says | Example |
| :-- | :-- | :-- |
| **Read by** | Which agents on this computer read the file | Claude, Codex |
| **Written to** | The file's full path, with a button to copy it. A Global skill has one per agent: **Claude file**, **Codex file**. It is always one document per file: nothing is written «together with another» | `~/.claude/CLAUDE.md` |
| **Size** | How much the file the agent reads weighs | 12.7 kB |
| **Backups** | How many this computer keeps, and **See** to open them | 3 backups · See, or None yet |
| **Format** | Markdown, or the script's kind; how its lines end, and the marks it keeps | Markdown · LF, or PY · CRLF · BOM |
| **Last written** | When the file last changed | today, 19:37 |
| **Original kept** | Whether backup #1 is there, the file as it was before Tisty touched it | Yes, before Tisty's first write |
| **Syncs on this computer** | Whether Tisty keeps it up to date here | Yes; Yes, except Codex (paused); Tisty only reads it; No, Agents does not sync; No: it waits for «Apply here»; No: its text waits for your approval; No: another computer's; Waits for you to choose, or No: another one rules here, when [two documents go to the same file](#if-two-documents-go-to-the-same-file); No: its folder and its text say different computers. With a proposal waiting, it adds «an agent's change waits for your approval» |
| **Shared** | Only when the document has a live shared link: whether it can be read or edited, and **See** shows the links with their copy button and the usual actions, such as **Stop sharing**. See [Sharing](Documents#sharing) | Read only · See |
| **The file's properties** | The other keys of its heading, in small key and value rows. When there are too many, or they are too long to read as fields, they show as written | `disable-model-invocation` true; `applyTo` `**/*.rs` |

**Codex's cap.** Codex reads its `AGENTS.md` up to 32 KiB and leaves the rest unread. When the file nears that cap, the **Size** row says so in amber: «29 kB · close to the 32 KiB Codex reads». Past it, it says so in red: «33.8 kB · past the 32 KiB Codex reads: it leaves the rest unread». It is a good moment to move long rules into a skill.

**The agent's text**, inside the same card, with no separate sheet. It is what you wrote, as is, with no sections or template imposed: Tisty draws it with compact headings, across the whole width of the card, and never turns its first line into a title. The file's heading is not repeated there, since the facts box already has it. What your agent reads does not change: the file stays the same, byte for byte.

**The buttons, at the foot**, held at the bottom while the whole card scrolls with its text:

- **Edit** opens Tisty's editor on the text, heading included, with the editor's usual bar for headings and lists, and **Done** closes it again.
- **Backups**, when the document is written to a single file and that file has any. It opens the same list as **See** in the facts box. A **Global** skill, written to several files, does not have it at the foot: each file's backups open with **See**, on its row.
- **History**: the document's earlier versions, not the file's. [[Earlier versions|Documents#earlier-versions]] tells it. The comparison with what is **Approved** reads from there: if you empty the history, or the approved version passes the limit, the card asks you to read the whole text before approving.
- **Copy link** copies its [link](Documents#a-documents-link).
- **Pause sync with Claude**, when the document is written to a single file. A Global skill has a button per agent: **Pause Claude**, **Pause Codex**…
- **Another computer's documents have no Edit, no Apply, no Delete and no pause**: the card says «It belongs to another of your computers: on this one it is only read.». What you can do is read it, move it, copy it as text and take it out. Words another computer approved for this one wait here each time, as [On several computers](#other-computers-documents) tells.

A memory document has no **Edit**: the agent writes it, and the card says «The agent writes this file itself: Tisty shows it and brings it in, and never writes over it.».

**An example.** You open Codex's **Working hours** rule, which goes into `~/.codex/AGENTS.md`. At the top you see «Agents / Home (this computer) / Codex» and the labels **Rule**, **Codex** and **In sync**. In the box, «Written to `~/.codex/AGENTS.md`», and «Size 29 kB · close to the 32 KiB Codex reads» in amber. You read the text, decide to move the long part into a skill, and press **Edit** at the foot.

### Each standing

| Standing | What it means | What to do |
| :-- | :-- | :-- |
| **In sync** | The file says what the document says | Nothing |
| **About to be written** | You changed the document and Tisty writes it in a moment | Nothing |
| **Changed outside** | Someone changed the file outside Tisty, or deleted it | Choose: [bring or overwrite](#a-change-made-outside-bring-it-in-or-overwrite) |
| **Changed on both sides** | Both the file and the document changed since the last write | Choose which to keep |
| **Written by an earlier Tisty** | An earlier version of Tisty wrote the file without noting which documents it came from, and today it does not say what they say | **See the file**, then **Overwrite with Tisty** if you keep the document |
| **On hold** | This document's text [changed without your approval](#to-approve), or the document left Agents by an agent's hand, so the file stays as it was. The card names it | Approve the document, or put it back and take it out yourself |
| **Paused** | You paused sync for that agent, or Agents does not sync on this computer; then the card adds «The Agents folder does not sync with your agents. Choose Sync in the sidebar or in Settings › Agents › Agents folder.». For Gemini, also when the line `@tisty-rules.md` that Tisty had put in `GEMINI.md` is missing | **Resume** whenever you like, or **Sync** the folder; for Gemini, [**Put the line back**](#gemini-paused) |
| **Not allowed on this computer** | This computer has not let you read your agents' files yet, for example because Agents came in brought from another computer | **Sync**, in the sidebar or in Settings › Agents › Agents folder, and choose in the first-time box |
| **Nothing to write** | The document is empty, and Tisty never empties an agent's file | Write something, or take the document out of Agents |
| **The file is a link** | The file, or a folder on its way, is a symbolic link; writing it would replace the link | Swap the link for the real file |
| **The code block is broken** | The block that holds a script is no longer whole | Mend the block in the document |
| **Cannot be read** | The file exists, but Tisty could not read it | Check its permissions |

Two more notices can show under the facts box:

- **«No agent on this computer reads this document yet.»** The agent is not on this computer, or the name is not a file that agent reads.
- **«On this computer another document writes this file.»** Two documents, the one in your folder and one at the root, go to the same file: only one is written, and the other's card says so.

And if the document is not applied on this computer yet, a notice says why, with the button that settles it: **Apply here** and **Not on this computer** if it waits for your yes; **This one** and **The other** if [two documents go to the same file](#if-two-documents-go-to-the-same-file); **Go by its folder** if its folder and its `agent:` line do not agree. [On several computers](#on-several-computers) tells it all.

## A change made outside: bring it in or overwrite

You can keep editing your agents' files in your editor, or let the agent change them. Tisty never writes over those changes. This happens while syncing: brought but not synced, Tisty neither writes those files nor asks you to decide anything, and what you change there stays there.

**An example.** You open `~/.claude/CLAUDE.md` in your editor and add «- Run the tests before every commit.». If Agents syncs, the next time Tisty looks (on start, when you come back to the window, or when you open the document), the card of **How I work**, the document of that `CLAUDE.md`, says **Changed outside**, with a notice:

- **See the file** shows what the file holds now.
- **Bring the change in** writes into the document what the file says. What the document said before stays in its [earlier versions](Documents#earlier-versions). If someone changed the document after the card was drawn, Tisty does not write over it: it asks you to read it again and choose again.
- **Overwrite with Tisty** asks «Write this document over `~/.claude/CLAUDE.md`? What the file holds now is kept as a backup.» and, if you say yes, writes the file again from the document.

Until you choose, Tisty does not write that file, even if you change the document.

**Bring the change in** shows when the document belongs to a single agent, not to Global, and you are not editing it: since a document is the whole file, what changed in the file is what passes into the document. If you are editing, press **Done** first.

**If the file was deleted**, the card says **Changed outside** without **See the file**: **Overwrite with Tisty** makes it again.

### Changed on both sides

If both the file and the document changed, the card says **Changed on both sides**. **Keep the file's version** shows, which asks «Make this document read as … does now? What it says now stays in its history.», next to **Overwrite with Tisty**.

The same can happen the first time you sync, when you already had a document in Agents for a file that changed in between. Then **It exists on both sides** opens, with the lines that differ: in red, with `−`, what is **In Tisty**; in green, with `+`, what is **In the file**. You choose **Keep Tisty's**, **Keep the file** or **Decide later**, and what you do not keep is still kept: the file in its [backups](#backups), the document in its earlier versions. If Tisty never brought that file, the box adds **Bring it into Agents**, as [A file you never brought](#a-file-you-never-brought) tells. From **Settings › Agents › Agents folder**, «1 file says something different in Tisty and in the agent.» and **Review** bring you back to that box.

## Backups

**Before writing over an agent's file, Tisty keeps what it held.** Backups live on this computer, in `data/agents/` inside Tisty's folder, never in your agents' folders, and they do not travel with sync or parcels.

- **Backup #1 is the original**: the file as it was before Tisty first wrote over it. That one is never removed.
- **Then, the last 20.** When the 21st comes, the oldest one after the original goes.
- **Taking a file away leaves one too.** If you [take away the file](#taking-a-rule-and-its-file-out) of a rule you took out, its backups stay on this computer, and if a rule writes that same path again, they show in its **Backups**.

So a file Tisty wrote over many times keeps up to 21: the original and the last 20. The box says «The last 20 are kept.», and #1 stays on top of them.

To see them, press **Backups** at the foot of the card, or **See** on the **Backups** row of the facts box, beside «3 backups»:

- **See** shows the whole backup.
- **Restore** asks «Put this backup back in …? What the file holds now is kept as a backup.» and writes it into the file.

**A restored backup does not change your document.** The file says again what the backup said, and the card turns to **Changed outside**: you choose **Bring the change in**, so the document says the same, or **Overwrite with Tisty**, if you changed your mind. That way you never lose either side.

The **document's** earlier versions, on the other hand, are those of any document: they are in its [history](Documents#earlier-versions).

## Pausing, stopping sync and emptying

**Pausing one agent.** While Agents syncs, the usual place is **Settings › Agents › Agents folder**: on the **Agents** card, turn off the **Sync Claude** switch, which shows only while the folder syncs. Also in the sidebar: right-click **Agents › Home (this computer) › Claude** and choose **Pause sync with Claude**; while paused, the same menu says **Resume sync with Claude**. Also from a document's card, with the buttons at its foot: **Pause sync with Claude**, on a document written to a single file, or one button per agent, **Pause Claude**, **Pause Codex**…, on a Global skill. Paused, Tisty neither opens nor writes that agent's files: they stay as the last write left them, and you keep editing your documents. **Resume sync with Claude** writes what you changed, except a file that changed outside in the meantime, which stays marked for you to decide.

**Stopping sync for the whole folder.** Right-click **Agents** and choose **Stop syncing**, or the **Stop syncing** button on its card in **Settings › Agents › Agents folder**. This happens, and nothing more:

- **No agent file is touched.** Your agents keep reading their files just as Tisty left them, and Tisty stops writing them and asking you to decide when they change outside.
- **No document is deleted.** They stay in Agents, and you read and edit them as always; what you change stays in Tisty.
- **Agents goes to state 2.** Its robot turns grey again and, when you open it, «Not synced with your agents · Sync» shows at the top. Each document's card says «The Agents folder does not sync with your agents. Choose Sync in the sidebar or in Settings › Agents › Agents folder.», and its facts box, under **Syncs on this computer**, «No, Agents does not sync».

**Syncing again.** With Agents in state 2, any of these three does the same: the **Sync** link in the line at the top, **Sync** on its right-click, or **Sync** on its card in Settings › Agents › Agents folder. If this computer had already let you read your agents' files, for example because you chose **Bring** or **Stop syncing** right here, it syncs at once, without the first-time box. If it never did, like a [second computer](#a-second-computer-step-by-step), the first-time box opens first.

When it syncs, Tisty writes what you changed in your documents in the meantime, with its backup first, except a file that changed outside, which stays marked for you to decide. What it finds different on both sides, or no longer belonging to any rule or skill, it asks about right then.

**The first time you sync something you had only brought**, a file still exactly as you brought it is taken as it stands: nothing is asked about it. A file that changed since you brought it, on the other hand, is not written over: **It exists on both sides** opens and you choose.

**An example.** Lucas wants to try some new rules by hand in his `CLAUDE.md` for a week, without Tisty touching them. He chooses **Stop syncing**. He edits the file in his editor and, meanwhile, keeps arranging his documents in Agents. At the end of the week he presses **Sync**: the card of **How I work** says **Changed outside**, and Lucas chooses **Bring the change in** to keep what he tried, or **Overwrite with Tisty** to go back to what his documents say.

Pausing, syncing and the permission belong to this computer: stopping sync here changes nothing on your other computers.

### Gemini paused

While syncing, Tisty puts the line `@tisty-rules.md` in your `GEMINI.md` once. If you later delete it, or delete `GEMINI.md` altogether, Tisty takes that as your decision and does not put it back: Gemini stays **Paused**, stops getting your rules, and Tisty stops writing `tisty-rules.md`. Only a line Tisty put there counts: if it never put it there, for example because you had no rules for Gemini yet, its absence does not pause Gemini. You see it in three places:

- **On the card** of each rule Gemini reads: the **Paused** label and «Gemini does not read this rule: the line @tisty-rules.md is missing from GEMINI.md, and Tisty does not put it back by itself.», with the **Put the line back** button.
- **In Settings › Agents › Agents folder**, under **Sync Gemini**: «Paused: the line @tisty-rules.md is missing from GEMINI.md.», with **Put the line back**.
- **When you go into Agents**, the box **Tisty's line is missing**: «The line @tisty-rules.md is no longer in GEMINI.md, so Gemini does not read your rules. Tisty does not put it back by itself: Gemini stays paused until you do.», with **Leave it paused** and **Put the line back**.

**Put the line back** adds the line again at the end of `GEMINI.md`, touching nothing else, and Tisty writes your rules to `tisty-rules.md` again. Until you do, the card and the Settings card keep saying so.

### Deleting a rule or skill

When you delete a rule or skill of yours from Agents, with **Delete** in its menu, and Tisty wrote it into an agent's folder, Tisty asks right there whether to take it away from your agents too. It is one box: on top it says what the usual question says (what goes with the document, its languages and its pages, and that it cannot be undone), and below it names every agent whose copy can go:

- **Take away too** deletes the document and takes each of those files out of your agents' folders. A backup of each file stays on this computer first.
- **Only from Tisty** deletes the document and leaves those files where they are, and your agents keep reading them.
- **Cancel** deletes nothing.

**An example.** Yisset wrote the Global skill **voice** in Agents, and Tisty put it in `~/.claude/skills/voice/SKILL.md` and in `~/.agents/skills/voice/SKILL.md`, which Codex and Copilot read. She no longer wants it and chooses **Delete**. The box says «Delete «voice»? The document and what it holds go for good. This cannot be undone.» and, in bold, «Take it away from Claude, Codex and Copilot too?». She chooses **Take away too**: both files go, each with its backup, and none of her agents reads that skill again. Only the files the box named go: anything that came loose some other way while it was open waits for its own question.

A document of another computer is not deleted from this one: here it is read, moved and taken out, and deleting it is that computer's business.

Some copies the box says apart, and they never go without you seeing them:

- **A copy changed outside Tisty**, for example one you edited by hand: «Changed outside Tisty in Codex and Copilot: that copy stays, and you see it after, on its own.». After deleting, the [Take the file away too?](#taking-a-rule-and-its-file-out) box opens for it, with its path and «It changed after Tisty last wrote it.». If it changes between the question and your answer, it stays too. **Only from Tisty** does not mark it as left: it stays for that question.
- **A copy a paused agent reads**: «Paused: Copilot. Its copy stays, and Tisty asks again once it syncs.». That holds for any agent that reads it, also one that only shares `~/.agents/skills` with another, and for Gemini while the line `@tisty-rules.md` is missing from its `GEMINI.md`. Tisty does not touch it. When the agent syncs again, that copy counts as a loose file: the **Agents** card in Settings says so with **Review**, and Tisty asks the usual question the next time you go into Agents.

When every copy is changed or paused, the box still shows, with **Only from Tisty** and **Cancel**. When nothing Tisty wrote comes loose (the document was only brought and never synced, Agents never synced on this computer, or every file is also written by another document), the usual «Delete «voice»?» is the whole question. If you stopped syncing, the box names each agent that reads a copy as paused, with **Only from Tisty** and **Cancel**. Deleting several documents at once asks one box for all of them, with the same list the usual question shows and every agent named together.

If a document cannot be deleted, for example because it is locked, none of its files is touched, and Tisty says why. If a file cannot be taken away, for example because its folder is a link, the document is still deleted and Tisty names that file and the reason; it stays for the usual question.

#### Files you left with your agents

What you leave with **Only from Tisty** stays yours: Tisty takes it as your file. The **Agents** card in **Settings › Agents › Agents folder** lists them apart: «2 files you left with your agents when you deleted them from Agents.», with **Review**, which asks about each one with [Take the file away too?](#taking-a-rule-and-its-file-out). Tisty never asks about them by itself and never writes over them, and a folder of Agents you empty still goes. If you later write a rule or skill that lands on the same file again, Tisty asks first, as with [a change made outside](#a-change-made-outside-bring-it-in-or-overwrite), with **Keep Tisty's** (backup first), or **Keep the file** when the document belongs to an agent on this computer. If you bring that same file back with [Bring another file](#bring-another-file), it is Tisty's again and leaves that list. If the document you deleted was the last one in Agents, the folder goes, but this list stays in the card with **Review**, so you can still change your mind; nothing else Agents wrote is offered there. This list is this computer's only: it does not travel to your other computers.

### Taking a rule and its file out

While syncing, Tisty stops writing a document when you take it out of Agents: when you archive it or move it out, when you change its `agent:` line to another file's, when you empty all its words, when you move it to another computer's folder, and when you go back to an [earlier version](Documents#earlier-versions) that had no card, or another one. Deleting it asks [right then](#deleting-a-rule-or-skill); this question is for every other way, and for the copies that question leaves for later.

Since a document is a whole file, no rule or skill writes that file any more, but it is still on disk and your agent keeps reading it. So about a second later, Tisty asks what to do with it.

**Emptying a document.** Leaving a rule with no words at all is the same as taking it out: Tisty never empties an agent's file, so instead it asks about it with **Take the file away too?**, as if you had taken it out.

**Moving it to another computer.** A document you move to another of your computers' folder stops being applied here, and Tisty asks about its file on this computer with the same question. If an agent's proposal is waiting on it, it will not move: the card says «An agent's proposal waits on this document. Approve or discard it before moving it to another computer». What you move to your computer's folder, or to the root, counts as applying it here; on the other computers it waits for its **Apply here**.

**An example.** You had the rule **Tests**, from `~/.claude/rules/tests.md`, and moved it out of Agents to **Notes**. **Take the file away too?** opens: «No rule or skill in Agents writes this Claude file any more, but it is still on disk and Claude keeps reading it.», with the path below and «If you take it away, a backup stays on this computer.».

- **Take the file away** keeps a backup and deletes it: Claude stops reading that rule. If it was the last file in a skill's folder, that folder goes with it; the agent's own folders, such as `rules`, stay.
- **Leave it** leaves it where it is, and Tisty stops answering for it: it neither writes it nor asks again. If one day you want it back in Agents, it shows in [Bring another file](#bring-another-file).

If the file changed after Tisty wrote it, for example because you edited it by hand, the box adds «It changed after Tisty last wrote it.». The backup keeps the file as it is now, with that change.

If you take away `tisty-rules.md`, Gemini's rules file, Tisty also takes the line `@tisty-rules.md` out of `GEMINI.md`, and nothing else.

Until you answer, the **Agents** card in **Settings › Agents › Agents folder** says «1 file no longer belongs to any rule or skill in Agents.», with **Review** to come back to the question.

Tisty does not ask, and leaves the file as it is:

- about agent memory, which the agent writes itself: neither each Claude Code project's memory nor Gemini's `GEMINI.md`;
- about a file of an agent you paused, nor about any while Agents does not sync: brought but not synced, taking a document out touches no file;
- about a file you only brought and Tisty never wrote: **Bring** does not note it as written by Tisty, so offering to take it away is not Tisty's to do;
- about a file whose document [waits for your approval](#to-approve);
- about a file that no longer exists.

Your other computers do the same: when the deleted rule reaches them through sync, each one asks about its own file with this question the next time Tisty looks, and the backup stays on that computer. A document deleted on another computer never takes away a file on this one until you answer here.

### Deleting a subfolder

Right-click a subfolder of Agents, such as **Agents › Home (this computer) › Claude › Tests**, and choose **Delete**. Tisty first asks «Delete «Tests»? Its documents go to Unfiled and its subfolders move up a level. Nothing is lost, but this cannot be undone.». With your yes:

1. Its documents go to **Unfiled**. As they leave Agents, Tisty stops writing them.
2. About a second later, Tisty asks you about each file they made up, just as in [Taking a rule and its file out](#taking-a-rule-and-its-file-out): **Take the file away** keeps a backup on this computer and deletes it; **Leave it** leaves it where it is.
3. The folder does not come back by itself: Tisty never makes again a folder of Agents you deleted.

As when you take a rule out, it does not ask about a paused agent, nor while Agents does not sync: Tisty was not writing any file then.

**An example.** Marisol kept her testing rules in a folder of her own, **Agents › Home (this computer) › Claude › Tests**, and no longer wants them. She right-clicks **Tests** and chooses **Delete**; a second later, Tisty asks **Take the file away too?** about `~/.claude/rules/tests.md`. She chooses **Take the file away**: the backup stays on this computer, Claude stops reading those rules, and its documents stay in **Unfiled**, in case she wants them back one day.

### Emptying Agents

If you take every document out of Agents while Tisty is open and the folder syncs, it first asks about the files its last rules wrote, one by one, as in [Taking a rule and its file out](#taking-a-rule-and-its-file-out). Once you answer the last one, the Agents folder and its subfolders leave Tisty, and the files you chose to leave stay as they were. Brought but not synced, there is nothing to ask: Tisty wrote no file, and the folder goes as soon as the last document leaves.

Only an Agents that held documents and that you emptied with the window open goes. An Agents empty some other way stays: one made with **Bring and sync** without finding any files, or one that was already empty when Tisty opened.

Emptied, Agents goes back to the sidebar as at the start, in state 1: the robot and the name in grey. If you want it again, a click on it opens the first-time box again, with the rules and skills it finds already ticked, the ones you left included, and memory unticked, as always.

### Hiding it from the sidebar

If you do not use Agents and do not want to see it in the sidebar, not even in grey, hide it. **It is never hidden and syncing**, so it never writes files you do not see.

1. Right-click Agents and choose **Hide from the sidebar**, in any of its three states. **Hide** on the **Agents** card in Settings › Agents › Agents folder works too.
2. If Agents syncs, it first warns: «To hide Agents, first stop syncing. Your agent files are not touched.», with **Stop syncing and hide** and **Cancel**.
3. **Stop syncing and hide** does both, in that order: Agents goes to state 2 and leaves the sidebar. **Cancel** leaves it as it was, in sight and syncing. If Tisty cannot tell whether Agents syncs, it warns all the same.

Agents leaves the sidebar on this computer without a trace, and sits in **Settings › Look and window › Hidden folders**, beside any other folder you hid: see [Hiding a folder from the sidebar](Documents#hiding-a-folder-from-the-sidebar). Its card in Settings › Agents › Agents folder says «Hidden from the sidebar on this computer, and not synced: it writes no agent file.», with **Show again**.

- **Show again**, in Hidden folders or on the card, puts it back in the sidebar in the state it had: in grey, nothing brought or brought but not synced.
- **Sync**, on the card of a brought and hidden Agents, syncs it and shows it again too: while syncing, Agents always shows.

**Hidden, Agents does not sync.** It writes none of your agents' files, and nothing from those files waits for your decision. Your documents are still there: search and **Move to** find them, and if an agent proposes a change, the proposal stays on its document's card until you approve or discard it.

**If in the earlier version you had hidden it while it wrote your agents' files**, you now find it in the sidebar again, syncing. From there, if you would rather not see it, **Stop syncing** and **Hide from the sidebar**.

**Its subfolders follow the same rule.** **Hide from the sidebar** is always on the right-click of **Global**, a computer's folder, **Claude** or any folder inside Agents; on another computer's folder it is the only thing offered. While Agents syncs, it goes through the same warning, and **Stop syncing and hide** stops syncing the whole of Agents before hiding that subfolder: what it holds should not keep being written out of your sight. And if a hidden subfolder starts syncing again, for example with **Sync** on the card, it comes back to the sidebar by itself.

### Agents stays at the top

Tisty knows the folder because it sits at the top, on the first level of your Library and not inside another folder, carries its robot icon and keeps the name `AGENTS`. In the window you see it as **Agents**, or **Agentes** with Tisty in Spanish, but inside it is still called `AGENTS`: that way earlier versions of Tisty on your other computers and the agents that read it through MCP find it.

That is why Agents cannot be put inside another folder, renamed, given another icon or archived, and while it holds anything, it cannot be deleted either. That holds whether you try it from the window, an agent asks for it through MCP or it arrives from another of your computers: Tisty does not do it, and your rules keep reaching your agents as before. If you try it from the window, for example by dragging it into **Work**, Tisty tells you «Agents stays at the top, with its name and its robot, so it keeps guarding your agents' rules. Its color and its place among your folders are yours to change».

What is yours: its colour and its order among your folders. To make it stop writing your agents' files, the way is **Stop syncing**; to stop seeing it, **Hide from the sidebar**; and to make it go, [empty it](#emptying-agents).

## The Agents card in Settings › Agents › Agents folder

The **Agents** card uses the same words and does the same as the sidebar. At the top, always, «Your agents' rules, skills and memory as Tisty documents. Bring copies them here; Sync also writes what you change to the file each agent reads.». Below, by state:

| State | What it says | Its buttons |
| :-- | :-- | :-- |
| **1. Nothing brought yet** | Which agents Tisty saw, such as «I found Claude and Codex on this computer.», or «I found no agent on this computer.», and «I only checked whether their folders exist; I read nothing inside them.» | **Bring** and **Bring and sync**, which both open the first-time box, and **Hide** |
| **2. Brought, not synced** | «Not synced with your agents» | **Sync**; **Bring another file**, once this computer has let you read your agents' files; and **Hide** |
| **3. Syncing** | What waits for your decision, such as «1 file says something different in Tisty and in the agent.», with **Review** | **Bring another file**, **Stop syncing** and **Hide**, and below them one switch per agent: **Sync Claude**, **Sync Codex**… |

With Agents syncing and one agent not synced, its switch says «Not synced: its files stay as they are, and nothing in Tisty is deleted.». An agent that syncs but has no file yet says so under its switch: «Gemini: no rules, skills or memory yet. It gets what you write in Global or in its folder.». If Tisty finds no agent on this computer, the card says «No agents found on this computer.». **Hide** shows in all three states while it is not hidden already; while syncing, it first asks you to stop syncing, with the same warning as the sidebar.

## If you come from the earlier version

In the earlier version, the folder showed in capitals in the sidebar and, while it did not write your agents' files, looked dimmed, in grey, with a word beside it. When you update you have nothing to do:

| How you saw it | How you find it |
| :-- | :-- |
| In its colour, writing your agents' files | In state 3, **syncing**, with its coral robot. It keeps writing the same files |
| Dimmed, with documents | In state 2, **brought, not synced**. None of your agents' files changed |
| Dimmed, with no documents and no folder | In state 1, **nothing brought yet**, in grey |
| Hidden, writing no files | Still hidden, in Settings › Look and window › Hidden folders |
| Hidden while it wrote files | Back in the sidebar, syncing |

From now on, the first step is **Bring** or **Bring and sync**, even when you tick nothing; with the folder already brought, **Sync**; and to stop it, **Stop syncing**, on its right-click or on its card. Your documents, their earlier versions and each file's backups stay as they were.

## On several computers

If you use Tisty on more than one computer, say the home one and the work one, your Agents documents travel with your sync like any other. But **each computer has its own folder and writes only its own**: the home one syncing does not make the work one sync, and what is in another computer's folder you read from here but do not write from here.

### Each computer, its own folder

```text
Agents
├── Global                      skills only
├── Codex                       only if you moved or made something here: it is for every computer
├── Home (this computer)
│   ├── Claude
│   └── Codex
└── Work                        another computer: faint
    └── Claude
```

- **Your folder** carries the name your computer has in Sync, with «(this computer)» after it. Tisty reads and syncs only that one: what you bring, and each agent it finds here, goes to that folder.
- **The other computers' folders** go last, faint, by name. If two computers have the same name, a short code tells them apart. There you read, move, copy as text and take out; you do not edit, apply, delete or pause.
- **The root is for all computers.** A document sits there only if you move it there or make it there: right-click the document, **Move to**, and **Agents** or the loose folder, such as **Codex**. It is for when you want the same rule on all your computers. On each computer it still waits for its **Apply here**, as the step by step tells.
- **Moving applies or stops applying.** Moving something to your computer's folder or to the root counts as applying it on this computer; on the others it waits for its **Apply here**. Moving a document to another computer's folder stops applying it here, and Tisty asks about its file, as [Taking a rule and its file out](#taking-a-rule-and-its-file-out) tells. A document with an agent's proposal waiting cannot be moved to another computer until you approve or discard it, and neither can a folder holding one.
- **Tisty asks before a move that changes whose it is.** When you move a document or a folder between the root, your computer's folder and another computer's folder, by dragging it or with **Move to**, a question first says what changes, and nothing moves until you press **Move**. **Cancel** leaves everything as it was. A move inside the same folder, or one that stays on the same computer, asks nothing.

**An example.** Yisset moves **Branches**, her Claude rule at the root, to **Home (this computer) › Claude**, because at work branches go another way. Tisty asks:

```text
Move «Branches» to this computer's folder?

On this computer, it keeps being written to CLAUDE.md.
Your other computers keep their file as it is, as a document of their own.
```

Each line says one thing that will happen:

| The line | What it means |
| :-- | :-- |
| **Tisty writes it to …** | This computer starts writing that file, with a backup first |
| **It keeps being written to …** | It was already written here, and it still is |
| **Another document also writes …** | Something at the root or in your folder writes the same file: Tisty asks which one rules here, as [If two documents go to the same file](#if-two-documents-go-to-the-same-file) tells |
| **Tisty stops writing … and asks** | It leaves this computer, and Tisty asks what to do with the file, with a backup |
| **Its words are not approved yet** | No computer writes it until you approve them |
| **No agent reads it yet** | Its agent is not on this computer, or its sync is paused here: nothing is written |
| **On your other computers, it waits for Apply here** | It goes to the root: each of your other computers chooses |
| **On …, an earlier Tisty** | That computer reads it as a document for all your computers until it updates to 1.50.2 |

When you move a folder, the question counts its documents and gives one line for each that changes something on this computer.

### A second computer, step by step

**An example.** On the home computer, Mario chose **Bring and sync**: his `CLAUDE.md`, **How I work**, sits in **Agents › Home (this computer) › Claude**, and he wrote another rule there, **Code style**. Now he installs Tisty on the work computer and syncs:

1. In the sidebar, Agents arrives in **state 2**, brought but not synced. He sees **Home** faint, at the end, with its documents: they belong to the other computer, and he can read them right away. At the top it says «Not synced with your agents · Sync». No file of the work computer's agents has changed.
2. He presses **Sync**, on that line, in the right-click or on the **Agents** card in **Settings › Agents › Agents folder**. Since this computer never let you read its agents' files, the first-time box opens first, with this computer's files. Those that already have a document say **already in Agents**.
3. Tick what you want to bring from this computer, such as its projects' memory, and press **Bring and sync**. **Work (this computer)** is made, with a folder for each agent it found, and Agents goes to state 3, with its coral robot. If you choose **Bring** instead, what you ticked comes in and this computer still does not sync.
4. **How I work** and **Code style** stay in **Home**: on the work computer they are written to no file. Their card says «It belongs to another of your computers: on this one it is only read.», with no **Edit**, **Apply here** or **Delete**.
5. If Mario wants **Code style** on the work computer too, he copies it as text into his work agent's folder, or moves it to the root: right-click the document, **Move to** and **Agents**. Moving to the root counts as applying it on the computer where he does it; on the other, the card shows an amber notice, «This document comes from another computer. Apply it to the agents on this one?», with two buttons:
   - **Apply here** writes it to the files of this computer's agents, with its backup first, as always. If a file on this computer already said something else, Tisty does not write over it: the card stands **Changed on both sides** and you choose.
   - **Not on this computer** leaves it out here. The card says «You chose not to apply it on this computer.», and **Apply here** stays by hand in case you change your mind.

What you choose holds for that document on that computer, and stays:

- **A document you already apply updates itself.** If tomorrow you change the root document at home, the work computer writes it as soon as it arrives, without asking again.
- **What you write or bring on a computer is applied on that computer with no click.** Only what reaches the root from another waits.
- **Only what this computer would read waits.** A Codex document asks nothing on a computer with no Codex.
- **Once applied, the card does not offer to undo it.** For an agent on this computer to stop getting it, pause that agent here, or stop syncing Agents on this computer; for no computer to get it, take it out of Agents.

### If two documents go to the same file

It happens when the document in your folder and one at the root write the same file, or the same skill. Until you choose, nothing changes on this computer: neither of the two is written. The card asks «On this computer «X» already goes to the same file. Which one rules here?» and offers two buttons, **This one** and **The other**.

**An example.** Marisol has her `CLAUDE.md` in **Home (this computer) › Claude**, and from home she moves to the root another document that also goes to `CLAUDE.md`. When it arrives, the work computer cannot tell which one should rule. Marisol opens either and presses **This one** on the one she wants: that one is written to `~/.claude/CLAUDE.md`, and the other stays in its folder, with the card «On this computer … rules.».

- **A skill is chosen whole**, with all its files.
- **It is chosen per file.** If you choose for Claude, a Global skill keeps working in the other agents.
- **You can change your mind**: the one that does not rule stays in its folder, and **This one** on its card makes it rule.
- **If the one that ruled goes away**, Tisty does not write the other by itself: it asks again.

### Other computers' documents

Those in another computer's folder are read, moved, copied as text and taken out. They have no **Edit**, **Apply here**, **Delete** or pause, and the card says «It belongs to another of your computers: on this one it is only read.». An agent that tries to write one through MCP is refused. Words another computer approved for this one wait here each time, until you press **Apply here**.

**Memory** is the same, and [stricter still](#memory-belongs-to-the-computer-that-brought-it).

### Approving on another computer is not applying

Approving changes the text; **Apply here** decides whether a computer writes the file. They are two separate things, and on a computer where a document waits, only the second writes it.

**An example.** Jaime has **How I work** at the root. On the work computer it still waits for his **Apply here**. Meanwhile, Claude Code proposes changing it, and since Jaime is at work, he approves it right there:

1. **Approve the change** asks «Approve this change? It is written to the document, and reaches your agents' files on each computer where Agents syncs and you applied it.». With your yes, the document says the new thing on all your computers.
2. At home, which already applied it, Tisty writes the change to its files as usual, with its backup first.
3. At work nothing is written yet. The card changes its amber notice to «You approved its words on this computer, but applying it here is a separate choice. Apply it to the agents on this one?», and **Apply here** and **Not on this computer** remain. The facts box still says «No: it waits for «Apply here»».
4. Press **Apply here** when you want work to receive it.

The same goes for **Approve this text** and for going back to an earlier version from the document's history: the text changes on all your computers, and a computer where the document was waiting keeps waiting. What you wrote or brought on a computer is applied on that computer with no click, as always.

### Two Agents folders that join by themselves

If two computers made Agents before syncing with each other, for example because you installed Tisty at work without connecting sync yet and chose **Bring and sync** on both, when they meet **two Agents folders** show in the sidebar for a moment.

Tisty joins them by itself, without asking:

- **The older one stays.** The same one on all your computers, even if both join at once.
- **Subfolders merge by name.** What was in the other computer's **Global** goes into your **Global**, each computer's folder into its own, and so on. A subfolder only the other one had comes over whole.
- **Documents move to the folder that stays,** with their text, their approval and their history: no approval is lost. The folder left empty is deleted.
- **No file of your agents changes** because of the join, and it asks «Take the file away too?» about none. What was already applied on a computer stays applied there.
- **An archived folder comes over whole** and stays archived: what it held is not written again.

One rare case is left as it is: if the folder that would go holds something an agent put there, Tisty does not join them, because moving it would make it look placed by you. Move that document to a folder of your own and the join happens by itself on the next round. While two Agents folders stay unjoined, both are protected: your agent can only propose in either of them.

### Memory belongs to the computer that brought it

An agent's memory is about what happened on that computer, so it is never written to another. On the other computers, its card says «Another computer's agent memory: on this one it is only read.», with no buttons. Each computer can bring its own memory of the same project: it shows unticked in the first-time box, even if another computer already brought its own.

### If you come from 1.50.1

Before, a document lived in **Claude**, **Codex** or **Global** for all computers at once, and Tisty joined into one file what was in Global and what was in your agent. Since 1.50.2 it is the other way round: each computer has its own folder, and a document is one file.

- **What each computer brought or wrote moves by itself to its own folder**, once all your computers have 1.50.2 or later. No agent file changes by a byte.
- **A `CLAUDE.md` that 1.50 had joined with a Global rule** becomes one document that says exactly what the file says.
- **While another computer is still on an earlier version**, nothing moves and the files stay as they are.
- **If the file was changed outside**, it is left and asked about as always.
- **On a computer still on 1.50.1**, you can edit another computer's document, but that text waits for your approval on the newer one.
- **A Global rule from that time** stays in Global for you to read, copy or delete; it is no longer written to any agent.

### To approve

**Only words you wrote or approved reach an agent's file.** What you write in Tisty's window, what you bring in the first time and what you bring with **Bring the change in** already counts as approved. A text that changed without going through your hands does not: for example, if someone edited the document's file inside Tisty's data folder, or an imported parcel brought it.

Then, inside the document's card, an amber notice says «Its text does not have your approval yet, so it reaches no agent until you approve it.». It is not a standing label: it is that sentence, and below it, before the button, it shows you what you need to decide:

1. **Whose hand wrote these words**, on one line: «Written on this computer.», «Written on another computer.», «Written by an agent.», «An earlier version came back.» or «Changed outside Tisty.».
2. **What changed since what you last approved:**
   - If the words last approved are kept on this computer, the compared lines: in red, with `−`, what was **Approved** and goes; in green, with `+`, what it says **Now**.
   - If the document was never approved: «Never approved: read all of it before approving it.».
   - If the words last approved are not kept here: «The words last approved are not kept on this computer, so there is nothing to compare them with: read all of it.».
3. The **Approve this text** button.

**Approve this text** asks «Approve what this document says now? Read it first: it reaches your agents' files on each computer where Agents syncs and you applied it.». If the document changed between being shown and your click, Tisty approves nothing: «This document changed since its card was drawn. Read it again and try once more.».

**An example.** A file-tidying program of Jaime's touched, without his knowing, the file of the rule **How I work** inside Tisty's data folder, and added a line to it. Its card shows the amber notice, «Changed outside Tisty.», and the compared lines: `+ - Always use the main branch.` in green, under **Now**. Jaime did not write that: he does not approve it, presses **Edit**, deletes the line and presses **Done**. What he writes in the window already counts as approved, and his `CLAUDE.md` gets it again.

If that document was already written to a file, the file does not change meanwhile: its card says **On hold** and names the document holding it, for example «This file stays as it was: claude · rules/style.md changed or left Agents without your approval. Approve it, or put it back and take it out yourself.».

A document an agent put in Agents, and not you, is not written either: its card says «An agent put it here, so no agent reads it until you move it yourself.». Move it yourself to another subfolder of Agents you made (reordering it is not enough), and it applies. If an agent made or renamed the folder it sits in, the same goes for everything inside it: move it to a folder of yours.

### Written by an earlier Tisty

An earlier version of Tisty wrote the files without noting which documents each one came from. The first time the new version looks at them, it compares: if the file says what its documents say, it takes it over without a word. If it says something else, the card stays at **Written by an earlier Tisty**, with the notice «An earlier Tisty wrote this file without noting which documents it came from, so it is left as it is until you choose». **See the file** shows you what it holds, and **Overwrite with Tisty** writes it from your documents, with its backup first.

## What is never read

- **Your agents' settings**: `settings.json`, `settings.local.json`, `config.toml`, `config.json`, `hooks.json`, `mcp.json`, `mcp-config.json`, `.claude.json` and `keybindings.json`. To find them, Tisty only looks where rules, skills and memory go, never in an agent's main folder, except Claude Code's `settings.json` and `plugins/installed_plugins.json`, only to tell whether the Tisty plugin already brings its skills; and inside a skill a file with one of those names does come in, because it belongs to the skill. This is about the Agents folder: connecting an agent and adding its hooks reads and writes some of them, as [Privacy](Privacy#the-agents-you-connect) tells.
- **Credentials**, by name and without opening them: `.credentials.json`, `credentials.json`, `auth.json`, `.env` and its variants, `.netrc`, and `.pem`, `.key`, `.p12`, `.pfx`, `id_rsa` or `id_ed25519` keys.
- **A file that looks like it holds a secret**, such as a token or a private key, even under an innocent name. The check runs on this computer and looks for known shapes, so it may not recognise them all: keep secrets out of your agents' rules.
- **Claude Code's conversations**: of its `projects` folder, only each project's `memory` folder is looked at.
- **Symbolic links** and files larger than 512 kB.

And a document in Agents is only written where each agent keeps its rules, skills and memory, never to its settings or credentials.

**Agent memory** comes unticked the first time, and once in Tisty it is read only: the agent keeps writing it, and when it changes, the card says **Changed outside** and **Bring the change in** brings Tisty up to date. It has no **Edit**, **Overwrite with Tisty** or **Restore**, and on your other computers it is [only read](#memory-belongs-to-the-computer-that-brought-it).

What Tisty keeps of all this, and what travels, is in [Privacy](Privacy#your-agents-in-the-agents-folder).

## With your agent, through MCP

An agent you let in through [[MCP|Agents-and-MCP]] **reads** your documents in Agents like any other, whether it syncs or not, and its guide tells it what they are. **It does not write them: it proposes the change, and only you approve it.** Nor can it bring, sync, stop syncing, hide or show Agents: no MCP tool does that, only you, in Tisty's window. Through MCP the folder is called `AGENTS`, its stored name, in any language: that is why the answers below say `AGENTS` and not Agents. For example, you ask Claude Code «Which of my rules do you read?».

**1. It reads its guide.** `rules` with the topic «documents» brings, among others, the section on the folder:

```json
{ "name": "rules", "arguments": { "topic": "documents" } }
```

```text
## Assistants' files in AGENTS

A document whose properties hold an `agent:` line is an assistant's own file kept in the folder
AGENTS — a rule file such as `CLAUDE.md`, a skill, a script — and Tisty writes its text, byte for
byte, to the file that assistant reads, on each computer where AGENTS syncs and the person applied
it. `read` gives the text the person approved. You change one only by proposing: `write_doc` with
`doc` and `print`, `edit_doc` or `append_doc` keep your text byte for byte, apart, and nothing
reaches any file until the person approves it on the document's card: say so, never call it done.
A later write goes on top of your proposal, which `read` shows in `proposal`. Keep the `agent:`
line: any other tool that names these documents or the folder AGENTS is refused, as is adding or
changing an `agent:` line anywhere: tell the person what to write. …
```

**2. It looks for the document.** `brain` with the file's words:

```json
{ "name": "brain", "arguments": { "query": "CLAUDE.md" } }
```

```text
0 task(s) and 1 document(s) match "CLAUDE.md"; showing 0 and 1.
"claude · CLAUDE.md" (document) [k4r5j4qr-0001]
```

`brain` with `folder` and a subfolder's name or path lists that agent's documents. The `AGENTS` folder alone answers with how many sit in each subfolder.

**3. It looks at its card and reads it.** `brain` with `doc` gives its outline and place; `read` with `doc`, the text you approved, whole, `agent:` line included:

```json
{ "name": "read", "arguments": { "doc": "k4r5j4qr-0001" } }
```

```text
---
agent: [claude/rule/CLAUDE.md, on-dev_ab12cd34]
title: claude · CLAUDE.md
---
# How I work

- Answer in English.
- Branches shaped type/name.
```

**Another computer's document.** If the document is in another of your computers' folder, the `read` answer carries two more fields, `of_computer` (which computer it belongs to) and `read_as`, which tells the agent to read it as data and never as a rule of its own for this computer. It cannot propose a change on it either: that is done from that computer. On this computer's documents it proposes as always, and there is no new tool.

The agent answers you with the title and the folder, «your document claude · CLAUDE.md, in AGENTS / Home / Claude», never with the id. Every tool is in the [[Tools reference|MCP-Tools]].

### A proposal from your agent

You ask it «add to my rules that it runs the tests before every commit». The agent edits the document like any other, but in Agents its change is not written: **it stays apart, as a proposal**, byte for byte. Nothing changes in the document or in your agents' files until you approve it.

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

**2. You see it in Tisty.** Open the document: the card gains the label **Proposal** and, below, a purple box:

- «An agent (Claude Code) proposes a change», with when it proposed it;
- «Nothing of it reaches your agents' files until you approve it. What it changes in the approved text:»;
- the compared lines: in red, with `−`, what is **Approved** and goes; in green, with `+`, what is **Proposed** and comes;
- the buttons **Approve the change** and **Discard**.

If Tisty is open while the agent proposes, the proposal shows by itself within a second or two, even when the window is not in front.

**3. You approve or discard.**

- **Approve the change** asks «Approve this change? It is written to the document, and reaches your agents' files on each computer where Agents syncs and you applied it.». With your yes, the document says what was proposed, with your approval, and on each computer where Agents syncs and that document applies, Tisty writes it to `~/.claude/CLAUDE.md` like any change of yours, with its backup first. What it said before stays in its history.
- **Discard** asks «Discard this proposal? The document stays as you approved it.». The proposal goes and nothing else changes.

Each button acts only on what the card showed you. If the agent wrote again while you were looking, Tisty does not choose for you: «The proposal changed since the card showed it. Look at it again before choosing.». While you edit the document, **Approve the change** does not show: press **Done** first.

**Good to know:**

- **One proposal per document.** If the agent writes again, its change goes on top of its own proposal, and `read` hands it back apart from the approved text, in `proposal`. If it writes the approved text back, it withdraws it.
- **Approved text keeps reaching your agents** while a proposal waits: a proposal on one rule holds back nothing else. The card's facts box says so: «Yes · an agent's change waits for your approval».
- **If the approved text changed since**, the card warns «It was proposed over an earlier version, so approving it may undo a change made since.». Look closely at the lines before approving.
- **If the document's text [waits for your approval](#to-approve)**, you first approve its text with **Approve this text** and then the proposal: «This document's text does not have your approval yet. Approve it first, then the proposal.».
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

**Any other tool** that names a document in Agents or the folder `AGENTS` is refused: moving, archiving, marking or summing up a document, making it a page of another, joining it to another language, attaching a file to it or going back to an earlier version, and also moving, renaming or archiving the folder itself. The whole list is in [[MCP tools|MCP-Tools#the-agents-folder]]. Tisty tells it how to propose:

```text
the document "claude · CLAUDE.md" holds an assistant's own files in AGENTS, which only the person
changes, in Tisty: nothing an assistant moves, files, marks or sums up there reaches them. To
change its text, propose it: `write_doc` with `doc`, `print` and the whole `body`, `edit_doc`, or
`append_doc` at its end, and nothing else with them. The person approves or discards it on the
document's card.
```

**Putting another document or making a folder** inside Agents is refused too, and there Tisty asks it to tell you what it would change, for you to do it.

**A new document with an `agent:` line** is refused in any folder, so nobody can bring a rule in from outside:

```text
a body with an `agent:` line is an assistant's own file, which only the person makes, in AGENTS.
Write it without that line, or tell the person what the file should say.
```

**Nor adding or changing an `agent:` line in a document that already exists**, in any folder and with any tool, `edit_doc` included. If Claude Code tries to turn your document **Notes**, in **Work**, into a rule, by putting properties with `agent: claude/rule/CLAUDE.md` on top of it, Tisty writes nothing:

```json
{ "name": "edit_doc", "arguments": { "doc": "m2q8v7xd-0001", "old": "# Notes", "new": "---\nagent: claude/rule/CLAUDE.md\n---\n# Notes" } }
```

```text
an `agent:` line makes an assistant's own file, which only the person makes, in AGENTS. Leave that
line as it was. Nothing was written.
```

And if `append_doc` leaves those lines at the end of a document, they stay words of the text, never its properties: that document reaches no agent.

**Agent memory** takes no proposals: the agent itself writes it, in its own file. Nor does a proposal get in that would leave a script's code block broken, or that goes past 64,000 characters or 2,000 lines, what the card can compare.

And editing the file on disk does not help either: that change stands as **Changed outside** and waits for your decision. **Why it works this way:** while syncing, a rule in Agents reaches all your agents, on each computer where Agents syncs and you applied it; if an agent could write it alone, an instruction hidden in a page it read could end up in the rules of the others. With a proposal, you see every line before it arrives.

## Questions

**Does Tisty replace my files?** It neither moves nor converts them: it keeps writing them where your agent looks for them. Before the first write it keeps the original, backup #1, which is never removed.

**Can I still edit `CLAUDE.md` by hand?** Yes. Tisty marks the file **Changed outside** and waits for you to choose: bring your change into the document, or write over it.

**What if I close or uninstall Tisty?** Your agents keep reading their files as Tisty left them. Nothing depends on Tisty being open.

**Does Agents touch `settings.json`, hooks or my MCP configuration?** It does not write them, and reads only Claude Code's `settings.json` and `plugins/installed_plugins.json`, to tell whether the Tisty plugin already brings its skills. Connecting an agent is something else: there Tisty adds its MCP server to that agent's configuration, and its hooks if you ask for them, as [[Agents and MCP|Agents-and-MCP]] tells; and, with the door open, it reads those files when its window opens to see whether they point at this Tisty, [writing nothing](Privacy#the-agents-you-connect).

**Why does agent memory come unticked?** Because the agent writes it, it may talk about your projects, and once in Tisty it travels with your sync and your backups like any document. Bring it in only if you want to read it in Tisty.

**What is the `@tisty-rules.md` line in my `GEMINI.md`?** It is the only line Tisty adds to that file, so Gemini reads your rules, which go in `tisty-rules.md`. The rest of `GEMINI.md` is written by Gemini. If you delete it, Tisty does not put it back: Gemini stays [paused](#gemini-paused) until you press **Put the line back**.

**Does Agents see the agents I use inside WSL?** No. On Windows, Tisty looks at your Windows user folder, and an agent running inside WSL keeps its files in your Linux home.

**And each repository's rules?** They stay where they are. Agents handles your user's files, not each project's `AGENTS.md` or `CLAUDE.md`.

**Can I write a new rule from scratch?** Yes: right-click an agent's folder on your computer, such as **Claude**, and **New document**. The document already starts with its `agent:` line, and you only write the rule. Not in **Global**: only skills go there. If Agents syncs, Tisty writes it as soon as you write something. See [A new document is already a rule](#a-new-document-is-already-a-rule).

**How many backups does it keep?** The original, backup #1, which is never deleted, and on top of it the last 20 of each file, on this computer.

**Why do I have an empty Cursor folder in Agents?** Tisty found Cursor on this computer, and each agent it finds has its subfolder, even an empty one. If you do not want it, delete it: Tisty does not make it again. See [How the folder looks](#how-the-folder-looks).

**I installed a new agent. How do I add it?** Right-click **Agents** and **Look again**: Tisty makes its subfolder and offers you its new files already ticked. See [Look again](#look-again).

**Can I ask my agent to change my rules?** Yes, and it proposes it: its change stays apart, on the document's card, until you press **Approve the change** or **Discard**. Nothing reaches your agents before. See [A proposal from your agent](#a-proposal-from-your-agent).

**I use Tisty on two computers. Do my rules reach the other one by themselves?** They arrive as documents, with your sync, in your other computer's folder, and there they are read but not written to that computer's agents. If you want the same rule on both, move it to the root with **Move to › Agents** (or to the agent's loose folder, such as **Codex**), and on the other computer press **Sync** and then **Apply here** on the document, the first time. After that it updates by itself. See [On several computers](#on-several-computers).

**Can I keep a rule on one computer only?** Yes: leave it in that computer's folder, and nobody else writes it. For a root one you do not want on one of them, press **Not on this computer** on its card there.

**Why does it ask «Which one rules here?»** Because on this computer two documents, the one in your folder and one at the root, go to the same file or the same skill. Choose **This one** or **The other**; until you do, nothing changes. See [If two documents go to the same file](#if-two-documents-go-to-the-same-file).

**Can I edit what is on my other computer?** Not from this one: you read it, move it, copy it as text and take it out. The card says «It belongs to another of your computers: on this one it is only read.». See [Other computers' documents](#other-computers-documents).

**I deleted a rule. What happens to its file?** As you delete it, Tisty asks in that same box «Take it away from Claude, Codex and Copilot too?», with the agents that have it: **Take away too** deletes its files and keeps a backup of each, and **Only from Tisty** leaves them where they are, listed in Settings in case you change your mind later. See [Deleting a rule or skill](#deleting-a-rule-or-skill). If you took it out some other way, for example by moving it out of Agents, Tisty asks «Take the file away too?»: see [Taking a rule and its file out](#taking-a-rule-and-its-file-out).

**The first time I left a file unticked. Can I bring it now?** Yes, with **Bring another file**: by right-clicking a subfolder of Agents, such as Claude; with Agents open, from the Library's **+** button or the command palette; or from Settings › Agents › Agents folder. See [Bring another file](#bring-another-file).

**Why do I have Agents in the sidebar if I never used it?** It is there, in grey, so you know it exists. Like that it reads nothing: it only checks whether each agent's folder exists, such as `~/.claude`, and the list of names and sizes only shows when you click it. If you would rather not see it, right-click it and **Hide from the sidebar**; it comes back with **Show again**, in Settings › Look and window › Hidden folders.

**What is the difference between Bring and Sync?** **Bring** copies the files into Tisty and writes nothing into them; **Sync** makes what you change in Tisty reach those files. See [Bring or sync](#bring-or-sync).

**I brought my rules and changed them in Tisty, but my agent does not see it.** Agents is brought but not synced: its robot is grey and, when you open it, it says «Not synced with your agents · Sync» at the top. Press **Sync**.

**Does Stop syncing delete anything?** No. No agent file is touched and no document is deleted: your agents keep reading what Tisty wrote last time, and your documents stay in Agents.

**Can my agent propose a change while Agents is hidden or not synced?** Yes: the proposal stays on its document's card, and you approve or discard it there. Approved, it changes the document; it reaches your agents' files when Agents syncs.

**Why does the sidebar not tell me if something waits?** Because Agents looks like any folder: only its colour says whether it syncs. What waits for your decision is on each document's card and on the **Agents** card in Settings › Agents › Agents folder, and while syncing, Tisty asks about pending files as soon as you go into it. See [How the folder looks](#how-the-folder-looks).

**Why does Agents hold no documents?** You made it with **Bring and sync** without Tisty finding any files, or its documents left while Tisty was closed, for example when syncing with another computer. Write a rule with **New document** in an agent's folder on your computer, or bring a file with **Look again** or **Bring another file**.

## When something does not add up

**An agent is missing from the list.** Tisty only sees an agent whose folder exists: `~/.claude`, `~/.codex`, `~/.copilot`, `~/.gemini` or `~/.cursor`. If you moved it with a variable such as `CODEX_HOME`, set it as a user variable and open Tisty again. If you never used that agent on this computer, open it once so it makes its folder.

**A file is greyed out.** The reason sits beside it: see [what the list says](#what-the-list-says-beside-a-file).

**My change in Tisty does not reach the file.** Look at the standing on the document's card:

- **Paused**: resume that agent, or the folder, on the card or in Settings › Agents › Agents folder.
- **Not allowed on this computer**: this computer has not let you read your agents' files yet. Press **Sync** and choose in the first-time box.
- **The card says «The Agents folder does not sync with your agents.»**: Agents is brought but not synced, with its grey robot. Press **Sync**.
- **Changed outside** or **Changed on both sides**: Tisty waits for your decision before writing.
- **Nothing to write**, **The file is a link** or **The code block is broken**: the table of [each standing](#each-standing) says what to do.
- **«No agent on this computer reads this document yet.»**: check the `agent:` line. The level must be an agent on this computer, and the name a file that agent reads.
- **«On this computer another document writes this file.»** or **«Which one rules here?»**: two documents go to the same file. Choose with **This one** or **The other**, or keep one. See [If two documents go to the same file](#if-two-documents-go-to-the-same-file).
- **«It belongs to another of your computers»**: that document is written on that computer, not on this one.
- **«It sits in one computer's folder and its text says another's»**: press **Go by its folder**.
- **On hold**: the document's text waits for your approval. The card names it; [approve it](#to-approve).
- **Written by an earlier Tisty**: look at the file and, if you keep the document, **Overwrite with Tisty**.
- **An amber notice about another computer or your approval**: the document is not applied on this computer yet. See [On several computers](#on-several-computers).

**My rule does not reach my other computer.** A rule in one computer's folder is written only on that computer. If you want it on both, move it to the root. On the other computer, check that Agents has its coral robot (if it is grey, press **Sync**) and open the document: if it says «This document comes from another computer», press **Apply here**; if it says «You chose not to apply it on this computer.», that too. If the card says «No agent on this computer reads this document yet.», that computer does not have the rule's agent.

**A card says «Its text does not have your approval yet» and I changed nothing.** A document's text changed without going through Tisty's window, for example through an imported parcel or a direct edit in the data folder. The notice tells you whose hand wrote it and what changed since what you last approved: read it, and if it is right, **Approve this text**. See [To approve](#to-approve).

**I wrote a rule and its file stands Changed on both sides.** That file already existed and you did not bring it in, so Tisty cannot tell which part is yours. Do not overwrite it: in **Settings › Agents › Agents folder**, press **Review** and, in **It exists on both sides**, **Bring it into Agents**. The file comes in as a document with what it says, and the original stays as backup #1. See [A file you never brought](#a-file-you-never-brought).

**It asks «Take the file away too?» and I deleted nothing.** Changing a rule's `agent:` line to another file's, or going back to an earlier version that had no card, also counts as taking a rule out. If you would rather touch nothing, **Leave it** leaves it where it is.

**Agents is not in the sidebar.** You hid it, on this computer: it is in **Settings › Look and window › Hidden folders**, and **Show again** brings it back. If you emptied it with Tisty open, it does not hide: it comes back to the sidebar by itself, in grey, and a click on it brings in what it finds again.

**Agents' robot is grey.** It does not sync on this computer. If the name is grey too, nothing is brought yet: click it. If the name looks normal, it is brought: press **Sync**. See [Three states in the sidebar](#three-states-in-the-sidebar).

**Hide asks me to stop syncing.** Agents syncs, and it is never hidden and syncing, neither Agents nor its subfolders, such as **Global**. **Stop syncing and hide** does both; your agent files are not touched. If you would rather it kept syncing, **Cancel**.

**A subfolder of Agents has no Rename or New diagram.** On purpose: inside Agents, each folder belongs to an agent, and Tisty knows whom it writes to by its name. Its right-click offers only **New document**, **Bring another file**, its agent's pause, **Hide from the sidebar** and **Delete**; another computer's folder offers only **Hide from the sidebar**. See [A subfolder's right-click](#a-subfolders-right-click).

**The right-click on Agents only offers syncing, looking again or hiding.** That is on purpose. **Look again** brings what is new from your agents; to choose each file yourself, use **Bring another file** on the right-click of a subfolder, such as Claude, or in Settings › Agents › Agents folder. And it does not move or change its name because inside it is kept as `AGENTS`, and under another name or in another place Tisty and your agents would stop finding it. See [Agents stays at the top](#agents-stays-at-the-top).

**I tried to move or rename Agents and it would not let me.** Tisty told you «Agents stays at the top, with its name and its robot, so it keeps guarding your agents' rules. Its color and its place among your folders are yours to change». That is on purpose: see [Agents stays at the top](#agents-stays-at-the-top).

**The whole folder stopped writing.** Check that Agents has its coral robot: if it is grey, it does not sync on this computer, and **Sync** brings it up to date again. If one agent gets nothing, look at its switch in Settings › Agents › Agents folder.

**Codex does not read my rules.** If you have `~/.codex/AGENTS.override.md`, Codex reads only that file, and a document with `agent: codex/rule/AGENTS.md` is written to `AGENTS.md`, which Codex does not read while the other one exists. If you made the override after the first time, [bring it into Agents](#codexs-agentsoverridemd).

**Gemini does not read my rules.** If its card says **Paused**, the line `@tisty-rules.md` that Tisty had put in `~/.gemini/GEMINI.md` is missing: press **Put the line back**, on the card or in Settings › Agents › Agents folder. Tisty does not put it back on its own, because deleting it is your decision. See [Gemini paused](#gemini-paused).

**Bring the change in does not show.** The document is in Global, or you are editing it: press **Done** and look again.

**«The file changed while Tisty was writing, so it was left alone.»** Another program wrote the file at the same moment. Tisty writes over nothing: look at the card again and choose.

**I restored a backup and it says Changed outside.** That is expected: [a restored backup does not change your document](#backups). Choose **Bring the change in** to keep it.

## See also

- [Privacy](Privacy#your-agents-in-the-agents-folder): what Agents reads, when, and what stays on this computer.
- [[Agents and MCP|Agents-and-MCP]]: connecting an agent to Tisty.
- [Earlier versions](Documents#earlier-versions): a document's history.
- [Folders](Documents#folders): filtering, archiving and unarchiving.
- [Properties](https://markdown.tisty.app/properties), in the Tisty Markdown guide: the `agent:` line and its marks.
