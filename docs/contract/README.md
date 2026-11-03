# The contract between Tisty and whoever shows what it shares

Version `1.0.0-draft.1`.

Tisty produces everything a shared document is made of. Whoever stores and shows
it, Folio by Tisty today and anyone else tomorrow, reads the formats written
here and implements them in its own way. The two sides share no code, and
neither depends on how the other works inside.

This contract lives in the official Tisty repository, `rgdevment/Tisty`, never
in a fork. When Tisty improves or adds something, the change comes here first,
with a new version.

## What it holds

| File | What it fixes |
|---|---|
| [envelope.md](envelope.md) | The bytes of a shared version: its manifest, parts, cipher and limits |
| [identity.md](identity.md) | Who signed: root keys, machine keys, certificates, revocations and verdicts |
| [sheet.md](sheet.md) | The HTML Tisty draws: its components, and how they are cleaned |
| [reader.md](reader.md) | What anything that shows a sheet has to offer |
| [allowed.json](allowed.json) | The closed list of tags, attributes, styles and URLs |
| [tokens.json](tokens.json) | The style rules, as design tokens |
| [vectors/](vectors) | Samples both sides have to pass |

## What it does not hold

The rules of a server are the server's. Its API, quotas, rate limits, proofs of
work, waits and refusals belong to whoever runs it, and it publishes them. A
Tisty reads what a server publishes, checks before it uploads and tells the
person, with no number of a server written into its code. The server decides
anyway, because anyone can call it without passing through Tisty.

What it looks like around a sheet is free too: the bar, the panels, the buttons
and the brand. Only the sheet itself has to look the same everywhere.

## Words

**Must** and **must not** are what a vector or a reader can catch. **Should** is
what a good implementation does, and leaving it out breaks nothing. **Writer**
is what seals and signs, which today is only Tisty. **Reader** is what opens and
shows. **Server** is what stores and serves without the key.

## Versions

The version is semantic. A change a conforming reader would refuse or show
wrongly is a new major version. An addition an older reader safely passes over
is a new minor one. A correction that changes no bytes is a new patch.

While it says `draft`, anything can still change, and nothing is promised
between drafts. It leaves draft once Tisty seals these formats and a reader
opens them.

A reader pins one version, by a tag or a commit of this repository, and moves
when it chooses. An envelope carries its own format version, so a reader can
tell it is too old rather than show something wrong.

## Passing it

A side conforms when it passes the vectors of the version it pins:

- [vectors/envelopes.json](vectors/envelopes.json): each envelope opens to what
  it says, or is refused for the reason it says.
- [vectors/identity.json](vectors/identity.json): each certificate, revocation
  and signature holds or does not.
- [vectors/hostile.json](vectors/hostile.json): each input leaves the cleaner
  without what it must not contain.
- [vectors/sheets/](vectors/sheets): real sheets drawn by Tisty stay whole
  through the cleaner and offer every component.

The keys in the vectors are public and for tests only.
[vectors/generate.mjs](vectors/generate.mjs) writes them from fixed seeds, and
[vectors/verify.mjs](vectors/verify.mjs) opens them again with only what a
browser has. Neither is an implementation to copy: each side writes its own and
checks it against the data.

Tisty runs `verify.mjs` with its conventions, and a test holds the translator
inside `allowed.json`, so neither can drift unseen.
