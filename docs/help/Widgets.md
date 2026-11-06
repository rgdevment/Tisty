# Widgets

A code block whose language is `widget` is drawn as a fixed view: HTML and CSS, and nothing else. It is how a document holds a panel of figures, a summary or a small table in Tisty's colours — something more than text, which stays as you wrote it.

Every form, with its examples drawn the way Tisty draws them, is in the Tisty Markdown guide: [diagrams, formulas and widgets](https://markdown.tisty.app/diagrams-formulas-and-widgets#widgets).

````markdown
```widget title="Walks"
<div class="stat"><small>This month</small><b>42</b></div>
```
````

> [!NOTE]
> GitHub does not draw widgets, so this page shows the code. In Tisty the block above is drawn as a card with its figure.

## Fixed, and the same everywhere

- A widget **runs nothing, ever**: no script, no `onclick`, no button or form.
- It **names nothing outside itself**: no link, no picture, font or style from an address, no frame, sound or video. A picture goes inside it, as `data:image/…`.
- It looks the same wherever the document is read.

## When a block is not a widget

A block with anything else is not a widget. It is shown as its code, with what is wrong, and none of it is drawn. An agent that tries to write one is told why, and nothing is written.

## Ready-made classes

It comes with Tisty's colours and ready-made classes, so plain HTML already looks right without writing CSS; every class, with its example, is in the [Markdown guide](https://markdown.tisty.app/diagrams-formulas-and-widgets#widgets).

## Showing the code

The widget's code stays folded under the drawing. Open it with **Show code**.

## An `.html` file is only a file

An `.html` file attached to a document is an attachment like any other: a card with its name and size. Tisty never runs it or draws it. Nothing a document holds ever runs.

## In a window of its own

**Open apart**, on a widget, opens it in a window of its own. Its bar sizes it as **Fit**, **Phone**, **Tablet** or **Desktop** (`Ctrl`/`⌘` + `1` to `4`), and **More** has **Reload**, **View code**, **Show in the document**, **Try it dark** and **Zoom to 100 %**.
