# What every reader offers

Anything that shows a sheet offers all of this, whatever it looks like. How it
is laid out is the reader's own, in its frame.

## The document

| Function | Must | What it does |
|---|---|---|
| Show the sheet | yes | Clean, in the document's language and in the reader's theme, light or dark |
| Paper | yes | Change the paper among those offered, for this visit |
| Sheets | yes | See it in sheets or as one long page |
| Turn | yes | Portrait or landscape, while in sheets |
| Index | yes | The headings of the sheet, leading to each one |
| Details | yes | Title, tags and the document's properties |
| Tables | yes | A wide table scrolls by itself and never widens the page |
| Formulas | yes | MathML as it arrives |
| Diagrams | yes | Inline SVG as it arrives, or the code when there is none |
| Attached files | yes | A card that says the file stays in Tisty |

## Who signed

A reader must show the verdict of [identity.md](identity.md), outside the sheet,
where the document cannot imitate it.

- It shows the signed name, marked as what its writer chose, and the whole
  fingerprint.
- It says how to compare the fingerprint over another channel.
- It does not show a document with no writer's signature.
- It shows the name so that its text direction cannot be turned, as with
  `<bdi>`.
- A short suffix of the fingerprint is not a proof, and a reader must not offer
  one as one.

## Pages and what it cites

- The pages of a document are shown with their place, such as «page 2 of 3», and
  lead to the next one.
- A card to a cited document opens it in place, with a way back, and the back
  arrow works too.
- The key of a cited document never goes in the browser's history.
- A document cited but not shared is plain text.

## Languages

- The reader opens, in this order: the language chosen earlier for this link,
  the first of the browser's languages the group has (exact, then by base
  language), then the anchor's.
- A selector shows each language by its own name, such as «Español» or
  «English».
- Changing language stays in the same section, counting headings. A section with
  no twin leads to the top.
- It goes to the page in the same place, and to the document if that language
  has no such page.
- If the sender's roots differ among the languages, the reader warns, because
  someone else signed one of them.
- The chosen language is kept on the reader's device and nowhere else.

## Leaving

| Function | Must | What it does |
|---|---|---|
| Open in Tisty | yes | `tisty://folio?link=…`, see [envelope.md](envelope.md) |
| Copy link | yes | The link as it was opened |
| Report | yes | What the server offers for this |

**Copying and downloading the words** are not offered yet, in this version. What
travels is the Tisty language, whose own marks read as plain text anywhere else,
and whether a reader may offer it, or Tisty seals a standard Markdown beside it,
is still open.

## What it keeps

A reader may keep its own code. It must not keep the words or the keys of a
document, in storage or in its history. Choices such as the language are fine,
because they say nothing of the document.

## Failing

| Refused as | What it says |
|---|---|
| `newer` | This was shared from a newer Tisty, and something here needs updating |
| `locked` | This link does not open it, or it was changed on the way |
| `damaged`, `foreign`, `not_a_picture` | It cannot be shown |

A reader must not show part of an envelope it refused.
