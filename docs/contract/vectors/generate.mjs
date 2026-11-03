// Writes the test vectors of the contract from fixed keys, so the same run gives the same bytes.
// These keys are public and for tests only: nothing real is ever signed or sealed with them.
import { createCipheriv, createHash, createHmac, createPrivateKey, createPublicKey, hkdfSync, sign } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { deflateRawSync } from "node:zlib";

const here = new URL(".", import.meta.url);
const b64 = (bytes) => Buffer.from(bytes).toString("base64url");
const u32 = (n) => {
  const out = Buffer.alloc(4);
  out.writeUInt32BE(n);
  return out;
};
const u64 = (n) => {
  const out = Buffer.alloc(8);
  out.writeBigUInt64BE(BigInt(n));
  return out;
};
const i64 = (n) => {
  const out = Buffer.alloc(8);
  out.writeBigInt64BE(BigInt(n));
  return out;
};
const sha = (...parts) => {
  const over = createHash("sha256");
  for (const part of parts) over.update(part);
  return over.digest();
};
const text = (said) => Buffer.from(said, "utf8");
const filled = (byte, n = 32) => Buffer.alloc(n, byte);

const ed = (seed) => {
  const der = Buffer.concat([Buffer.from("302e020100300506032b657004220420", "hex"), seed]);
  const secret = createPrivateKey({ key: der, format: "der", type: "pkcs8" });
  const open = createPublicKey(secret).export({ format: "der", type: "spki" }).subarray(-32);
  return { secret, open };
};
const signed = (key, message) => sign(null, message, key.secret);

const second = (iso) => Date.parse(iso) / 1000;
const attest = (label, root, share, when) =>
  b64(signed(root, Buffer.concat([text(label), root.open, share, i64(second(when))])));

const certificate = (root, share, since = "2026-11-03T12:00:00Z") => ({
  root: b64(root.open),
  share: b64(share.open),
  since,
  sig: attest("tisty-share.certificate.v1\0", root, share.open, since),
});
const revocation = (root, share, at = "2026-11-03T13:00:00Z") => ({
  root: b64(root.open),
  share: b64(share.open),
  at,
  sig: attest("tisty-share.revocation.v1\0", root, share.open, at),
});
const message = (root, share, about, said) =>
  Buffer.concat([text("tisty-share.signed.v1\0"), root.open, share.open, u64(text(about).length), text(about), said]);
const vouched = (root, share, about, said) => ({
  certificate: certificate(root, share),
  sig: b64(signed(share, message(root, share, about, said))),
});

const root = ed(filled(1));
const share = ed(filled(2));
const otherRoot = ed(filled(3));
const otherShare = ed(filled(4));
const LINK = filled(10);

const PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==",
  "base64",
);
const MEDIA = { png: "image/png", jpeg: "image/jpeg", webp: "image/webp", gif: "image/gif" };

const drawnContent = (textHash, cited, html, config, pictures) => {
  const parts = [text("folio.written.v1\0"), text("drawn")];
  for (const [named, said] of [
    ["text", text(textHash)],
    ["cited", cited ?? Buffer.alloc(0)],
    ["html", text(html)],
    ["config", config],
  ]) {
    parts.push(text(named), u64(said.length), said);
  }
  parts.push(u64(pictures.length));
  for (const picture of pictures) {
    for (const said of [MEDIA[picture.kind], b64(sha(picture.bytes))]) parts.push(u64(text(said).length), text(said));
  }
  return sha(...parts);
};

const vouchedWords = (content, id, at, name) => {
  const parts = [content, u64(text(id).length), text(id), i64(second(at))];
  if (name === undefined) parts.push(Buffer.from([0]));
  else parts.push(Buffer.from([1]), u64(text(name).length), text(name));
  return Buffer.concat(parts);
};

const DEFAULT = {
  key: LINK,
  salt: filled(11),
  id: "k7Qm2xPa",
  version: 1,
  from: null,
  config: { title: "Paseos", tags: ["rina"], lang: "es", paper: "a4", fields: [{ key: "lang", values: ["es"] }] },
  html: '<h1 data-at="0-8">Paseos</h1><p>Rina sale <mark data-pen="green">dos veces</mark> al día.</p>',
  words: "---\nlang: es\n---\n# Paseos\n\nRina sale ==dos veces=={green} al día.\n",
  pictures: [],
  cited: null,
  name: "Mario",
  at: "2026-11-03T12:30:00Z",
  writer: [root, share],
  sender: [root, share],
};

// Every hook sits where a forger would reach in, so each refusal below comes from one lie and no other.
const seal = (asked = {}, hooks = {}) => {
  const a = { ...DEFAULT, ...asked };
  const config = text(JSON.stringify(a.config));
  const cited = a.cited ? text(JSON.stringify(a.cited)) : null;
  const textHash = b64(sha(text(a.words)));
  const covered = drawnContent(textHash, cited, a.html, config, a.pictures);
  const said = vouchedWords(covered, hooks.writtenId ?? a.id, a.at, a.name);
  const wrote = {
    id: hooks.writtenId ?? a.id,
    at: a.at,
    ...(a.name === undefined ? {} : { name: a.name }),
    ...(a.pictures.length ? { pictures: a.pictures.map((one) => b64(sha(one.bytes))) } : {}),
    text: textHash,
    signed: vouched(a.writer[0], a.writer[1], "folio.text.v1", said),
  };
  const inner = text(JSON.stringify(wrote));

  let plain = [
    { kind: "config", len: config.length, sealed: deflateRawSync(config, { level: 9 }) },
    { kind: "signature", len: inner.length, sealed: inner },
    ...(cited ? [{ kind: "cited", len: cited.length, sealed: deflateRawSync(cited, { level: 9 }) }] : []),
    { kind: "html", len: text(a.html).length, sealed: deflateRawSync(text(a.html), { level: 9 }) },
    ...a.pictures.map((one) => ({ kind: one.kind, len: one.bytes.length, sealed: one.bytes })),
    { kind: "text", len: text(a.words).length, sealed: deflateRawSync(text(a.words), { level: 9 }) },
  ];
  if (hooks.plain) plain = hooks.plain(plain);

  const versionKey = Buffer.from(hkdfSync("sha256", a.key, a.salt, text("folio.version.v1\0"), 32));
  let manifest = {
    v: 5,
    id: a.id,
    version: a.version,
    from: a.from,
    salt: b64(a.salt),
    commit: b64(createHmac("sha256", a.key).update(text("folio.commit.v1\0")).update(a.salt).digest()),
    parts: plain.map((one) => ({ kind: one.kind, size: one.sealed.length + 16, len: one.len })),
  };
  if (hooks.manifest) manifest = hooks.manifest(manifest);
  const written = text(JSON.stringify(manifest));
  const aad = Buffer.concat([text("folio.part.v1\0"), sha(written)]);
  let parts = plain.map((one, at) => {
    const nonce = Buffer.concat([Buffer.alloc(8), u32(at)]);
    const cipher = createCipheriv("aes-256-gcm", versionKey, nonce);
    cipher.setAAD(aad);
    return Buffer.concat([cipher.update(one.sealed), cipher.final(), cipher.getAuthTag()]);
  });
  if (hooks.parts) parts = hooks.parts(parts);
  const index = Buffer.concat(parts.map((one) => sha(one)));
  const writtenFrame = hooks.writtenAfter ? text(hooks.writtenAfter(written.toString())) : written;
  const digest = sha(text("folio.indexed.v1\0"), u64(writtenFrame.length), writtenFrame, u64(index.length), index);
  const answered = Buffer.concat([u64(text(a.id).length), text(a.id), u64(a.version), digest]);
  const outer = vouched(a.sender[0], a.sender[1], "folio.envelope.v1", answered);
  const trailer = hooks.trailer ? hooks.trailer(JSON.stringify(outer)) : JSON.stringify(outer);
  let bytes = Buffer.concat([
    Buffer.from([0xff]),
    text("FOL"),
    Buffer.from([1]),
    u32(writtenFrame.length),
    writtenFrame,
    u32(index.length),
    index,
    u32(text(trailer).length),
    text(trailer),
    ...parts,
  ]);
  if (hooks.bytes) bytes = hooks.bytes(bytes);
  return b64(bytes);
};

const opened = (asked = {}) => {
  const a = { ...DEFAULT, ...asked };
  return {
    config: a.config,
    cited: a.cited ?? [],
    html: a.html,
    text: a.words,
    pictures: a.pictures.map((one) => ({ kind: one.kind, sha256: b64(sha(one.bytes)) })),
    name: a.name ?? null,
    at: a.at,
    person: b64(root.open),
    id: a.id,
    version: a.version,
    from: a.from,
  };
};

const good = (name, note, asked = {}, expected = {}) => ({
  name,
  note,
  key: b64(asked.key ?? LINK),
  envelope: seal(asked),
  revoked: expected.revoked ?? [],
  expect: { sender: expected.sender ?? "holds", writer: expected.writer ?? "holds", ...opened(asked) },
});
const bad = (name, note, refused, asked = {}, hooks = {}, key = LINK) => ({
  name,
  note,
  key: b64(key),
  envelope: seal(asked, hooks),
  revoked: [],
  expect: { refused },
});
const flipped = (bytes, from) => {
  const out = Buffer.from(bytes);
  out[from] ^= 1;
  return out;
};
const pictured = { pictures: [{ kind: "png", bytes: PNG }] };
const anchored = {
  cited: [
    { doc: "", id: "enGlish1", key: b64(filled(12)), title: "Walks", lang: "en" },
    { doc: "Rina", id: "citedDoc", key: b64(filled(13)), title: "Rina", page: [1, 2] },
  ],
};

const envelopes = [
  good("plain", "The least a version holds: config, signature, the sheet and the words."),
  good("second-version", "A later version says which one it was written on.", { salt: filled(14), version: 2, from: 1 }),
  good("picture", "A picture sits between the sheet and the words, and is not deflated.", pictured),
  good("anchor", "A language map and a cited page, in the part before the sheet.", anchored),
  good("no-name", "The signed name is optional; the reader then shows the fingerprint alone.", { name: undefined }),
  good("revoked-key", "A good signature whose key the root took back reads as revoked.", {}, {
    sender: "revoked",
    writer: "revoked",
    revoked: [revocation(root, share)],
  }),
  good("other-root-revocation", "A revocation from another root takes nothing back.", {}, {
    revoked: [revocation(otherRoot, share)],
  }),
  bad("foreign", "Not an envelope at all.", "foreign", {}, { bytes: (b) => flipped(b, 1) }),
  bad("newer-layout", "A layout byte above 1 belongs to a newer Tisty.", "newer", {}, {
    bytes: (b) => {
      const out = Buffer.from(b);
      out[4] = 2;
      return out;
    },
  }),
  bad("newer-format", "A manifest version above 5 belongs to a newer Tisty.", "newer", {}, {
    manifest: (m) => ({ ...m, v: 6 }),
  }),
  bad("older-format", "No format below 5 is read.", "damaged", {}, { manifest: (m) => ({ ...m, v: 4 }) }),
  bad("wrong-key", "Another key fails at the commitment, before any part is opened.", "locked", {}, {}, filled(15)),
  bad("flipped-part", "A sealed byte that no longer matches the index.", "damaged", {}, {
    bytes: (b) => flipped(b, b.length - 1),
  }),
  bad("flipped-part-new-index", "The index was rewritten too, so the seal is what catches it.", "locked", {}, {
    parts: (parts) => parts.map((one, at) => (at === 2 ? flipped(one, 0) : one)),
  }),
  bad("swapped-parts", "Two parts traded places, with their sizes and hashes, and the nonce catches it.", "locked", {}, {
    parts: (parts) => {
      const [config, signature, ...rest] = parts;
      const copy = [config, signature, ...rest];
      copy[copy.length - 1] = rest[0];
      copy[copy.length - 2] = rest[rest.length - 1];
      return copy;
    },
    manifest: (m) => {
      const parts = [...m.parts];
      const html = parts[parts.length - 2];
      const words = parts[parts.length - 1];
      parts[parts.length - 2] = { ...html, size: words.size };
      parts[parts.length - 1] = { ...words, size: html.size };
      return { ...m, parts };
    },
  }),
  bad("changed-version", "The manifest changed after sealing, so no part opens.", "locked", {}, {
    writtenAfter: (written) => written.replace('"version":1', '"version":7'),
  }),
  bad("truncated", "The last part was cut short.", "damaged", {}, { bytes: (b) => b.subarray(0, b.length - 1) }),
  bad("trailing-bytes", "Bytes past the last part.", "damaged", {}, {
    bytes: (b) => Buffer.concat([b, Buffer.from([0])]),
  }),
  bad("no-signature", "Part 1 has to be the writer's signature.", "damaged", {}, {
    plain: (plain) => plain.filter((one) => one.kind !== "signature"),
  }),
  bad("words-not-last", "The words have to come last.", "damaged", {}, {
    plain: (plain) => [plain[0], plain[1], plain[3], plain[2]],
  }),
  bad("no-sheet", "A version without a sheet shows nothing.", "damaged", {}, {
    plain: (plain) => plain.filter((one) => one.kind !== "html"),
  }),
  bad("unfolds-past-len", "A part may not open to more than its manifest says.", "damaged", {}, {
    plain: (plain) => plain.map((one) => (one.kind === "html" ? { ...one, len: 4 } : one)),
  }),
  bad("spelled-another-way", "The sender's signature is taken only in the one spelling.", "damaged", {}, {
    trailer: (said) => ` ${said}`,
  }),
  bad("not-a-picture", "The bytes do not start like the kind they declare.", "not_a_picture", {
    pictures: [{ kind: "png", bytes: Buffer.from("this is not a picture") }],
  }),
  {
    ...good("signed-for-another-link", "The writer signed for another id, so that signature is refused."),
    envelope: seal({}, { writtenId: "another1" }),
    expect: { sender: "holds", writer: "refused", ...opened({}), name: "Mario" },
  },
  {
    ...good("writer-is-not-sender", "Written by another key than the one that uploaded: refused.", {
      writer: [otherRoot, otherShare],
    }),
    expect: { sender: "holds", writer: "refused", ...opened({}), person: b64(otherRoot.open) },
  },
];

const identity = {
  fingerprint: { person: b64(root.open), shown: fingerprint(root.open) },
  certificates: [
    { name: "good", certificate: certificate(root, share), holds: true },
    { name: "forged-root", certificate: { ...certificate(root, share), root: b64(otherRoot.open) }, holds: false },
    { name: "changed-date", certificate: { ...certificate(root, share), since: "2026-11-03T12:00:01Z" }, holds: false },
    { name: "fraction-of-second", certificate: { ...certificate(root, share), since: "2026-11-03T12:00:00.5Z" }, holds: false },
  ],
  revocations: [
    { name: "good", revocation: revocation(root, share), holds: true },
    { name: "forged", revocation: { ...revocation(root, share), root: b64(otherRoot.open) }, holds: false },
  ],
  signatures: [
    {
      name: "good",
      about: "folio.envelope.v1",
      said: b64(text("what was said")),
      signed: vouched(root, share, "folio.envelope.v1", text("what was said")),
      holds: true,
    },
    {
      name: "about-something-else",
      about: "folio.text.v1",
      said: b64(text("what was said")),
      signed: vouched(root, share, "folio.envelope.v1", text("what was said")),
      holds: false,
    },
    {
      name: "small-order-key",
      note: "An identity point as the key and an empty signature verify under loose rules; strict ones refuse them.",
      about: "folio.envelope.v1",
      said: b64(text("what was said")),
      signed: {
        certificate: (() => {
          const small = Buffer.concat([Buffer.from([1]), Buffer.alloc(31)]);
          return {
            root: b64(root.open),
            share: b64(small),
            since: "2026-11-03T12:00:00Z",
            sig: attest("tisty-share.certificate.v1\0", root, small, "2026-11-03T12:00:00Z"),
          };
        })(),
        sig: b64(Buffer.concat([Buffer.from([1]), Buffer.alloc(63)])),
      },
      holds: false,
    },
  ],
};

function fingerprint(open) {
  const digest = sha(text("tisty-share.person.v1\0"), open);
  const wide = BigInt(`0x${digest.subarray(0, 9).toString("hex")}`) >> 6n;
  const digits = wide.toString().padStart(20, "0");
  return digits.match(/.{5}/g).join(" ");
}

const write = (name, data) => writeFileSync(new URL(name, here), `${JSON.stringify(data, null, 2)}\n`);
const version = JSON.parse(readFileSync(new URL("../allowed.json", here))).version;
write("envelopes.json", { version, envelopes });
write("identity.json", { version, ...identity });
