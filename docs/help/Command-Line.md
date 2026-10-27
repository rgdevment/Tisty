# Command line

Since 1.31, Tisty has no command line. The window is the way in, for your tasks and your documents alike.

What remains of the `tisty` program is one thing: **`tisty mcp`**, the door your assistant comes through. Any other command only prints a notice that the command line was retired and sends you to the window.

## Where it lives

It comes inside the app: you do not install it on its own, and it does not need to be on your PATH. Since 1.32.1 it also answers to plain `tisty` while the door for assistants is open (see [[Assistants and MCP|Assistants-and-MCP]]). **Settings › Assistants** shows where it sits on this computer, and connects the assistants it finds installed.

## Connecting an assistant with it

For an assistant Tisty does not know, one line does it, with the full path that **Settings › Assistants › How to connect one** shows, ready to copy:

```console
$ <your-assistant> mcp add tisty -- <path-to-tisty> mcp
```

Where `<your-assistant>` is whatever yours is called. Then let it in from **Settings › Assistants**. The rest is in [[Assistants and MCP|Assistants-and-MCP]].

## If you used the command line

- What it did for maintenance lives in the window: **Sync now** and **Send everything again** in Settings › Sync, confirming another machine on its notice or under **Settings › Sync › Your computers**, the store review in **Settings › Maintenance**, and the backup zip in **Settings › Your data**.
- Scripts that called `tisty add` or `tisty ls --json` stop working. The [[MCP door|Assistants-and-MCP]] is how a program reaches Tisty now.
- The Homebrew formula `tisty-cli` and the loose zips of the command line are no longer published. If that is all you installed, it stays on 1.30: install the app instead ([[Installation]]), and it finds your tasks and documents where they are.
