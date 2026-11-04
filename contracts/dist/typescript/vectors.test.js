// Opens the sample envelopes the way a page does: with what a browser brings, and nothing else.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { check, checkHtml, picturesIn, referencesIn, VERSION } from "./index.js";

const VECTORS = JSON.parse(readFileSync(new URL("./vectors.json", import.meta.url), "utf8"));
const { subtle } = globalThis.crypto;
const words = (said) => new TextEncoder().encode(said);
const hex = (said) => Uint8Array.from(Buffer.from(said, "hex"));
const b64 = (said) => Uint8Array.from(Buffer.from(said, "base64url"));
const joined = (...pieces) => Uint8Array.from(Buffer.concat(pieces));
const same = (one, other) => Buffer.from(one).equals(Buffer.from(other));
const sha = async (...pieces) => new Uint8Array(await subtle.digest("SHA-256", joined(...pieces)));
const be = (number) => {
  const out = Buffer.alloc(8);
  out.writeBigUInt64BE(BigInt(number));
  return out;
};

// This reader's own, and no number of the contract's: each reader knows what it can hold.
const OPENS_AT_MOST = 64 * 1024 * 1024;

// A manifest only says how much will come, so nothing is set aside for it and reading stops the moment it passes.
const inflated = async (bytes, size) => {
  assert.ok(size <= OPENS_AT_MOST, "it says more than this reader holds");
  const reader = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("deflate-raw")).getReader();
  const pieces = [];
  let held = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    held += value.length;
    if (held > size) {
      await reader.cancel();
      assert.fail("a part gives more than its manifest says");
    }
    pieces.push(value);
  }
  assert.equal(held, size, "a part gives what its manifest says");
  return joined(...pieces);
};

const majorOf = (said) => said.split(".")[0];
const opensHere = (said) => (said.includes("-") || VERSION.includes("-") ? said === VERSION : majorOf(said) === majorOf(VERSION));

const verified = async (key, signature, ...pieces) => {
  const held = await subtle.importKey("raw", key, "Ed25519", false, ["verify"]);
  assert.ok(await subtle.verify("Ed25519", held, signature, joined(...pieces)), "the signature holds");
};

const opened = async (bytes, asked, linkKey) => {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  assert.ok(same(bytes.subarray(0, 4), words("TENV")), "its mark");
  let at = 4;
  const piece = (size) => {
    const out = bytes.subarray(at, at + size);
    at += size;
    return out;
  };
  const length = () => {
    at += 4;
    return view.getUint32(at - 4);
  };
  const manifestLength = length();
  assert.ok(manifestLength <= 16384, "its manifest is within its size");
  const manifestBytes = piece(manifestLength);
  const manifest = JSON.parse(new TextDecoder().decode(manifestBytes));
  // Before anything else: a newer version may say what this one would call damaged.
  assert.ok(opensHere(String(manifest.contract)), "written under a version this reader opens");
  assert.deepEqual(check("envelope", manifest), []);
  assert.equal(manifest.id, asked.id, "the document it asked for");
  assert.equal(manifest.kind, asked.kind, "the kind it asked for");
  assert.ok(manifest.from === undefined || manifest.from < manifest.version, "it starts from an earlier version");
  assert.ok(manifest.parts.every((one) => !one.compressed || one.name === "first" || one.name === "rest"), "a picture or a file is never compressed");
  assert.ok(manifest.sealed || !linkKey, "a key answers only for what is sealed");
  const names = manifest.parts.map((one) => one.name);
  const inOrder =
    asked.kind === "main"
      ? names[0] === "first" && names[1] === "rest" && names.slice(2).every((one) => one === "picture")
      : names.every((one) => one === "file");
  assert.ok(inOrder, "its parts are in their order");
  const signatureLength = length();
  assert.ok(signatureLength <= 2048, "its signature is within its size");
  const signature = JSON.parse(new TextDecoder().decode(piece(signatureLength)));
  assert.deepEqual(check("signature", signature), []);
  const index = piece(32 * manifest.parts.length);
  const root = b64(signature.root);
  const machine = b64(signature.machine);
  await verified(root, b64(signature.certificate), words("tisty-share.certificate.v1\0"), root, machine, be(Date.parse(signature.since) / 1000));
  const about = words("envelope");
  await verified(machine, b64(signature.signed), words("tisty-share.signed.v1\0"), root, machine, be(about.length), about, await sha(manifestBytes), index);
  let key;
  if (manifest.sealed) {
    const held = await subtle.importKey("raw", linkKey, { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
    const commitment = new Uint8Array(await subtle.sign("HMAC", held, joined(words("tisty-envelope-commit"), b64(manifest.head))));
    assert.ok(same(commitment, b64(manifest.commitment)), "the key is the one it was sealed with");
    const base = await subtle.importKey("raw", linkKey, "HKDF", false, ["deriveKey"]);
    key = await subtle.deriveKey({ name: "HKDF", hash: "SHA-256", salt: b64(manifest.head), info: words("tisty-envelope-key") }, base, { name: "AES-GCM", length: 256 }, false, ["decrypt"]);
  }
  const parts = [];
  for (const [number, part] of manifest.parts.entries()) {
    const travelled = piece(part.sealed_size);
    assert.ok(same(await sha(travelled), index.subarray(32 * number, 32 * number + 32)), "its hash");
    const nonce = new Uint8Array(12);
    new DataView(nonce.buffer).setUint32(8, number);
    const held = manifest.sealed
      ? new Uint8Array(await subtle.decrypt({ name: "AES-GCM", iv: nonce, additionalData: manifestBytes }, key, travelled))
      : travelled;
    assert.ok(part.compressed || held.length === part.open_size, "a part gives what its manifest says");
    parts.push(part.compressed ? await inflated(held, part.open_size) : held);
  }
  assert.equal(at, bytes.length, "nothing follows its parts");
  return { manifest, manifestBytes, index, parts };
};

const asJson = (bytes) => JSON.parse(new TextDecoder().decode(bytes));
const atsOf = (list) => (list ?? []).map((one) => one.at);

// All a reader looks at once it is open: only it can, and a writer's word counts for nothing.
const documentIn = ({ manifest, parts }) => {
  const [first, rest] = parts.slice(0, 2).map(asJson);
  assert.ok(!("body" in first) && !("pages" in first), "the first part holds neither body nor pages");
  assert.ok(Object.keys(rest).every((one) => one === "body" || one === "pages"), "the rest holds only those");
  const held = { ...first, ...rest };
  assert.deepEqual(check("document", held), []);
  assert.equal(held.id, manifest.id, "the document inside is the one the manifest says");
  const pictures = held.pictures ?? [];
  assert.equal(pictures.length, parts.length - 2, "as many pictures as it says");
  pictures.forEach((one, at) => {
    assert.equal(one.at, at, "a picture in its place");
    assert.equal(one.size, parts[at + 2].length, "a picture of its size");
  });
  const pages = held.pages ?? [];
  assert.ok(pages.every((one) => one.page_of === held.id), "its pages are its own");
  assert.ok(pages.every((one, at) => at === 0 || pages[at - 1].at < one.at), "its pages by their place, each in its own");
  assert.equal(new Set(pages.map((one) => one.id)).size, pages.length, "each page with an id of its own");
  assert.equal((held.files ?? []).length > 0, held.files_hash != null, "files and the hash that answers for them go together");
  const there = { page: atsOf(pages), picture: atsOf(pictures), file: atsOf(held.files), cited: (held.cited ?? []).map((_, at) => at) };
  for (const html of [held.html, ...pages.map((one) => one.html)]) {
    assert.deepEqual(checkHtml(html), []);
    for (const { kind, at } of referencesIn(html)) assert.ok(there[kind].includes(at), `tisty:${kind}/${at} is there`);
    for (const { at, width, height } of picturesIn(html)) {
      assert.ok(width === String(pictures[at].width) && height === String(pictures[at].height), "a picture drawn at its own size");
    }
  }
  return held;
};

const reading = async (bytes, asked, linkKey) => {
  const main = await opened(bytes, asked, linkKey);
  return { main, held: documentIn(main) };
};

const filesFor = async (held, bytes, asked, linkKey) => {
  const files = await opened(bytes, asked, linkKey);
  assert.ok(same(await sha(files.manifestBytes, files.index), b64(held.files_hash)), "the envelope its document answers for");
  const said = held.files ?? [];
  assert.equal(said.length, files.parts.length, "as many files as it says");
  said.forEach((one, at) => {
    assert.equal(one.at, at, "a file in its place");
    assert.ok(one.size === undefined || one.size === files.parts[at].length, "a file of its size");
  });
  return files;
};

// What a reader looks at before it draws one. A picture that fails shows its alt, and the words still read.
const pictureHolds = (bytes, { kind, size, width, height }) => {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (bytes.length !== size) return false;
  let said = null;
  if (kind === "image/png") {
    if (!same(bytes.subarray(0, 8), Uint8Array.of(0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a))) return false;
    for (let at = 8; at + 12 <= bytes.length; ) {
      const length = view.getUint32(at);
      const name = String.fromCharCode(...bytes.subarray(at + 4, at + 8));
      if (at === 8 && name === "IHDR" && length >= 8 && at + 16 <= bytes.length) said = [view.getUint32(at + 8), view.getUint32(at + 12)];
      at += 12 + length;
      if (name === "IEND") return at === bytes.length && said?.[0] === width && said?.[1] === height;
    }
    return false;
  }
  if (bytes[0] !== 0xff || bytes[1] !== 0xd8 || bytes[2] !== 0xff) return false;
  for (let at = 2; at + 1 < bytes.length; ) {
    const mark = bytes[at + 1];
    if (bytes[at] !== 0xff) at += 1;
    else if (mark === 0xd9) return at + 2 === bytes.length && said?.[0] === width && said?.[1] === height;
    else if (mark === 0xff) at += 1;
    else if (mark === 0x00 || mark === 0x01 || (mark >= 0xd0 && mark <= 0xd8)) at += 2;
    else {
      if (at + 4 > bytes.length) return false;
      const isSize = mark >= 0xc0 && mark <= 0xcf && mark !== 0xc4 && mark !== 0xc8 && mark !== 0xcc;
      if (isSize && at + 9 <= bytes.length) said = [view.getUint16(at + 7), view.getUint16(at + 5)];
      at += 2 + view.getUint16(at + 2);
    }
  }
  return false;
};

// What a reader says of a signature that holds, from what the server serves beside it. Whole seconds in one form sort as they read.
const verdictOf = async ({ root, machine }, { received, revocations = [] }) => {
  const counting = [];
  for (const one of revocations) {
    if (one.root !== root || one.machine !== machine) continue;
    const held = await subtle.importKey("raw", b64(root), "Ed25519", false, ["verify"]);
    const said = joined(words("tisty-share.revocation.v1\0"), b64(root), b64(machine), be(Date.parse(one.at) / 1000));
    if (await subtle.verify("Ed25519", held, b64(one.signed), said)) counting.push(one.at);
  }
  if (!counting.length) return "holds";
  return received !== undefined && received < counting.sort()[0] ? "retired" : "doubtful";
};

const hexOf = (parts) => parts.map((one) => Buffer.from(one).toString("hex"));
const linkKey = b64(VECTORS.link_key);
const MAIN = { id: "k7Qm2xPa", kind: "main" };
const FILES = { id: "k7Qm2xPa", kind: "files" };
const PUBLIC = { id: "wordsOnly", kind: "main" };

test("the samples are written under this very version", async () => {
  const { manifest } = await opened(hex(VECTORS.main.envelope), MAIN, linkKey);
  assert.equal(manifest.contract, VERSION, "run node contracts/vectors/make.mjs");
});

test("no head is used twice under one key, across every sample", async () => {
  const written = JSON.parse(readFileSync(new URL("./written-by-tisty.json", import.meta.url), "utf8"));
  assert.equal(written.link_key, VECTORS.link_key);
  const heads = [
    await opened(hex(VECTORS.main.envelope), MAIN, linkKey),
    await opened(hex(VECTORS.files.envelope), FILES, linkKey),
    await opened(hex(VECTORS.files_refused[0].files), FILES, linkKey),
    await opened(hex(written.main), { id: "writtenByTisty", kind: "main" }, linkKey),
    await opened(hex(written.files), { id: "writtenByTisty", kind: "files" }, linkKey),
  ].map((one) => one.manifest.head);
  assert.equal(new Set(heads).size, heads.length);
});

for (const { says, envelope } of VECTORS.refused) {
  test(`a reader refuses ${says}`, async () => {
    await assert.rejects(reading(hex(envelope), { id: "hostile", kind: "main" }));
  });
}

for (const { says, main, files } of VECTORS.files_refused) {
  test(`a reader refuses ${says}`, async () => {
    const { held } = await reading(hex(VECTORS.main.envelope), { id: main.id, kind: "main" }, linkKey);
    await assert.rejects(filesFor(held, hex(files), { id: main.id, kind: "files" }, linkKey));
  });
}

for (const { says, document, files } of VECTORS.files_lying) {
  test(`a reader refuses ${says}`, async () => {
    await assert.rejects(filesFor(document, hex(files), { id: "hostile", kind: "files" }));
  });
}

for (const { says, served, verdict } of VECTORS.verdicts) {
  test(`a signature is ${verdict} when ${says}`, async () => {
    assert.deepEqual(check("served", served), []);
    assert.equal(await verdictOf(VECTORS.signed_by, served), verdict);
  });
}

for (const { says, envelope } of VECTORS.opening) {
  test(`a reader opens ${says}`, async () => {
    await reading(hex(envelope), { id: "hostile", kind: "main" });
  });
}

test("a newer major version is told from a damaged one, whatever it holds", async () => {
  await assert.rejects(reading(hex(VECTORS.a_newer_major), { id: "hostile", kind: "main" }), /version/);
});

for (const one of VECTORS.pictures.hold) {
  test(`a real ${one.kind} holds`, () => assert.ok(pictureHolds(hex(one.bytes), one)));
}

for (const one of VECTORS.pictures.fail) {
  test(`a reader shows the alt of ${one.says}`, () => assert.ok(!pictureHolds(hex(one.bytes), one)));
}

test("a reference is read as a browser reads it, and only where it leads", () => {
  assert.deepEqual(referencesIn(VECTORS.references.html), VECTORS.references.said);
});

test("the fingerprint is what the contract's words make of a root", async () => {
  const digest = await sha(words("tisty-share.person.v1\0"), b64(VECTORS.identity.root));
  const number = BigInt(`0x${Buffer.from(digest.subarray(0, 9)).toString("hex")}`) >> 6n;
  assert.equal(number.toString().padStart(20, "0").match(/.{5}/g).join(" "), VECTORS.identity.fingerprint);
});

test("a main envelope gives its document, its pages and its pictures, with no other request", async () => {
  const { main, held } = await reading(hex(VECTORS.main.envelope), MAIN, linkKey);
  assert.deepEqual(held, VECTORS.main.document);
  assert.deepEqual(hexOf(main.parts.slice(2)), VECTORS.main.pictures);
  held.pictures.forEach((one, at) => assert.ok(pictureHolds(main.parts[at + 2], one)));
});

test("a files envelope gives its files, and the main envelope answers for it", async () => {
  const files = await filesFor(VECTORS.main.document, hex(VECTORS.files.envelope), FILES, linkKey);
  assert.deepEqual(hexOf(files.parts), VECTORS.files.files);
});

test("what Tisty sealed in Rust opens here with what a browser brings", async () => {
  const written = JSON.parse(readFileSync(new URL("./written-by-tisty.json", import.meta.url), "utf8"));
  const key = b64(written.link_key);
  const { held } = await reading(hex(written.main), { id: "writtenByTisty", kind: "main" }, key);
  await filesFor(held, hex(written.files), { id: "writtenByTisty", kind: "files" }, key);
});

test("words alone need one envelope", async () => {
  const { manifest } = await opened(hex(VECTORS.public.envelope), PUBLIC);
  assert.equal(manifest.parts.length, 2);
  assert.equal(VECTORS.public.document.files_hash, undefined);
});

test("a public envelope opens without a key", async () => {
  const { held } = await reading(hex(VECTORS.public.envelope), PUBLIC);
  assert.deepEqual(held, VECTORS.public.document);
});

test("words are counted in bytes, as a machine holds them", () => {
  assert.deepEqual(check("document", { id: "x", html: "<p>x</p>", body: "a".repeat(512000) }), []);
  assert.notDeepEqual(check("document", { id: "x", html: "<p>x</p>", body: "€".repeat(200000) }), []);
});

test("a revocation is signed by the root", async () => {
  const said = VECTORS.revocation;
  assert.deepEqual(check("revocation", said), []);
  const root = b64(said.root);
  await verified(root, b64(said.signed), words("tisty-share.revocation.v1\0"), root, b64(said.machine), be(Date.parse(said.at) / 1000));
});

test("another key does not open it", async () => {
  await assert.rejects(opened(hex(VECTORS.main.envelope), MAIN, new Uint8Array(32)));
});

test("a link with a key refuses an envelope that is not sealed", async () => {
  await assert.rejects(opened(hex(VECTORS.public.envelope), PUBLIC, linkKey));
});

test("another document does not pass for the one asked for", async () => {
  await assert.rejects(opened(hex(VECTORS.public.envelope), MAIN));
});

test("a files envelope does not pass for the main one", async () => {
  await assert.rejects(opened(hex(VECTORS.files.envelope), MAIN, linkKey));
});

for (const [says, at] of [["its last byte", -1], ["a byte of its manifest", 40]]) {
  test(`changing ${says} does not open it`, async () => {
    const bytes = hex(VECTORS.main.envelope);
    bytes[at < 0 ? bytes.length + at : at] ^= 1;
    await assert.rejects(opened(bytes, MAIN, linkKey));
  });
}

test("cutting it does not open it", async () => {
  await assert.rejects(opened(hex(VECTORS.main.envelope).slice(0, -1), MAIN, linkKey));
});
