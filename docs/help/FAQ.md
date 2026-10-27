# FAQ

## The basics

**Is Tisty free?**
Yes, and it stays free. AGPL-3.0, no paid tier, no feature held back, no trial that runs out. [Commercial terms](https://github.com/rgdevment/Tisty/blob/main/docs/COMMERCIAL.md) exist only for organisations that cannot comply with the AGPL.

**Does it need an account or a connection?**
Neither. Tisty makes one request you did not ask for — to see whether a newer version exists — and sends nothing to ask it. Take the machine off the network and everything else works the same. See [[Privacy]].

**Where is my data kept?**
In one folder on your own disk. **Settings › Your data** prints the path and *Show folder* opens it. See [Installation](Installation#where-your-data-lives).

**What happens to my notes if Tisty disappears?**
They stay where they are: Markdown and plain text, readable with `cat`, searchable with `grep`, openable in any editor.

**Can I export my tasks?**
There is no export button, because there is nothing to convert: they are already open files. Documents are Markdown in `docs/`. Tasks live in `store/`, one folder per computer, as an append-only history with one JSON event per line, so any tool reads them. Every task ever created, for instance:

```sh
cat store/*/*.tisty | jq -c 'select(.op == "task.add") | {id, title: .d.title}'
```

That is the history, not today's state: a later event renames, finishes or moves a task. For the tasks as they stand now, ask your assistant through [[MCP|Assistants-and-MCP]] to list them, or **Settings › Your data** writes a zip with everything. If you need a specific format to take Tisty elsewhere, say which in an issue.

**Is my data encrypted?**
On your disk, no, by decision: your operating system's permissions are the protection, and the files stay readable with tools you already have. In a cloud Tisty reaches directly, it can be: see [Encrypting your cloud](Sync-and-Backup#encrypting-your-cloud). [PRIVACY.md](https://github.com/rgdevment/Tisty/blob/main/PRIVACY.md) explains why.

**Can I use it with my team?**
No. Tisty is for one person by design — no assignees, no permissions, no boards.

**Is there a mobile app? Linux?**
No mobile app: Tisty is a desktop program for Windows and macOS. Your documents are Markdown in a folder, so a phone that reaches that folder can read and edit them with any app. There is no Linux version either; an assistant that runs on Linux inside Windows, under WSL, reaches Tisty all the same from Settings › Assistants.

**Does it work with a screen reader?**
Not as far as anybody knows. The whole window works from the keyboard and that path is tested, but it has never been tried with a real screen reader. If you use one, an [issue](https://github.com/rgdevment/Tisty/issues) saying where it falls apart would be very welcome.

## AI and assistants

**Does Tisty use AI?**
No, and none is coming. The natural language is rules on your machine. If you use an assistant of your own, Tisty leaves an [[MCP door|Assistants-and-MCP]] — you open it, and you can close it.

**An assistant says one of its tasks is done. Did it close it?**
No. Nothing an assistant does closes a task. It marks it, with its account in the journal, and the task waits under **To confirm** until you decide. See [[What an assistant may do|MCP-Permissions]].

**Can an assistant write over a document I am working on?**
Only over the version it read. To replace a whole document it must send back the print of the text it read; if you wrote in between, nothing is written and it has to read again. What it replaced is kept, and the window offers to put the document back — only when that copy is exactly one write back, the only time it can do what it says. If you try to save over a change you had not seen, it asks which one stands.

## Documents

**Can I bring my notes over from another app?**
Documents, yes: *Import a document* takes `.md`, `.markdown`, `.txt`, `.html` or Word `.docx`, one at a time, and turns it into Markdown — Word to Markdown, HTML to Markdown, Obsidian to Markdown. Pasting from Word, Google Docs or a web page converts too. A `.tistyx` from another Tisty unpacks whole. Tasks have to be typed. See [Documents](Documents#importing).

**Can I read a Markdown file without bringing it into Tisty?**
Yes: choose Tisty under **Open with**, or follow a `tisty://open?path=…` link. It opens read only in a window of its own, and **Edit in Tisty** brings it in when you want it. See [Documents](Documents#opening-a-markdown-file-from-outside).

**A document opened read-only and I did not ask for that.**
It is archived, or a page of it (or its document) is archived. The bar at the foot says which. A document holding HTML tags is not read-only: it opens, edits and saves whole. See [The Markdown editor](Markdown-Editor#when-a-document-opens-read-only).

**Do diagrams and formulas need the internet?**
No. Both are drawn by code inside Tisty: nothing is fetched, no font is loaded from anywhere, and they work with the network unplugged. Both stay Markdown that GitHub draws too. See [diagrams, formulas and widgets](https://markdown.tisty.app/diagrams-formulas-and-widgets) in the Tisty Markdown guide.

**Why does my table's column width survive, and where does it live?**
In how long the rule under the header is drawn. See [tables](https://markdown.tisty.app/tables) in the Tisty Markdown guide.

## Sync

**Can I use it on two computers?**
Yes, through a folder you already sync or through your Google Drive, OneDrive or WebDAV server directly. See [[Sync and backup|Sync-and-Backup]].

**I changed something on one machine and it is not on the other.**
Almost always the cloud program is not running, is paused, or does not sync that folder. The full checklist is in [Sync and backup](Sync-and-Backup#something-changed-on-one-machine-and-is-not-on-the-other).

**Can I sync through Google Drive without Google's program?**
Yes, since 1.30: **Settings › Sync › Your cloud service › Google Drive › Connect**. Tisty asks only for a hidden space of its own in your Drive, sees nothing else there, and tells you on its card when something is wrong. What each message means is in [Sync and backup](Sync-and-Backup#syncing-through-google-drive-directly).

**Where are my copies in Google Drive, and how do I delete them?**
In a hidden space of your Drive that only Tisty opens. Disconnecting keeps them, and so does *Disconnect from Drive* in **Google Drive › Settings › Manage apps**, which only withdraws Tisty's access. Only **Delete hidden app data** there, or **Delete my copies in Google Drive for good** in Tisty, deletes them — and that space has no bin.

**Can I sync through OneDrive without Microsoft's program?**
Yes, since 1.31: **Settings › Sync › Your cloud service › OneDrive › Connect**. Tisty keeps its copies in *Apps › Tisty*, a folder you can see in your OneDrive; leave it to Tisty. See [Sync and backup](Sync-and-Backup#syncing-through-onedrive-directly).

**Can I sync through my own server?**
Yes, since 1.32, over WebDAV: Nextcloud, ownCloud, Synology and others. In **Settings › Sync › Your cloud service › WebDAV** you type the server's address, your user and a password, and Tisty checks the server before connecting; no browser opens. The password stays on this computer, kept by the system, and the copies go in a *Tisty* folder on the server. Only `https://` is accepted, except to this very computer. See [Sync and backup](Sync-and-Backup#syncing-through-your-own-webdav-server).

**Can I encrypt my cloud?**
Yes, since 1.36, for Google Drive, OneDrive or WebDAV: **Settings › Security › Encrypt my cloud…** encrypts what Tisty keeps there with a 24-word phrase only you know, so the provider can no longer read your tasks or documents. A folder kept in step by another program is not encrypted. Your other computers get the key on their own; a new one is let in with twenty digits typed on a computer that already has the key, or with your phrase. The provider still sees file names and sizes, how many computers you have, and earlier unencrypted versions for a while. Without the phrase or one of your computers, nobody can open it, not even us. See [Encrypting your cloud](Sync-and-Backup#encrypting-your-cloud).

**Tisty says something is blocking Google Drive.**
A proxy, a filter at work or an antivirus stands between this computer and Google, or inspects secure connections so Tisty cannot trust them. Your work stays safe on this computer meanwhile. On a work network, ask whoever runs it to let Tisty through; on your own computer, allow Tisty in your antivirus. Never go around your company's protection on your own. What each message means is in [Sync and backup](Sync-and-Backup#when-something-stands-in-the-way-of-google).

**One of my machines stopped syncing after an update.**
Update the one that is behind. A machine only reads what it understands, so the two stop syncing in both directions until they agree. A computer still on Tisty 1.30 to 1.33 stops and asks you to update instead of writing over a newer one. Nothing is lost, and what you write meanwhile stays on that machine.

## Comparing

**How is it different from Obsidian, Notion or Todoist?**
Tisty keeps tasks and documents in one store and one search, and finishing a task keeps everything it gathered instead of hiding it. The README has [a comparison table](https://github.com/rgdevment/Tisty#alternatives) that also says where each of the others is stronger.
