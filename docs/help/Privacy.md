# Privacy

Tisty collects nothing and keeps nothing of yours on a server. The one exception is yours to make: a document you share by link goes, sealed, to [Folio](https://github.com/rgdevment/Folio-by-Tisty), a separate service. No account, no telemetry, no analytics, no crash reports. That is not a promise you have to take on trust — it is a property of the code, which is [public](https://github.com/rgdevment/Tisty). Run a network monitor against it if you like.

This page is a summary. The full policy, including the parts that are not reassuring, is [PRIVACY.md](https://github.com/rgdevment/Tisty/blob/main/PRIVACY.md), and the security reasoning is in [SECURITY.md](https://github.com/rgdevment/Tisty/blob/main/SECURITY.md).

![Software out of conviction, not subscription: no account, no subscription, no telemetry, and your files stay on your computer](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/web/en-10-close.png)

## What is stored

Everything lives in one folder on your disk:

- **Tasks and events** — plain-text JSONL, an append-only log. Readable with `cat`, searchable with `grep`, parseable with `jq`.
- **Documents** — plain Markdown files.
- **Agents, if you use it** — your assistants' rules, skills and memory, as documents. If you sync it, Tisty also writes, outside this folder, the files each assistant reads ([what it reads and when](#your-assistants-in-agents)).
- **Earlier versions of your documents** — Markdown like the documents, in the `history` folder. Each is kept 30 days from when it was replaced, or what you choose in **Settings › Documents and signature › Keep earlier versions for**; the latest of each document, and what **Put back what it said before** would bring back, never run out. They stay on this computer: never synced or put in a parcel, but carried by your backups. [How they work](Documents#earlier-versions).
- **Your diary** — Markdown like any document, a book a year. No agent sees it until you open it from the Diary itself, on Tisty 1.48 or later; it is never shared by link, and it travels with your sync and your backups like the rest, and inside a `.tistyx` parcel when you export everything.
- **Attachments** — the files themselves, unmodified.
- **Brainsty's base** — how your things are tied, the weighty words of each document and task and which things are alike, in this computer's cache. It rebuilds itself from what you keep, so it is not synced and does not go in a parcel, to Folio or to an export. It does not go in your backups either. Hidden and dropped tasks, your diary and the documents in Agents leave no words there, and of the rest only the prose counts: not code, not paths, not a task's progress notes. [How it works](Brainsty-Network#alike).
- **An assistant's note on a document**, the one it leaves with `sum_up` for the next reader: one more event in the log, so it travels to your other computers with sync, to your cloud and to your backups. It never leaves through Folio or in a `.tistyx`, and the window does not show it. [What each tool does](MCP-Tools#documents).
- **Local settings** — including this machine's identifier and signing key. The private half of the key never leaves the machine.
- **Your phrase, if you make one** — the 24 words that encrypt your cloud, kept on the computer where you made them, in the system keychain on macOS or a file sealed for your user on Windows. Never synced, backed up or put in a parcel. Nobody from Tisty or Folio will ever ask you for it; anyone who does is trying to trick you.
- **What you share by link**: each link's key and the tokens that manage or edit it live in the log, so your other computers can manage it. They travel with your sync, so whoever can read an unencrypted copy can open those links. Once you stop sharing a document, the names of whoever edited it stay in the log too, so sharing it again can remind you who they were. How long Folio asked this computer to wait, and which documents it shared for the first time today, stay on this computer.
- **Reminders booked with the system** — the hour, the task's title and its identifier, for each reminder of the coming month, are handed to Windows or macOS so they ring with Tisty closed. They stay on this computer.
- **A `private/` folder** — anything placed there never leaves the machine, by any route.

Nothing is obfuscated, and nothing is in a format only Tisty can read.

**Nothing on your computer is encrypted at rest.** Your operating system's permissions are the protection. That is a decision, not an oversight: encryption would break the block-by-block merge of documents that syncing depends on. What Tisty keeps in a cloud it reaches directly is another matter: since 1.36 you can encrypt it with your phrase ([how it works](Sync-and-Backup#encrypting-your-cloud)). A folder kept in step by another program is never encrypted.

> [!WARNING]
> **An append-only log does not forget.** If a password ends up in a task, deleting the task removes it from what you see, not from the history underneath — and if it already reached the shared folder, it is on every machine that pulled it, plus your provider's version history. Keep secrets out, or in `private/`.
>
> A document does not forget at once either: a text you delete on purpose stays in its [earlier versions](Documents#earlier-versions), on each computer that held it when it was replaced. It leaves them once the version that holds it is past its limit and neither the latest nor what **Put back what it said before** would bring back, or when you empty its history or delete the document. Backups made in the meantime keep it: yours, and the last few Tisty writes on its own inside its private folder before a change, such as changing how you sync.

## Who can reach your data

| You choose | Who else can reach it |
| :-- | :-- |
| **Local only** (the default) | Nobody |
| **A folder your cloud client syncs** | That provider, under their terms |
| **Google Drive, connected from Tisty** | Google, under its terms. Tisty sees only its own hidden space there ([how it works](Sync-and-Backup#syncing-through-google-drive-directly)) |
| **OneDrive, connected from Tisty** | Microsoft, under its terms ([how it works](Sync-and-Backup#syncing-through-onedrive-directly)) |
| **Your own WebDAV server, connected from Tisty** | Whoever runs that server ([how it works](Sync-and-Backup#syncing-through-your-own-webdav-server)) |
| **Any of those three, encrypted with your phrase** | Nobody can read your files without your phrase or one of your computers. The provider still sees their names, sizes and how many computers you have, and may keep earlier unencrypted versions for a while |
| **A folder on your own hardware** (NAS, external drive) | Whoever can reach that hardware |
| **A document you share by link, through Folio** | Whoever has the link. Folio holds it sealed and cannot read it, but sees what [the full policy](https://github.com/rgdevment/Tisty/blob/main/PRIVACY.md#sharing-by-link) lists |
| **A backup zip** | Wherever you put it |
| **A `.tistyx` parcel you hand to someone** | Them, and whoever they hand it to |
| **A `.tistyx` locked with six digits** | Whoever has both the file and the number |
| **An assistant you let in** | Whatever service that assistant talks to |
| **What you keep in Agents** | The same as your other documents and, if you sync it, also each assistant that reads those files and whatever service it talks to, even if you never let it in |
| **A task you save to your calendar** | Google or Microsoft, under their terms; with Apple Calendar, the Calendar app on your Mac and whatever it syncs with |

## The network requests Tisty makes

**On its own:** when it opens and once a day after that, it downloads a small file to see whether a newer version exists. It sends nothing — no identifier, no task, no name. A Microsoft Store copy asks Windows what the Store has, and reads the Store's public catalog to see which version that is: when it opens, every six hours, and when you ask. It downloads that same file only if the catalog does not answer. While you sync with Google Drive, OneDrive or a WebDAV server, it also reads, at most once an hour, a small signed file on GitHub that can pause syncing or lower its limits; it carries nothing of yours. And when you open a document you shared that holds a language you kept from someone else, it asks Folio whether that language changed, at most every quarter of an hour.

**Only when you ask:**

- **Update** in **Settings › Help and about** downloads the release file and the installer, and Tisty refuses anything not signed with the key built into your copy.
- **Bring the preview** on a [link card](https://markdown.tisty.app/links) asks that page once for its title and picture.
- **The help** inside Tisty (the `?` at the foot of the sidebar) comes with your version and asks nothing. Its **Tisty's Markdown** guide (also `F1` while you write) reads it from [markdown.tisty.app](https://markdown.tisty.app), so it is always the latest; offline, it shows the copy that came with your version. Its pictures are downloaded from GitHub only when you press **Download** above the help.
- **Show them**, on the line above a Markdown file opened from outside, asks each address the file names for its picture, one request per picture. Nothing is fetched until you press it, it asks again each time you open the file, and it refuses an address on this computer or your own network. This is the viewer's alone: a document in your store draws no picture from the internet.
- **Opening a link**, with `Ctrl`/`⌘` + click while you write or a click while you read, opens your browser.
- **Save to my calendar**, on a task, opens Google Calendar's or Outlook's page in your browser with the title, the day and the hour, and the task's link in the address, and the event lands in your calendar only if you press **Save** there. Tisty does not make that request: your browser does, and the address stays in its history. With Apple Calendar, Tisty leaves a `.ics` file in its cache and the Calendar app opens it. The task's description and notes never go.
- **Share**, on a document, reads a small signed file on GitHub to learn whether [Folio](https://github.com/rgdevment/Folio-by-Tisty) is open. What you then confirm goes to Folio, encrypted on this computer, and so do Sync, Pause, Stop sharing and opening a shared link. Nothing is asked until you press.
- **Google Drive**, **OneDrive** or **your WebDAV server**, once you connect it from the window, is where Tisty sends and fetches your copies. Assistants never touch the network.
- **Checking a WebDAV server** — only when you connect it: Tisty signs in with what you typed and looks at its own folder there, and nothing else. Only `https://` is accepted, except to this very computer, and the password never travels to any other address.
- **Checking the way to OneDrive** works the same as the check for Google Drive below, with one request to `graph.microsoft.com` that carries nothing of yours.
- **Checking the way to Google Drive** — only while you connect Google Drive or move to it, Tisty sends one request to Google's API host, `www.googleapis.com`, with no account, no token and nothing of yours in it. It waits at most 10 seconds and goes through your system's proxy, never around it. It only lets Tisty tell you whether a proxy, a filter at work or an antivirus is in the way, or Google itself is having trouble. It never runs while syncing. When a round of syncing cannot reach Google, Tisty reads the failure the same way, sending nothing more, and keeps on this computer only a word for what stood in the way, how many rounds in a row failed and since when, to warn you once it lasts. A connection cut halfway through a transfer leaves no word at all.

A [widget](Widgets) runs nothing and names nothing outside itself. An attached `.html` file is only a file: Tisty never runs it.

## Your identity when you share

**What it is.** A pair of keys Tisty makes on this computer. One is secret and never leaves your computers; the other is public and only serves to check your signature.

**What it is for.** Tisty signs each document you share with it. Whoever opens it sees a fingerprint, a row of numbers that is always the same for you: that is how they know two documents come from the same person and that nobody altered them. You can sign with an alias; it does not have to be your name.

**What Folio sees.** Your document is encrypted on this computer before it leaves. The key that opens it travels inside the link, after the #, and that part never reaches Folio, so Folio cannot read the title or what you wrote. It sees your public key, to tell that two documents come from the same person, and never your name or your email.

**Your 24-word phrase.** It is your identity's backup: with it, your other computers sign as you. Keep it outside Tisty, on paper or in a PDF apart. If you lose the phrase and all your computers, the identity cannot be recovered. Nobody from Tisty or Folio will ever ask you for it.

**If you lose a computer.** Remove it in **Settings › Security**, under **Your computers**, with **Remove**. Its key stops counting, and Folio tells whoever reads what it signed after that.

## Your assistants in Agents

In the **Agents** folder you keep your assistants' rules, skills and memory as documents (Claude Code, Codex, GitHub Copilot, Gemini CLI and Cursor). With **Bring** you only copy them into Tisty; with **Sync**, Tisty also writes underneath the files each one reads. How to use it, file by file and assistant by assistant, is in [[Agents|AGENTS-Folder]].

**What it reads, and when**

- **Agents in grey in the sidebar, and its card in Settings › Assistants › Agents folder** before anything is brought, only check whether each assistant's folder exists, such as `~/.claude` or `~/.codex`, to tell you which ones it found. They look at nothing inside.
- **When you click Agents and the first-time notice opens**, Tisty only looks at which files are in your assistants' folders, with their names and sizes, to show you the list. It opens none of them, and those starting with a dot it does not even list. Closing the notice with the X or `Esc` reads nothing more.
- **When you go into Agents, also on a computer that has not given permission yet**, for example one where Agents came in brought from another computer, Tisty makes that same list, to know whether there is anything to ask you: names and sizes only, opening no file. **Look again** on that computer makes the same list to show you the first-time notice, and opens nothing before your **Bring** or your **Bring and sync** either.
- **With Bring or Bring and sync**, it opens only the files you ticked in that list and brings them in as documents. That permission is this computer's: each of your computers asks for its own.
- **With Bring, Tisty does not write to your agents' files**, nor note any as written by it: they stay as they are until you choose Sync. To show you what there is to bring, as in **Bring another file** or **Look again**, it looks again at which files are there, with their names and sizes. Making in Agents the subfolder of each assistant it finds touches no file either.
- **After that**, it opens only the files that belong to a document in Agents: to see whether they changed outside and, if it syncs, to write them. A **Global** document is written to every assistant Tisty finds on this computer, even one whose files you did not tick. A document that comes from another of your computers is not written until you press **Apply here** on this one, and only words you wrote or approved reach a file. A change made outside is never written over: it stays marked until you decide. In `GEMINI.md` it only adds one line, `@tisty-rules.md`; the rest of that file is Gemini's.
- **The memory an assistant writes itself** is brought in and shown, but Tisty never writes over it.

**What Agents never brings in**

- `settings.json` and your assistants' other settings files (`config.toml`, `hooks.json`, `mcp.json`…), except one a skill carries in its own folder, and no credentials file (`.credentials.json`, `auth.json`, `.env`, `.pem`, `.key` or `id_rsa` keys…). It skips them by name, without opening them. And a document in Agents is only written where each assistant keeps its rules, skills and memory, never to its settings or credentials files.
- A file that looks like it holds a token, a password or a private key is checked on this computer and left out, with a notice. The check looks for known shapes and may not recognise them all: keep secrets out of your assistants' rules.
- Nor does it follow symbolic links or bring in files over 512 kB.

**What travels and what stays**

- **Your documents in Agents travel like the rest**: with your sync, your cloud (encrypted, if you encrypted it), your backups and a `.tistyx` parcel that carries everything. An assistant you let in through MCP reads them like any document, but does not change them, move them or make one, nor put an `agent:` line on any document of yours: it can only propose a change, which waits apart until you approve it on the card ([how](AGENTS-Folder#a-proposal-from-your-agent)). The proposal travels with your documents, and nothing of it reaches your assistants before your approval.
- **While syncing, what you write in Agents reaches your assistants**: each one reads its file and sends it to the service it works with, even if you never connect it to Tisty. Whether it syncs belongs to each computer, and does not travel.
- **Stays on this computer**, in `data/agents/` inside Tisty's folder: this computer's permission, whether it syncs, which assistants you paused, which documents you applied or left out on this computer, a fingerprint of each file it wrote, and the backups. None of that travels with sync, backups or parcels.
- **Before writing over** one of your assistants' files, Tisty keeps the version that was there. Of each file it holds the original, backup #1, which is never deleted, and on top of it the last 20, and it leaves no copies in your assistants' folders.

**Stopping sync without losing anything**

With **Bring and sync** or **Sync**, sync stays on for this computer, and you can pause it for one assistant or stop syncing the whole folder, with **Stop syncing** on its right-click or in Settings › Assistants › Agents folder: no agent file is touched and no document is deleted. Not synced or hidden from the sidebar, Agents writes none of your assistants' files. Paused, Tisty neither opens nor writes those files on its own: they stay as the last write left them, and your documents stay in Tisty. A pause belongs to this computer. If you take a rule out of Agents while it syncs, Tisty asks whether to take its file away too: it deletes it only if you say yes, and keeps a backup on this computer first. If you take every document out, it first asks the same about each file, and then the folder leaves Tisty and Agents goes back to the sidebar in grey, nothing brought; what you chose to leave stays as it was. Tisty never empties an assistant's file, and never deletes one without asking you.

**Your assistants' hooks**

- Agents never touches your assistants' settings files. Hooks do, and only when you ask: in **Settings › Assistants › Connected**, **Add** tells you which file it will write and, with your yes, puts Tisty's hooks there, leaving the rest of that file as it is. **Remove** takes them out.
- With the hooks in place, when a session starts the assistant gets the tasks it left open and what moved. After each edit and each call to Tisty a mark is left in this computer's cache, without reading the conversation, to know whether a turn edited files without writing anything in Tisty. Those marks are not synced and never leave the computer.

## No AI

Tisty does not use AI or machine learning on your data, and sends nothing to any model. The natural language is deterministic rules on your machine.

The same holds for what [Brainsty](Brainsty-Network#alike) thinks is alike. When it ties «Pagar la cuenta del agua» to «Pay the water bill», it understands and translates nothing: it looks each word up in a fixed list of Spanish–English pairs, *pagar ≈ pay*, *agua ≈ water*, and counts the ones that match. That list comes from the [Open Multilingual Wordnet 2.0](https://github.com/omwn/omw-data), made by linguists, comes inside the installer and never leaves your computer or is looked up online. It learns nothing from what you write: it is the same for everyone and changes only with a new version of Tisty. Its authors and its licence are under **Settings › Help and about › Third-party notices**.

If you connect an assistant of your own through [[MCP|Assistants-and-MCP]], whatever it reads goes wherever that assistant goes. That door comes open on a new install, and **Stop it**, in Settings › Assistants › Door, closes it.
