# The envelope

One envelope holds one version of one document. The manifest is in the clear, so
a server can hold its limits without the key. Everything else is sealed, part by
part.

All integers are big-endian. `base64url` has no padding. `H(x)` is SHA-256. Text
labels end in a zero byte, written `\0`.

## The link

A reading link is `https://HOST/ID#KEY`, where `KEY` is 32 random bytes in
`base64url`. An editing link adds a token: `https://HOST/ID#KEY.TOKEN`.
Everything after the `#` never reaches a server.

`KEY` never comes from a password. A password is a door the server keeps, and
opens nothing by itself.

Opening a link in Tisty is `tisty://folio?link=LINK`, with the whole link
percent-encoded. Tisty takes it only for servers it was built to trust.

## The bytes

| Field | Size | What |
|---|---|---|
| Magic | 4 | `FF 46 4F 4C` |
| Layout | 1 | `01` |
| Manifest length | 4 | At most 16 KiB |
| Manifest | | JSON, kept as written |
| Index length | 4 | 32 times the number of parts |
| Index | | `H(sealed part)` for each part, in order |
| Sender length | 4 | At most 2 KiB |
| Sender | | The sender's signature, see [identity.md](identity.md) |
| Parts | | Each sealed part, one after the other, nothing after the last |

A first byte that does not match is `foreign`. A layout above 1 is `newer`, and
any other is `damaged`.

## The manifest

```json
{
  "v": 5,
  "id": "k7Qm2xPa",
  "version": 2,
  "from": 1,
  "salt": "base64url of 32 random bytes",
  "commit": "base64url of 32 bytes",
  "parts": [{ "kind": "config", "size": 120, "len": 180 }]
}
```

| Field | Rule |
|---|---|
| `v` | 5. Above it is `newer`, and below it is `damaged` |
| `id` | 1 to 64 of `A-Z a-z 0-9 - _` |
| `version` | At least 1 |
| `from` | The version this one was written on, or `null`. Below `version` |
| `salt` | New for every version |
| `commit` | Ties these bytes to one key, see below |
| `parts[].kind` | `config`, `signature`, `cited`, `html`, `png`, `jpeg`, `webp`, `gif` or `text` |
| `parts[].size` | Sealed, tag included |
| `parts[].len` | Opened and inflated. Every limit holds on this |

A later version of this contract may add fields, and a reader passes over the
ones it does not know.

## The parts

They come in this order, so a reader paints from the first bytes and stops
early:

| Order | Kind | Must | Deflated | What |
|---|---|---|---|---|
| 1 | `config` | yes | yes | How the writer read the document |
| 2 | `signature` | yes | no | The writer's signature |
| 3 | `cited` | no | yes | What it leads to, and its languages |
| 4 | `html` | yes | yes | The sheet, see [sheet.md](sheet.md) |
| 5 | pictures | no | no | In the order `tisty:picture/N` numbers them, from 0 |
| 6 | `text` | yes | yes | The words, in the Tisty language |

Anything else out of order is `damaged`. A `cited` part is never sealed empty.

A picture must start like the kind it declares, or it is `not_a_picture`: PNG
`89 50 4E 47 0D 0A 1A 0A`, JPEG `FF D8 FF`, WebP `RIFF` then `WEBP` at byte 8,
GIF `GIF87a` or `GIF89a`. The writer recompresses pictures before sealing, so
nothing they hide leaves with them. That is a promise of the writer, and no
server can check it.

## Sealing

```text
commit      = HMAC-SHA256(KEY, "folio.commit.v1\0" || salt)
version key = HKDF-SHA256(ikm = KEY, salt = salt, info = "folio.version.v1\0", 32 bytes)
nonce       = 8 zero bytes || part number as 4 bytes, from 0
aad         = "folio.part.v1\0" || H(manifest as written)
sealed part = AES-256-GCM(version key, nonce, aad, part), with its 16-byte tag at the end
```

The commitment comes first because the cipher alone does not tie bytes to one
key, and it stops an envelope from reading one way to a reader and another way
to a moderator. A nonce repeats only under another `salt`, and so under another
key.

Parts marked deflated go through raw DEFLATE, RFC 1951, with no zlib or gzip
wrapper, before sealing. A deflated part weighs at most `len + len/16 + 64`
bytes before the tag. A part not deflated weighs exactly `len`.

Sealing covers the manifest, so changing a size, a kind or the order opens
nothing.

## Limits of the format

These are fixed, and not a server's to change:

| What | At most |
|---|---|
| Manifest | 16 KiB |
| Sender frame | 2 KiB |
| `signature` part, opened | 2 KiB + 48 bytes for each picture, plus one |
| `config` part, opened | 16 KiB |
| `cited` part, opened | 64 KiB and 64 entries |
| Title | 200 characters |
| A signed name | 80 characters, see [identity.md](identity.md) |

How much text, sheet and pictures an envelope carries is a server's rule, which
it publishes.

## `config`

```json
{
  "title": "Walks",
  "tags": ["rina"],
  "lang": "es",
  "paper": "a4",
  "sheets": true,
  "landscape": true,
  "fields": [{ "key": "lang", "values": ["es"] }]
}
```

Each field may be left out. `title` is 1 to 200 characters. There are up to 64
`tags`, each of 1 to 200 characters. `lang` is a language tag of 2 to 35
characters of `A-Z a-z 0-9 -`. `paper` is 1 to 16 lowercase letters, and a paper
a reader does not know is shown as a plain page. `landscape` only goes with
`sheets`. There are up to 32 `fields`, each key of 1 to 200 characters and each
value of up to 500.

A `config` that breaks any of this is `damaged`. A field a later Tisty adds is
passed over.

## `cited`

A JSON list of the documents this one leads to, each in its own envelope, with
the key that opens it:

```json
[
  { "doc": "", "id": "enGlish1", "key": "base64url", "title": "Walks", "lang": "en" },
  { "doc": "Rina", "id": "citedDoc", "key": "base64url", "title": "Rina", "page": [1, 2] }
]
```

`doc` is the name the words call it by in a card, and only an entry with `lang`
may leave it empty. `page` is its place among the pages of its document, from 1.
`lang` makes it one of the languages of this document, which is then the anchor.

## Languages

Each language is a document of its own, so each document of each language
travels in its own envelope. The anchor is the one the link opens, and only it
holds the map of languages, in `cited`. Adding a language seals only that
language and the anchor.

The map comes before the sheet on purpose. A reader whose language is another
one stops reading the anchor there and asks for the right envelope, so a second
language costs little. The cipher already tells it only the holder of the key
wrote that map.

The words of each language say `lang:` in their properties and carry
`translationKey:`, which is how a Tisty that opens them joins them to the
others.

Paper, sheets and turn belong to the group: every language says the same.

If whoever shares takes the anchor's language out of the group, the link stays
with its document, in that language alone. The other languages leave the share,
and Tisty warns before it does this.

## Opening

1. Read the frames. Check each size against the manifest and each part against
   the index.
2. A server stops here. It checks the sender's signature and its limits, and
   holds the bytes.
3. Check the commitment with `KEY`. A difference is `locked`, and nothing more
   is tried.
4. Derive the version key. Open each part, and a failure is `locked`. Inflate
   into exactly `len` bytes, and anything else is `damaged`.
5. Check the writer's signature over the sheet, see [identity.md](identity.md).
6. Clean the sheet and show it, see [sheet.md](sheet.md).
7. Each picture and the words, when they arrive, must hash to what the writer
   listed. A difference is `damaged`.

A reader may stop after the sheet, at the end of its `html` part, and read the
rest only for pictures or for the words. Reading in pieces changes nothing
above.

## Refusals

| Word | Means |
|---|---|
| `foreign` | Not an envelope |
| `newer` | Sealed by a newer Tisty: ask the reader to update |
| `damaged` | Broken, or made by hand against these rules |
| `locked` | This key does not open it, or something changed it |
| `not_a_picture` | A picture is not the kind it declares |
