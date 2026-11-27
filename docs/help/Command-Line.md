# Command line

Since 1.31, Tisty has no command line. The window is the way in, for your tasks and your documents alike.

What remains of the `tisty` program is what your agent uses: **`tisty mcp`**, the door it comes through, and **`tisty hook`**, which the hooks Tisty adds to it call when a session starts, before and after some tools and when a turn ends (see [[Agents and MCP|Agents-and-MCP]]). `tisty hook` reads what the agent sends, answers in that agent's own shape and always ends without an error, so it never stops a turn. There is also **`tisty run-hook`**, the one line through which an agent runs a hook you made and registered in Agents. Like `tisty hook`, it never fails the turn: only your script ending with 2 before a tool stops it. Any other command only prints a notice that the command line was retired and sends you to the window.

## Where it lives

It comes inside the app: you do not install it on its own, and it does not need to be on your PATH. Since 1.32.1 it also answers to plain `tisty` while the door for agents is open (see [[Agents and MCP|Agents-and-MCP]]). **Settings › Agents › Connected** shows where it sits on this computer, and connects the agents it finds installed.

## Connecting an agent with it

For an agent Tisty does not know, one line does it, with the full path that **Settings › Agents › Connected › How to connect one** shows, ready to copy:

```console
$ <your-agent> mcp add tisty -- <path-to-tisty> mcp
```

Where `<your-agent>` is whatever yours is called. If you closed the door, open it again in **Settings › Agents › Door**. The rest is in [[Agents and MCP|Agents-and-MCP]].

## If you used the command line

- What it did for maintenance lives in the window: **Sync now** and **Send everything again** in **Settings › Sync**, confirming another machine on its notice or under **Settings › Security › Your computers**, the store review in **Settings › Health**, and the backup zip in **Settings › Copies**.
- Scripts that called `tisty add` or `tisty ls --json` stop working. The [[MCP door|Agents-and-MCP]] is how a program reaches Tisty now.
- The Homebrew formula `tisty-cli` and the loose zips of the command line are no longer published. If that is all you installed, it stays on 1.30: install the app instead ([[Installation]]), and it finds your tasks and documents where they are.
