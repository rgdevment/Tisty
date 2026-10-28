# Diagrams

A diagram in Tisty is a `mermaid` block inside a document: a few lines of text that Tisty draws, and that GitHub, Notion, Obsidian or an assistant read just the same. You can write them yourself, but you don't have to: the diagram window draws it with you and saves the result as that same text.

How each kind of diagram is written is in the Tisty Markdown guide: [diagrams, formulas and widgets](https://markdown.tisty.app/diagrams-formulas-and-widgets).

## A new diagram

- **Outside a document:** in the Documents **+**, or in a folder's menu, choose **New diagram**. It asks for a name and the kind: mind map, database schema, flowchart, class diagram or other (Mermaid text). Tisty makes a plain document with a short example of that kind and opens the window. The document is made in the folder you are in, or the one whose menu you used; outside every folder, in the **Diagrams** folder, made when it is missing.
- **Inside a document:** type `/` and choose **New diagram**. The example is written where you are and the window opens on it. **Mermaid block as text** leaves an empty block, to write by hand.

## Opening one that exists

Hover over a diagram and press **Open in the diagram window**, or double-click it. A single click still opens it large, to look at. The window only opens in documents you can write, and each diagram has just one.

## The window

On one side is the drawing, and on the other, **What goes into the document**: the Mermaid text, which changes live with the drawing and which you can type into too.

- The mouse wheel, or dragging the background, moves the drawing; with `Ctrl`/`⌘` it zooms in and out where the pointer is. **Zoom in**, **Zoom out** and **Fit** do the same with buttons.
- **Save to «…»**, or `Ctrl`/`⌘` + `S`, writes the diagram back into its block and touches nothing else in the document.
- If the document changed while the window was open, the window tells you: **Use the document's** or **Save mine over it**.
- If you close with unsaved changes, the window asks first.

## Drawing by hand

Four kinds are drawn and edited right on the drawing. The others (sequences, Gantt, pies…) are edited in the text, with the drawing beside it.

- **Mind map.** **+ Child** and **+ Sibling**, or `Tab` and `Enter`; `Delete` removes the chosen idea; a double click or `F2` renames it; dragging an idea onto another moves it there, with its branch. The panel changes its shape.
- **Database schema.** **+ Table** and **+ Column**; each column has a type, PK, FK and UK keys, and can be **Required**. Drag a table by its title to move it, and a column's dot onto another table to relate them. **Copy as SQL** and **Copy as DBML** take the schema to another tool.
- **Flowchart.** **+ Node** and **+ Group**; drag a node's dot onto another to join them. The panel changes a node's text and **Shape**, its group, the chart's **Direction**, and an arrow's label and kind. `Delete` removes what is chosen.
- **Class diagram.** **+ Class**, **+ Attribute** and **+ Method**; drag a class's dot onto another to relate them, and choose the relation's **Kind**, how many at each end and its label.

What only the window needs to remember, such as where you left each table of the schema, goes in `%% tisty:` lines inside the same block. Mermaid treats them as comments, so on the page, in the PDF or on GitHub the diagram is laid out by Mermaid, and the window opens it again just as you left it.

A diagram the window cannot keep without changing it, such as a mind map with two roots or a class diagram with a `namespace`, still opens, with Mermaid's drawing and its text to edit.

## Exporting

**Export ▾**, in the window's header, and **Export**, on the diagram's bar in the page, offer the same:

- **PNG, transparent**, **PNG, light** or **PNG, dark**.
- **JPG, light** or **JPG, dark**.
- **SVG**, light and without a background.
- **Copy as image**, on the background of the theme you are in, to paste into a chat or an email.

Images come out at twice their size so they stay sharp (a very large one somewhat less, as far as the system can paint it), and the file is named after the document, with "-2", "-3"… from its second diagram on. Mind maps, schemas, flowcharts and class diagrams come out just as you see them in the window, with your tables where you left them; other kinds, as Mermaid draws them.

## With an assistant

An assistant connected through [[MCP|Assistants-and-MCP]] writes diagrams like any other text in the document: a `mermaid` block. The `language` tool tells it which kinds the window opens and that `%% tisty:` lines are kept as they are, so it does not undo what you laid out.
