# FAQ

## The basics

**Is Tisty free?**
Yes, Tisty is free and will stay free, under the AGPL-3.0 licence. No accounts, no telemetry, and it keeps being developed and improved. [Commercial terms](https://github.com/rgdevment/Tisty/blob/main/docs/COMMERCIAL.md) exist only for organisations that cannot comply with the AGPL.

**Does it need an account or a connection?**
Neither. On its own, Tisty only asks whether a newer version exists, and it sends nothing to ask. Once you sync with a cloud or share a document, it also reads the few small files [[Privacy]] lists. Take the machine off the network and everything else works the same. See [[Privacy]].

**Do I need an AI agent?**
No. Everything Tisty brings is there from the start: tasks in natural language with their steps, linked documents, Ideas, the Diary and [[Brainsty|Brainsty-Network]]. It works without an agent too, the traditional way: your notes, documents and tasks, by hand and at your own pace. Say you keep Rina's vet visits in a document and her next vaccine as a task that rings the evening before: no agent sees any of it until you connect one, whenever you like, as [[Assistants and MCP|Assistants-and-MCP]] tells.

**Where is my data kept?**
In one folder on your own disk. **Settings › Storage** prints the path and **Show folder** opens it. See [Installation](Installation#where-your-data-lives).

**What happens to my notes if Tisty disappears?**
They stay where they are: Markdown and plain text, readable with `cat`, searchable with `grep`, openable in any editor.

**Can I export my tasks?**
There is no export button, because there is nothing to convert: they are already open files. Documents are Markdown in `docs/`. Tasks live in `store/`, one folder per computer, as an append-only history with one JSON event per line, so any tool reads them. Every task ever created, for instance:

```sh
cat store/*/*.tisty | jq -c 'select(.op == "task.add") | {id, title: .d.title}'
```

That is the history, not today's state: a later event renames, finishes or moves a task. For the tasks as they stand now, ask your assistant through [[MCP|Assistants-and-MCP]] to list them, or **Create backup…**, in **Settings › Copies**, writes a zip with everything. If you need a specific format to take Tisty elsewhere, say which in an issue.

**Is my data encrypted?**
On your disk, no, by decision: your operating system's permissions are the protection, and the files stay readable with tools you already have. In a cloud Tisty reaches directly, it can be: see [Encrypting your cloud](Sync-and-Backup#encrypting-your-cloud). [PRIVACY.md](https://github.com/rgdevment/Tisty/blob/main/PRIVACY.md) explains why.

**Can I use it with my team?**
Not as a shared board. Tisty is one person's by design — no assignees, no permissions, no shared database. Each person keeps their own tasks, documents and agent, and when somebody needs what you know, you hand them the document through Folio, encrypted and signed. See [Sharing](Documents#sharing).

**Is there a mobile app? Linux?**
No mobile app: Tisty is a desktop program for Windows and macOS. Your documents are Markdown in a folder, so a phone that reaches that folder can read and edit them with any app. There is no Linux version either; an assistant that runs on Linux inside Windows, under WSL, reaches Tisty all the same from Settings › Assistants › Connected.

**Do reminders ring with Tisty closed?**
Yes, since 1.48, while the computer is on: Tisty books them with Windows or macOS, and a click on the notice opens the task. See [Reminders](Spread-and-Reminders#reminders).

**Can Tisty remind me on my phone?**
Not by itself. **Save to my calendar**, on a task with a date, puts it in Google Calendar, Outlook or Apple Calendar, and your calendar reminds you there. See [In your calendar](Spread-and-Reminders#in-your-calendar).

**Is there a diary, or daily notes?**
Yes, since 1.48: **Diary** in the Library, a book a year with a page a month and a section a day. See [Diary](Documents#diary).

**Can my assistant read my diary?**
Not until you press **Only you** at the top of the Diary and it reads **Open to agents**; even then it only adds to the end of today, when you ask. **Diary** does not start your diary while any computer of yours runs a Tisty older than 1.48, and once it exists, a computer on an older version stops syncing before it receives it.

**Does it work with a screen reader?**
Not as far as anybody knows. The whole window works from the keyboard and that path is tested, but it has never been tried with a real screen reader. If you use one, an [issue](https://github.com/rgdevment/Tisty/issues) saying where it falls apart would be very welcome.

## AI and assistants

**How do I give my AI agent a memory between sessions?**
Connect Tisty over MCP, as [[Assistants and MCP|Assistants-and-MCP]] shows: Settings › Assistants connects GitHub Copilot, Claude Code, Codex, Cursor… and [[every other one it knows|Assistants-and-MCP#connecting-an-assistant]]. From then on the agent starts a session by asking what moved since the last one, files its work as tasks with steps and a journal, and keeps what it learns in documents. The next session, on any project, starts from there. See [A day with your agent](Assistants-and-MCP#a-day-with-your-agent).

**Does it replace CLAUDE.md, AGENTS.md or a memory-bank folder?**
For what changes, yes; for what does not, no. Instructions that rarely move — how to build, the conventions — belong in the files your repository already hands the assistant. What moves — progress, decisions, what was ruled out, what is left — belongs in Tisty, where it is dated, searchable, read a section at a time, and never lost between branches, machines or projects.

And the files in your home folder, like `~/.claude/CLAUDE.md` or `~/.codex/AGENTS.md`, the [[Agents|AGENTS-Folder]] folder brings into Tisty as documents, if you ask it to. **Bring** only copies them; **Sync** also writes into them what you change in Tisty, with a backup first. Your repository's files it does not touch.

**Can my agent change my rules in the Agents folder?**
Not on its own. It reads them like any document, but a change of its own stays as a proposal: on the card you see what is **Approved** and what is **Proposed**, and only **Approve the change** carries it to your agents' files, on every computer where Agents syncs and you applied it. See [[A proposal from your agent|AGENTS-Folder#a-proposal-from-your-agent]].

**Does Tisty use AI?**
No. Tisty is not AI and has none inside, and that is not going to change. The natural language is rules on your machine. If you use an assistant of your own, Tisty leaves an [[MCP door|Assistants-and-MCP]] — it comes open, and you close it whenever you like.

**An assistant says one of its tasks is done. Did it close it?**
No. Nothing an assistant does closes a task. It marks it, with its account in the journal, and the task waits under **To confirm** until you decide. See [[What an assistant may do|MCP-Permissions]].

**Can an assistant write over a document I am working on?**
Only over the version it read. To replace a whole document it must send back the print of the text it read; if you wrote in between, nothing is written and it has to read again. What it replaced is kept as a version in its history, and from **See the history** you go back to it whenever you want ([Earlier versions](Documents#earlier-versions)). The window also offers to put the document back at once — only when that copy is exactly one write back, the only time it can do what it says. If you try to save over a change you had not seen, it asks which one stands.

## Documents

**Can I bring my notes over from another app?**
Documents, yes: *Import a document* takes `.md`, `.markdown`, `.txt`, `.html` or Word `.docx`, one at a time, and turns it into Markdown — Word to Markdown, HTML to Markdown, Obsidian to Markdown. Pasting from Word, Google Docs or a web page converts too. A `.tistyx` from another Tisty unpacks whole. Tasks have to be typed. See [Documents](Documents#importing).

**Can I read a Markdown file without bringing it into Tisty?**
Yes: choose Tisty under **Open with**, or follow a `tisty://open?path=…` link. It opens read only in a light viewer of its own, up to 1 MB, and **Edit** brings a copy in when you want it. See [Documents](Documents#opening-a-markdown-file-from-outside).

**Can I link to a document from outside, or hand one to my agent?**
Yes: **Copy link**, under **Export** or in the document's menu, gives you a `tisty://doc/…?id=…` address. Followed, it opens the document read only in the light viewer on a computer that holds it, and your agent reads or changes that very document without searching for it. See [Documents](Documents#a-documents-link).

**A document opened read-only and I did not ask for that.**
It is archived, or a page of it (or its document) is archived. The bar at the foot says which. A document holding HTML tags is not read-only: it opens, edits and saves whole. See [The Markdown editor](Markdown-Editor#when-a-document-opens-read-only).

**Do diagrams and formulas need the internet?**
No. Both are drawn by code inside Tisty: nothing is fetched, no font is loaded from anywhere, and they work with the network unplugged. Both stay Markdown that GitHub draws too. See [diagrams, formulas and widgets](https://markdown.tisty.app/diagrams-formulas-and-widgets) in the Tisty Markdown guide.

**Why does my table's column width survive, and where does it live?**
In how long the rule under the header is drawn. See [tables](https://markdown.tisty.app/tables) in the Tisty Markdown guide.

**Can I go back to what a document said a few days ago?**
Yes, on the computer where it changed: **See the history**, on the **History** row of the panel beside the document, lists its earlier versions, **What changed** shows in red and green what differs from now, and **Go back to this version** brings one back. Tisty keeps one when someone writes after 30 minutes without changes or when the one writing changes, you or an assistant, so what your agent replaced always stays apart. They are kept 30 days by default, are not synced and **Empty the history** deletes them from that computer. See [Documents](Documents#earlier-versions).

**How is the Brainsty network different from Obsidian's graph?**
Obsidian's graph shows links between notes. Brainsty joins documents and tasks, with each task's state and its history, and the same network serves your agent as an index to search without reading everything. It draws with lines what is written (links, cards, pages, parts, where a task was born and tags), and what is written weighs more than a tag as it settles; folders and lists give the colour, they are not dots. With an amber curve apart, what Tisty thinks is alike: because they share words, because one has a typo or because one is the other's translation, and it always says why. It opens in the content panel, beside the sidebar, with a legend of its colours, and costs nothing once you move to another view. See [[The Brainsty network|Brainsty-Network]].

**Does Brainsty use AI to tell what is alike?**
No. It uses fixed rules and a list of words: two uncommon words in common, a word a single change away from another, like *imrpesora* and *impresora*, or a pair from a Spanish–English dictionary made by linguists, like *pagar ≈ pay*. It neither understands nor learns from what you write, it reads only the prose (never code, paths or your agents' files), and all of it runs on your computer. See [No AI](Privacy#no-ai).

**Does Brainsty upload anything?**
No. The network is worked out on each computer and rebuilds itself: it is not synced, and it does not go to Folio, a `.tistyx`, an export or a backup. What travels is your documents and tasks, as always. The same holds for what Brainsty notes as alike: see [The Brainsty network](Brainsty-Network#alike).

## Sync

**Can I use it on two computers?**
Yes, through a folder you already sync or through your Google Drive, OneDrive or WebDAV server directly. See [[Sync and backup|Sync-and-Backup]].

**I changed something on one machine and it is not on the other.**
Almost always the cloud program is not running, is paused, or does not sync that folder. The full checklist is in [Sync and backup](Sync-and-Backup#something-changed-on-one-machine-and-is-not-on-the-other).

**Can I sync through Google Drive without Google's program?**
Yes, since 1.30: **Settings › Sync › Turn on… › Your cloud service › Google Drive › Connect**. Tisty asks only for a hidden space of its own in your Drive, sees nothing else there, and tells you on its card when something is wrong. What each message means is in [Sync and backup](Sync-and-Backup#syncing-through-google-drive-directly).

**Where are my copies in Google Drive, and how do I delete them?**
In a hidden space of your Drive that only Tisty opens. Disconnecting keeps them, and so does *Disconnect from Drive* in **Google Drive › Settings › Manage apps**, which only withdraws Tisty's access. Only **Delete hidden app data** there, or **Delete my copies in Google Drive for good** in Tisty, deletes them — and that space has no bin.

**Can I sync through OneDrive without Microsoft's program?**
Yes, since 1.31: **Settings › Sync › Turn on… › Your cloud service › OneDrive › Connect**. Tisty keeps its copies in *Apps › Tisty*, a folder you can see in your OneDrive; leave it to Tisty. See [Sync and backup](Sync-and-Backup#syncing-through-onedrive-directly).

**Can I sync through my own server?**
Yes, since 1.32, over WebDAV: Nextcloud, ownCloud, Synology and others. In **Settings › Sync › Turn on… › Your cloud service › WebDAV** you type the server's address, your user and a password, and Tisty checks the server before connecting; no browser opens. The password stays on this computer, kept by the system, and the copies go in a *Tisty* folder on the server. Only `https://` is accepted, except to this very computer. See [Sync and backup](Sync-and-Backup#syncing-through-your-own-webdav-server).

**Can I encrypt my cloud?**
Yes, since 1.36, for Google Drive, OneDrive or WebDAV: **Settings › Security › Encrypt my cloud…** encrypts what Tisty keeps there with a 24-word phrase only you know, so the provider can no longer read your tasks or documents. A folder kept in step by another program is not encrypted. Your other computers get the key on their own; a new one is let in with twenty digits typed on a computer that already has the key, or with your phrase. The provider still sees file names and sizes, how many computers you have, and earlier unencrypted versions for a while. Without the phrase or one of your computers, nobody can open it, not even Tisty. See [Encrypting your cloud](Sync-and-Backup#encrypting-your-cloud).

**Tisty says something is blocking Google Drive.**
A proxy, a filter at work or an antivirus stands between this computer and Google, or inspects secure connections so Tisty cannot trust them. Your work stays safe on this computer meanwhile. On a work network, ask whoever runs it to let Tisty through; on your own computer, allow Tisty in your antivirus. Never go around your company's protection on your own. What each message means is in [Sync and backup](Sync-and-Backup#when-something-stands-in-the-way-of-google).

**One of my machines stopped syncing after an update.**
Update the one that is behind. A machine only reads what it understands, so the two stop syncing in both directions until they agree. A computer still on Tisty 1.30 to 1.33 stops and asks you to update instead of writing over a newer one. Nothing is lost, and what you write meanwhile stays on that machine. A computer still on a Tisty older than 1.48 also stops at a place that holds a [Diary](Documents#diary), before it receives any of it.

## Comparing

**How is it different from Obsidian, Notion or Todoist?**
Tisty keeps tasks and documents in one store and one search, and finishing a task keeps everything it gathered instead of hiding it. The README has [a comparison table](https://github.com/rgdevment/Tisty#alternatives-to-obsidian-notion-and-todoist) that also says where each of the others is stronger.
