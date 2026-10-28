# Widgets

A code block whose language is `widget` is drawn as a small page: HTML, styles and, if you like, a little script. It is how a document holds a dashboard, a counter, a small calculator — something more than text.

Every form, with its examples drawn the way Tisty draws them, is in the Tisty Markdown guide: [diagrams, formulas and widgets](https://markdown.tisty.app/diagrams-formulas-and-widgets#widgets).

````markdown
```widget title="Walks"
<div class="stat"><small>This month</small><b>42</b></div>
```
````

> [!NOTE]
> GitHub does not run widgets, so this page shows the code. In Tisty the block above is drawn as a card with its figure.

## Sealed off

- A widget runs in a frame that is allowed **no network at all**: it cannot fetch anything, load a picture from an address, or send what it shows anywhere.
- It cannot reach the window around it or your other documents.
- A link inside it opens your browser only when you click it.

## Ready-made classes

It comes with Tisty's colours and ready-made classes, so plain HTML already looks right without writing CSS; every class, with its example, is in the [Markdown guide](https://markdown.tisty.app/diagrams-formulas-and-widgets#widgets).

## Showing the code

The widget's code stays folded under the drawing. Open it with **Show code**.

## When it grows: attach an `.html` file

If a widget grows large, attach it instead: an `.html` file attached to a document is drawn the same way, sealed off just the same, and its bar lets you show it as an attachment again.

## In a window of its own

**Open apart**, on a widget or an attached `.html` page, opens it in a window of its own, sealed off just the same. Its bar sizes it as **Fit**, **Phone**, **Tablet** or **Desktop** (`Ctrl`/`⌘` + `1` to `4`), and **More** has **Reload**, **View code**, **Show in the document**, **Try it dark** and **Zoom to 100 %**.
