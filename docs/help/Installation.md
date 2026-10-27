# Installation

Tisty runs on **Windows** and **macOS**. There is no Linux build today and no mobile app.

## Windows

**Microsoft Store** — [Tisty in the Microsoft Store](https://apps.microsoft.com/detail/9PGVWXD8X93N). The Store keeps it updated; *About* can ask the Store for an update on the spot.

## macOS

**Homebrew** — the tap is added once. After that Tisty keeps itself up to date:

```console
$ brew tap rgdevment/tap
$ brew install --cask tisty
```

The `tisty-cli` formula, which installed only the command line, is no longer published since 1.31: see [[Command line|Command-Line]].

## Direct download

[GitHub Releases](https://github.com/rgdevment/Tisty/releases/latest) carries the signed installers:

- Windows: `.exe`
- macOS: `.dmg`, one per chip — `aarch64` for Apple Silicon, `x86_64` for Intel. Apple menu › *About This Mac* says which yours is. Homebrew picks on its own.

## Are the downloads signed?

Yes. The macOS disk images are signed and notarised by Apple, and the Windows installer is signed too. Tisty also refuses to install an update that is not signed with a key compiled into the copy you are running. Which licence applies to which build is explained in [DISTRIBUTION.md](https://github.com/rgdevment/Tisty/blob/main/docs/DISTRIBUTION.md).

## The first time it opens

Tisty asks two things:

1. **Which language** — English or Spanish. The window, the guide and the natural language all follow it, and Settings changes it whenever you like.
2. **Where your copies should go** — a folder for [[syncing|Sync-and-Backup]], or nowhere if you only use one machine.

Then it **writes you a guide and opens it**. The guide is an ordinary document in your own store, in a folder of its own — yours to read, edit or throw away — and it shows from the inside what a document can hold. You can come back to it from Settings.

Everything else has an obvious default you can change later: Tisty starts with the computer and waits out of the way.

## Settings

Settings takes over the sidebar with its sections:

- **General** — how Tisty behaves towards you: language, look, start-up, the quick capture, notices and the welcome.
- **Sync** — where your copies go, your computers, and where large attachments stay.
- **Security** — your 24-word phrase and [encrypting your cloud](Sync-and-Backup#encrypting-your-cloud).
- **Your data** — where it lives, backups, documents from another Tisty, and leaving.
- **Assistants** — the [[MCP door|Assistants-and-MCP]].
- **Maintenance** — the store review, loose attachments, freeing up space and reporting a problem.
- **About** — the version, updates and release candidates.

## Updates

Tisty checks once when it opens, and once a day after that, whether a newer version exists — it sends nothing to ask. If one exists, a line appears in *About*:

- a Microsoft Store copy is updated by the Store;
- a copy that owns its own folder gets a button.

Nothing is downloaded unless you press it. Release candidates can be turned on in *About*. Details in [[Privacy]].

## Where your data lives

- **Windows:** `.tisty` in your user folder — hidden, and closed to the other accounts on the computer. Turn on **Hidden items** in File Explorer's **View** menu to see it.
- **macOS:** `Library/Application Support/tisty` and `Library/Caches/tisty` in your home folder.

**Settings › Your data** prints the path, and *Show folder* opens it.

## Uninstalling

Uninstalling takes the program away and **leaves your tasks and documents where they are**, so reinstalling finds them as they were. To leave for good, delete that folder yourself.
