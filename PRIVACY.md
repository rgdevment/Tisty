# Privacy policy

**Last updated:** November 23, 2026

## The short version

Tisty does not collect anything and sends nothing of yours to a server of ours.
No account, no telemetry, no analytics, no crash reporting. The one exception is
yours to make: a document you choose to share by link goes, sealed, to Folio,
when Folio has opened for new documents.

It makes **one** check on its own — when it opens, and once a day after that:
it downloads a small file and its signature to see whether a newer version
exists. It sends nothing. Two more requests follow only if you press the button
that offers you an update, one or two only if you ask a link in a document for
its preview (the page, then the picture it offers), one more when you open the
guide inside Tisty, one per picture if you press **Download** above the help,
one per picture from the internet if you ask for them in the viewer of a
Markdown file from outside, and those of sharing by link if you press **Share**
or **Sync**, manage a share, open a shared link, open a document you shared with
a language you kept from someone else, or delete a document you shared. Never
otherwise. A copy installed from the Microsoft Store asks Windows what the Store
has and leaves the rest to the Store, and reads the Store's public catalog to
see which version that is; it reads that same small file only if the catalog
does not answer. All of them are described in full below.

Since version 1.30, Tisty can also sync through your own Google Drive, since
1.31 through your own OneDrive, and since 1.32 through a WebDAV server of your
own, talking to it directly. It runs only if you choose it and connect your
account in the Tisty window, it reaches nothing but that provider or that
server, plus, at most once an hour, the signed `cloud.json` beside the version
manifest, and what it sends and receives is described in full below.

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
| **A `.tistyx` parcel of documents you hand to somebody** | Them, and whoever they hand it to. You decide what goes in it and when. It carries the documents, where they sat, what the archive held and what an assistant marked — never your device identifier. Once you have your identity it is signed with this computer's key, so it carries your root's public key, which reads as your fingerprint, this computer's own public key with the certificate that ties it to your root, and the name you sign with, and no longer your store's name. That computer key is the same in every parcel it signs, so whoever holds two of them can tell they came from the same computer. For each document you shared through Folio it also carries the Folio document's identifier and who edited it there: their names, keys and fingerprints, as Folio handed them to you |
| **A `.tistyx` parcel locked with a number, for another machine of yours** | Whoever has both the file and the number. Without it nothing here opens it — though six digits are six digits, and the lock only makes each guess expensive. Inside it carries the same as an open one, and it is signed on the same terms: only once you have your identity |
| **A document you share by link through Folio** | Whoever has the link. Folio holds the document sealed and cannot read what is in it, but sees what [Sharing by link](#sharing-by-link) lists |
| **What you keep in Agents** | The same as your other documents, plus, while it syncs, each assistant whose files Tisty writes, and whatever service that assistant talks to — even one you never let in. See [Your assistants' files, in Agents](#your-assistants-files-in-agents) |
| **A task you save to your calendar** | Google or Microsoft, under their terms and their jurisdiction, when the calendar is theirs; with Apple Calendar, the Calendar app on your Mac and whatever account it keeps that calendar in. Only the task you pressed the button on, and only what [Saving a task to your calendar](#saving-a-task-to-your-calendar) lists |

With a folder, Tisty only ever reads and writes local paths. It has no network
code for it, no credentials, and no idea which provider — if any — is behind the
folder you named. Whatever keeps that folder in step between your machines is
software you already installed and already trust. Reaching your cloud account
directly is the one way that changes this, and each provider is described in
its own section below. Encrypting is offered only for those clouds, never for a
folder.

There is no option where Tisty hosts your store. There is no paid tier that
changes this. **Nothing you keep reaches a server of ours unless you press Share
on a document**. Then that document goes to Folio, sealed so that Folio cannot
read it. Apart from Folio, the one site of ours
Tisty ever asks is markdown.tisty.app, for the guide, and that request carries
nothing of yours. So there is nothing on our side to breach, subpoena, sell, or
lose, beyond the sealed copies of what you chose to share.

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
- **Agents, if you use it** — your assistants' rules, skills and memory, kept
  as documents like the rest. While it syncs, Tisty also writes, outside its
  folder, the files each assistant reads; [Your assistants' files, in
  Agents](#your-assistants-files-in-agents) says what it reads, when, and what
  stays on this computer.
- **Brainsty's base** — how your things are tied, the weighty words of each
  document and task, and which things are alike, in Tisty's cache on this
  computer. It is read only from the prose you wrote, never from code, paths,
  a task's progress notes or the documents in Agents, and hidden or dropped
  tasks and your diary leave nothing there. It rebuilds itself from what you
  keep, so it is never synced, backed up, put in a parcel, shared by link or
  exported. The alike pairs come from rules and a fixed Spanish–English word
  list that ships with Tisty; nothing is learned from what you write.
- **Earlier versions of your documents** — plain Markdown files, one per
  version, in a `history` folder beside the documents, with a small marker per
  document saying who wrote last and which version undoes the last change. Each
  is kept 30 days from when it was replaced, or the 1, 5 or 15 you choose in
  Settings › Documents and signature; the latest of each document, and the one
  that undoes the last change, never run out. They stay on this computer: they
  are never synced, shared by link or put in a parcel, but a backup zip carries
  them, including the ones Tisty writes on its own before a change. Deleting a
  document deletes its versions; **Empty the history** deletes them without
  deleting the document.
- **Words a save refused** — if a document got locked or too long while you were
  writing, Tisty keeps what it could not save in the window's own storage on
  this computer, until you put it back, discard it or delete the document in
  this window. It is never synced, backed up, put in a parcel or shared by link.
- **Attachments** — the files themselves, unmodified. The log keeps a note of
  each one: the name it is kept under (a lowercased form of the file's name
  with a short fingerprint), its size and fingerprint, how it came in (dropped
  in the window, attached by an assistant, imported beside a document, from a
  parcel or the guide) and, when it is retired, whether it went unused or was
  joined with a copy of itself. Never the folder it came from.
- **Your diary** — Markdown documents like the rest, a book a year with
  `diary: on` in its properties. It syncs, goes into a backup and into a parcel
  that carries everything, and is never shared by link. Whether it is open to
  assistants is an event in your log that only your own computers write, so it
  reaches all of them and an assistant cannot open it for itself.
- **Reminders booked with the system** — so a reminder rings with Tisty closed,
  Tisty hands Windows or macOS, ahead of time, the notices of the coming month,
  at most 256 on Windows and 64 on macOS: for each, the hour, the task's title
  and the task's identifier. The system's own notification service keeps them
  on this computer until the hour comes, or until Tisty takes them back: when
  the reminder changes, when you turn the system notification off in Settings,
  when you leave from Settings › Copies, or when the Windows installer
  removes Tisty. Nothing goes over the network.
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
  you to star the project, whether it has already offered your assistant a
  door and whether it has already offered to open at sign-in for your
  reminders, so that each of those is asked once and never again. It holds too
  whether important reminders are on and the calendar you chose for **Save to my
  calendar**. Which assistants
  are on this machine is read when the offer is decided and is not written down.
  Those that take Tisty's hooks or skills, once you connect them from Settings,
  are noted in the window's own storage, so each is offered them once.
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
- **An alias, if you write one** — it exists so that a document you hand over
  in a parcel or a PDF still says who wrote it. It is optional, it lives in the
  log like everything else, and so it reaches your other machines through the
  folder you sync and travels inside any parcel you hand out. A PDF carries it
  only if you add the card while exporting and have no name to sign with. It
  goes out with the sync before you hand anything over, which is why it should
  be a name you are content for others to read.
  A log only ever grows: an alias you write today can be changed tomorrow, but
  the old one stays written. The same goes for a name to sign with, which is
  also optional, lives in the log and goes on a PDF's card in place of your
  alias. Your address is **not** asked for anywhere, and the field reserved for
  it in the log is written by nothing in this version.
- **The time zone you were in** — every event carries the IANA zone of whoever
  wrote it, so an hour reads back where it happened instead of drifting when you
  travel. It goes to the shared folder with the rest, and a long enough history
  shows where you have been. It is named here because it is the one field that
  says something about you rather than about your work; everything else in that
  folder is already your own words.
- **What you share by link** — so that your other computers can manage a share,
  the log keeps, for each one, its identifier, its version, the key of its link
  and the tokens that manage or edit it, with a print of what was last sent of
  each document so that a Sync sends only what changed. Of a document others
  added languages to, it also keeps the owner's signed word on each one reviewed
  and, on the owner's side, the key of each one left out, which no version
  carries, so it can be reviewed again from any of your computers. They travel
  with the log to the folder or cloud you sync through, so whoever can read the
  log can open that link, and the languages left out of it. They are not put in
  a parcel. The limits Folio last published and the terms you accepted stay in
  `private/`, and so do the waits that pace sharing: when you pressed Sync on
  each share, until when Folio asked this computer to wait, for a document, for
  your network or for the rest of the day, which documents were shared for the
  first time today, counted by the day as Folio counts them, and when this
  computer last asked Folio after the languages others added to a document.
  None of those leaves this computer.
- **The languages others added to what you share** — since 1.46, `private/`
  also keeps, by document identifier, the highest word of a document's owner
  this computer has seen (`folio-reviewed.json`), so an older word served again
  never counts, and the number and print of the latest version of each of a
  shared document's languages it sent or read (`folio-languages-printed.json`),
  so it can tell you that a language you kept has changed. When you keep a
  language someone else added, it also keeps a copy of its title and words as
  you read them, in `folio-kept-copies/`, to show you what changed since: that
  copy is someone else's writing, and only the computer where you pressed
  **Keep** holds it. It goes when you leave the language out, and it and the
  language's print go when Folio no longer holds that language, or when the
  share that brought it stops or is forgotten. The pictures of a language under
  review are written to Tisty's cache so the window can show them, and the next
  language you review replaces them. None of these leaves this computer.
- **A `private/` folder** — anything placed here never leaves the machine
  under any transport.
- **The guide** — the one thing Tisty writes for you rather than the other way
  round. Its words and its images ship inside the program and are copied into
  your documents in the language you picked. It comes signed by its maker, and
  Tisty checks that signature against the maker's root, fixed in the program,
  before copying it. Nothing is fetched to do it, and once it is there it is an
  ordinary document: edit it, or delete it.

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

The same holds, more briefly, for a document: a text you delete on purpose
stays in its earlier versions on each computer that held it when it was
replaced, until the version is past the limit and neither the latest nor the
one that undoes the last change, or until you empty its history or delete the
document.

The `private/` folder exists for exactly this. Prevention is the only real
defence, because the remedy barely exists.

## What Tisty does not do

- Does not send data to any server you did not choose: only your cloud, if you
  connect one, and Folio, for a document you press **Share** on.
- Does not share a document unless you press **Share** on it. See "Sharing by
  link" below.
- Does not create accounts or profiles.
- Does not use cookies, analytics, advertising, or tracking of any kind.
- Does not share anything with third parties on its own.
- Does not use AI or machine learning on your data. The natural-language
  parser is deterministic rules running locally, and Tisty itself sends nothing
  to any model. If you connect an assistant of your own, read "the door you can
  open" below: what it carries is its doing and your choice, never Tisty's.
- Does not upload crash reports.
- Does not make background network calls, except the update check below and,
  only if you connect Google Drive, OneDrive or a WebDAV server, the requests
  that sync with it, and the `cloud.json` check, at most once an hour, that
  rides along. Opening a document you shared with a language you kept from
  someone else also asks Folio after that language, at most every quarter of an
  hour, as *Sharing by link* describes.

An earlier version of this file promised that if Tisty ever needed a network
request it would be read-only, documented here before it shipped, and visible
as a code change first. That is what happened: the check went into the public
repository, and this section was written before the release that carries it.

## Sharing by link

Sharing goes through [Folio by Tisty](https://github.com/rgdevment/Folio-by-Tisty),
a separate service with its own terms and its own privacy policy. Folio decides
when it opens for new documents. Tisty does not reach it, or anything about it,
until you ask.

Every document shows a **Shared** row. Pressing **Share** reads the file in the
first row below. While that file does not say sharing is open, Tisty shows a
notice instead, with a link to `tisty.cl` that opens in your browser only
if you click it. Only starting a new share waits for that file: a link reaching
Tisty and a share already out do not.

| | |
| :--- | :--- |
| **When you press Share or Sync** | Tisty reads `https://raw.githubusercontent.com/rgdevment/Folio-by-Tisty/limits/limits.json` and its signature, `limits.json.minisig`, to learn whether sharing is open, what it allows and whether Folio's terms changed. Whether sharing is open counts for an hour; whether the terms changed is asked again on each Share or Sync, at most once a minute. A try that reached nothing, or brought a file whose signature did not hold, does not count, so the next press asks again. Nothing is uploaded, GitHub serves them, not Folio, and the request says only `tisty/<version>`. If that file says sharing is closed, or it cannot be read, it stops there. If sharing is open and you have not accepted the current terms, it reads `TERMINOS.md` or `TERMS.md` from `main/docs/` in the same repository |
| **Folio's private address** | `tisty.cl`, unless `domains/hosts.json` in the same repository names another. Tisty reads that file and its signature, `hosts.json.minisig`, from `https://raw.githubusercontent.com/rgdevment/Folio-by-Tisty/domains/` before talking to Folio, when you press **Share**, and when the **Shared** row shows a document's links. It keeps the last one whose signature held and asks again only once that copy is two days old, or within the hour while it holds none. An older one never replaces a newer one, nothing is uploaded, and the request says only `tisty/<version>`. Below, «that address» is this one |
| **When you confirm** | Tisty asks `https://<private address>/api/v1` for an identifier, then sends what you chose, sealed on this computer: the document with its pages, pictures and files, its other languages, the documents it cites and the name you sign it with. When others edited it through its link, the names each of them signed with travel too, each signed by its own key, so whoever opens it sees who created it and who edited it. A language someone who edits adds stays theirs, under the name they sign with: readers see it only once you keep it, as the version you read, credited to them and to you as who reviewed it. The key travels in the link, after the `#`, and never reaches the server. Neither does the token of an edit link, which signs on this computer. Requests to that address follow no redirect, wait at most two minutes and say `tisty/<version>` |
| **What Folio sees** | Not the title, the words, the pictures, the files or the names anyone signs with: those are inside the seal, and so are your reviews of the languages others added. In the clear it gets each envelope's manifest (identifier, version, sizes, signature, the version of Folio's terms you accepted, with no date, and the public keys of your identity and of this computer, so it can tell that two documents come from one person), how many days the link lasts, the token that manages it, a print of the sealed files, and any notice that your identity took back a computer's key. Once you review a language someone else added to a document of yours, it also gets, beside that document, your word on all your reviews of it: signed by this computer, it names the document, a number that grows each time you send it and a hash of your reviews, never which languages you kept or left out. Folio serves that word, and the notices of whoever signed, owns or edited a document, to whoever opens it, so their Tisty can weigh the signatures. Like any server, it also gets the address and the time of each request |
| **When a link reaches Tisty** | A link arrives as `tisty://edit?link=…`, from the button on Folio's page or from any other page that fires it, and whether sharing is open does not change it. Tisty accepts only an `https` link on `tisty.cl`, `folio.tisty.app` or Folio's private address, and takes from it only the identifier and the key. Before anything leaves, it asks whether to look for the document, and says Folio will see this computer's network address. Only on **Look for it** does it ask `https://<private address>/api/v1/docs/<id>`, and for the files if there are any, never the address the link names, and the key stays here. The document is opened on this computer, a second question shows its title and who signed it, with «verified» beside what the maker of Tisty signed under the root Folio's signed limits name, which Tisty already holds. When a signer's alias is a domain, Tisty checks it against the list Folio signs, reading one small file from `https://raw.githubusercontent.com/rgdevment/Folio-by-Tisty/domains/` and keeping it for a day, and nothing is kept unless you press **Bring it**. Bringing it also asks for each of the document's other languages, up to 19, under the same address, and keeps only the owner's own and those the owner kept, as the version the owner read. Each goes under the name its author signs with, the owner's own under the owner's, so one someone else added reads «Translated by» that person «· reviewed by» the owner, unless both sign with one name. A share Tisty already remembers asks again before looking for languages added since, and asks nothing more if you say no; on a Tisty not set up the link waits for the welcome. A read link leaves a copy of your own, and nothing of it goes back |
| **When you press Sync, See reads and expiry, Pause, Resume, Appeal, Change the edit link or Stop sharing** | Tisty talks to that same address, for that document and for those that travel with it. Changing the edit link sends Folio only the public key of the new link's token, never the token, and then the next version. Sync first asks for the latest version of the document and of each of its languages, brings a newer one in apart and keeps any wait Folio names, then sends only the documents of the share that changed, if any. On a document you own, once you have reviewed a language someone else added, Sync sends your word on your reviews in the same request as the document's version, with the token that manages it, so Folio keeps both or neither. Before sending, and at most once a day for that document, it also asks for the latest version of each language others added to it, the ones you left out included, to let go of any Folio no longer holds. A language you add through someone else's edit link goes up as your own, under the name you sign with, and waits for their word. Each press repeats any notice that your identity took back a computer's key. While a wait Folio named still runs, a press asks nothing and says how long is left |
| **When you review a language someone else added** | Only on a document you own. **Review** asks that same address for the language as Folio holds it, its words and pictures, and opens it here to show you; **Keep** asks for it once more, so that what you sign is the version you read. Your word, kept or left out, is signed on this computer and goes up with the next Sync, which the row presses for you |
| **When you delete a shared document** | Tisty asks that same address to stop its share. If the document was deleted on another of your computers, this one asks the next time you delete any document or go through **Share** here. A share you only held through an edit link is forgotten with no request |
| **On its own** | One thing. When you open a document you shared that holds a language you kept from someone else, its **Shared** row asks that same address for the latest version of each such language, at most once every quarter of an hour for that document. Only a version this computer has not seen is brought, its words and pictures, and opened here, to tell you whether it changed since you kept it. Nothing else is sent, and nothing is asked while you keep no one else's language. Otherwise a shared row does not check on Folio by itself; it finds out when you press something. To say how long a wait has left, it reads only this computer |

## The request Tisty makes on its own

| | |
| :--- | :--- |
| **Why** | To tell you a newer Tisty exists |
| **What it asks for** | `https://raw.githubusercontent.com/rgdevment/Tisty/manifest/release-manifest.json`, and its signature beside it, `release-manifest.json.sig` |
| **Method** | GET. Nothing is uploaded |
| **What it sends** | The headers a request cannot avoid, and a user agent that says `tisty/<version>` — the same thing the download itself would reveal. No identifier, no store, no task, no name |
| **How often** | When Tisty opens, and at most once a day for as long as it keeps running |
| **Timeout** | 5 seconds |
| **If it fails** | Nothing is said. If the file or its signature cannot be fetched, nothing is retried: the next look is the next day, or the next time Tisty opens. If both arrive but do not verify — a cache on the way can hand back a stale pair — it asks for both once more with `?at=<now>` added, so no cache answers, and gives up there: one look makes at most four GETs |

The file it downloads contains version numbers and nothing else — no address, so
nothing that arrives from it can send you anywhere. It sits in a branch of its
own that holds those files and nothing else, which is why the address above
stays the same however many versions come out.

If you turn on release candidates in *Help and about*, this same request is what
carries them: nothing else changes, and nothing more is sent.

## If Tisty came from the Microsoft Store

That copy asks Windows what the Store has for it, through the system's own Store
interface, and Windows is what talks to Microsoft — with the identity that
machine already uses for every other app it installed from there. Tisty adds
nothing of its own: no identifier, no store, no task, no name. It is the question
Windows asks on its own schedule anyway; the button only asks it now.

Each time it asks, it also reads Tisty's page in the Store's public catalog, to
see which version the Store really has. That request goes from Tisty straight to
Microsoft, and is the same for everyone who makes it. Only if it does not answer
does Tisty read the file described above instead, with the same headers.

| | |
| :--- | :--- |
| **Why** | To tell you a newer Tisty exists, and to install it if you say so |
| **What it asks** | Windows, through `StoreContext`, and `https://displaycatalog.mp.microsoft.com/v7.0/products/9PGVWXD8X93N?market=US&languages=en-US`. If that does not answer, GitHub for the file above. No server of ours is involved |
| **What it sends** | To Windows, nothing of Tisty's: what travels is what the Store already sends for any app installed from it. To the catalog, the headers a request cannot avoid, and a user agent that says `tisty/<version>`. The market and language in the address are fixed, not yours |
| **How often** | When Tisty opens, every six hours for as long as it keeps running, and whenever you ask *Help and about* to look |
| **Timeout** | 10 seconds to ask Windows, 5 seconds for the catalog |
| **If it fails** | Nothing is said, and *Help and about* falls back to sending you to the Store yourself |

Pressing **Update** there downloads nothing through Tisty: the Store fetches and
installs the package, and Windows closes Tisty to put it in place.

## The requests only you can start

On every copy but the Store's, pressing **Update** in *Help and about* makes
Tisty fetch a second file, which does name an address, and then the installer at
that address. They go to `raw.githubusercontent.com` and to `github.com`, which
hands the download on to `objects.githubusercontent.com`, all over HTTPS, and
both carry what any download carries: an address to send the bytes back to, and
a user agent — here `tauri-plugin-updater/<version>`, the library doing the
fetching. On a Mac the installer asked for is the one built for its chip, so the
name of that download says whether the machine is Apple Silicon or Intel — as a
download made by hand from the releases page would. Nothing about you, nothing
about what you have written.

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

A widget in a document is HTML and CSS and nothing else: it runs nothing, names
nothing outside itself, and a block with anything more is shown as its code. An
attached `.html` file is a file like any other: Tisty never runs it or draws it.

A Markdown file you open from outside — with *Open with*, or a
`tisty://open?path=…` link — is shown read-only in a viewer of its own and stays
out of your store until you choose **Edit**, which makes a copy and leaves the
file where it was. Such a link opens only a readable text file on this computer.

A document's link, `tisty://doc/…?id=…`, opens one of your own documents
read-only in that same viewer, and only on a computer that holds it. It carries
the document's folder names and its title in plain words, and an identifier
that begins with a shortened form of the device identifier of the computer
that created it, so whoever you send it to reads them.

Its pictures come from one of two places. One beside the file is read from the
file's folder or below it, and from nowhere else. One on the internet is not
fetched when the file opens: the viewer offers a button above the page, and
only if you press it does Tisty make one request per picture to the address the
file names, with a user agent that says `tisty/<version>`. That site sees your
IP address, as with any picture a browser loads. Tisty refuses an address that
leads to this computer or to your own network, and checks again on a redirect.
It takes only an answer that says it is a picture, up to 20 MB, and keeps
nothing: the picture is gone when the window closes, and the next time you open
the file it asks again. This is the viewer's alone. A document in your store
draws no picture from the internet.

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

The help inside Tisty — the `?` at the foot of the sidebar, or the Help menu on
a Mac — comes with this version and asks nothing of the network. Two parts of it
reach out, and only when you open them.

Tisty's Markdown guide — `F1` while you write, or **Tisty's Markdown** in the
help — is read where it is published, so what it says is always the latest: one
request for `https://markdown.tisty.app/llms-full.txt` (`/es/llms-full.txt` in
Spanish), over HTTPS, with the user agent `tisty/<version>`. It sends nothing
else and nothing of yours; the site learns an address to answer to and the time
of day, like any visit. It waits at most 4 seconds, and without an answer it
shows the copy of the guide that came with this version, so the guide works
offline too.

The help's pictures stay on GitHub until you press **Download** above the help:
then one request per picture to
`https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/`, a few
megabytes in all, over HTTPS with the same user agent, and they are kept in
Tisty's cache on this computer. Until then each picture is named where it would
be, and nothing is asked for.

**Nothing is downloaded or installed unless you ask for it.** What the daily
check brings back is a line of text you can ignore. If a newer version exists, a
line appears in *Help and about*, and what it offers depends on how this copy was
installed: the Microsoft Store updates itself and Tisty leaves it alone, and a
copy that owns the folder it lives in gets a button.

That button is the only thing that fetches an installer, and what it does is
described above. If you never press it, nothing is downloaded and the line just
sits there.

## Saving a task to your calendar

**Save to my calendar**, on a task with a date, hands that one task to a
calendar: the one you chose under *Settings › Tasks and reminders › Your
calendar* or, while that says **Ask every time**, the one you pick when the
button asks, used for that event only. Tisty keeps in its local settings only a
calendar chosen in Settings, and nothing else: no account, no key, no connection
to your calendar.

With Google Calendar or Outlook, Tisty opens your browser on that provider's
page for a new event, with the task's title, its day and hour, and its link in
Tisty (`tisty://task?id=…`) written into the address. That identifier names the
task, and since it is a ULID, it also tells when the task was made. Tisty
makes no request of its own: your browser loads the page, so the provider
receives that address, as any page you open does, and your browser keeps it in
its history. The event is stored only if you press
**Save** on that page. The task's description, steps and notes never go. The
page is `calendar.google.com` for Google Calendar, `outlook.live.com` for
Outlook.com or Hotmail and `outlook.cloud.microsoft` for Outlook for work or
school.

With Apple Calendar, on a Mac, Tisty writes the same event as a `.ics` file in
its cache and hands it to the Calendar app. Nothing leaves through Tisty; what
Calendar does with the event, such as syncing it through iCloud, is up to
Calendar. The file is named with the task's identifier, carries the hours of its
reminders, or one Tisty adds when the task has none, and stays in the cache
until you save a task to Apple Calendar again ten minutes or more later, which
clears it.

The event does not change when the task does: moving the date in Tisty later
moves nothing in your calendar.

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
| **Who it talks to** | Google: `accounts.google.com` and `oauth2.googleapis.com` to sign in, and `www.googleapis.com` for Drive, plus, at most once an hour, the signed `cloud.json` on GitHub described under *The remote switch* below. Never a server of ours |
| **What Google lets Tisty see** | The permission asked for is `drive.appdata`: Tisty sees and changes only its own hidden space in your Drive, which no other app and no Drive window shows. The rest of your Drive — your documents, your photos, what others share with you — stays out of its reach. If Google grants anything wider, Tisty gives the access back and stops. It also reads the address of the account you connected, to show you which one it is and to refuse to mix two accounts |
| **What it sends** | The files of your store — the history, the documents and `tisty.toml` — into that hidden space of your Drive, as they are on your disk unless you encrypt your cloud (described below). They count against your Drive storage, and *Manage apps* in Drive's settings shows how much. Attachments go up too, but stay on the computer they were added on; on your other computers each one comes down only when you open it. When a machine stops syncing with a place, it leaves there a short signed note (`moved.json`) saying where it went — a folder's name or the cloud account's address — so your other machines stop instead of going on alone |
| **What Google sees** | What any provider of a synced folder sees: the files above, their names and sizes, when they change, and the address and time of each request. Google already holds your account; Tisty adds nothing about you |
| **What it receives** | The same files, written by your other machines, and Drive's list of what changed |
| **How you allow it** | Only from the Tisty window: assistants never connect an account and never touch the network. You sign in on Google's own page in your browser. Tisty never sees your password; it receives an access you can withdraw at any time. On macOS it keeps that access in the system keychain. On Windows it keeps it in a file sealed with Windows' own protection for your user (DPAPI, the same lock Credential Manager uses), inside Tisty's folder under your profile, so uninstalling Tisty removes it. Only the window touches it; it never reaches the log, a backup or a parcel |
| **Letting go** | *Disconnect this computer* forgets the access on that machine only. *Remove Tisty's access on every computer* asks Google to withdraw it, which Google does for every machine at once. You can also withdraw it yourself from your Google account, under *Security* › *Third-party apps and services* (<https://myaccount.google.com/connections>). Uninstalling withdraws nothing. On Windows it also removes this machine's copy. macOS runs nothing when an app is removed, so there the copy stays in the system keychain until you disconnect first or delete it there; Google drops an access unused for six months |
| **Checking the way first** | Only while you are choosing Google Drive — connecting it, or moving to it — Tisty first checks that Google's API host answers: a single request to `https://www.googleapis.com/drive/v3/about`, with no account, no token and nothing of yours in it, which Google answers at once by turning a stranger away. It waits at most 5 seconds to reach it and 10 seconds in all, and goes through whatever proxy your system names, never around it. From how it ends, Tisty only tells you whether something on the way — a proxy, a filter at work, an antivirus — blocks Google or inspects secure connections, whether Google was only slow to answer, or whether Google itself is having trouble. It never runs on a round of syncing, and what it learns stays on your screen and in Tisty's own log on this computer. When a round of syncing cannot reach Google, Tisty reads how it failed the same way, sending nothing more, and keeps on this computer only that word — blocked, filtered or slow — with how many rounds in a row failed and since when, to warn you once it lasts: a second failed round in a row, or two minutes after the first. A connection cut halfway through a transfer keeps no word at all |
| **How often** | Only while Tisty is running, including when it sits in the tray or started on its own at login; never once it is closed. It asks whether anything changed at most every few minutes, and less often while nothing does. There is no background service |
| **Limits** | A daily allowance per computer — not per person — of 2,000,000 request units and 50 GB downloaded, so one copy cannot spend the shared quota Google grants this project. Attachments are capped at 650 MB, on a task or inside a document alike, and past 25 MB one can stay on the computers you choose instead of travelling to all of them; nothing over 750 MB is taken in from the cloud. *Free up space* lets go of attachments you have not opened in a while; opening one again brings it back down and counts against the same allowance |
| **The remote switch** | A small file, `cloud.json`, published beside the version manifest at `https://raw.githubusercontent.com/rgdevment/Tisty/manifest/cloud.json`, can lower those limits or pause syncing for a while on every copy, should the project's quota be at risk. It is signed with minisign against a key fixed in the program when it is built, read only up to 16 KiB, and an older one never replaces a newer one. It is fetched at most once an hour while you sync with Google Drive, OneDrive or a WebDAV server, together with its signature, `cloud.json.sig`, following no redirect and giving up after 10 seconds; it sends nothing but the request itself, and carries numbers and nothing of yours. A copy built without that key, as one built from source may be, never asks for it and keeps the limits it ships with |
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
| **Who it talks to** | Microsoft: `login.microsoftonline.com` to sign in, and `graph.microsoft.com` for OneDrive, plus, at most once an hour, the signed `cloud.json` on GitHub described under *The remote switch* for Google Drive. Never a server of ours |
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
| **Who it talks to** | The server at the address you type, plus, at most once an hour, the signed `cloud.json` on GitHub described under *The remote switch* for Google Drive. Never a server of ours |
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

## Your assistants' files, in Agents

The Agents folder (`AGENTS` through MCP and on disk) keeps your assistants'
rules, skills and memory as documents — for Claude Code, Codex, GitHub
Copilot, Gemini CLI and Cursor. It has two ways of working, and you choose:
**Bring** copies the files you tick into Tisty and writes nothing back;
**Bring and sync** also writes, underneath, the files each assistant reads.
It makes no network request either way; it reads and writes files on this
computer, and only these:

- **Grey in the sidebar**, before anything is brought, it only checks whether
  each assistant's folder exists, such as `~/.claude` or `~/.codex`, to tell you
  which ones it found. It looks at nothing inside.
- **When you click Agents**, and whenever **Look again** or **Bring another
  file** shows what there is to bring, it looks at which files sit in your
  assistants' folders — their names, their sizes, and whether they are links —
  to show you the list. It opens none of them, and does not even list the ones
  whose names start with a dot.
- **With your yes**, **Bring** or **Bring and sync**, it opens only the files
  you ticked in that list and brings them in as documents. The yes belongs to
  this computer: it is kept on it, stamped with its identifier, and a copy that
  reaches another computer counts for nothing there, so each of your computers
  asks for its own.
- **After that**, it opens only the files that belong to a document in Agents,
  to see whether they changed outside and, only while it syncs, to write them;
  and, while it syncs, those it wrote for one that is gone, also the ones you
  chose to leave, to see whether they changed. Brought but not synced, it writes
  none of them. Only words you approved reach a file: a document whose text
  waits for your approval is not written, and one that arrives from another
  computer waits for **Apply here** on this one. A document in Global is written
  to every assistant Tisty finds on this computer, even one whose files you did
  not tick. A change made outside is never written over; it stays marked until
  you bring it in or write Tisty's version over it. In Gemini CLI's `GEMINI.md`,
  which Gemini writes itself, Tisty only ever adds one line, `@tisty-rules.md`,
  and takes nothing out.
- **The memory an assistant writes for itself** is brought in and shown, and
  never written back.
- **An assistant connected through the door below** reads these documents like
  any other, but cannot change, move or create them: what it writes there stays
  as a proposal until you approve it on the document's card, and only then can
  it reach a file.

What Agents never brings in:

- **Settings and credentials, by their names**: `settings.json`,
  `settings.local.json`, `config.toml`, `config.json`, `hooks.json`, `mcp.json`,
  `mcp-config.json`, `.claude.json` and `keybindings.json` — except a file of
  those names inside a skill's own folder, which is part of the skill — and,
  anywhere, `.credentials.json`, `credentials.json`, `auth.json`, `.netrc`,
  `.env` and `.env.*`, `id_rsa*`, `id_ed25519*`, and files ending in `.pem`,
  `.key`, `.p12`, `.pfx` or `.env`. These are skipped without being brought in.
  Apart from adding or taking out Tisty's hooks when you ask, Claude Code's
  `settings.json` is opened for two things, and neither keeps anything of it:
  with your yes to Agents or to Tisty's own skills, to see whether the Tisty
  plugin is turned off (`enabledPlugins`); and, for the hooks row in Settings ›
  Agents › Connected, to see whether Tisty's hooks are in it. Nor is an Agents
  document ever written to an assistant's own settings or credentials: a
  document goes only where each assistant keeps its rules, its skills and its
  memory.
- **A file that reads like it holds a credential.** A file you ticked is
  checked on this computer, before it becomes a document, for the shapes of
  tokens, passwords and private keys; if it matches, it is left out and you
  are told why. The same check runs on a change you bring in from outside. It
  recognises known shapes, not every secret there is, so keep secrets out of
  your assistants' rules.
- **Links and large files.** A symbolic link is never followed, read or written
  through, and a file over 512,000 bytes is not brought in.

What travels, and what stays:

- **The documents travel like any document.** They sync, go into Tisty's
  backups, into a `.tistyx` parcel that carries everything, and into your
  cloud — encrypted, if you encrypted it. They never take part in Brainsty's
  alike pairs, and the network keeps them hidden until you show them.
- **While it syncs, what you approve in Agents reaches your assistants.** Each
  one reads its file and sends it to whatever service it works with, whether
  or not it is connected to Tisty. That is the point of the folder, and it is
  also a way your words leave this computer.
- **This computer's own record stays on it**, in `data/agents/` inside Tisty's
  folder, out of everything sync, backups and parcels carry: your yes to Agents
  and your yes to Tisty's own skills, whether Agents syncs here, which
  assistants you paused, which documents you applied or left out on this
  computer, a fingerprint of each file Tisty last wrote, which of those files
  you left with your assistants when you deleted what wrote them, and the
  backups. It is readable by your user alone, where the system allows it.
- **Backups before every overwrite.** Before Tisty writes over one of your
  assistants' files, it keeps the version that was there. Backup #1 is the
  file as it was before Tisty first touched it, and it is never removed; after
  it, the last 20 are kept, so up to 21 per file, and no copies are left in
  your assistants' folders.

Turning it off loses nothing. You can pause one assistant, or **Stop syncing**
for the whole folder; either way Tisty neither opens nor writes those files on
its own: they stay as its last write left them, and your documents stay in
Tisty. A pause belongs to this computer, like the yes. Deleting a rule or skill
from Agents asks once, right then, whether to take it away from the assistants
Tisty wrote it for: **Take away too** deletes each file after a backup, and
**Only from Tisty** leaves them and notes them on this computer, listed with
**Review** in Settings › Agents › Agents folder. A copy changed outside, or one
a paused assistant reads, stays and is asked about apart; a file other rules
still make up is written again without this one, after a backup. When a rule
leaves Agents any other way, or is deleted on another computer, Tisty asks here
before deleting its file. Short of those answers Tisty never empties or deletes
an assistant's file, apart from its own skills' copies, below. The help page
[The Agents folder](https://www.tisty.app/docs/agents-folder/) tells each step.

## Tisty's own skills

Tisty ships three skills that teach your agents to work with it: `tisty`,
`consolidate` and `writing-for-the-next-agent`. They live inside the app, not in
your store: they never sync, never go into a backup, a parcel or your cloud,
and no agent can name or change them through the MCP. Only the bytes of the
release, checked against a fingerprint built into the app, are written.

- **They have their own yes, on this computer.** Settings › Agents › Connected
  shows them in a row of their own, and **Turn on here** is that yes. It needs
  no Agents folder, and a record copied from another computer does not count.
- **Before that yes, and with no yes to Agents either, their card opens nothing
  inside your agents' folders.** It looks only at which agents' folders exist,
  as Settings › Agents already does, and, for copies Tisty wrote before, whether
  they are still there. With the yes to Agents alone, it reads the plugin's
  records named below and any copy Tisty wrote before, to tell whether it
  changed.
- **With it, Tisty reads and writes only this:** one folder per skill in each
  agent's skills folder on this computer (such as `~/.claude/skills/tisty`,
  `~/.cursor/skills/tisty` and the shared `~/.agents/skills/tisty`), the other
  files already in those folders, to leave alone any skill that is not
  Tisty's, and, for Claude Code, `plugins/installed_plugins.json` and
  `settings.json`, only to tell whether the Tisty plugin already brings them.
  Nothing else in those files is kept, copied or shown. No rule, memory or
  other file of your agents is opened.
- **Pausing Agents does not reach them.** A pause, for one agent or for the
  whole folder, holds only your own documents; the skills follow their own
  switch.
- **They never write over your files.** A skill folder holding a file Tisty did
  not write, or a copy you changed, is left as it is. The one exception: a
  folder holding exactly the files of this release, byte for byte and nothing
  else, counts as Tisty's copy, so it updates and goes when you turn them off.
  They update by themselves when you open a newer Tisty.
- **Turning them off** takes out of your agents' folders only the copies still
  as Tisty wrote them, after a backup kept in `data/agents/`; the skills stay in
  Tisty, ready to turn on again, and are never deleted. A copy you changed is
  left for you to take away from the same card. No file is ever taken away on
  its own or through the MCP.

## The door you can open for an assistant

Tisty has no AI inside it and makes no request to any model. It does speak
[MCP](https://modelcontextprotocol.io) when you ask it to, so an assistant
already running on this machine can file a task on your behalf. None of it
happens on its own:

- **You install it.** Nothing listens until you run `tisty mcp`. The assistant
  reaches it through that process's own input and output — on this machine,
  with no port, no account and no token. It is not a server and nothing can
  reach it from the network.
- **You hold the door.** It comes open on its own until you choose — on a new
  install, and on an update if you never closed it — and Settings › Agents ›
  Door closes it with **Stop it**; while it is closed, everything an assistant
  tries is refused. Letting an assistant in and throwing it out are events in
  your log, not lines in a settings file, so an assistant cannot let itself in
  by editing a file it can reach. That you chose to keep the door closed is kept
  in this computer's settings.
- **What it may do is bounded.** File a task, add to the journal, say a task is
  done, yours too — which marks it for you and closes nothing — work any task
  the way it works a document, write a
  document, write one again whole, mark a document that had its day for you to
  delete, archive or keep, put one away or bring it back when you ask, jot
  ideas on your Ideas sheet and mark the old ones the same way, keep a copy of a
  file you point it at, and read what is already there. Writing a document
  in Agents only leaves a proposal: nothing reaches the files your assistants
  read until you approve it on the document's card, and then only on the
  computers where Agents syncs and you applied it. It may not
  close or delete anything, tick a step of yours or finish a task of yours,
  move a day you set, reach a task you hid or dropped — where you linked one,
  it reads «a task out of sight», and your link comes back intact if it
  rewrites that document — read your diary until you open it
  to agents from the Diary itself, change what you wrote in it once you do —
  open, it only adds to the end of today, signed, and makes the month's page if
  today has none — or start a diary where there is none, or
  take files from anywhere but Downloads, Documents, Pictures, the desktop and
  the temporary folder. The diary's door holds on Tisty 1.48 or later, so the
  Diary does not start one while a computer of yours runs an older version, and
  once one exists a computer on an older version stops syncing before it
  receives it.
  Bringing a markdown file in copies what that file's text points at, and only
  what sits in that file's own folder or below it, so what leaves your disk is
  bounded by the file you named rather than by what its text asks for. To
  replace a whole document it has to send back the print it read that document
  at, so what you wrote while it was thinking cannot be taken. What it reads
  comes back without the paths of your disk.
- **Its hooks, if you add them.** Settings › Agents › Connected can add Tisty's
  hooks to an assistant's own settings file; it names the file and asks first,
  and leaves the rest of that file as it is. If the file held anything, a copy
  of it as it was stays beside it, as `.before-tisty`. With them, a session
  starts with the tasks it left open, what moved and how much is pressing, and
  after each edit and each call to Tisty a mark is kept in Tisty's cache on this
  computer, without reading the conversation. The marks never leave it, and
  **Remove** takes the hooks out.
- **What it keeps for itself.** Where each assistant last caught up is kept
  in Tisty's private data, so the next session hears only what moved, and
  never leaves this computer. A summary it leaves on a long document is an
  event in your log, so the next assistant on any of your computers reads it:
  it syncs, goes into your cloud and your backups, and never leaves through a
  Folio link or a `.tistyx` parcel. Neither is shown in the window.

The part worth knowing before you connect one: **whatever that assistant reads,
it takes with it.** If it runs against a service, then the moment you ask it to
look, what it looks at is on its way to that service. Tisty still sends nothing
— but this door is real, and besides syncing, sharing a document by link and the
files Agents writes for your assistants while it syncs, it is the only thing
here that can carry your data off this machine. It comes open on a new install,
and **Stop it** closes it whenever you like.

## Children's privacy

Tisty collects no personal information from anyone, of any age. There are no
accounts and no registration, and nothing of yours is sent to us, short of a
document you share by link, which goes sealed.

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
