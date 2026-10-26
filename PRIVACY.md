# Privacy policy

**Last updated:** October 26, 2026

## The short version

Tisty does not collect anything and has no server to transmit to. No account,
no telemetry, no analytics, no crash reporting.

It makes **one** network request on its own — when it opens, and once a day
after that: it downloads a small file to see whether a newer version exists.
It sends nothing. Two more follow only if you press the button that offers you
an update, one more only if you ask a link in a document for its preview, and
one more when you open the guide inside Tisty. Never otherwise. A copy
installed from the Microsoft Store makes no update request: it asks Windows
what the Store has, and the Store does the rest. All of them are described in
full below.

Since version 1.30, Tisty can also sync through your own Google Drive, since
1.31 through your own OneDrive, and since 1.32 through a WebDAV server of your
own, talking to it directly. It runs only if you choose it and connect your
account in the Tisty window, it reaches nothing but that provider or that
server, and what it sends and receives is described in full below.

Since 1.36, what Tisty keeps in any of those three can be encrypted with a
24-word phrase only you hold, so the provider stores files it cannot read. That
too runs only if you turn it on, and what it hides and what it does not is
described below.

That is not a policy promise you have to take on trust — it is a property of
the code, which is [public and auditable](https://github.com/rgdevment/Tisty).
Run a network monitor against it if you like.

## The part that is not "everything stays local"

Most privacy policies for local applications stop at "your data never leaves
your machine". Tisty's cannot, because **Tisty is designed to synchronise**,
and being vague about that would be dishonest.

Here is the accurate statement:

> **Tisty never sends your data anywhere on its own initiative. If you turn on
> synchronisation, your data goes exactly where you told it to go, and whoever
> runs that destination can technically read it.**

There is one exception, and you choose it: a cloud Tisty reaches directly can be
encrypted with your phrase. Its provider then holds your files but cannot read
what is in them.

What you can choose, and what each implies:

| You choose | Who else can reach your data |
|---|---|
| **Local only** (the default) | Nobody. It never leaves the machine |
| **A folder your cloud client keeps in step** (Google Drive, OneDrive, iCloud, Dropbox, pCloud…) | That provider, under their terms and their jurisdiction |
| **Your own Google Drive, reached by Tisty directly** (since 1.30) | Google, under its terms and its jurisdiction. Nobody else: there is no server of ours in between |
| **Your own OneDrive, reached by Tisty directly** (since 1.31) | Microsoft, under its terms and its jurisdiction. Nobody else: there is no server of ours in between |
| **Your own WebDAV server, reached by Tisty directly** (since 1.32) | Whoever runs that server — you, or the company that hosts it for you. Nobody else: there is no server of ours in between |
| **Any of those three clouds, encrypted with your phrase** (since 1.36) | Nobody can read what is in your files without your phrase or one of your computers. The provider still holds them, sees their names, sizes and how many computers you have, and may keep earlier unencrypted versions for a while |
| **A folder on hardware you own** (a NAS, an external drive) | Whoever can reach that hardware |
| **A backup zip you keep somewhere** | Wherever you put it |
| **A `.tistyx` parcel of documents you hand to somebody** | Them, and whoever they hand it to. You decide what goes in it and when. It carries the documents, where they sat, what the archive held and what an assistant marked — never your device identifier |
| **A `.tistyx` parcel locked with a number, for another machine of yours** | Whoever has both the file and the number. Without it nothing here opens it — though six digits are six digits, and the lock only makes each guess expensive |

With a folder, Tisty only ever reads and writes local paths. It has no network
code for it, no credentials, and no idea which provider — if any — is behind the
folder you named. Whatever keeps that folder in step between your machines is
software you already installed and already trust. Reaching your cloud account
directly is the one way that changes this, and each provider is described in
its own section below. Encrypting is offered only for those clouds, never for a
folder.

There is no option where Tisty hosts anything. There is no paid tier that
changes this. **We never operate a server that holds your data**, so there is
nothing on our side to breach, subpoena, sell, or lose.

Choosing a destination is choosing who you trust. Tisty makes that choice
visible instead of making it for you.

## What is stored, and how

- **Tasks, projects and events** — plain-text JSONL files. Readable with
  `cat`, searchable with `grep`, parseable with `jq`.
- **Documents** — plain Markdown files. The log keeps a note of each one: its
  title, its size, and the tags it carries, all three read out of the body you
  wrote. A tag is a word you typed with a hash against it, so nothing is
  recorded that you did not write in the document itself, and taking the file
  elsewhere takes its tags with it.
- **Attachments** — the files themselves, unmodified. The log keeps a note of
  each one: the name it is kept under (a lowercased form of the file's name
  with a short fingerprint), its size and fingerprint, how it came in (dropped
  in the window, attached by an assistant, imported beside a document, from a
  parcel or the guide) and, when it is retired, whether it went unused or was
  joined with a copy of itself. Never the folder it came from.
- **Local configuration** — including the device identifier. The *file* never
  leaves this machine on its own: it lives outside the roaming `%APPDATA%`, and
  should a roaming profile or a copied home carry it to another computer, Tisty
  sees it is somewhere else and stops speaking as this machine. The
  identifier itself does appear in the shared folder — it names your device's
  directory and stamps every event — because that is what tells two writers
  apart. What must never be shared is the file that binds it to this machine.
  Beside it, in `private/`, sits the key this machine signs what it writes
  with: its public half is written into the log so your other machines can
  check the signatures, and the private half is never synced, backed up or put
  in a parcel. Which of your other machines' keys you have confirmed is kept on
  this machine alone.
  It also holds the day this copy first opened, whether it has already asked
  you to star the project, and whether it has already offered your assistant a
  door, so that each of those is asked once and never again. Which assistants
  are on this machine is read when the offer is decided and is not written down.
  The day this copy opened is, with the time zone below, the only field here
  that says anything about you rather than about your work, and none of them
  leaves this machine: the file is not synced, not backed up, and not put in a
  parcel.
- **The version each computer runs** — since 1.34, each machine writes into the
  log the Tisty release it runs, once each time it changes. It travels with the
  log like everything else, and it is what lets Tisty tell you which of your
  computers needs an update before your cloud can be encrypted.
- **Your identity and its phrase, if you create one** — the 24-word phrase is
  made on your computer the first time you ask for it in Settings › Security,
  and it never leaves that computer through Tisty: it is kept in the system
  keychain on macOS and, on Windows, in a file sealed with DPAPI for your user,
  apart from the cloud tokens. It is never written into the log, a backup or a
  parcel. What the log does carry is its public half, which Settings › Security
  shows as your fingerprint, and for each of your computers a key that half
  vouches for. Whether you confirmed the phrase, and the key of an encrypted
  cloud that another of your computers handed to this one, stay on this
  computer.
- **An alias, if you write one** — it exists so that a document you share still
  says who wrote it. It is optional, it lives in the log like everything else,
  and so it reaches your other machines through the folder you sync and travels
  inside any parcel you hand out. It goes out with the sync before you share
  anything, which is why it should be a name you are content for others to read.
  A log only ever grows: an alias you write today can be changed tomorrow, but
  the old one stays written. Your real name and your address are **not** asked
  for anywhere, and the two fields reserved for them in the log are written by
  nothing in this version.
- **The time zone you were in** — every event carries the IANA zone of whoever
  wrote it, so an hour reads back where it happened instead of drifting when you
  travel. It goes to the shared folder with the rest, and a long enough history
  shows where you have been. It is named here because it is the one field that
  says something about you rather than about your work; everything else in that
  folder is already your own words.
- **A `private/` folder** — anything placed here never leaves the machine
  under any transport.
- **The guide** — the one thing Tisty writes for you rather than the other way
  round. Its words and its images ship inside the program and are copied into
  your documents in the language you picked. Nothing is fetched to do it, and
  once it is there it is an ordinary document: edit it, or delete it.

**Nothing is encrypted at rest on your computer.** Your operating system's file
permissions are the only protection for what sits on your disk. This was a
deliberate decision, not an oversight: encryption breaks the three-way merge of
documents, which is precisely where synchronisation needs it most — two machines
editing the same document are reconciled block by block, and that cannot be done
on bytes nobody can read. The reasoning is in [SECURITY.md](SECURITY.md).

**The copy in your cloud can be.** Encrypting your cloud changes only what the
provider holds: each of your computers decrypts what it brings down and
reconciles it on its own disk, in plain text, exactly as before. A folder kept in
step by a desktop client is never encrypted.

**Nothing is obfuscated.** If Tisty disappeared tomorrow, your data would
still be readable with tools you already have. That is the point.

## One consequence worth knowing before it bites you

**An append-only log does not forget.** If a credential ends up in a task,
deleting the task removes it from the current state but not from the history
underneath — and if it already travelled to the shared folder, it is on every
machine that pulled it, plus whatever the provider keeps in its own version
history.

The `private/` folder exists for exactly this. Prevention is the only real
defence, because the remedy barely exists.

## What Tisty does not do

- Does not send data to any server.
- Does not create accounts or profiles.
- Does not use cookies, analytics, advertising, or tracking of any kind.
- Does not share anything with third parties.
- Does not use AI or machine learning on your data. The natural-language
  parser is deterministic rules running locally, and Tisty itself sends nothing
  to any model. If you connect an assistant of your own, read "the door you can
  open" below: what it carries is its doing and your choice, never Tisty's.
- Does not upload crash reports.
- Does not make background network calls, except the update check below and,
  only if you connect Google Drive, OneDrive or a WebDAV server, the requests
  that sync with it.

An earlier version of this file promised that if Tisty ever needed a network
request it would be read-only, documented here before it shipped, and visible
as a code change first. That is what happened: the check went into the public
repository, and this section was written before the release that carries it.

## The request Tisty makes on its own

| | |
| :--- | :--- |
| **Why** | To tell you a newer Tisty exists |
| **What it asks for** | `https://raw.githubusercontent.com/rgdevment/Tisty/manifest/release-manifest.json` |
| **Method** | GET. Nothing is uploaded |
| **What it sends** | The headers a request cannot avoid, and a user agent that says `tisty/<version>` — the same thing the download itself would reveal. No identifier, no store, no task, no name |
| **How often** | When Tisty opens, and at most once a day for as long as it keeps running |
| **Timeout** | 5 seconds |
| **If it fails** | Nothing is said and nothing is retried: the next look is the next day, or the next time Tisty opens |

The file it downloads contains version numbers and nothing else — no address, so
nothing that arrives from it can send you anywhere. It sits in a branch of its
own that holds those files and nothing else, which is why the address above
stays the same however many versions come out.

If you turn on release candidates in *About*, this same request is what carries
them: nothing else changes, and nothing more is sent.

## If Tisty came from the Microsoft Store

That copy asks GitHub nothing. It asks Windows what the Store has for it,
through the system's own Store interface, and Windows is what talks to
Microsoft — with the identity that machine already uses for every other app it
installed from there. Tisty adds nothing of its own: no identifier, no store, no
task, no name. It is the question Windows asks on its own schedule anyway; the
button only asks it now.

| | |
| :--- | :--- |
| **Why** | To tell you a newer Tisty exists, and to install it if you say so |
| **What it asks** | Windows, through `StoreContext`. No server of ours is involved |
| **What it sends** | Nothing of Tisty's. What travels is what the Store already sends for any app installed from it |
| **How often** | When Tisty opens, every six hours for as long as it keeps running, and whenever you ask *About* to look |
| **Timeout** | 10 seconds to ask |
| **If it fails** | Nothing is said, and *About* falls back to sending you to the Store yourself |

Pressing **Update** there downloads nothing through Tisty: the Store fetches and
installs the package, and Windows closes Tisty to put it in place.

## The requests only you can start

On every copy but the Store's, pressing **Update** in *About* makes Tisty fetch
a second file, which does name an address, and then the installer at that
address. They go to `raw.githubusercontent.com` and to `github.com`, which
hands the download on to `objects.githubusercontent.com`, all over HTTPS,
and both carry what any download carries: an address to send the bytes back to,
and a user agent — here `tauri-plugin-updater/<version>`, the library doing the
fetching. On a Mac the installer asked for is the one built for its chip, so
the name of that download says whether the machine is Apple Silicon or Intel —
as a download made by hand from the releases page would. Nothing about you,
nothing about what you have written.

This is the one place where an address that arrived over the network is opened,
which is why it is fenced twice. The address has to be where this project's
releases live, or nothing is downloaded at all. And what does arrive is checked
against a key compiled into the copy you already have, before a single byte is
run or written to disk.

On macOS, if the application sits somewhere your account cannot write, the
system asks for an administrator password to put the new copy in place. You can
refuse; nothing is moved.

A link **you write or paste into a document** is a different matter: clicking it
opens your browser, the way any Markdown reader does. Tisty does not follow it
on its own, and never opens anything without a click. Worth knowing if a
document reached you from a shared folder someone else can write to — the words
of a link and where it goes are not obliged to agree, in Tisty or anywhere else.
If LinkUnbound is installed, a web link is handed to it instead, so it can ask
which browser to use; nothing else changes, and nothing leaves through Tisty.

A widget in a document, or an attached `.html` page, is drawn by its own code
in a frame that is allowed no network at all: it cannot fetch anything, load a
picture from an address or send what it shows anywhere. A link inside it opens
your browser only when you click it, like any other.

A Markdown file you open from outside — with *Open with*, or a
`tisty://open?path=…` link — is shown read-only in a window of its own and stays
out of your store until you choose *Edit in Tisty*. Such a link opens only a
readable text file on this computer, and a widget inside that file is not run
until you bring the file in.

Tisty's own screens carry a few links of their own: the repository, the pages
for sponsorship and for a coffee, the AlternativeTo listing, the pages of the
author's two other tools, CoPanda and LinkUnbound, and — on a copy the
Microsoft Store keeps — the Store's own rating window. Where each one leads is
fixed in the program, not fetched from anywhere, and they behave like any link
above: nothing is requested, nothing is sent, and your browser opens only when
you click. The card that asks for a star is the same: it reads no count and
checks nothing over the network, it only opens the page if you press it.

A link left alone in its own paragraph is drawn as a card, and that card is made
of the address itself: the site's name and the words you wrote. Nothing is
fetched to draw it. If you press **Bring the preview**, Tisty asks that page
once for its title, its description and the picture it offers — and that request
is like any other visit: the site learns an address to answer to and the time of
day. Tisty reads its answer up to 256 KiB and the picture it offers up to
400 KiB, and stops there — read to the limit off the wire, so a page cannot
decide how much of your machine it takes. What comes back is kept here, so the
card never asks again, not tomorrow and not on your other computer, which would
have to be told to bring it too. The picture is stored inside that record rather
than in your documents, so nothing of the world ends up in the folder your
machines share, and the whole store of them is capped by weight, so old cards go
rather than pile up.

Opening the guide inside Tisty — `F1`, the `?` at the foot of the sidebar, or
the Help menu on a Mac — reads it where it is published, so what it says is
always the latest: one request for `https://markdown.tisty.app/llms-full.txt`
(`/es/llms-full.txt` in Spanish), over HTTPS, with the user agent
`tisty/<version>`. It sends nothing else and nothing of yours; the site learns an
address to answer to and the time of day, like any visit. It waits at most 4
seconds, and without an answer it shows the copy of the guide that came with
this version, so the guide works offline too.

**Nothing is downloaded or installed unless you ask for it.** What the daily
check brings back is a line of text you can ignore. If a newer version exists, a
line appears in *About*, and what it offers depends on how this copy was
installed: the Microsoft Store updates itself and Tisty leaves it alone, and a
copy that owns the folder it lives in gets a button.

That button is the only thing that fetches an installer, and what it does is
described above. If you never press it, nothing is downloaded and the line just
sits there.

## Syncing with your own Google Drive

Since version 1.30, Tisty can keep your machines in step through your own
Google Drive account, without Google's desktop program installed. OneDrive
and WebDAV have their own sections below; Dropbox is not reached directly, only
through a folder its desktop program keeps in step. A copy built without this
project's Google credentials offers no Google Drive and makes none of these
requests.

| | |
| :--- | :--- |
| **Why** | To keep your machines in step through your Google Drive, without its desktop program installed |
| **Who it talks to** | Google only: `accounts.google.com` and `oauth2.googleapis.com` to sign in, and `www.googleapis.com` for Drive. Never a server of ours, because there is none |
| **What Google lets Tisty see** | The permission asked for is `drive.appdata`: Tisty sees and changes only its own hidden space in your Drive, which no other app and no Drive window shows. The rest of your Drive — your documents, your photos, what others share with you — stays out of its reach. If Google grants anything wider, Tisty gives the access back and stops. It also reads the address of the account you connected, to show you which one it is and to refuse to mix two accounts |
| **What it sends** | The files of your store — the history, the documents and `tisty.toml` — into that hidden space of your Drive, as they are on your disk unless you encrypt your cloud (described below). They count against your Drive storage, and *Manage apps* in Drive's settings shows how much. Attachments go up too, but stay on the computer they were added on; on your other computers each one comes down only when you open it. When a machine stops syncing with a place, it leaves there a short signed note (`moved.json`) saying where it went — a folder's name or the cloud account's address — so your other machines stop instead of going on alone |
| **What Google sees** | What any provider of a synced folder sees: the files above, their names and sizes, when they change, and the address and time of each request. Google already holds your account; Tisty adds nothing about you |
| **What it receives** | The same files, written by your other machines, and Drive's list of what changed |
| **How you allow it** | Only from the Tisty window: assistants never connect an account and never touch the network. You sign in on Google's own page in your browser. Tisty never sees your password; it receives an access you can withdraw at any time. On macOS it keeps that access in the system keychain. On Windows it keeps it in a file sealed with Windows' own protection for your user (DPAPI, the same lock Credential Manager uses), inside Tisty's folder under your profile, so uninstalling Tisty removes it. Only the window touches it; it never reaches the log, a backup or a parcel |
| **Letting go** | *Disconnect this computer* forgets the access on that machine only. *Remove Tisty's access on every computer* asks Google to withdraw it, which Google does for every machine at once. You can also withdraw it yourself from your Google account, under *Security* › *Third-party apps and services* (<https://myaccount.google.com/connections>). Uninstalling withdraws nothing. On Windows it also removes this machine's copy. macOS runs nothing when an app is removed, so there the copy stays in the system keychain until you disconnect first or delete it there; Google drops an access unused for six months |
| **Checking the way first** | Only while you are choosing Google Drive — connecting it, or moving to it — Tisty first checks that Google's API host answers: a single request to `https://www.googleapis.com/drive/v3/about`, with no account, no token and nothing of yours in it, which Google answers at once by turning a stranger away. It waits at most 5 seconds to reach it and 10 seconds in all, and goes through whatever proxy your system names, never around it. From how it ends, Tisty only tells you whether something on the way — a proxy, a filter at work, an antivirus — blocks Google or inspects secure connections, whether Google was only slow to answer, or whether Google itself is having trouble. It never runs on a round of syncing, and what it learns stays on your screen and in Tisty's own log on this computer. When a round of syncing cannot reach Google, Tisty reads how it failed the same way, sending nothing more, and keeps on this computer only that word — blocked, filtered or slow — with how many rounds in a row failed and since when, to warn you once it lasts: a second failed round in a row, or two minutes after the first. A connection cut halfway through a transfer keeps no word at all |
| **How often** | Only while Tisty is running, including when it sits in the tray or started on its own at login; never once it is closed. It asks whether anything changed at most every few minutes, and less often while nothing does. There is no background service |
| **Limits** | A daily allowance per computer — not per person — of 2,000,000 request units and 50 GB downloaded, so one copy cannot spend the shared quota Google grants this project. Attachments are capped at 25 MB per task and 650 MB inside a document, and nothing over 750 MB is taken in from the cloud. *Free up space* lets go of attachments you have not opened in a while; opening one again brings it back down and counts against the same allowance |
| **The remote switch** | A small file, `cloud.json`, published beside the version manifest at `https://raw.githubusercontent.com/rgdevment/Tisty/manifest/cloud.json`, can lower those limits or pause syncing for a while on every copy, should the project's quota be at risk. It is signed with minisign against a key fixed in the program when it is built, read only up to 16 KiB, and an older one never replaces a newer one. It is fetched at most once an hour while you sync with Google Drive, sends nothing but the request itself, and carries numbers and nothing of yours |
| **What it never does** | Send your data to us or to anyone but Google, read a file in your Drive outside its own space, or write tokens and signed addresses into its own log |
| **Leaving** | Choose *Only on this machine* or a folder through *Move to another place*. Tisty brings home what lives only in your Drive before it lets go of it, and leaves the copies there; disconnecting keeps them too. *Delete my copies in Google Drive for good* deletes them, since that space has no bin. In Drive's settings, under *Manage apps*, *Disconnect from Drive* only withdraws Tisty's access; *Delete hidden app data* deletes the copies as well. When a machine leaves a folder, it writes a short note in it, `README - Tisty.txt`, saying where its copies went and how to come back; it takes the note away when it chooses that folder again |

### Google API Services

Tisty's use and transfer of information received from Google APIs adheres to
the [Google API Services User Data Policy](https://developers.google.com/terms/api-services-user-data-policy),
including the Limited Use requirements. In plain words: what Tisty reads from
your Drive is used only to sync your own store between your own machines. It
is never sent to us or to anyone else, never used for advertising, never sold,
never used to train any model, and no person reads it.

## Syncing with your own OneDrive

Since version 1.31, Tisty can do the same through your own personal OneDrive,
without Microsoft's desktop program in the way. It is the same engine as Google
Drive, so what the section above says of the files, the limits, the remote
switch and how often it asks holds here too; what changes is below. A copy built
without this project's Microsoft client offers no OneDrive and makes none of
these requests.

| | |
| :--- | :--- |
| **Who it talks to** | Microsoft only: `login.microsoftonline.com` to sign in, and `graph.microsoft.com` for OneDrive. Never a server of ours, because there is none |
| **What Microsoft lets Tisty see** | The permissions asked for are `Files.ReadWrite.AppFolder`, `User.Read` and `offline_access`: Tisty sees and changes only its own folder, *Apps › Tisty* in your OneDrive, reads the address of the account to show you which one it is and to refuse to mix two accounts, and keeps the access between sessions. The rest of your OneDrive stays out of its reach. If Microsoft grants anything wider, Tisty refuses the sign-in |
| **Where the copies are** | Unlike Google's hidden space, *Apps › Tisty* is a folder you can see in OneDrive. Leave it to Tisty: what you change there by hand is read as a change from another computer. Deleting it sends the copies to OneDrive's recycle bin, and *Delete my copies in OneDrive for good* does the same |
| **How you allow it** | Only from the Tisty window. You sign in on Microsoft's own page in your browser; Tisty never sees your password. Tisty is a public client with no secret of its own. The access is kept exactly as Google's is, in the system keychain or, on Windows, sealed with DPAPI |
| **Letting go** | *Disconnect this computer* forgets the access on that machine only. Microsoft offers no way for an app to withdraw a personal account's access by itself, so *Remove Tisty's access on every computer* forgets it here and opens your Microsoft account at <https://account.live.com/consent/Manage>, where you remove Tisty for every machine at once |
| **Checking the way first** | Only while you are choosing OneDrive, Tisty checks that Microsoft's API host answers with a single request to `https://graph.microsoft.com/v1.0/me`, with no account, no token and nothing of yours in it, read the same way as Google's |
| **The remote switch** | `cloud.json` speaks to OneDrive only under its own entry in the `providers` field, so a pause or a limit meant for Google never stops or slows OneDrive, nor the other way round |
| **What it never does** | Send your data to us or to anyone but Microsoft, read a file in your OneDrive outside its own folder, or write tokens into its own log |

## Syncing with your own WebDAV server

Since version 1.32, Tisty can keep your machines in step through a WebDAV
server of your own — Nextcloud, ownCloud, Synology and others — with no desktop
program in the way and no browser sign-in. It is the same engine as Google Drive
and OneDrive, so what the Google Drive section says of the files and how they
travel holds here too; what changes is below.

| | |
| :--- | :--- |
| **Who it talks to** | The server at the address you type, and nothing else. Never a server of ours, because there is none |
| **Where the copies are** | In a folder named `Tisty` that Tisty makes inside the folder your address points at. Nothing outside it is listed or touched. You can see it on your server; leave it to Tisty, since what you change there by hand is read as a change from another computer |
| **What it sends** | The same files as to Google Drive, into that folder, and with every request your user and password, as HTTP Basic authentication. Attachments always stay on every computer as well, so *Free up space* is not offered |
| **How the line is guarded** | Only `https://` is accepted, because the password travels with every request. Plain `http://` is refused, except to this very computer (`localhost` or a loopback address). The server's certificate is checked by the system as for any other secure connection, and Tisty never follows a redirect, which would carry the password to wherever it points. An address that carries a user or a password of its own is refused |
| **How you allow it** | Only from the Tisty window: in Settings › Sync you type the *Server address*, the *User* and the *Password*. Before anything is kept, Tisty checks the server: it signs in with what you typed, looks at its own folder and, if the folder is empty, writes a small file called `tisty-probe` twice to see that the server refuses to write over a copy it was not told about, then deletes it. A server that would let two computers write over each other is refused, and nothing is connected |
| **How the password is kept** | Only once the check passes. On macOS it goes into the system keychain. On Windows it goes into a file sealed with DPAPI for your user, inside Tisty's folder under your profile, beside the cloud tokens, so uninstalling Tisty removes it. Tisty holds it in memory for at most twelve hours before reading it again, and it never reaches the log, a backup or a parcel. An app password from your server is better than your main one: you can remove it there |
| **How often** | It looks for changes every two minutes while Tisty runs, since no WebDAV server says what changed. If the server asks to slow down, Tisty waits. The same daily allowance per computer applies, and the remote switch speaks to WebDAV only under its own entry |
| **Letting go** | *Disconnect this computer* forgets the password on that machine only and keeps your copies, here and on the server. There is no button to withdraw access on every computer: remove the app password, or change the password, on your server. *Delete my copies in WebDAV for good* deletes them from the server, which sends them to its bin if it keeps one |
| **What it never does** | Send your data to us or to anyone but that server, touch a file outside its own folder, send the password over plain `http` to another machine, or write the password into its own log |

## Encrypting your cloud

Since version 1.36, what Tisty keeps in your Google Drive, your OneDrive or
your WebDAV server can be encrypted, from Settings › Security, with
*Encrypt my cloud*. It is offered only for those three, which Tisty reaches
itself; a folder kept in step by a desktop client is never encrypted. It changes
nothing on your computers: what sits on your disk stays plain, as described
above.

| | |
| :--- | :--- |
| **What it needs** | A 24-word phrase, made on your computer and confirmed by typing three of its words, and every one of your computers on 1.36 or later. Tisty names any computer that is behind |
| **What it encrypts** | Every file of your store in that cloud: the history, the documents and the attachments. Each one goes into a `sealed/` folder beside the old copy, under the same name, sealed with XChaCha20-Poly1305 under a key that comes from your phrase. Each seal is bound to the cloud and to the file's name, so a file cannot be passed off as another or moved |
| **What the provider still sees** | As Tisty itself warns before you start: the provider keeps earlier unencrypted versions for a while, in its history and its bin; the names and sizes of the files and how many computers you have stay visible; and while it encrypts, your cloud takes twice the room, until you delete the old copy. Documents are named by an identifier rather than their title, but an attachment's name is a lowercased form of the file's own name. The provider still sees when files change and the address and time of each request |
| **What stays unencrypted** | `tisty.toml`, which says what the folder is and now also that it is encrypted and with which store; `moved.json`, the signed note left when a computer moves elsewhere; the store's identifier in `store/`; a small mark, `sealed/sealed.mark`, which lets a phrase be checked before anything comes down; and the `knock/` folder, where your computers ask each other for the key. An ask in `knock/` carries the name you gave that computer in Tisty |
| **The old copy** | It stays as it was until you press *Delete the unencrypted copy*, which deletes only the files whose encrypted copy is already there. Until the last step, cancelling loses nothing. A file too big for the provider to take encrypted is left out and stays as it is |
| **Where the phrase lives** | On the computer where you made it, in the system keychain on macOS or, on Windows, in a DPAPI-sealed file apart from the cloud tokens, which the installer's uninstaller leaves in place. It is never synced, backed up or put in a parcel. You can show it again in Settings › Security or save it as a PDF; keep that outside Tisty. It never passes through our hands, so nobody can give it back to you |
| **Your other computers** | A computer that already syncs with you asks for the key on its own, and another of yours that holds it answers, as soon as one of them has Tisty open. A new computer chooses *Let in from another computer* and shows twenty digits, which you type on one of your computers that already reads the encrypted cloud. Or you can type your 24 words on it with *Type my phrase*: only the key of your cloud stays on that computer, never the phrase. Either way the key travels only inside the cloud, sealed for that one computer, and is kept in its keychain or, on Windows, in the same sealed folder as the phrase |
| **Who can open it** | Only someone with your phrase or one of your computers. Not the provider, and not us: there is no copy of the key anywhere else, so if you lose the phrase and all your computers, the encrypted data is lost |
| **Going back** | *Decrypt my cloud* copies everything back unencrypted and deletes the encrypted copy. It leaves a signed note, `knock/unsealed.json`, so your other computers follow without you doing anything on each one. Your phrase stays yours |

## The door you can open for an assistant

Tisty has no AI inside it and makes no request to any model. It does speak
[MCP](https://modelcontextprotocol.io) when you ask it to, so an assistant
already running on this machine can file a task on your behalf. None of it
happens on its own:

- **You install it.** Nothing listens until you run `tisty mcp`. The assistant
  reaches it through that process's own input and output — on this machine,
  with no port, no account and no token. It is not a server and nothing can
  reach it from the network.
- **You let it in.** The assistant shows up in Settings › Assistants as a device
  you have to admit, and everything it tries before that is refused. Admitting
  it is an event written into your log rather than a line in a settings file,
  so you can see when it happened and throw it out whenever you want.
- **What it may do is bounded.** File a task, add to the journal, say a task it
  filed or one you opened to agents is done — which marks it for you and closes
  nothing — write a document, write one again whole, keep a copy of a file you
  point it at, and read what is already there. It may not close or delete
  anything, say a task you wrote and kept closed to it is done, reach a task
  you hid, or take files from outside the folders where a download lands.
  Bringing a markdown file in copies what that file's text points at, and only
  what sits in that file's own folder or below it, so what leaves your disk is
  bounded by the file you named rather than by what its text asks for. To
  replace a whole document it has to send back the print it read that document
  at, so what you wrote while it was thinking cannot be taken. What it reads
  comes back without the paths of your disk.

The part worth knowing before you open it: **whatever that assistant reads, it
takes with it.** If it runs against a service, then the moment you ask it to
look, what it looks at is on its way to that service. Tisty still sends
nothing and still has no server of its own — but this door is real, and it is
the only thing here that can carry your data off this machine. Which is why it
stays shut until you open it, and why closing it is one click.

## Children's privacy

Tisty collects no personal information from anyone, of any age. There are no
accounts, no registration and no data transmission.

## Changes to this policy

Any change is committed to the public repository with a clear message, updates
the date above, and appears in the release notes. Because the project is open
source, a change in privacy behaviour would be visible as a code change before
it ever reached you.

## Contact

<rgdevment@tisty.app>

---

*Tisty is in early development. This document describes decisions already
made; it will be expanded with concrete storage paths and behaviour as the
implementation lands, and never in the direction of collecting more.*
