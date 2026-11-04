// Opens the sample envelopes the way a page does: with what a browser brings, and nothing else.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { check, checkHtml, VERSION } from "./index.js";

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

const inflated = async (bytes, size) => {
  const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("deflate-raw"));
  const out = new Uint8Array(await new Response(stream).arrayBuffer());
  assert.equal(out.length, size, "a part gives what its manifest says");
  return out;
};

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
  assert.ok(manifestLength <= 65536, "its manifest is within its size");
  const manifestBytes = piece(manifestLength);
  const manifest = JSON.parse(new TextDecoder().decode(manifestBytes));
  assert.deepEqual(check("envelope", manifest), []);
  assert.equal(manifest.id, asked.id, "the document it asked for");
  assert.equal(manifest.kind, asked.kind, "the kind it asked for");
  assert.ok(manifest.sealed || !linkKey, "a key answers only for what is sealed");
  const names = manifest.parts.map((one) => one.name);
  const inOrder =
    asked.kind === "main"
      ? names[0] === "first" && names[1] === "rest" && names.slice(2).every((one) => one === "picture")
      : names.every((one) => one === "file");
  assert.ok(inOrder, "its parts are in their order");
  const signatureLength = length();
  assert.ok(signatureLength <= 4096, "its signature is within its size");
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
    parts.push(part.compressed ? await inflated(held, part.open_size) : held);
  }
  assert.equal(at, bytes.length, "nothing follows its parts");
  return { manifest, manifestBytes, index, parts };
};

// The first two parts of a main envelope are its words; what follows them is pictures.
const documentOf = ({ parts }) => Object.assign({}, ...parts.slice(0, 2).map((one) => JSON.parse(new TextDecoder().decode(one))));
const hexOf = (parts) => parts.map((one) => Buffer.from(one).toString("hex"));
const linkKey = b64(VECTORS.link_key);
const MAIN = { id: "k7Qm2xPa", kind: "main" };
const FILES = { id: "k7Qm2xPa", kind: "files" };
const PUBLIC = { id: "wordsOnly", kind: "main" };

test("the samples are written under this very version", async () => {
  const { manifest } = await opened(hex(VECTORS.main.envelope), MAIN, linkKey);
  assert.equal(manifest.contract, VERSION, "run node contracts/vectors/make.mjs");
});

test("no head is used twice under one key", async () => {
  const main = await opened(hex(VECTORS.main.envelope), MAIN, linkKey);
  const files = await opened(hex(VECTORS.files.envelope), FILES, linkKey);
  assert.notEqual(main.manifest.head, files.manifest.head);
});

test("a main envelope gives its document, its pages and its pictures, with no other request", async () => {
  const main = await opened(hex(VECTORS.main.envelope), MAIN, linkKey);
  assert.deepEqual(documentOf(main), VECTORS.main.document);
  assert.deepEqual(check("document", documentOf(main)), []);
  assert.deepEqual(checkHtml(documentOf(main).html), []);
  assert.equal(documentOf(main).id, main.manifest.id, "the document inside is the one the manifest says");
  assert.deepEqual(hexOf(main.parts.slice(2)), VECTORS.main.pictures);
});

test("a files envelope gives its files, and the main envelope answers for it", async () => {
  const files = await opened(hex(VECTORS.files.envelope), FILES, linkKey);
  assert.deepEqual(hexOf(files.parts), VECTORS.files.files);
  const hash = await sha(files.manifestBytes, files.index);
  assert.ok(same(hash, b64(VECTORS.main.document.files_hash)));
});

test("what Tisty sealed in Rust opens here with what a browser brings", async () => {
  const written = JSON.parse(readFileSync(new URL("./written-by-tisty.json", import.meta.url), "utf8"));
  const key = b64(written.link_key);
  const main = await opened(hex(written.main), { id: "writtenByTisty", kind: "main" }, key);
  const held = documentOf(main);
  assert.deepEqual(check("document", held), []);
  assert.deepEqual(checkHtml(held.html), []);
  assert.equal(held.id, main.manifest.id);
  const files = await opened(hex(written.files), { id: "writtenByTisty", kind: "files" }, key);
  assert.ok(same(await sha(files.manifestBytes, files.index), b64(held.files_hash)));
});

test("words alone need one envelope", async () => {
  const { manifest } = await opened(hex(VECTORS.public.envelope), PUBLIC);
  assert.equal(manifest.parts.length, 2);
  assert.equal(VECTORS.public.document.files_hash, undefined);
});

test("a public envelope opens without a key", async () => {
  const held = documentOf(await opened(hex(VECTORS.public.envelope), PUBLIC));
  assert.deepEqual(held, VECTORS.public.document);
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
