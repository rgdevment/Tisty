# Settings

Settings is where Tisty fits itself to you: how it looks, how it tells you things, where it leaves your copies, who writes in it and how everything stands. I laid it out so that what needs something from you comes first, in **Overview**, and so that any setting turns up when you type its name, without going page by page. It works the same whether you work with your agent or on your own: nothing here asks you to connect an agent.

## Opening Settings

- **The gear** at the foot of the sidebar. **Back**, at the top left, or `Esc` take you back to where you were.
- **The command palette**: type «settings» and choose **Settings** (the shortcuts are in [Keyboard shortcuts](Keyboard-Shortcuts)).
- **From a notice**: the foot of the sidebar, when sync needs something, takes you to the page that sorts it out. When an agent can no longer find Tisty, the notice at the foot of the sidebar has **See and fix**, which opens **Agents**.

The gear and the palette always open it on **Overview**. On the left you have, in this order, **Back**, **Search a setting**, **Overview** and the ten pages in four groups:

| Group | Pages | What it holds |
|---|---|---|
| **This computer** | [Look and window](#look-and-window), [Tasks and reminders](#tasks-and-reminders), [Documents and signature](#documents-and-signature) | How Tisty looks and behaves on this computer, how you jot things down and how it tells you, and the name you sign with |
| **Your data** | [Sync](#sync), [Security](#security), [Copies](#copies), [Storage](#storage) | Where your copies travel, who you are on your computers, your backups and how much room it all takes |
| **Connections** | [Agents](#agents) | The door for your agent and the agents connected on this computer |
| **Tisty** | [Health](#health), [Help and about](#help-and-about) | The check of your history, the version you have and where to ask for help |

A page with something to attend to carries a number to the right of its name, the same one **Overview** counts: with **Security 1** and **Copies 1** you know, without opening anything, that there is one thing in each.

With the keyboard, `↑` and `↓` walk the column, `Home` and `End` jump to the first and the last page, and `Enter` opens the one you have marked: the arrows only move the focus, so passing over **Health** does not start its review until you open it.

Each page starts with its name and a line saying what it is for, for example **Copies**: «A backup is a file of yours, outside Tisty. Syncing does not replace it: the cloud also copies a mistake or a deletion to all your computers.».

## Overview

**Overview** tells you how your Tisty is doing at a glance: «How your Tisty is doing, at a glance. What needs something from you comes first.». It has up to three parts, top to bottom.

### Make Tisty yours

After you install, **Overview** offers five steps, each with **Go ›**, which takes you to the exact row:

1. **Choose where your copies go** — or stay on this computer only. Takes you to [Sync](#sync).
2. **Create your 24-word phrase** — your identity on all your computers. Takes you to **Your identity**, in [Security](#security).
3. **Make your first backup** — a file of yours, outside Tisty. Takes you to **Save a copy**, in [Copies](#copies).
4. **Put your alias on what you write** — what you sign your documents with. Takes you to **Alias**, in [Documents and signature](#documents-and-signature).
5. **Connect an agent** — optional. Takes you to the **Connected** tab of [Agents](#agents).

The title counts what is done, **Make Tisty yours · 1 of 5**, and each finished step is struck through with a ✓. Once the first four are done, the list goes by itself: the fifth is optional and does not hold it back.

**Example.** Marisol installs Tisty on a new laptop and, in the welcome, chooses **Only on this machine**. Settings opens on **Make Tisty yours · 1 of 5**: step 1 is already struck through, because staying on this computer is a choice too. **Go ›** on step 2 leads to **Security**, where Marisol creates the phrase and comes back: now it says **2 of 5**. A backup and the alias «Marisol» later, the list goes away, even with no agent connected yet.

### To attend to

Below, **To attend to · 2** gathers what needs something from you, each with an amber dot and its button, which takes you to the row where it is sorted out:

| What it says | Button | Takes you to |
|---|---|---|
| **Tisty 1.50.0 is out** (or **The Store has a new version**) | **Update** when Tisty can install it from here; otherwise **See** | Help and about |
| **Your cloud asks you to sign in again** | **See** | Sync |
| **Sync has something for you to look at** | **See** | Sync |
| **Sync is paused on this computer** | **See** | Sync |
| **Days without anything from your other computers** (three days or more) | **See** | Sync |
| **Your phrase is not confirmed yet** | **Confirm it** | Security › Your identity |
| **Jaime's laptop writes without you confirming its key** | **Confirm** | Security › Your computers |
| **Your last backup is from Mon, Oct 5 11:47** (30 days or more) | **Create backup** | Copies › Save a copy |
| **3 things to look at in your history** | **Review** | Health |
| **Quick capture was left without a shortcut** | **Choose another** | Tasks and reminders › Quick capture |
| **Claude Code points to a Tisty that no longer exists**, itself or its hooks; with several, a single line: **3 agents point to a Tisty that no longer exists** | **Fix** | Agents › Connected, in the box above the tabs |
| **Codex points to another Tisty**, one still on this computer | **Fix** | Agents › Connected, in the box above the tabs |
| **Tisty's skills are to be brought up to date in your agents** | **Fix** | Agents › Connected, in the box above the tabs |

When nothing is left, it says **Nothing needs your attention.** with a green dot. That line only shows if Tisty could read everything it checks on opening: if a read failed or is still on its way, it does not say it, because it cannot promise it. While the steps of **Make Tisty yours** are showing, it does not either.

Below there may be a line that asks for nothing: **Your agents can use Tisty's hooks and skills · Optional**, or just the hooks, or just the skills, with **See** and **Not now**. It shows with the door open, when an agent you already had connected lacks them and was not offered them when you connected it, and none has lost Tisty. It counts in no number and sits well with **Nothing needs your attention.** **See** takes you to the box in **Agents** where they are added; **Not now** takes it away from here and from there.

**Example.** Jaime comes back from holiday and opens Settings. **To attend to · 2** says **Your phrase is not confirmed yet** and **Your last backup is from Mon, Oct 5 11:47**, and in the column **Security** and **Copies** carry a 1. **Create backup ›** lands on **Save a copy**, with the row highlighted for a moment; Jaime presses **Create backup…**, keeps the file on an external drive and, back in **Overview**, one thing is left.

Each of these also shows on its row, as an amber **Attention** note (your agents', in the box in [Agents](#agents)): it is the same one, not another, and sorting it out clears it from both places and from the column's number at once. More in [Attention, on its row](#attention-on-its-row).

Some things are only known after a look: the computers waiting for your confirmation show once you have opened **Security** or **Health**, and what to look at in your history once you have opened **Health**. **Overview** does not start those checks on its own. Your agents do not wait for that: with the door open, Tisty looks at whether they point at this Tisty when its window opens and when Settings opens, writing nothing. What it reads is in [Privacy](Privacy#the-agents-you-connect).

### Status

**Status** has six tiles, one per page: **Sync**, **Copies**, **Security**, **Agents**, **Health** and **Help and about**. Each one says what matters with a coloured dot, and pressing it opens its page.

- **Green dot**: it is fine and Tisty knows it first-hand.
- **Amber dot**: something still works, but it is worth a look.
- **Grey dot**: it is off, there is nothing yet, or Tisty cannot vouch that it is fine.
- **«…» with a grey dot**: the fact has not arrived yet. A value Tisty did not read is never drawn.

| Tile | Green | Amber | Grey |
|---|---|---|---|
| **Sync** | **In Google Drive** (or your folder, your OneDrive, your WebDAV) with a round that ended well or is under way | Something asks for you, **nothing new since …** (three days with nothing from your other computers) or **Chosen by a newer Tisty** | **Off** · only on this computer; offline, at today's limit or waiting; or **the cloud's state could not be read yet** |
| **Copies** | **Last backup** with its date | The last backup is 30 days old or more | **No backup** · none yet |
| **Security** | **Phrase confirmed** | **Phrase not confirmed**, **Phrase on another computer**, or a computer waiting for your confirmation | **No phrase yet** · your identity is born with it |
| **Agents** | **Door open** · signs as «palmera 95» | **Door open** · 2 to fix: each agent that, itself or its hooks, can no longer find Tisty, and Tisty's skills to bring up to date | **Door closed** · what an agent asks is turned down |
| **Health** | **Everything in order** · checked 10:42 | **3 things to look at**, or **Some checks could not run** | **Not checked yet** · checked when you open Health |
| **Help and about** | **Tisty 1.49.0** · up to date, if Tisty checked for versions today and got an answer | **1.50.0 is out** · you have 1.49.0 | **no check for updates yet**, or **last checked …** when the last answer is not from today |

**Health** is the only one not checked when Settings opens: it shows the last check you made in this window, with its time, so you see how old it is.

**On another computer**, **Overview** speaks for that computer: each one keeps its own backups, its **Health** check and its agents, so on your desktop it may say **No backup** even though the laptop made one yesterday. Your phrase, though, is yours on all of them: on a computer that does not keep it, because you created it on another or because you reinstalled Tisty there, **Security** says **Phrase on another computer** · type it here to recognise this one.

## Search a setting

**Search a setting**, at the top of the column, finds any setting by its name, its description or words people use for it. You need not type it whole or with accents, and capitals do not matter: every word you type has to appear in the setting.

1. Click **Search a setting** and type, for example, «shortcut».
2. The page shows «shortcut» as its title, how many it found, **2 settings**, and one row for each with its page on the right: **Quick capture** · Tasks and reminders and **Quick idea** · Tasks and reminders.
3. Press a row: its page opens and the row is highlighted for a moment in the middle.

Results come in the order of the column, from **Look and window** to **Help and about**. Some examples:

| You type | It finds |
|---|---|
| backup | **Save a copy** and **Documents from another Tisty**, in Copies |
| language | **Language**, in Look and window, and **The welcome**, in Help and about |
| encrypt | **Your identity** and **Encrypt my data in the cloud**, in Security, among others |
| keyboard | **Quick capture** and **Quick idea**, in Tasks and reminders |
| folder | **Hidden folders**, **Where it syncs** and **Turn syncing off**, among others |

When nothing matches, it says «Nothing with those words. Try «backup», «shortcut» or «language».». A setting that exists but is not showing right now, such as **Your computers** in Sync while you do not sync, opens its page from the top, with nothing highlighted.

`Esc` inside the search box clears what you typed; with the box empty, it leaves Settings.

Search a setting searches Settings. To search this help, use the help's own search; for your tasks and documents, the window's.

## This computer

What you see and hear on the computer in front of you.

### Look and window

«How Tisty looks and what its window does on this computer.»

- **Look**: **Language** (English or Spanish; if you do not choose, Tisty speaks your computer's language), **Look** (**Light**, **Dark** or **Follow the computer**) and **Let Tisto peek in**, the beaver that peeks over the bar while you type a task.
- **This window**: **When you close the window**, for example **Leave it in the tray**, and **Start with the computer**, which opens Tisty tucked away, with no window, when you sign in. If Windows took Tisty's startup away, from Task Manager or from Settings → Apps → Startup, the row says «Tisty opens only when you open it.», the switch turns grey and the **Held by the system** note tells you where to turn it back on, with **Open Windows Settings**. While Windows keeps it off, Tisty does not ask **Open Tisty when you sign in?** either.
- **Hidden folders** («This computer only»): the folders you took out of the sidebar on this computer, each with **Show again**. They still keep everything, and search, the palette and «Move to» find them. If you hid none, it says «No folder is hidden.».

### Tasks and reminders

«How you jot things down without opening Tisty, and how it tells you a reminder is due.»

- **Quick capture**: **Quick capture** picks the shortcut that opens a small field over whatever you are doing, `Ctrl` + `Shift` + `Space` by default. What you type goes to Tasks, with no list. **Quick idea** picks another shortcut to jot straight into [Ideas](Documents#ideas); with no shortcut, `Tab` in the quick capture still turns it into an idea.
- **Notices**: **A notification from the system**, **A short tone**, **Important reminders** (a reminder may come through «Do not disturb» once you allow it in the system) and **Your calendar**, where «Save to my calendar» keeps a task with a date.

**Important reminders** depends on two things. With **A notification from the system** off, its switch turns grey and the **Missing** note says «Turn on «A notification from the system» first: without it no reminder reaches the system.», with **Turn it on**. Once it is on, the **Held by the system** note reminds you of the step left in Windows or macOS, with **Open the system settings**. If your system or your copy of Tisty does not allow it, the row says so instead of offering it.

If another program already uses your shortcut, Tisty takes another one and says so on that same row; if it is left with none, **Overview** asks you with **Quick capture was left without a shortcut**. Notices are prepared and shown on this same computer: none of this goes out to the internet. More in [Spread and reminders](Spread-and-Reminders).

### Documents and signature

«The name you sign what you write with, and how much of its past each document keeps.»

- **Who writes here**: **Alias** («Travels to your computers»), the name you sign what you write with, with **Sign the older ones** to put it on what you wrote before; and **Name to sign with**, how you appear in your PDFs. Each has **Learn more** with the long explanation.
- **Document history**: **Keep earlier versions for** («This computer only»), 1, 5, 15 or 30 days. More in [Earlier versions](Documents#earlier-versions).

**Example.** Lucas types «Lucas» as the alias on the desktop. Once it syncs, the laptop signs «Lucas» too, because the alias travels. If **Keep earlier versions for** is set to 5 days on the laptop, the desktop stays at 30: each computer keeps its own history.

## Your data

Your copies, your identity and your room. The three pages with something that cannot be undone, **Sync**, **Security** and **Copies**, keep it in their [Handle with care](#handle-with-care) zone.

### Sync

«Where Tisty leaves copies for your other computers to pick up.»

- **Where it syncs**: the state, for example «Off. Your tasks live only on this computer.», and **Turn on…**, which lets you choose **Your cloud service** (Google Drive, OneDrive or WebDAV, directly) or **Cloud through a folder or NAS**. Syncing with a folder: **Sync now**, **Open the folder**, when your last round was and the last thing that came from another computer; and under **Set-up:**, **Change folder** and, when this copy can connect a cloud, **Move to another place…**, with **Send everything again** inside **More options**. With a direct cloud, your cloud's card with what it says and what you can do.
- **Your computers**: while syncing, a row with **See in Security**, because the list of your computers lives in Security.
- **Handle with care**: with a folder, **Turn syncing off** (**Turn off**): this computer stops leaving copies and what is already in the folder stays there. With a direct cloud, **Disconnect this computer**, which lets go of your cloud here only; if your cloud no longer lets Tisty in, **Leave without Google Drive** (or without the cloud you use); and with Google Drive or OneDrive, **Remove Tisty's access on every computer**. Each one asks you first: see [What it asks you first](#what-it-asks-you-first).

Everything that happens on this page is in [Sync and backup](Sync-and-Backup).

### Security

«Your identity, the computers you recognise as yours, and your cloud's encryption.»

- **Your identity**: it is born with your 24-word phrase. **Create my phrase** creates it; **Keep my phrase on this computer** takes it and leaves it kept here, if you created it on another computer or reinstalled Tisty on this one. **See the phrase** shows it again on each computer that keeps it. If you only want to open your encrypted cloud, use **Open my cloud with the phrase**, further down, which does not keep the phrase.
- **Your computers**: one single list with every computer that writes where you sync, this one marked **This computer**. Each with its code and **See the code**; the others, with **Remove**. While syncing, below, **Rename** and **Show large** for this computer.
- **Encrypt my data in the cloud**: **Encrypt my cloud…**; with the cloud encrypted, **Let a new computer in**. On a computer that cannot open your encrypted cloud yet, the steps it is missing, with **Reconnect** and **Open my cloud with the phrase** ([how](Sync-and-Backup#syncing-again-on-a-computer)). Without a direct cloud, the row carries the **Missing** note: «Connect a cloud in Sync to encrypt what Tisty keeps there.», with **Go to Sync**. With the cloud encrypted, the **Handle with care** zone keeps **Delete the unencrypted copy** and **Decrypt my cloud**.

Step by step in [Confirming your machines](Sync-and-Backup#confirming-your-machines) and [Encrypting your cloud](Sync-and-Backup#encrypting-your-cloud).

### Copies

«A backup is a file of yours, outside Tisty. Syncing does not replace it: the cloud also copies a mistake or a deletion to all your computers.»

- **Save a copy**: **Create backup…** writes one file with your whole history, from every computer, with what it carries and when the last one was.
- **Documents from another Tisty**: **Export…** takes your documents to another Tisty and **Import…** brings in the ones somebody handed you. It is not a backup: importing a file twice makes a second copy.
- **Handle with care**: **Restore** (**Restore from a file…**) replaces everything on this computer with whatever the file carries, and does not show if you sync with Google Drive or OneDrive; **Leave Tisty** (**Leave…**) guides you to back up, disconnect the cloud and leave this computer clean, with your tasks and documents in their folder.

More in [Backups](Sync-and-Backup#backups).

### Storage

«Where your files live on this computer and how much room they take.»

- **Where it lives**: **Your files**, with its path and **Show folder**. It cannot be changed: always knowing where your files are is part of them being yours.
- **How much room it takes**: tasks, lists and events in your history, and how much the history, the documents and the attachments weigh.
- **Attachments**: **Where large attachments stay**, for those over 25 MB. With no shared folder, the picker turns grey and the **Missing** note says «There is no shared folder yet, so everything stays on this computer.», with **Go to Sync**.
- **Free up space**: **Attachments not opened in a while**, **Attachments nothing uses** and **The same file kept twice**, with **See which**. With a direct cloud, also **Cloud use today**. If there is nothing to let go of, it says «Nothing to free up right now».

More in [Large attachments](Sync-and-Backup#large-attachments) and [Privacy](Privacy).

## Connections

### Agents

«Tisty has no AI inside. Your agent brings it; Tisty is the memory you share.» The page has five tabs. With the door open you come in on **Connected**; with the door closed, on **Door**:

- **Door**: **An agent can file work here**, with **Stop it** to close it (and **Let an agent file here** to open it again), **Ask it to keep its task up to date** and **Remind it when a turn starts and ends**.
- **Connected**: **Agents on this computer**, with **Look again**, a search box for the ones Tisty knows and did not find, **Wrote in Tisty from elsewhere**, the **Tisty's skills** row, with **See**, and **How to connect one**, folded, to connect it by hand. Each row says how that agent stands, **Connected**, **Not connected**, **Points at another Tisty**, **May be here**, **Set up your own way**, **Could not be read** or **By hand**. Under each connected agent that takes them, the **Hooks: when a turn starts and ends** row, with **Add** or **Remove**, and the **Tisty's skills** row, with its standing; and when you connect one, once, a box that offers its hooks and the skills. [Hooks and skills](Hooks-and-Skills) tells it.
- **Agents folder**: the **Agents** card, to bring and sync your agents' rules, skills and memory. More in [Agents: rules, skills and memory](AGENTS-Folder).
- **How its signature looks**: **Who signs what an agent writes**, with a **Preview of a row** that changes at once, **Only a diamond**, **Name the agent in the task list** and **Name the agent inside a task**.
- **What it can and cannot do**: **What it can do** and **What it can never do**, side by side.

**The box above the tabs.** With the door open, what happens with your agents shows on top, whichever tab you are on:

- **2 agents to fix**: the ones that point, themselves or their hooks, to a Tisty that is no longer on this computer. Each one says what is fixed (**connection**, **hooks** or **connection and hooks**), what it points at today and, under **It writes in:**, the files that will be touched. If Tisty's skills are to be brought up to date, they go right there with their folders. **Fix all**, or **Fix** if there is a single thing, does it, and in its place it leaves **Fixed**, with a ✓ for each one, or **Some could not be fixed**, with the reason.
- **Codex points to another Tisty**: «That Tisty is still on this computer. To bring one here, use Fix on its row.». **Fix all** does not touch it, and **Not now** takes away the box and the line in **Overview**.
- **Hooks and skills for your agents**: the optional part, in one step. **Add hooks and turn on skills** first shows you two ticks, with the files and folders that will be written, and only writes what you left ticked when you press it again. What you untick is not offered again; with **Not now**, nothing is. The usual rows are still there.

**Example.** Lucas had Tisty from the installer and now installs it from Microsoft Store and uninstalls the other one. On opening it, the foot of the sidebar says **2 agents can no longer find Tisty** · «The Tisty they point to is no longer on this computer.». Lucas presses **See and fix** and lands on the box: **Claude Code · connection and hooks**, with `~/.claude.json` and `~/.claude/settings.json`, and **Codex · connection**, with `~/.codex/config.toml`. After **Fix all**, the box reads **Fixed**, and what is left is to quit both agents completely and open them again. One by one, it is in [When an agent can no longer find Tisty](Agents-and-MCP#when-an-agent-can-no-longer-find-tisty).

With the keyboard, `←` and `→` change tabs, and `Home` and `End` jump to the first and the last.

With the door closed, the page reminds you where it matters. In **Door**, **Ask it to keep its task up to date** and **Remind it when a turn starts and ends** turn grey with the **Missing** note: «These two settings act while the door is open.». In **Connected**, if you have agents connected, **Agents on this computer** says «Even so, they file nothing until you open the door.», with **Go to Door**. And in **How its signature looks**, with **Only a diamond** on, the two name switches turn grey: «The two above stay off while «Only a diamond» is on.».

**Stop it** asks you before closing: «No agent will be able to file in Tisty.», everything it wrote stays where it is, and connected agents stay connected, but whatever they ask is refused. It needs no tick, because **Let an agent file here** opens it again.

If you work without an agent, this page can wait: the open door does nothing while no agent calls. Everything else in [Agents and MCP](Agents-and-MCP).

## Tisty

### Health

«Tisty checks your history, your documents and your attachments when you open this page.»

- At the top, the result, for example **Everything in order · checked 10:42**, with **Check again**. If it finds something, **To look at** lists it with what you can do in each case, such as documents missing here or a computer signing with another key.
- **Tools**: **Quick index**, with **Rebuild the quick index**, which reads your history again; nothing you wrote changes.
- **If something goes wrong**: **Report a problem**, with **Prepare…**, puts together a file with what it takes to understand a fault, which you read before you share it. Nothing is sent.

### Help and about

«The version you have, how it updates, and where to ask for help.»

- The version, the licence and what Tisty is. **Check for updates**, and **Update** when there is a new version; **Take release candidates**, to try versions before they are finished.
- **Help**: **The welcome**, with **Show it again** (it asks again about language, where your copies go, whether it starts with the computer and what closing the window does, without touching what you wrote) and **Open the guide**; and **Help on tisty.app**, with **Open ↗**.
- **Support Tisty**, with **Star on GitHub**, and **Other tools**.

When there is a new version, its row carries the **Note**: «This notice also shows in the Overview and as a dot in the bar, not only here.». If **Check for updates**, **Update** or **Take release candidates** fail, the error shows on that same row, with its cause. With no connection, for example, **Check for updates** tells you Tisty could not reach its list of versions. The row says «You are on the newest version» only after an answer from today; otherwise it says «No check for updates has been answered yet» or the day of the last one.

On the Microsoft Store copy, the Store keeps Tisty up to date: when it offers a version, **Update** asks it for it; when the Store is already bringing it on its own, **Overview** lets you **See** how it goes. There, **Take release candidates** carries the **Doesn't apply here** note: «The Microsoft Store keeps this copy up to date and offers no release candidates.».

## Handle with care

What cannot be undone, or changes a lot at once, lives in a red box at the end of its page, **Handle with care**, away from the everyday rows:

| Page | In Handle with care |
|---|---|
| **Sync** | With a folder, **Turn syncing off**. With a direct cloud, **Disconnect this computer**; **Leave without Google Drive** when your cloud no longer lets Tisty in; and, with Google Drive or OneDrive, **Remove Tisty's access on every computer** |
| **Security** | **Delete the unencrypted copy**, while that copy is left, and **Decrypt my cloud**, with the cloud encrypted |
| **Copies** | **Restore**, unless you sync with Google Drive or OneDrive, and **Leave Tisty** |

Its buttons are red, and each row says in one line what its button does. If the page has nothing delicate at the moment, for example **Sync** with syncing off, it does not show the zone.

**Example.** Yisset stops using an old laptop. Before wiping it, Yisset opens **Copies**, makes a backup with **Create backup…** and only then goes down to **Handle with care** and presses **Leave…**: Tisty guides the way to back up, disconnect the cloud and leave that computer clean, and the tasks and documents stay in their folder.

### What it asks you first

No red button does anything on the first click. It opens a window with its name, what will happen in a short list, **Cancel** and the button that does it. When there is no way back, that button stays grey until you tick **I understand this can't be undone**, and the box starts unticked every time the window opens.

| You press | The window says | Button | Tick |
|---|---|---|---|
| **Turn off** | **Turn syncing off**: «Your tasks will live on this computer only.» What is already in the shared folder stays there, and your other computers stop getting what you write here. | **Turn off** | No |
| **Disconnect this computer** | «This computer stops reaching Google Drive; your other computers keep syncing.» Your copies stay in your Google Drive, and you can connect it again whenever you like. | **Disconnect** | No |
| **Remove Tisty's access on every computer** | «Google takes Tisty's access away from this account on every computer at once.» Your copies stay in your Google Drive, and each computer will have to connect again. | **Remove access** | No |
| **Leave without Google Drive** | «Google Drive can no longer be reached from this computer.» Tisty keeps your tasks and documents here and syncs with nothing. Before asking, it counts the attachments that stay only in the cloud, for example «3 attachments are only in Google Drive and stay there, out of Tisty's reach.», or tells you «Every attachment is already on this computer.». | **Leave without Google Drive** | Yes |
| **Delete the unencrypted copy** | **Delete the old unencrypted copy?**: «Only files whose new copy is already in Google Drive are deleted.» It may take a few minutes, and you can keep working. This can't be undone. | **Delete the copy** | Yes |
| **Decrypt my cloud** | «Google Drive will see your data unencrypted again. Your phrase stays yours.» It takes a few minutes, and you can keep working. | **Decrypt** | No |
| **Restore from a file…**, after choosing the zip | **Restore from a file**: «Restore goes back to that moment, and everything after it is lost.» It replaces everything on this computer, and this computer stops sharing its folder: choosing it again asks you which side wins. | **Restore** | Yes |
| **Leave…** | The usual guided dialog, step by step: the backup, the cloud, what Tisty left on this computer and what is left in your hands. | **Close Tisty** | No |

Four buttons outside the zone ask the same way, on their own row:

- **Remove**, on a computer in **Security › Your computers**: **Remove this computer?** For example, «Jaime's laptop can no longer write in the shared folder.»; what it already wrote stays there and every computer still sees it; if it comes back, it joins as a new computer, backed up and emptied first; and when it last wrote. Button **Remove the computer**, with a tick.
- **Stop it**, in **Agents › Door**: see [Agents](#agents). No tick.
- **Move 5 to the bin · 41 MB**, in **Storage › Attachments nothing uses**: **Move 5 attachments nothing uses to the bin?**, with «Those on this computer stay there for 30 days.» and the note that those kept only in the folder you share are deleted from it, and your cloud keeps them in its own bin. No tick.
- **Delete for good**, in **Health › To look at**: **Delete «Recipes» for good?**, with «No copy is left here.» for a loose document, or «The document and what it holds go for good, on every computer.» for a missing one. With a tick.

What can be undone, or is a step inside a guided dialog, keeps a simple question: sending everything again, cancelling a half-done encryption, moving a single attachment to the trash and deleting the old cloud's copies when you move.

![The Restore from a file window, with its three lines, the I understand this can't be undone box unticked and the Restore button still grey](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/settings/en-02-restore.png)

**Example.** Lucas deleted a whole list by mistake and wants to go back to Friday's backup. In **Copies**, Lucas presses **Restore from a file…** and picks the zip. Tisty shows **Restore from a file** and the **Restore** button stays grey: what was written since Friday is lost and, since this computer syncs with a folder, it stops sharing it until the folder is chosen again. After thinking it over, Lucas ticks **I understand this can't be undone** and restores. Pressing **Cancel** would have changed nothing.

## What each row tells you

What happens when you touch something shows right below the row that caused it, never at the top of the page, so you always know what it refers to. If you use a screen reader, it announces it as soon as it appears.

| What you see | What it means |
|---|---|
| **Saved**, in green | Tisty has saved the change. It goes away by itself in a couple of seconds; a longer line, such as «Ready the next time you sign in», stays a few seconds more. |
| **Saving… 3 s**, or **Syncing…**, **Saving the backup…**, **Restoring…**, **Checking…** | That row is working, and the seconds count how long it has taken. It only shows if it takes a while, and only on that row. |
| **Waiting for «Backup» to finish.** | Another row is working with your data, and this one waits its turn with its button grey. |
| **Error**, in red | Something failed, and the note says why. When it can be repeated, such as a backup, a restore, a review, encrypting or preparing a report, it brings **Try again**. |
| **Missing**, with a dotted border | This setting depends on another one that is off or does not exist yet. The note's button takes you to fix it. |
| **Held by the system**, with a dotted border | Windows or macOS decides it, not Tisty. The note says where to change it and, when it can, opens that screen for you. |
| **Doesn't apply here**, with a dotted border | On this copy of Tisty that setting makes no sense, and the note says why. |
| **Attention**, in amber | The same thing **To attend to** counts, here on the row where it is sorted out. |
| **Note**, in blue | Context that asks nothing of you. |

A grey switch is never broken: it always has the note below that says why and, when it can, the button that frees it.

![The Tasks and reminders page, with Important reminders grey and below it the Missing note asking to turn on A notification from the system, with the Turn it on button](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/settings/en-01-notes.png)

| Page › row | Note | When | What it says | Button |
|---|---|---|---|---|
| Look and window › **Start with the computer** | **Held by the system** | Windows took Tisty's startup away | «Windows holds this switch now. Turn Tisty back on under Settings → Apps → Startup.» | **Open Windows Settings** |
| Tasks and reminders › **Important reminders** | **Missing** | **A notification from the system** is off | «Turn on «A notification from the system» first: without it no reminder reaches the system.» | **Turn it on** |
| Tasks and reminders › **Important reminders** | **Held by the system** | It is on and still needs allowing in the system | On Windows: «Under Settings → System → Notifications → Tisty, allow important notifications. Windows may also ask you in a notice the first time: allow it there too.» On a Mac, the step in System Settings and Focus | **Open the system settings** |
| Security › **Encrypt my data in the cloud** | **Missing** | There is no direct cloud | «Connect a cloud in Sync to encrypt what Tisty keeps there.» | **Go to Sync** |
| Storage › **Where large attachments stay** | **Missing** | There is no shared folder | «There is no shared folder yet, so everything stays on this computer.» | **Go to Sync** |
| Agents › Door › **Remind it when a turn starts and ends** | **Missing** | The door is closed | «These two settings act while the door is open.» | — |
| Agents › Connected › **Agents on this computer** | **Missing** | There are agents connected and the door is closed | «Even so, they file nothing until you open the door.» | **Go to Door** |
| Agents › How its signature looks › **Name the agent inside a task** | **Missing** | **Only a diamond** is on | «The two above stay off while «Only a diamond» is on.» | — |
| Help and about › **Take release candidates** | **Doesn't apply here** | The Microsoft Store copy | «The Microsoft Store keeps this copy up to date and offers no release candidates.» | — |
| Help and about › the update | **Note** | There is a new version | «This notice also shows in the Overview and as a dot in the bar, not only here.» | — |

### One thing at a time with your data

Everything that reads or rewrites your history, the shared folder or your cloud goes one thing at a time: syncing, backing up and restoring, encrypting and decrypting, your phrase and your computers, taking and bringing documents, the **Health** review, rebuilding the quick index, freeing space, large attachments, the **Handle with care** buttons and **Leave Tisty**. While one works, the others say **Waiting for «…» to finish.** with the name of the one at work, and their buttons wait in grey.

The everyday things stay free: language, look, Tisto, what closing the window does, starting with the computer, the shortcuts, the notices, your calendar, your alias, your name for signing, document history, the door and your agent's signature, the guide and the help.

**Example.** Jaime presses **Create backup…** and picks an external drive. After a moment, **Saving the backup… 4 s** shows under **Save a copy**, and in **Handle with care** **Restore** says «Waiting for «Backup» to finish.», with its button grey. Meanwhile Jaime goes to **Look and window** and picks **Dark**: **Saved** shows at once, because the look does not touch the data. If the drive comes loose halfway, the backup row shows **Error** with the cause and **Try again**; with the drive plugged back in, Jaime tries again from there.

### Attention, on its row

What **To attend to** gathers in **Overview** also shows on the row where it is sorted out, in amber, with the title in bold and, when needed, the why below:

| Page › row | Title | The why |
|---|---|---|
| Sync › **Where it syncs** | **Days without anything from your other computers** | «They may be off, or the program that uploads this folder may not be running.» |
| Security › **Your identity** | **Your phrase is not confirmed yet** | «Type it once to check you kept it right. Without that you can't encrypt your cloud.» |
| Security › **Your computers** | **Jaime's laptop writes without you confirming its key** | «It hasn't said what it signs with. Nothing it writes is checked until you confirm it.» |
| Tasks and reminders › **Quick capture** | **Quick capture was left without a shortcut** | «Another app already uses them. Choose another combination.» |
| Copies › **Save a copy** | **Your last backup is from Mon, Oct 5 11:47** | — |
| **Health**, at the top | **3 things to look at in your history** | — |

With a folder, **Where it syncs** also shows there whether your cloud asks you to sign in, whether there is something to look at or whether syncing is paused; with a direct cloud, your cloud's card says so, with its button. The update does not repeat its notice: its row already is the offer, with the **Note**. Your agents' do not go on a row either: they show in the [box above the tabs](#agents) of **Agents**.

**Example.** Marisol presses **Confirm it ›** in **Overview** and lands on **Your identity**, with the amber note **Your phrase is not confirmed yet** and, below it, «Type it once to check you kept it right. Without that you can't encrypt your cloud.». Once the words are typed, the note leaves the row, **To attend to** and the 1 next to **Security** at once.

## This computer only and Travels to your computers

Some settings carry a grey label beside their name, so you know what happens on your other computers without trying it:

- **This computer only**: the setting applies here and nowhere else. **Hidden folders** and **Keep earlier versions for** carry it: hiding a folder on the laptop does not hide it on the desktop.
- **Travels to your computers**: the setting reaches your other computers when you sync. **Alias** carries it: you type it once and they all sign the same.

The labels go where the answer is not obvious. What travels with your copies and what stays, in full, is in [Sync and backup](Sync-and-Backup).

## How it works

Every page but **Overview** has **How it works ↗** at the top right. It opens this help on that page's article, and sometimes on the exact section: in **Security**, for example, it opens [Encrypting your cloud](Sync-and-Backup#encrypting-your-cloud); in **Copies**, [Backups](Sync-and-Backup#backups); in **Look and window** and in **Health**, their section of this article. In **Help and about** it opens the start of the help. The help comes inside Tisty, so it opens offline too, and it suggests other articles for that same page.

**Learn more**, under some rows, is something else: it unfolds the setting's long explanation right there, without leaving the page.

## Where each thing went

If you came from an earlier version, when Settings had the pages General, Sync, Your data, Maintenance and About:

| Before | Now |
|---|---|
| General › Language, Look, Let Tisto peek in | Look and window › Look |
| General › When you close the window, Start with the computer | Look and window › This window |
| General › Hidden folders | Look and window › Hidden folders |
| General › Quick capture, Quick idea | Tasks and reminders › Quick capture |
| General › Notices and Your calendar | Tasks and reminders › Notices |
| General › Alias, Name to sign with | Documents and signature › Who writes here |
| General › The welcome, Open the guide | Help and about › Help |
| Sync | Sync |
| Sync › Your computers | Security › Your computers, one single list |
| Sync › Where large attachments stay | Storage › Attachments |
| Your data › Backup, Restore, Documents from another Tisty, Leave Tisty | Copies |
| Your data › Document history | Documents and signature › Document history |
| Your data › Your files | Storage › Where it lives |
| Maintenance › the check, To look at, the quick index, Report a problem | Health |
| Maintenance › Free up space and the figures | Storage › Free up space and How much room it takes |
| About | Help and about |
| Agents, on a single page | Agents, in five tabs |

## When something does not look like this

- **Just installed**: **Overview** starts with **Make Tisty yours** and nearly every tile grey. That is expected: grey means «nothing yet», not «something is wrong».
- **Not syncing**: **Sync** shows only **Where it syncs** with **Turn on…**; **Your computers** and its **Handle with care** zone show up once you sync.
- **A read that fails**: the tile stays on «…» or grey, and **Overview** does not say **Nothing needs your attention.** until it can read everything. Going back to **Overview** reads it again.
- **Another computer**: each computer has its own **Overview**, backups and check; what carries **Travels to your computers** arrives when you sync.
- **The Microsoft Store copy**: updates come from the Store. While it has not said which number it brings, **Overview** says **The Store has a new version**.
- **An agent**: no agent can touch Settings. **What it can and cannot do** says so: «Touch syncing, settings or machines» is among what it can never do.
