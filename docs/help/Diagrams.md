# Diagrams

A diagram in Tisty is a `mermaid` block inside a document: a few lines of text that Tisty draws, and that GitHub, Notion, Obsidian or an assistant read just the same. You can write them yourself, but you don't have to: the diagram window draws it with you and saves the result as that same text.

How each kind of diagram is written is in the Tisty Markdown guide: [diagrams, formulas and widgets](https://markdown.tisty.app/diagrams-formulas-and-widgets).

## A new diagram

- **Outside a document:** in the Documents **+**, or in a folder's menu, choose **New diagram**. It asks for a name and the kind: mind map, database schema, flowchart, class diagram or other (Mermaid text). Tisty makes a plain document with a short example of that kind and opens the window. The document is made in the folder you are in, or the one whose menu you used; outside every folder, in the **Diagrams** folder, made when it is missing.
- **Inside a document:** type `/` and choose **New diagram**. The example is written where you are and the window opens on it. **Mermaid block as text** leaves an empty block, to write by hand. The **Diagram** button on the floating bar offers the same without the menu.
- **Reusing one:** **Existing diagram…**, in the `/` menu or on the **Diagram** button, opens a search over the diagrams of all your documents. Each shows its document, the section it sits in and its title, if it has one; type part of any of them or of the kind to narrow them. The one chosen is drawn beside the list. Choosing it writes a copy, and from then on each is its own: changing one does not change the other.

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
- **Flowchart.** The palette on the left holds the shapes, grouped in Basic, Data, People and systems, and Notes: drag one onto the canvas, or press it to put it in the middle. Each shape stays where you leave it. Drag one of its dots onto another shape to join them, or onto empty space to make a new shape already joined; arrows run at right angles between the sides that face each other. Double-click a shape or an arrow to change its words. The palette's **Lanes** are columns or rows: they are dragged with everything in them, sized from their edges, and a shape you let go inside one becomes part of that lane (and stops being part of it when let go outside). `Ctrl` or `⌘` + click, or `Shift` + drag, chooses several shapes at once. The bar above undoes and redoes (`Ctrl`/`⌘` + `Z`), deletes, duplicates (`Ctrl`/`⌘` + `D`), gives a fill and a line colour, changes the chosen arrow's kind, lines several shapes up, **Lay out by itself** orders them again, and zooms in, out or to fit the window. **+ Group** adds a group. The panel changes a node's text and **Shape**, its group, the chart's **Direction**, and an arrow's label and kind. `Delete` removes what is chosen.
- **Class diagram.** It works like the flowchart, on the same canvas. The palette holds class, interface, abstract class and enumeration: drag them onto the canvas or press them, and each class stays where you leave it. Drag a class's dot onto another to relate them, or onto empty space to make a class already related. Each relation is drawn with its head (inheritance, realization, composition, aggregation, association, dependency) and how many at each end. Double-click to rename a class or label a relation. The bar changes the chosen relation's kind without turning it round, and also undoes, deletes, colours, aligns, lays out and zooms. In the panel, **+ Attribute** and **+ Method** add members; each one has its **Visibility** (+, -, #, ~). You also choose the **Kind of class** and, for a relation, its kind, how many at each end and its label.

What only the window needs to remember, such as where you left each table of the schema or each shape of the flowchart, goes in `%% tisty:` lines inside the same block. Mermaid treats them as comments, so on the page, in the PDF or on GitHub the diagram is laid out by Mermaid, and the window opens it again just as you left it.

A diagram the window cannot keep without changing it, such as a mind map with two roots or a class diagram with a `namespace`, still opens, with Mermaid's drawing and its text to edit.

## Exporting

**Export ▾**, in the window's header, and **Export**, on the diagram's bar in the page, offer the same:

- **PNG, transparent**, **PNG, light** or **PNG, dark**.
- **JPG, light** or **JPG, dark**.
- **SVG**, light and without a background.
- **Copy as image**, on the background of the theme you are in, to paste into a chat or an email.

Images come out at twice their size so they stay sharp (a very large one somewhat less, as far as the system can paint it), and the file is named after the document, with "-2", "-3"… from its second diagram on. Mind maps, schemas, flowcharts and class diagrams come out just as you see them in the window, with your tables and shapes where you left them (a flowchart with icons or images comes out as Mermaid draws it); other kinds, as Mermaid draws them.

## With an assistant

An assistant connected through [[MCP|Assistants-and-MCP]] writes diagrams like any other text in the document: a `mermaid` block. The `language` tool tells it which kinds the window opens and that `%% tisty:` lines are kept as they are, so it does not undo what you laid out.
