# Assistants and MCP

**Tisty is not AI and has none inside.** The natural language that turns «call the bank at 3» into a task is rules running on your machine: no model, no request, no cloud. That is not going to change.

But if you already use an assistant, you are probably telling it things worth keeping — the school group says card stock on Monday, the invoice is due on the 30th, the bug you just fixed and how. So Tisty leaves a door, and you decide whether to use it.

**Your agents' work, in the open.** Through Tisty, an agent plans in documents and works in tasks: it writes the plan, what it found and what it decided as Markdown, breaks the work into tasks with steps and ticks them off as it goes. Nothing stays buried in a chat: every stage lands in Tisty, on your computer, where you can follow it as it happens and audit it later. When it says a task is done, you are the one who closes it.

![Bring your favourite agents through MCP: they organise, document, schedule and prioritise while everything stays on your computer](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/store/en-07-mcp.png)

## How the door works

Tisty speaks [MCP](https://modelcontextprotocol.io) through `tisty mcp`, over its own standard input and output. That `tisty` comes inside the app and is not a command line any more: it opens only the assistant's door (see [[Command line|Command-Line]]).

- **On this machine only.** No port, no account, no token. It is not a server, and nothing can reach it from the network.
- **Nothing listens until you connect it.** The assistant starts `tisty mcp` when it needs it.
- **You hold the door.** It is open on a new install, and **Settings › Assistants** closes it with one click. While it is closed, everything an assistant tries is refused. Opening and closing it is written into your log.

Since 1.32.1, while the door is open, Tisty also answers to plain `tisty` from any terminal: the installer's copy on Windows and the Homebrew one on a Mac. If you have the Microsoft Store copy and the installer's at once, the newest answers. Close the door and `tisty` stops answering. An assistant that was already open needs a restart to notice.

## Connecting an assistant

### From Settings (the easy way)

**Settings › Assistants** finds the assistants installed on this computer among the ones Tisty knows (Claude Code, Claude Desktop, Codex, GitHub Copilot CLI, VS Code, Cursor, Devin Desktop and Windsurf, Antigravity, Gemini CLI, Qwen Code, OpenCode, Kilo Code, Zed, Junie, Kiro, Warp, LM Studio, Amp, Augment, Factory Droid, Trae, Cline, Roo Code, BoltAI, Goose and Continue) and says how each one stands:

- **Connected** — it starts this Tisty.
- **Not connected** — **Connect** writes Tisty into its settings, in the shape that assistant reads.
- **Points at another Tisty** — an older copy, or one that moved. **Fix** points it here.
- **May be here** — only its folder is left, so it may have been uninstalled.
- **By hand** — Goose and Continue keep their settings in YAML, which Tisty does not write. The row shows what to paste.

**Connect the ones found** connects every one waiting at once. Some files serve more than one assistant: GitHub Copilot CLI's is also read by VS Code, and Codex's by ChatGPT and Warp.

An assistant reads its settings when it starts, so after connecting it, quit it completely and open it again. If it has a window, closing it is not enough: on Windows it keeps running by the clock until you quit it from there, and on a Mac until **⌘Q**.

Tisty only touches its own entry. Anything you added to it, like `env` or the tools you allowed, stays, and so does the rest of the file. The first version of each file is kept beside it as `.before-tisty`, and the one just before each write as `.before-tisty-last`.

The **⋯** on every row shows how to connect that assistant by hand: its own command when it has one, and the exact lines for its file. The search box below the list does the same for an assistant Tisty knows but did not find.

### With a plugin

Some assistants take Tisty as a plugin. The plugin carries a small launcher that finds the Tisty installed on this computer (the Microsoft Store copy, the installer's, the Mac app or Homebrew), uses the newest when there are several, and starts that copy's own `tisty mcp`, so what the assistant writes always matches your app and your data. If Tisty is not installed, the assistant gets a single tool, `install_tisty`, that tells you how to get it. The launcher needs Node.js.

In Claude Code:

```console
/plugin install tisty --marketplace rgdevment/mcp-plugins
```

Any other assistant that starts a server by command, such as Cursor, Cline or VS Code, takes the same launcher from npm:

```json
{
  "mcpServers": {
    "tisty": { "command": "npx", "args": ["-y", "tisty-mcp"] }
  }
}
```

The first start downloads the launcher and can take a moment; later ones are immediate. If Tisty lives somewhere unusual, set `TISTY_PATH` to the full path of its `tisty`.

Connect each assistant one way, the plugin or **Settings › Assistants**, not both: otherwise it runs Tisty's server twice and sees every tool twice. The plugin's source and its issues are in [mcp-plugins](https://github.com/rgdevment/mcp-plugins).

Tisty is also listed where assistants look for servers: the official MCP Registry as `io.github.rgdevment/tisty`, the [GitHub MCP Registry](https://github.com/mcp/rgdevment/tisty), [npm](https://www.npmjs.com/package/tisty-mcp), [Glama](https://glama.ai/mcp/servers/rgdevment/Tisty) and [MCP Market](https://mcpmarket.com/server/tisty).

### Assistants inside WSL

On Windows, an assistant that runs in a Linux distro under WSL reads its settings in Linux, where a `C:\...` path does not exist. **Settings › Assistants** lists your distros; **Look in Ubuntu** reads that distro's files (starting it if it was stopped) and connects its assistants to the same Tisty, through its Linux path:

```console
$ claude mcp add --scope user tisty -- /mnt/c/Users/<you>/AppData/Local/Programs/Tisty/tisty.exe mcp
```

It is the same Tisty and the same tasks and documents as on Windows. If a distro has Windows programs turned off (`[interop] enabled = false` in `/etc/wsl.conf`), Tisty says so and writes nothing there.

### By hand

For an assistant Tisty does not know, one line does it, with the full path to the `tisty` that came with the app. **Settings › Assistants › How to connect one** shows both forms below with that path already filled in for this computer, ready to copy, so nothing needs to be on your PATH:

```console
$ <your-assistant> mcp add tisty -- <path-to-tisty> mcp
```

For example, with Claude Code:

```console
$ claude mcp add tisty -- <path-to-tisty> mcp
```

Clients configured with a JSON file take the usual shape:

```json
{
  "mcpServers": {
    "tisty": { "command": "<path-to-tisty>", "args": ["mcp"] }
  }
}
```

Restart the assistant. If you closed the door, open it again in **Settings › Assistants**.

You do not have to remember this step. When an assistant tries to work with Tisty while the door is closed, the window tells you — «Claude Code tried to come in» — and **Let it in** opens the door right there. It also tells you when your assistants are connected but cannot do anything yet because the door is closed.

## What it can do, in short

File a task with its steps and its date, set a reminder, add to a task's journal, write and edit documents, file them into folders and pages, keep a copy of a file you point it at, search and read what is there — and say a task it filed is done, **for you to confirm**.

When it tells you what it did, it names a task or a document the way you would: by its title and where it is — the list, or the folder. It never shows you an id.

It can **never** close or delete a task, delete a document, or move a day you set. On a task you wrote it only says what it did — a step or the whole task is done — as its word, which you take or not.

The full rules are in [[What an assistant may do|MCP-Permissions]], and every tool is in the [[tools reference|MCP-Tools]].

## Signed by name

What an assistant writes is signed with the name of the program it spoke through — «by Claude Code», «by Codex» — in the row, the detail and the journal. Settings › Assistants counts what each one wrote. What was written before programs said their name reads «by an assistant».

## It reads sparingly

Tisty keeps a small card for each document — its headings, how long it is, what it leans on — worked out from the text itself, on this machine, and never sent anywhere. An assistant reads that card to pick the document it needs, then asks for only the part it wants, or changes that part without reading the rest. A long document is not poured into a model because an assistant wanted one paragraph of it.

Having read a document, it can leave a short summary for the next assistant — kept on this machine, never synced, and not part of what you wrote.

## The one thing worth knowing before you open it

> [!WARNING]
> Whatever an assistant reads travels wherever that assistant travels. If it runs against a cloud service, what it looks at is on its way to that service. Tisty still sends nothing and has no server — but this door is real, and it is the only thing in Tisty that can carry your data off the machine. That is why it stays shut until you open it, and why closing it is one click.
