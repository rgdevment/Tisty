// Writes the sample envelopes every language must open. What it wrote is kept, since another zlib squeezes the same words differently.
import { createHash, webcrypto } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { deflateRawSync } from "node:zlib";

const { subtle } = webcrypto;
const CONTRACT = readFileSync(new URL("../VERSION", import.meta.url), "utf8").trim();
const words = (said) => new TextEncoder().encode(said);
const b64 = (bytes) => Buffer.from(bytes).toString("base64url");
const sha = (...pieces) => createHash("sha256").update(Buffer.concat(pieces)).digest();
const be8 = (number) => {
  const out = Buffer.alloc(8);
  out.writeBigUInt64BE(BigInt(number));
  return out;
};
const u32 = (number) => {
  const out = Buffer.alloc(4);
  out.writeUInt32BE(number);
  return out;
};

const filled = (byte) => Buffer.alloc(32, byte);
const LINK_KEY = filled(0x11);
// Every envelope has a head of its own, even here: under one key, a head used twice breaks the seal.
const HEADS = { main: filled(0x22), files: filled(0x23), other: filled(0x24) };
const SINCE = "2026-11-03T12:00:00Z";

const signer = async (seed) => {
  const pkcs8 = Buffer.concat([Buffer.from("302e020100300506032b657004220420", "hex"), seed]);
  const key = await subtle.importKey("pkcs8", pkcs8, "Ed25519", true, ["sign"]);
  const jwk = await subtle.exportKey("jwk", key);
  return { key, public: Buffer.from(jwk.x, "base64url") };
};
const root = await signer(filled(0x33));
const machine = await signer(filled(0x44));
const sign = async (who, ...pieces) => Buffer.from(await subtle.sign("Ed25519", who.key, Buffer.concat(pieces)));

const certificate = await sign(root, words("tisty-share.certificate.v1\0"), root.public, machine.public, be8(Date.parse(SINCE) / 1000));

const revocation = async (by, of, at) => ({
  root: b64(by.public),
  machine: b64(of.public),
  at,
  signed: b64(await sign(by, words("tisty-share.revocation.v1\0"), by.public, of.public, be8(Date.parse(at) / 1000))),
});
const revoked = await revocation(root, machine, SINCE);

const seal = async ({ kind, id, version, sealed, parts, from, contract = CONTRACT, head = HEADS[kind], after = [], more = {}, spelled = (one) => one, forged, signatureBytes }) => {
  const told = parts.map(({ name, compressed, bytes, saying, junk = [], more }) => {
    const squeezed = compressed ? Buffer.concat([deflateRawSync(bytes, { level: 9 }), Buffer.from(junk)]) : bytes;
    return { name, compressed, squeezed, open_size: saying ?? bytes.length, sealed_size: squeezed.length + (sealed ? 16 : 0), more };
  });
  const opening = sealed ? { head: b64(head), commitment: b64(await hmac(head)) } : {};
  const manifest = words(
    spelled(JSON.stringify({
      contract,
      kind,
      id,
      version,
      ...(from === undefined ? {} : { from }),
      sealed,
      ...opening,
      parts: told.map(({ name, compressed, sealed_size, open_size, more }) => ({ name, compressed, sealed_size, open_size, ...more })),
      ...more,
    })),
  );
  const key = sealed ? await envelopeKey(head) : undefined;
  const travelling = [];
  for (const [at, part] of told.entries()) {
    travelling.push(
      sealed
        ? Buffer.from(await subtle.encrypt({ name: "AES-GCM", iv: nonce(at), additionalData: manifest }, key, part.squeezed))
        : Buffer.from(part.squeezed),
    );
  }
  const index = Buffer.concat(travelling.map((one) => sha(one)));
  const about = words("envelope");
  const signed = await sign(machine, words("tisty-share.signed.v1\0"), root.public, machine.public, be8(about.length), about, sha(manifest), forged === "signed" ? words("otro") : index);
  const vouched = forged === "certificate" ? await sign(root, words("otro")) : certificate;
  const signature = signatureBytes ?? words(JSON.stringify({ root: b64(root.public), machine: b64(machine.public), since: SINCE, certificate: b64(vouched), signed: b64(signed) }));
  const bytes = Buffer.concat([words("TENV"), u32(manifest.length), manifest, u32(signature.length), signature, index, ...travelling, ...after]);
  return { bytes, hash: b64(sha(manifest, index)) };
};

// Written only from the contract's words, so a reader that reads them gets the same.
function fingerprintOf(key) {
  const digest = sha(words("tisty-share.person.v1\0"), key);
  const digits = (BigInt(`0x${digest.subarray(0, 9).toString("hex")}`) >> 6n).toString().padStart(20, "0");
  return digits.match(/.{5}/g).join(" ");
}

const nonce = (at) => {
  const out = Buffer.alloc(12);
  out.writeUInt32BE(at, 8);
  return out;
};

async function hmac(head) {
  const key = await subtle.importKey("raw", LINK_KEY, { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  return Buffer.from(await subtle.sign("HMAC", key, Buffer.concat([words("tisty-envelope-commit"), head])));
}

async function envelopeKey(head) {
  const base = await subtle.importKey("raw", LINK_KEY, "HKDF", false, ["deriveKey"]);
  return subtle.deriveKey({ name: "HKDF", hash: "SHA-256", salt: head, info: words("tisty-envelope-key") }, base, { name: "AES-GCM", length: 256 }, false, ["encrypt"]);
}

const textParts = ({ body, pages, ...first }) => [
  { name: "first", compressed: true, bytes: words(JSON.stringify(first)) },
  { name: "rest", compressed: true, bytes: words(JSON.stringify({ body, pages })) },
];

// A real picture, one pixel wide, so a reader that decodes what it receives opens it too.
const PICTURES = [Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M/wHwAEBgIApD5fRAAAAABJRU5ErkJggg==", "base64")];
const FILES = [words("%PDF-1.7 cartilla")];
const files = await seal({
  kind: "files",
  id: "k7Qm2xPa",
  version: 1,
  sealed: true,
  parts: FILES.map((bytes) => ({ name: "file", compressed: false, bytes })),
});

const whole = {
  id: "k7Qm2xPa",
  title: "Paseos",
  lang: "es",
  sheet: { paper: "letter", on: true, orientation: "landscape" },
  signed_as: "Mario",
  pictures: [{ at: 0, name: "picture-0.png", kind: "image/png", size: PICTURES[0].length, width: 1, height: 1 }],
  files: [{ at: 0, name: "cartilla.pdf", kind: "application/pdf", size: FILES[0].length }],
  files_hash: files.hash,
  html: '<h1 id="paseos">Paseos</h1><img src="tisty:picture/0" alt="Rina" width="1" height="1"><a href="tisty:page/1">Enero</a><a href="tisty:file/0">Cartilla</a>',
  body: "# Paseos\n\n![Rina](tisty:picture/0)\n",
  pages: [{ id: "p1", page_of: "k7Qm2xPa", at: 1, title: "Enero", html: '<h1 id="enero">Enero</h1>', body: "# Enero\n" }],
};
const main = await seal({
  kind: "main",
  id: "k7Qm2xPa",
  version: 1,
  sealed: true,
  parts: [...textParts(whole), ...PICTURES.map((bytes) => ({ name: "picture", compressed: false, bytes }))],
});

const plain = { id: "wordsOnly", title: "Solo palabras", html: "<p>Hola</p>", body: "Hola\n" };
const open = await seal({ kind: "main", id: "wordsOnly", version: 1, sealed: false, parts: textParts(plain) });

// What a reader refuses by itself, each one well signed: a writer's word counts for nothing here.
const HOSTILE = "hostile";
const asWords = (first, rest) => [
  { name: "first", compressed: true, bytes: words(JSON.stringify(first)) },
  { name: "rest", compressed: true, bytes: words(JSON.stringify(rest)) },
];
const fair = { id: HOSTILE, html: "<p>a</p>" };
const fairRest = { body: "a" };
const aPicture = { at: 0, name: "picture-0.png", kind: "image/png", size: PICTURES[0].length, width: 1, height: 1 };
const asPicture = { name: "picture", compressed: false, bytes: PICTURES[0] };
const withHtml = (html) => asWords({ ...fair, html }, fairRest);
const page = { id: "p1", page_of: HOSTILE, at: 1, html: "<p>b</p>", body: "b" };
const CASES = [
  ["a rest that says what the first one said", { parts: asWords({ ...fair, title: "Paseos" }, { ...fairRest, title: "Otro" }) }],
  ["a rest that brings its own HTML", { parts: asWords(fair, { ...fairRest, html: "<p>otro</p>" }) }],
  ["a first that holds the body", { parts: asWords({ ...fair, body: "a" }, {}) }],
  ["another document inside", { parts: asWords({ ...fair, id: "otroDoc" }, fairRest) }],
  ["an owner, which only its signature says", { parts: asWords({ ...fair, owner: b64(root.public) }, fairRest) }],
  ["a field that left what travels", { parts: asWords({ ...fair, by: "Mario" }, fairRest) }],
  ["more pictures than it says", { parts: [...asWords(fair, fairRest), asPicture] }],
  ["fewer pictures than it says", { parts: asWords({ ...fair, pictures: [aPicture] }, fairRest) }],
  ["a picture in another place than it says", { parts: [...asWords({ ...fair, pictures: [{ ...aPicture, at: 1 }] }, fairRest), asPicture] }],
  ["a picture of another size than it says", { parts: [...asWords({ ...fair, pictures: [{ ...aPicture, size: 1 }] }, fairRest), asPicture] }],
  ["a picture that says it is compressed", { parts: [...asWords({ ...fair, pictures: [aPicture] }, fairRest), { ...asPicture, compressed: true }] }],
  ["its parts out of their order", { parts: asWords(fair, fairRest).reverse() }],
  ["a part that gives more than it says", { parts: [{ ...asWords(fair, fairRest)[0], saying: 3 }, asWords(fair, fairRest)[1]] }],
  ["a part that swells far past what it says", { parts: [{ name: "first", compressed: true, bytes: Buffer.alloc(16 * 1024 * 1024), saying: 10 }, asWords(fair, fairRest)[1]] }],
  ["a part that says more than any reader holds", { parts: [{ ...asWords(fair, fairRest)[0], saying: 2 ** 40 }, asWords(fair, fairRest)[1]] }],
  ["something after its parts", { parts: asWords(fair, fairRest), after: [Buffer.from([0])] }],
  ["a from that is not earlier", { parts: asWords(fair, fairRest), version: 2, from: 7 }],
  ["a from that is its own version", { parts: asWords(fair, fairRest), version: 2, from: 2 }],
  ["a newer major version", { parts: asWords(fair, fairRest), contract: "99.0.0" }],
  ["an older major version", { parts: asWords(fair, fairRest), contract: "0.9.0" }],
  ["a version that is none", { parts: asWords(fair, fairRest), contract: "paseos" }],
  ["a signature that does not hold", { parts: asWords(fair, fairRest), forged: "signed" }],
  ["a certificate that does not hold", { parts: asWords(fair, fairRest), forged: "certificate" }],
  ["a signature that cannot be read", { parts: asWords(fair, fairRest), signatureBytes: words("no es una firma") }],
  ["a part with something past its end", { parts: [{ ...asWords(fair, fairRest)[0], junk: [1, 2, 3] }, asWords(fair, fairRest)[1]] }],
  ["a draft of this version", { parts: asWords(fair, fairRest), contract: "1.0.0-draft.8" }],
  ["two pages out of their order", { parts: asWords(fair, { ...fairRest, pages: [{ ...page, at: 2 }, { ...page, id: "p2" }] }) }],
  ["two pages with one id", { parts: asWords(fair, { ...fairRest, pages: [page, { ...page, at: 2 }] }) }],
  ["a picture drawn at another size than its own", { parts: [...asWords({ ...fair, pictures: [aPicture], html: '<img src="tisty:picture/0" alt="a" width="900" height="1">' }, fairRest), asPicture] }],
  ["files without the hash that answers for them", { parts: asWords({ ...fair, files: [{ at: 0, name: "cartilla.pdf" }] }, fairRest) }],
  ["a hash without files", { parts: asWords({ ...fair, files_hash: files.hash }, fairRest) }],
  ["a picture that gives another size than its manifest says", { parts: [...asWords({ ...fair, pictures: [aPicture] }, fairRest), { ...asPicture, saying: 3 }] }],
  ["a script in its HTML", { parts: withHtml("<script>alert(1)</script>") }],
  ["a script in a page", { parts: asWords(fair, { ...fairRest, pages: [{ ...page, html: "<script>alert(1)</script>" }] }) }],
  ["a page of another document", { parts: asWords(fair, { ...fairRest, pages: [{ ...page, page_of: "otroDoc" }] }) }],
  ["two pages in one place", { parts: asWords(fair, { ...fairRest, pages: [page, { ...page, id: "p2" }] }) }],
  ["a page that is not there", { parts: withHtml('<a href="tisty:page/9">a</a>') }],
  ["a picture that is not there", { parts: withHtml('<img src="tisty:picture/3" alt="a" width="1" height="1">') }],
  ["a file that is not there", { parts: withHtml('<a href="tisty:file/0">a</a>') }],
  ["a document that is not cited", { parts: withHtml('<a href="tisty:cited/0">a</a>') }],
  ["a reference hidden past a slash", { parts: withHtml('<a title/href=tisty:page/9>a</a>') }],
  ["a reference in a page to what its document does not hold", { parts: asWords(fair, { ...fairRest, pages: [{ ...page, html: '<a href="tisty:picture/0">a</a>' }] }) }],
];
const refused = [];
for (const [says, { parts, ...over }] of CASES) {
  const { bytes } = await seal({ kind: "main", id: HOSTILE, version: 1, sealed: false, parts, ...over });
  refused.push({ says, envelope: bytes.toString("hex") });
}

// What a reader opens all the same: a later minor only adds what it can leave aside, and a number is a number however it is written.
const OPENING = [
  ["a later minor version", { parts: asWords(fair, fairRest), contract: "1.9.0" }],
  [
    "fields it does not know, wherever they are",
    {
      contract: "1.9.0",
      more: { novel: "x" },
      parts: [
        { ...asWords({ ...fair, novel: "x", sheet: { paper: "a5", orientation: "diagonal", novel: 1 } }, fairRest)[0], more: { novel: 1 } },
        asWords(fair, { ...fairRest, pages: [{ ...page, novel: "x" }] })[1],
      ],
    },
  ],
  ["numbers written with a point", { parts: asWords(fair, { ...fairRest, pages: [{ ...page, at: 1 }] }), spelled: (one) => one.replace(/"(open_size|sealed_size|version)":(\d+)/g, '"$1":$2.0') }],
];
const opening = [];
for (const [says, { parts, ...over }] of OPENING) {
  const { bytes } = await seal({ kind: "main", id: HOSTILE, version: 1, sealed: false, parts, ...over });
  opening.push({ says, envelope: bytes.toString("hex") });
}
const aNewerMajor = await seal({ kind: "main", id: HOSTILE, version: 1, sealed: false, contract: "2.0.0", parts: [{ name: "fonts", compressed: false, bytes: words("x") }] });

// A main envelope that answers for another files envelope than the one it is handed.
const otherFiles = await seal({ kind: "files", id: "k7Qm2xPa", version: 1, sealed: true, head: HEADS.other, parts: [{ name: "file", compressed: false, bytes: words("otro") }] });
const moreFiles = await seal({ kind: "files", id: HOSTILE, version: 1, sealed: false, parts: [...FILES, ...FILES].map((bytes) => ({ name: "file", compressed: false, bytes })) });
const compressedFile = await seal({ kind: "files", id: HOSTILE, version: 1, sealed: false, parts: [{ name: "file", compressed: true, bytes: FILES[0] }] });
const aFile = { at: 0, name: "cartilla.pdf", kind: "application/pdf", size: FILES[0].length };
const inOtherPlace = await seal({ kind: "files", id: HOSTILE, version: 1, sealed: false, parts: [{ name: "file", compressed: false, bytes: FILES[0] }] });
const countedOver = { id: HOSTILE, html: "<p>a</p>", body: "a", files: [aFile], files_hash: moreFiles.hash };
const squeezedOver = { id: HOSTILE, html: "<p>a</p>", body: "a", files: [aFile], files_hash: compressedFile.hash };

const stranger = await signer(filled(0x55));
const BEFORE = "2026-11-04T00:00:00Z";
const AT = "2026-11-05T00:00:00Z";
const AFTER = "2026-11-06T00:00:00Z";
const broken = { ...(await revocation(root, machine, AT)), at: AFTER };
const verdicts = [
  { says: "no revocation names it", served: { received: AFTER }, verdict: "holds" },
  { says: "the server says nothing and names no revocation", served: {}, verdict: "holds" },
  { says: "it arrived before the revocation's at", served: { received: BEFORE, revocations: [await revocation(root, machine, AT)] }, verdict: "retired" },
  { says: "it arrived exactly at it, which counts as after", served: { received: AT, revocations: [await revocation(root, machine, AT)] }, verdict: "doubtful" },
  { says: "it arrived after it", served: { received: AFTER, revocations: [await revocation(root, machine, AT)] }, verdict: "doubtful" },
  { says: "the server does not say when it arrived", served: { revocations: [await revocation(root, machine, AT)] }, verdict: "doubtful" },
  { says: "of several, the earliest counts", served: { received: AT, revocations: [await revocation(root, machine, AFTER), await revocation(root, machine, BEFORE)] }, verdict: "doubtful" },
  { says: "a revocation of another machine counts for nothing", served: { received: AFTER, revocations: [await revocation(root, stranger, AT)] }, verdict: "holds" },
  { says: "a revocation by another root counts for nothing", served: { received: AFTER, revocations: [{ ...(await revocation(stranger, machine, AT)) }] }, verdict: "holds" },
  { says: "a revocation whose signature does not hold counts for nothing", served: { received: AFTER, revocations: [broken] }, verdict: "holds" },
];

const png = PICTURES[0];
const jpeg = Buffer.from("/9j/4AAQSkZJRgABAgAAAQABAAD/wAARCAABAAIDAREAAhEBAxEB/9sAQwAIBgYHBgUIBwcHCQkICgwUDQwLCwwZEhMPFB0aHx4dGhwcICQuJyAiLCMcHCg3KSwwMTQ0NB8nOT04MjwuMzQy/9sAQwEJCQkMCwwYDQ0YMiEcITIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIy/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/9oADAMBAAIRAxEAPwDk64z9KP/Z", "base64");
const asKind = (kind, bytes, width, height) => ({ kind, width, height, size: bytes.length, bytes: Buffer.from(bytes).toString("hex") });
const PNG = "image/png";
const JPEG = "image/jpeg";
const pictures = {
  hold: [asKind(PNG, png, 1, 1), asKind(JPEG, jpeg, 2, 1)],
  fail: [
    ["an SVG that says it is a PNG", asKind(PNG, words('<svg xmlns="http://www.w3.org/2000/svg" onload="alert(1)"/>'), 1, 1)],
    ["a JPEG that says it is a PNG", asKind(PNG, jpeg, 2, 1)],
    ["a PNG that says it is a JPEG", asKind(JPEG, png, 1, 1)],
    ["a PNG of another size than it says", asKind(PNG, png, 2, 1)],
    ["a JPEG of another size than it says", asKind(JPEG, jpeg, 1, 2)],
    ["something after the end of a PNG", asKind(PNG, Buffer.concat([png, words("mas")]), 1, 1)],
    ["something after the end of a JPEG", asKind(JPEG, Buffer.concat([jpeg, words("mas")]), 2, 1)],
    ["a PNG cut short", asKind(PNG, png.subarray(0, png.length - 4), 1, 1)],
    ["a JPEG cut short", asKind(JPEG, jpeg.subarray(0, jpeg.length - 2), 2, 1)],
  ].map(([says, one]) => ({ says, ...one })),
};

const said = {
  link_key: b64(LINK_KEY),
  identity: { root: b64(root.public), fingerprint: fingerprintOf(root.public) },
  references: { html: `<a HREF='tisty:page/1'>a</a><img alt=a src=tisty:picture/0 width=1 height=1><q cite=" tisty:file/2 ">b</q><a title/href="tisty:cited/3">c</a><a href="https://a.example/tisty:page/9">tisty:page/8</a>`, said: [{ kind: "page", at: 1 }, { kind: "picture", at: 0 }, { kind: "file", at: 2 }, { kind: "cited", at: 3 }] },
  refused,
  opening,
  a_newer_major: aNewerMajor.bytes.toString("hex"),
  signed_by: { root: b64(root.public), machine: b64(machine.public) },
  verdicts,
  files_refused: [
    { says: "another files envelope than the one it answers for", main: { id: "k7Qm2xPa" }, files: otherFiles.bytes.toString("hex") },
  ],
  files_lying: [
    { says: "more files than it says", document: countedOver, files: moreFiles.bytes.toString("hex") },
    { says: "a file that says it is compressed", document: squeezedOver, files: compressedFile.bytes.toString("hex") },
    { says: "a file at another place than it says", document: { id: HOSTILE, html: "<p>a</p>", body: "a", files: [{ ...aFile, at: 3 }], files_hash: inOtherPlace.hash }, files: inOtherPlace.bytes.toString("hex") },
    { says: "a file of another size than it says", document: { id: HOSTILE, html: "<p>a</p>", body: "a", files: [{ ...aFile, size: 1 }], files_hash: inOtherPlace.hash }, files: inOtherPlace.bytes.toString("hex") },
  ],
  pictures,
  revocation: revoked,
  main: { envelope: main.bytes.toString("hex"), document: whole, pictures: PICTURES.map((one) => one.toString("hex")) },
  files: { envelope: files.bytes.toString("hex"), files: FILES.map((one) => Buffer.from(one).toString("hex")) },
  public: { envelope: open.bytes.toString("hex"), document: plain },
};
writeFileSync(new URL("./vectors.json", import.meta.url), `${JSON.stringify(said, null, 2)}\n`);
