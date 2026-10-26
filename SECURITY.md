# Security policy

Tisty stores tasks, notes and documents — material that is often personal and
sometimes confidential. This document explains what the design protects, what
it deliberately does not, and how to report a problem. For what is collected
and where it goes, see [PRIVACY.md](PRIVACY.md) — the short answer is nothing
and nowhere.

## Reporting a vulnerability

**Do not open a public issue.** Use either private channel:

- **Email** <rgdevment@tisty.app>, subject `[SECURITY] short description`.
- **GitHub Security Advisory** — the *Security* tab, *Report a vulnerability*.

Useful in a report: what an attacker could do and to whom, steps to reproduce,
the affected version, and your operating system. A proof of concept helps but
is not required.

You will get an acknowledgement within 48 hours. Fixes for anything serious
are aimed at days, not weeks. You will be credited in the release notes unless
you prefer otherwise.

Good-faith security research will never be met with legal threats, and a
report will never be dismissed for being inconvenient.

## What the design protects

- **The data never leaves your machine unless you tell it to.** No account, no
  telemetry, no remote server. There is no service of ours to breach because we
  run none. Tisty makes one request of its own — a daily GET for a
  version manifest, carrying nothing but the version you are running — and two
  more only if you press the button that offers you an update. A copy from the
  Microsoft Store makes neither request: it asks Windows what the Store has for
  it. It is written out in full in [PRIVACY.md](PRIVACY.md), down to the
  headers.
- **A cloud account is reached only if you choose it.** Tisty can sync straight
  with your own Google Drive since 1.30, your own OneDrive since 1.31 and a
  WebDAV server of your own since 1.32; Dropbox is reached only through a folder
  its desktop program keeps in step. With Google it asks only for
  `drive.appdata` — so it sees its own hidden space and nothing else in your
  Drive — and gives back any access granted wider than that; with Microsoft it
  asks only for its own *Apps › Tisty* folder and refuses a sign-in that grants
  more. A WebDAV server is reached only over `https`, except on this very
  computer, never through a redirect, and only inside a `Tisty` folder of its
  own; it is not connected unless it proves, before anything is kept, that it
  refuses to write over a copy it was not told about. Tisty keeps the access it
  was given, or the WebDAV password, in the system keychain on macOS and, on
  Windows, in a file sealed for your user with DPAPI under your profile that
  uninstalling removes, and masks tokens, passwords and signed addresses in its
  own log. Only the window holds it and only the window connects an account;
  assistants never reach the network. Nothing goes to a server of ours. A daily
  allowance per computer keeps one copy from spending the project's shared
  quota, and the one remote switch that can lower it or pause syncing,
  `cloud.json`, is refused unless it is signed with minisign against a key
  fixed in the build, is no larger than 16 KiB and is newer than the last one
  heard. You can withdraw Tisty's access from the window, from your Google or
  Microsoft account, or by removing the password on your server, at any time.
  What it asks and sends is in [PRIVACY.md](PRIVACY.md).
- **An update is verified before it is installed.** What the button fetches is
  checked against a public key compiled into the copy already on your machine.
  A build signed with any other key is refused, so a release that is not the
  maintainer's cannot arrive this way, whatever it claims to be. Two narrower
  guards sit in front of that one: the address an update is fetched from must be
  where this project's releases live, not merely whatever the feed named, and the
  signature must name the very version offered, so an older signed installer
  cannot be passed off as a newer one; and
  the download is given a deadline, so a server that answers slowly forever is
  eventually hung up on. A copy kept by the Microsoft Store never fetches an
  installer at all: it asks the Store what it has and lets the Store install it,
  so what stands behind that package is Microsoft's signature rather than the
  key below. The rule that nothing older than what is running may be offered
  holds there too.
- **What the world sends is read to a limit, never to the end.** One preview of
  one link, asked for by a button, is the only request Tisty makes to an address
  it did not choose. A page is read to 256 KiB and the picture it offers to
  400 KiB — read *to* the cap off the wire, not fetched whole and measured
  afterwards, so how much of your memory a page takes is not that page's
  decision. The request has a deadline of six seconds, the answer is parsed
  without executing anything, and what is kept is bounded by weight as well as
  by count, so the cache cannot quietly become a folder of somebody else's
  pictures.
- **Synchronisation is yours.** Tisty copies files into a folder you name, and
  whatever already keeps that folder in step between your machines is what moves
  them — or into a cloud of your own that it reaches itself. There is no
  Tisty-operated backend at any tier, and no transport of ours to attack: with a
  folder, the code that syncs reads and writes local paths, and with a cloud it
  speaks only to that provider or that server.
- **A cloud can be encrypted so that its provider cannot read it.** Since 1.36,
  what Tisty keeps in Google Drive, OneDrive or a WebDAV server can be sealed
  with a key that comes from a 24-word phrase only you hold. Neither the
  provider nor this project holds that key. It is described in its own section
  below.
- **An older Tisty stops instead of writing over a newer one.** Since 1.34,
  every place Tisty syncs through says in its `tisty.toml` the oldest release
  that may write there (`needs`), and every release since 1.30 reads it: one
  older than that stops and asks to be updated, and neither reads nor writes.
  An encrypted cloud raises it, so a release that does not know encryption never
  writes plain files beside the sealed ones. Each machine also writes into the
  log the release it runs, so Tisty can say which computer is behind.
- **No plugin system.** Nothing loads third-party code into the process.
- **`unsafe` is forbidden** at the workspace level, not merely discouraged. The
  window's crate is the one exception, and a narrow one: it denies `unsafe`
  too, and lets it in only where the operating system offers no other door — the
  webview on both systems, `sysctl` on macOS, and on Windows the shell, the
  imaging component, a program's version information, the notice that the
  path changed, DPAPI and the Store's dialog — each function marked one by one,
  in six files, and a test fails if a mark appears in any other.
- **A release is held to what CI already proved.** A tag on a commit that did
  not pass CI on `main` is not released. The updater's signature is checked
  against the key the build ships before anything is published, and every file
  on the releases page comes with a `SHA256SUMS` and a build-provenance
  attestation, so a download can be traced back to the workflow that made it.
- **The window is fenced by a content policy.** Everything it runs is shipped
  inside the program: no script from anywhere else, no page from anywhere else,
  and the only address it may talk to is the local channel to the Rust side.
  Three openings are worth naming because they are real. Scripts may compile
  WebAssembly, which is what draws the PDF you see before exporting it; and a
  frame may show a `blob:`, which is that PDF, built in memory on your machine
  and never fetched. Neither lets remote code in: a blob has no origin to be
  loaded from, and there is no path by which one arrives.

  The third does run code that was not shipped: a widget block in a document,
  or an attached `.html` page, is drawn live — its own HTML, CSS and script —
  in a frame of its own. That frame is sandboxed with scripts and nothing else,
  so it has no origin and cannot reach into the window, and it is served under
  a policy of its own that allows no network at all: no fetch, no remote
  picture, font or frame, no form. An attached page that is a whole document
  may also build scripts out of `blob:` and `data:` it makes itself, and frames
  out of `blob:` only, which bundled pages need in order to unpack, and still
  reaches no address.
  All it can say to the window is how tall it is and which link was clicked
  inside it, and a link is opened only after a real click there. A widget is as
  trustworthy as the document that carries it, and a document can arrive
  through the shared folder or in a parcel.

## What it deliberately does not protect

Being explicit here matters more than sounding reassuring.

- **Data is stored in plain text on your computer.** Tasks, notes and documents
  sit readable on disk, protected only by your operating system's file
  permissions. At-rest encryption on disk was evaluated and rejected: it would
  put every reader of your data behind a key Tisty would then have to manage,
  and the promise here is that your files stay readable with tools you already
  have. Encrypting your cloud does not change this: it seals only the copy the
  provider holds, and each of your computers decrypts what it brings down and
  keeps it plain on its own disk. A folder kept in step by a desktop client is
  never encrypted.
- **An append-only log does not forget.** If a credential ends up in a task,
  deleting the task removes it from the current state but not from the history
  underneath — and if it already travelled to the shared folder, it is on the
  other machines too. Prevention is the only real defence: keep anything
  sensitive in the `private/` folder, which never leaves the machine.
- **The installer and the feed are both signed.** Each installer is signed with
  the version it was cut for, and Tisty requires that version to match the one
  it was offered, so an older Tisty — genuinely signed, genuinely ours — cannot
  pose as a newer one. Every file of the feed — the manifest that announces
  versions (`release-manifest.json`) and the channel files the updater reads
  (`latest.json`, `candidate.json`) — carries a minisign signature of its own,
  under a key kept apart from the updater's and only where the release publishes
  the feed. A copy built with its public half announces no version from a
  manifest it cannot verify, and installs nothing unless the channel the updater
  read is the very one that key signed.
  What a signature cannot do is make the feed speak: whoever could write to it
  could still leave it unchanged. Copies from before the feed was signed read it
  as they always did, and update the same way.
- **The updater's signing key is a single point of failure.** One private key
  signs every release, it is held by the maintainer alone, and the format offers
  no way to rotate it: the public half is already compiled into every copy out
  there. If it leaked, a build signed with it would be accepted by installed
  copies; if it were lost, those copies would go on working and simply stop
  being able to update themselves. Either way the fix is a new release installed
  by hand, and it would be announced on the releases page. None of this reaches
  a copy kept by the Store, which stands on Microsoft's signature instead — a
  single point of its own, but not one this project holds.
- **Whoever holds the shared folder holds your data.** Syncing through a
  provider's folder means that provider stores your tasks, under their terms.
  That is the trade you make when you turn it on, and Tisty states it rather
  than hiding it. An encrypted cloud narrows it to what the encryption cannot
  hide, listed in its section below.
- **Anyone with access to your user account has access to your data.** Tisty
  adds no second authentication layer on top of your operating system's. On
  Unix it does narrow the permissions it controls — its directories to `0700`
  and its files to `0600`, owner only — but that stops another local account,
  not you, and not anything running as you. On Windows the folder that holds
  the signing key and the store's secret is cut off from the permissions it
  would inherit, and your account is given full control of it. The whole
  `.tisty` folder in your profile is treated the same way and hidden, so
  other accounts on the computer cannot open it and it does not sit in plain
  sight where it could be moved or deleted by mistake. The system account
  keeps its access, so a backup tool that runs as a service still copies it;
  administrators lose the access they inherit from your profile, but can still
  take ownership of it, as of any folder. The key folder inside it stays yours
  alone. On other systems,
  what keeps another account out is that your profile's own folders are
  yours.
- **Whoever can write to the shared folder can try to write to your history.**
  Each machine signs what it writes: an Ed25519 key of its own, kept in the
  `private/` folder of its local configuration — never synced, never in a
  backup — signs every write to its log, in the same line and the same write as
  what it signs, so a log never travels ahead of its signature; logs written
  before the 1.25 keep theirs in a file beside them, and the first signed write
  answers for them too. A machine that has said what it signs with owes a
  signature from then on, and
  a history written at the signed schema or later that arrives with none is not
  taken in at all. A machine whose key nobody here has confirmed is not trusted
  on the folder's word: what it writes waits until you confirm its key, in the
  window, or until you let that computer into an encrypted cloud with the
  twenty digits it shows, which confirms its key in the same step. Those
  confirmations are kept on this machine only, and a key once
  confirmed is replaced only on its own word: a change of key that the
  confirmed key itself signed is followed, and a new key nobody vouched for
  still waits for you. Nothing in this version
  changes a key; it only reads such a change, so that a later one can. The same
  word does not help against a key already stolen, since whoever holds it can
  sign a change too: that machine is removed and confirmed again by looking. The
  one moment the folder is taken on its
  word is the first time this machine reaches a folder, before it has confirmed
  any other: the keys of the machines already writing there are adopted as
  found, so the folder you join should be one you trust.

  The signatures answer for the log, and not for everything beside it. A
  document body that matches none of the prints the log holds for it is never
  taken in: when only another machine changed it, it waits up to an hour on
  this machine's clock for its history, and then you are asked. A document
  body that the log holds no print of at all, from before every body had one,
  is taken in with what this machine held set aside, since nothing answers for
  it either way. An attachment is held to the full digest the log records for
  it, by the sync round and by the window alike; one the log has no digest for
  is held only to the prefix its name carries. And `tisty.toml`, which names
  the store, is not signed: what it says of encryption is held to the signed
  log, as the encrypted cloud's section below explains. Someone who can write to
  the folder can also delete from it, and no signature brings back what was
  removed.

  What it does check besides is narrower and worth stating: it never writes
  through a symbolic link, into the folder or into any directory inside it; an
  attachment must hold the bytes its own name vouches for; one that was retired
  is not carried back in; a document body past the reader's ceiling is refused
  rather than swapped in for one that opens; a store carrying a different name
  stops everything until you decide; a place that asks for a later release than
  this one is neither read nor written; and a machine that was removed is refused
  outright — that removal is absorbing, so a removed identifier is never valid
  again and a machine that returns comes back as a new one.

- **A parcel of documents is a file somebody hands you**, and it is treated as
  such. Every path inside the zip must be an ordinary relative path under
  `docs/` or `attachments/` — anything climbing out with `..`, an absolute path
  or a drive prefix is dropped rather than written. The names written in the
  manifest go through the same door as any other document name, so a manifest
  cannot point the reader at a file elsewhere on your disk. The manifest itself
  is read under a ceiling, and so is the parcel: bytes actually written are
  counted rather than the sizes the archive declares, and both the number of
  files and the number of entries in the manifest are capped. A folder name or
  an alias arriving inside is trimmed to the same limits the window applies, and
  an icon or a colour it does not recognise is dropped instead of stored. A
  parcel can say a folder was in the archive, and one it creates lands that way;
  what it cannot do is close a folder you already have, because joining by name
  leaves the folder that was already here deciding for itself. The same holds for
  a page put away on its own and for what an assistant marked: both travel, and
  both land as what they are, but the mark arrives without the machine that wrote
  it, because a parcel goes to other people.

  What a parcel cannot do is prove who wrote what: anyone can edit the manifest
  in a zip, and a name in a parcel is a claim, not a signature. What it can prove
  is which store wrote it. Every store keeps a secret apart from its name — the
  name travels in every parcel, the secret never leaves the machine, and it is
  kept outside the store so that nothing walking it, neither a sync nor a backup,
  can carry it off — and the manifest carries an HMAC of itself under it. So a
  parcel that arrives wearing your store's name, from somebody who read that name
  off a parcel you once handed them, lands as a stranger's: they cannot forge the
  seal, and neither claiming your name nor stripping the seal off gets them any
  further.

  Keeping it out of the copies has a price, and it is worth saying plainly. A
  backup carries the store's name and not what proves it, so a store restored on
  another machine cannot recognise the parcels it handed out before: they land as
  a stranger's writing. Taking Tisty off this machine leaves the secret where it
  is, beside your documents, so reinstalling here keeps that recognition. And a
  backup made before this was so still has the secret inside it, in the clear:
  the copies you already have are worth protecting like the documents they hold.

  The secret belongs to the store, not to the machine, and it is named after it:
  a machine that holds two stores holds two secrets, and neither can write over
  the other. That is also what makes starting over reversible — it mints a new
  name for the store, so the old secret simply stays under the old one, and the
  safety copy written moments before can be put back and recognised with nothing
  done by hand. A secret is never destroyed, only displaced: what a build has to
  move out of the way is kept under the hour it was replaced, and putting one
  back is renaming it. Leaving takes
  the diary and the settings, never any of this.

  The other side of that: a store opened with somebody else's settings — a data
  directory named on its own, without the configuration that goes with it — has
  its secret left exactly where it is. Nothing is moved out of a store that this
  install cannot account for.

  The one parcel that carries proof is the one you lock. Exporting everything to
  another machine of your own seals it with a number: XChaCha20-Poly1305 over
  64 KiB blocks, under a key scrypt grinds out of that number, with each block's
  nonce carrying its count and a last-block byte, so a file cut short will not
  open as a whole one. Opening it is the proof — nobody who lacks the number can
  claim what is inside as their own writing — and what was somebody else's where
  it was packed stays theirs at the other end. Lose the number and the parcel is
  gone: nothing here can open it for you. A short number is a short number, and
  scrypt only makes each guess expensive rather than impossible.

## An encrypted cloud, if you turn it on

Since 1.36, what Tisty keeps in your Google Drive, your OneDrive or your WebDAV
server can be encrypted from Settings › Security. It is never offered for a
folder kept in step by a desktop client. What follows is what holds it, and
what it leaves in view.

**One phrase, and keys that come from it.** The phrase is 24 words from the
BIP-39 lists, in English or in Spanish, and it is the whole of a 32-byte seed
made from the system's randomness on your computer. From that seed, by
HKDF-SHA256 under labels of their own, come the key of your cloud and the
Ed25519 root of your identity; the log carries only the root's public half, as
your fingerprint, and the certificates it signs for each computer. The seed is
kept on the computer where it was made, in the system keychain on macOS and, on
Windows, in a DPAPI-sealed folder apart from the cloud tokens, so the
installer's uninstaller takes the tokens and leaves the phrase. It is never synced, backed up or
put in a parcel. Encrypting asks that the phrase be confirmed first, by three of
its words.

**What seals the files.** Each file is sealed with XChaCha20-Poly1305 over
64 KiB blocks, under a nonce that carries a random head for the file, the
block's place and whether it is the last, so no block can be dropped, cut short
or moved. The seal is bound to the cloud and to the file's name, so one sealed
file cannot pass for another, and a file that does not open is refused rather
than kept in part. A small mark, `sealed/sealed.mark`, sealed the same way,
tells whether a phrase is this cloud's before anything comes down; the
identifier it carries is tied to the cloud too, so two clouds' marks never show
that they belong to one person.

**What it does not hide.** The names of the files, their sizes, when they
change and how many computers you have stay in view; documents are named by an
identifier, but an attachment's name is a lowercased form of its file's own.
`tisty.toml`, `moved.json`, the store's identifier and the `knock/` folder stay
plain. Whatever the provider kept before you encrypted — its version history,
its bin, the unencrypted copy until you delete it — is still there. A file too
big for the provider to take encrypted is left out and stays as it was.

**The provider cannot turn it off for you.** The log, which every machine
signs, says that the cloud is sealed and with which key, and only that key —
or, on a computer whose log has not heard yet, the one that comes from the
phrase the log names as yours — is used to read or to mark it. If `tisty.toml`
stops saying the cloud is encrypted and none of your computers left a signed
notice that it turned encryption off, Tisty writes nothing there unencrypted
and tells you. That notice, `knock/unsealed.json`, is believed only from a
machine the log seats, signed with the key the log knows for it, naming the key
the log names, and later than the log's word that the cloud was sealed.

**How the key reaches your other computers.** It travels only inside the
cloud, in the plain `knock/` folder, and never as itself. A computer that needs
it puts up an ask signed with its own Ed25519 key, carrying a one-use X25519
key that lives only in its memory. A computer that holds the cloud's key
answers with that key sealed for the one-use key alone: XChaCha20-Poly1305
under a key drawn by HKDF-SHA256 from the X25519 exchange and bound to the ask.
A replayed ask opens for nobody, since its one-use key is gone. Each file there
is read to 4 KiB at most.

**A computer already in your log gets it on its own.** Its ask is answered only
if it is signed with the key the log knows for that computer, and its answer is
taken only from a computer the log seats with the key it knows, or one whose
key your identity's root has certified. The mark sits with the provider, who
could seal a mark of its own, so a mark that opens is never enough on its own.

**A new computer is let in by comparing twenty digits.** It shows a code drawn
with scrypt from its ask, from the key of the computer that took the ask, and
from a secret it committed to in the ask and reveals only after that computer
took it — about 66 bits, so a provider standing in the middle can neither pick
nor grind out a code that matches. You type the twenty digits on the computer
letting it in, and that is what confirms the new computer's signing key there;
then you say on the new computer that the other took them, and only then does
it accept the answer. If you type nothing, nothing is handed over. An ask from a
new computer lasts an hour.

**Typing the phrase keeps only the cloud's key.** On a computer that holds
neither, you can type your 24 words instead. The mark is checked first, the
identity signs for that computer once, and only the cloud's key is kept, in
the keychain or the sealed folder; the phrase is not.

**Nobody else can open it.** There is no copy of the key outside your
computers and the phrase. If you lose the phrase and every computer that holds
the key, the encrypted cloud cannot be opened by you, by the provider or by
this project.

**Going back.** *Decrypt my cloud* copies everything back unencrypted, leaves
the signed notice so your other computers follow, lets go of a key it was
handed, and deletes the encrypted copy. While a turn either way is under way,
`tisty.toml` says so, and for its last steps the other computers wait rather
than write; until the very last, cancelling loses nothing.

## An assistant, if you admit one

Tisty speaks MCP so an assistant already running on your machine can file work
for
you. That door is shut until you open it, and what follows is what holds it.

**Admitting one is an event in your log, not a line in a settings file.**
Nothing
arriving over the wire can register an agent; the log decides who may write, so
an
assistant cannot grant itself a voice by editing the file it can reach.

**It writes under a device of its own.** Throwing it out leaves everything it
wrote in place. What it files is tagged,
and it
may only file into a list that already exists — it cannot make one.

**It cannot reach what you hid.** A task you folded away is neither returned nor
counted; a count of one would say the thing exists. What it reads comes back
without
the absolute paths of your disk.

**What it says about work being finished only ever adds.** An assistant can mark
a task it filed as one it has finished, with the account of what it did. It
cannot mark a task you wrote unless you opened that task to agents, it cannot
close anything, and the mark takes nothing out of your list: what is due today
or overdue stays in the band you look for it in. Saying it twice over a mark you
have not seen yet is refused, so a text it reads somewhere cannot fill your
journal by repetition. Taking the mark off is yours alone — and doing so keeps
the account, so you can see what it claimed and why you disagreed.

**A body it did not read is a body it cannot replace.** An assistant may write a
whole document again, not only add to it — but only by sending back the print
that document read at when it last looked. If you have written in it since, the
write is refused and it has to read again. It can be wrong about a document; it
cannot quietly take what you wrote while it was thinking. What it wrote over is
kept beside the documents either way.

**It can take one step back, and only when that step is the whole distance.**
That kept body is not only held, it is reachable: `restore_doc` puts it back,
and what it replaces is kept in turn, so going back destroys nothing and going
back twice leaves the document where it started. But not every write keeps a
body — adding to the end does not, and your own saves from the window do not —
so what is kept is not always one step behind. Kept alongside it is the print
the document read at when it was set aside, and going back is refused unless
the document still reads exactly that way. An assistant that wrote this
morning does not undo an afternoon of yours by reaching for the wrong tool.

It can still do it on purpose: `even_if_more` goes back over writing done
since, and an assistant that has read the document may decide that is what it
wants. What that call writes over is kept in turn, so the same call again puts
your afternoon back — and the answer says it went further than one write, which
is the thing to look for if a document is suddenly shorter than you left it.

**Attachments are the one thing that leaves the machine.** They reach the shared
folder, and from there whatever cloud client you run. So the file an assistant
may
copy is judged by its bytes rather than its name: pictures, PDFs, plain text and
office documents pass; a private key, a PKCS#12 bundle or a file shaped like an
environment file is refused even when renamed to `.png`. Files outside
Downloads,
Documents, Pictures, Desktop and the temporary folder are refused before
anything is
read, and the path is canonicalised first, so traversal does not reach past it.

**And a file an assistant copies is one you pointed at, not one a document
chose.** Bringing a markdown file in copies what its text points at beside it, so
the document does not break the day it moves — and «beside it» is the rule rather
than a turn of phrase: only what sits in that file's own folder, or below it, is
followed. A link that climbs out with `..` is not fetched; its link is taken out,
its words stay, and the answer names it. The five folders above are still the
ceiling, but they are no longer the fence: the text of a markdown file somebody
sent you cannot pick which of your own files are copied into a store that syncs.

**What this does not protect against.** An assistant under prompt injection is
still
an assistant with your permission: it can file nonsense, write documents you did
not ask for, write over one you already had with a version of its own, put back
over a document a body from before you wrote in it — which it has to ask for,
and which is kept and can be put back — and
put a screenshot holding a password into a task, because a
screenshot is a picture and Tisty cannot read what is in it. Anything it reads
travels wherever that assistant travels — that is between it and you, and it is
why the door stays shut until you open it, and why closing it is one click.

## Supported versions

Until the first stable release, only the latest published version receives
fixes.
