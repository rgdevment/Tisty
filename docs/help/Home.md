<p align="center">
  <img src="https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/icons/tisty-256.png" width="110" height="110" alt="Tisty"/>
</p>

<h1 align="center">Tisty's help</h1>

<p align="center"><strong>How to use each thing, step by step and with examples.</strong></p>

<p align="center"><strong>English</strong> · <a href="Inicio">Español</a></p>

---

This is the help for Tisty: how to use each thing, with examples, whether you work with an AI agent or on your own. What Tisty is and why it exists is on [tisty.app](https://www.tisty.app/); here you go straight to what you want to do.

## Start with what you want to do

### On your own

- **Install it and take the first steps:** [[Installation]].
- **Write a task down the way you would say it:** [[Natural language|Natural-Language]].
- **Put the day and the week in order:** [[Tasks]] and [[The spread and reminders|Spread-and-Reminders]].
- **Keep what you know in a document:** [[Documents]] and [[The editor|Markdown-Editor]].
- **Catch an idea before it goes:** [Ideas](Documents#ideas).
- **Write what happened today:** [Diary](Documents#diary).
- **See everything you keep, joined:** [[The Brainsty network|Brainsty-Network]].
- **Find how you solved it last time:** [[The archive|Archive]].

### With your AI agent

- **Connect it:** [[Agents and MCP|Agents-and-MCP]].
- **Know what it may do and what it never does:** [[What an agent may do|MCP-Permissions]].
- **See how it works in a real session:** [[How an agent works|How-an-Agent-Works]].
- **Have it pick up by itself and leave work ready for the next one:** [[Hooks and skills|Hooks-and-Skills]].
- **Keep the rules of all your agents in one place:** [[Agents|AGENTS-Folder]].
- **Go back to what a document said before a change:** [Earlier versions](Documents#earlier-versions).

### Your files and your privacy

- **Have the same on two computers:** [[Sync and backup|Sync-and-Backup]].
- **Share a document with someone:** [Sharing](Documents#sharing).
- **Know what is stored and what leaves your computer:** [[Privacy]].

## A task or a document?

**Something you will do is a task. Something you will come back to is a document.** A task ends and stays in the archive, where a search tells you months later how you solved it. A document is read again, corrected and extended for years. And an idea is a thought caught on the fly, which later becomes a task, goes into a document or is set aside. [[The whole idea, with an example|Tasks#a-task-or-a-document]].

## Start here

- [[Installation]] — Microsoft Store, winget, Homebrew or a direct download.
- [First steps](Installation#the-first-time-it-opens) — what Tisty asks the first time, and the guide it writes for you.
- [[Settings]] — the Overview, the settings search and what each page holds.
- [[Keyboard shortcuts|Keyboard-Shortcuts]]

## Tasks and the archive

- [[Tasks]] — capture, views, the Eisenhower matrix, lists, tags, steps and parts.
- [[Natural language|Natural-Language]] — what the capture bar understands.
- [[The spread and reminders|Spread-and-Reminders]] — dealing work onto days, reminders that ring with Tisty closed, and saving a task to your calendar.
- [[The archive|Archive]] — stories, routines and the trace.

## Memory for your AI agent, through MCP

Tisty is not AI and has none inside, and that is not going to change. But it speaks [MCP](https://modelcontextprotocol.io), so the agent you already use — GitHub Copilot, Claude Code, Codex, Cursor, any MCP client — keeps its memory here: it files tasks with steps and a journal, writes documents, and picks up where it left off, on this machine, with no account and nothing over the network.

- [[Agents and MCP|Agents-and-MCP]] — connecting one, letting it in, a day with your agent and what it costs.
- [[Hooks and skills|Hooks-and-Skills]] — the hooks that tell Tisty when each turn starts and ends, and Tisty's three skills that teach your agent to pick up, tidy up and leave work ready for the next one.
- [[What an agent may do|MCP-Permissions]] — the limits, its word on your own tasks, and **To confirm**.
- [[MCP tools reference|MCP-Tools]] — every tool the server offers.
- [[Agents: your agents' rules, skills and memory|AGENTS-Folder]] — one folder for the files Claude Code, Codex, Copilot, Gemini and Cursor read. Bring copies them to read and arrange; Sync has Tisty write them, with backups.

## Documents

Documents are a Markdown notebook beside the tasks: callouts, tables, diagrams, formulas, widgets, code and pages, all kept as plain `.md` files any other editor reads. How each one is written is in the Tisty Markdown guide; the pages here are about the app.

- [[Documents]] — folders, tags, several languages, attachments, import and export, and reading a Markdown file from outside.
- [Ideas](Documents#ideas) — ideas of the moment as sticky notes, sorted out later into a task or a document.
- [Diary](Documents#diary) — what happened today, a book a year, kept from your agent until you open it.
- [Earlier versions](Documents#earlier-versions) — what a document said before a change, yours or your agent's, what changed and going back to it, on each computer.
- [A document's link](Documents#a-documents-link) — reaching it from a chat, your browser or your agent.
- [[The editor|Markdown-Editor]] — writing where you read, the bar, the `/` menu, tables and shortcuts.
- [[Pages and PDF|Pages-and-PDF]] — a document in chapters, page breaks, and Tisty's own PDF.
- [[Diagrams]] — the diagram window: a mind map, a schema, a freeform diagram or a class diagram drawn by hand, and exported as an image.
- [[Widgets]] — a fixed view of HTML and CSS inside a document, and opening it apart.
- [Tisty Markdown guide ↗](https://markdown.tisty.app) — every mark a document can hold, with its examples drawn.

## Brainsty: everything joined

What you keep was already joined: a document cites another, a task was born in some minutes, a book has pages. Brainsty makes those ties a single gesture to write, shows them as one network and serves them to your agent as an index it walks without reading everything.

- [[Linking documents|Linking-Documents]]: `[[` to link a document or one of its sections, the preview and **Linked from**.
- [[Tasks in documents|Tasks-in-Documents]]: citing a task as a live card, creating one with `/` without leaving the document, and **Born in**.
- [[The Brainsty network|Brainsty-Network]]: everything you keep drawn as a network, from afar and up close, with its search, its filters and the keyboard.
- [[How an agent works|How-an-Agent-Works]]: a real session, call by call: `brain`, a task's state, Tisty's questions and the hooks.

## Your data

- [[Sync and backup|Sync-and-Backup]] — two machines through a folder you already sync, or your Google Drive, OneDrive or WebDAV server directly, with what you keep there encrypted if you want.
- [[Privacy]] — what is stored, and the few requests Tisty ever makes.
- [[Command line|Command-Line]] — retired in 1.31; only what agents use remains: `tisty mcp`, their door, and `tisty hook`.
- [[FAQ]]

---

Tisty is free and open source under the AGPL-3.0. The source, the issues and the releases are in the [main repository](https://github.com/rgdevment/Tisty), and its site is [www.tisty.app](https://www.tisty.app/). Everyone taking part — in issues, pull requests or discussions — follows the [Code of Conduct](https://github.com/rgdevment/Tisty/blob/main/CODE_OF_CONDUCT.md).
