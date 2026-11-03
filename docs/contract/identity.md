# Who signed

This is Tisty's identity, and it may change once the service has one of its own.
Any change comes as a new version of this contract.

Keys and signatures are Ed25519, in `base64url` with no padding. A verifier must
refuse a key of small order and a signature whose `S` is not canonical, as RFC
8032 and WebCrypto ask. A date is RFC 3339 in UTC, in whole seconds, like
`2026-11-03T12:00:00Z`, and a fraction is refused.

The labels below say `tisty-share` because certificates already live in people's
records under them, so they stay as they are.

## The keys

- **The root** is one per person. It stays on the machine it was born on, comes
  from their phrase, and only vouches for other keys. What others see of a
  person is the root.
- **The machine key** is one per machine, and signs everything. Losing one costs
  a new certificate, never the person's name.

## A certificate

```json
{ "root": "…", "share": "…", "since": "2026-11-03T12:00:00Z", "sig": "…" }
```

`sig` is the root's signature over:

```text
"tisty-share.certificate.v1\0" || root (32) || machine key (32) || since in seconds (8, signed)
```

## A revocation

```json
{ "root": "…", "share": "…", "at": "2026-11-03T13:00:00Z", "sig": "…" }
```

Signed the same way, with the label `tisty-share.revocation.v1\0` and `at`. Only
the root that vouched can take a key back, and for good. If there are several,
the earliest `at` counts, so every reader tells the same date.

## A signature

```json
{ "certificate": { "root": "…", "share": "…", "since": "…", "sig": "…" }, "sig": "…" }
```

`sig` is the machine key's signature over:

```text
"tisty-share.signed.v1\0" || root (32) || machine key (32)
  || length of about (8) || about || said
```

`about` names what is signed, so a signature over one kind of thing never
answers for another. The root is signed too, or another root could certify this
key and claim what it signed.

## The sender's signature

It goes in the envelope's sender frame and is checked without the key. It
answers for these sealed bytes, uploaded as this id and version.

```text
digest = H("folio.indexed.v1\0" || length of manifest (8) || manifest
           || length of index (8) || index)
about  = "folio.envelope.v1"
said   = length of id (8) || id || version (8) || digest
```

The frame holds exactly this JSON, with its keys in this order and no spaces,
because it is the one part of the bytes no seal covers:
`{"certificate":{"root":…,"share":…,"since":…,"sig":…},"sig":…}`. Any other
spelling is `damaged`.

A server refuses an upload without a good one.

## The writer's signature

It goes in the `signature` part, sealed. It answers for these words and
pictures, shared as this id, at this moment, under this name.

```json
{
  "id": "k7Qm2xPa",
  "at": "2026-11-03T12:30:00Z",
  "name": "Mario",
  "pictures": ["base64url of H(picture)"],
  "text": "base64url of H(words)",
  "signed": { "certificate": {}, "sig": "…" }
}
```

`name` and `pictures` may be left out.

```text
covered = H("folio.written.v1\0" || "drawn"
            || "text" || length (8) || text as written above
            || "cited" || length (8) || the cited part opened, or nothing
            || "html" || length (8) || the sheet opened
            || "config" || length (8) || the config part opened
            || number of pictures (8)
            || for each picture:
                 length (8) || its media type || length (8) || its hash as written above)
about   = "folio.text.v1"
said    = covered || length of id (8) || id || at in seconds (8, signed)
          || name: 01 || length (8) || name, or 00
```

The media types are `image/png`, `image/jpeg`, `image/webp` and `image/gif`.

The sheet can be checked before the words and pictures arrive, because their
hashes stand in for them.

A reader must not show a document with no writer's signature. The verdict is
`refused` when `id` is not the manifest's, or when this certificate does not
have the root and machine key of the sender's.

## A name

A name is a line a person can read: at most 80 characters, no spaces at either
end, at least one letter, no more digits than letters, so it never passes for a
fingerprint, and nothing unseen that hides a letter or turns the text around.
Control characters and these code points are refused:

```text
00AD 034F 061C 115F 1160 17B4-17B5 180B-180F 200B-200F 2028-202E
2060-206F 2800 3164 FEFF FFA0 FFF9-FFFB 1D173-1D17A E0000-E007F
```

## The verdict

| Verdict | Means | What a reader says |
|---|---|---|
| `holds` | The signature is good and the key stands | Who signed, by name and fingerprint |
| `revoked` | Good, but its root took the key back | It was taken back and when. Nothing signed says when a signature was made, so older work and a thief's look alike |
| `refused` | It does not verify | It cannot be trusted |

## The fingerprint

This is what two people read to each other over another channel. It is 66 bits,
in four groups of five digits:

```text
d = H("tisty-share.person.v1\0" || root)
n = the first 9 bytes of d, as a number, shifted right by 6 bits
shown = n in decimal, padded to 20 digits, split in groups of 5 by spaces
```

The name is never a proof: anyone can choose any name. The fingerprint is.
