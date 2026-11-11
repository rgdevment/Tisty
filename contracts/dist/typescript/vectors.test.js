// Opens the sample envelopes the way a page does: with what a browser brings, and nothing else.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { check, checkHtml, picturesIn, playableAs, referencesIn, VERSION } from "./index.js";

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
const minorOf = (said) => {
  const written = said.split(".")[1];
  return String(Number(written)) === written ? Number(written) : Number.NaN;
};
const FIRST_SHARED = { major: "1", minor: 2 };
const opensHere = (said) => {
  if (said.includes("-") || VERSION.includes("-")) return said === VERSION;
  if (majorOf(said) !== majorOf(VERSION) || Number.isNaN(minorOf(said))) return false;
  return majorOf(said) !== FIRST_SHARED.major || minorOf(said) >= FIRST_SHARED.minor;
};
const underALaterMinor = (bytes) => {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const said = String(JSON.parse(new TextDecoder().decode(bytes.subarray(8, 8 + view.getUint32(4)))).contract);
  return !said.includes("-") && majorOf(said) === majorOf(VERSION) && minorOf(said) > minorOf(VERSION);
};
const toldApart = async (bytes, going) => {
  try {
    return await going();
  } catch (wrong) {
    if (underALaterMinor(bytes)) throw new Error("newer: update to read it");
    throw wrong;
  }
};

const verified = async (key, signature, ...pieces) => {
  const held = await subtle.importKey("raw", key, "Ed25519", false, ["verify"]);
  assert.ok(await subtle.verify("Ed25519", held, signature, joined(...pieces)), "the signature holds");
};

// The one rule of who may sign a version: its owner, or whoever holds the token its owner allowed.
const ownerAnswers = async (manifest, signature, manifestBytes, index) => {
  const { owner } = manifest;
  const root = b64(owner.root);
  const machine = b64(owner.machine);
  await verified(root, b64(owner.certificate), words("tisty-share.certificate.v1\0"), root, machine, be(Date.parse(owner.since) / 1000));
  const about = words("owner");
  const id = words(manifest.id);
  const permit = owner.permit ? b64(owner.permit) : new Uint8Array(32);
  await verified(machine, b64(owner.signed), words("tisty-share.signed.v1\0"), root, machine, be(about.length), about, be(id.length), id, permit, b64(owner.name_tag));
  if (owner.root === signature.root) return;
  assert.ok(owner.permit && signature.permit_signed, "another root needs its owner's permit");
  await verified(permit, b64(signature.permit_signed), words("tisty-share.permit.v1\0"), b64(signature.root), b64(signature.machine), await sha(manifestBytes), index);
};

// An editor answers only for having signed this name for this document, and one that does not hold is left aside.
const vouched = async (entry) => {
  const root = b64(entry.root);
  const machine = b64(entry.machine);
  await verified(root, b64(entry.certificate), words("tisty-share.certificate.v1\0"), root, machine, be(Date.parse(entry.since) / 1000));
  return machine;
};

const editorHolds = async (entry) => {
  try {
    const root = b64(entry.root);
    const machine = await vouched(entry);
    const about = words("editor");
    const id = words(entry.in);
    const name = words(entry.name);
    const owned = new Uint8Array([entry.owned === true ? 1 : 0]);
    await verified(machine, b64(entry.signed), words("tisty-share.signed.v1\0"), root, machine, be(about.length), about, be(id.length), id, owned, be(name.length), name);
    return true;
  } catch {
    return false;
  }
};

test("a document keeps only the editors signed for it", async () => {
  const { id, editors, kept } = VECTORS.editors_in_a_document;
  const holding = await Promise.all(editors.map(async (one) => one.in === id && (await editorHolds(one))));
  assert.deepEqual(
    editors.filter((_, at) => holding[at]),
    kept,
  );
});

for (const { says, served, kept } of VECTORS.editors_weighed.cases) {
  test(`an editor's machine is weighed against what was revoked: ${says}`, async () => {
    const { id, editors } = VECTORS.editors_weighed;
    const standing = await Promise.all(
      editors.map(async (one) => {
        if (one.in !== id || !(await editorHolds(one))) return false;
        const verdict = await verdictOf(one, served);
        return verdict === "holds" || verdict === "retired";
      }),
    );
    assert.deepEqual(
      editors.filter((_, at) => standing[at]),
      kept,
    );
  });
}

for (const { says, holds, entry } of VECTORS.editors) {
  test(`an editor ${holds ? "holds" : "is left aside"}: ${says}`, async () => {
    assert.equal(await editorHolds(entry), holds);
    assert.equal(await editorHolds({ ...entry, since: "x" }), false);
  });
}

const ownerNameHolds = async (manifest, first, linkKey) => {
  const id = words(manifest.id);
  const name = words(JSON.parse(new TextDecoder().decode(first)).owner_name ?? "");
  const held = await subtle.importKey("raw", linkKey, { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  const tag = new Uint8Array(await subtle.sign("HMAC", held, joined(words("tisty-owner-name\0"), be(id.length), id, name)));
  assert.ok(same(tag, b64(manifest.owner.name_tag)), "its owner's name is the one its owner signed");
};

const opened = (bytes, asked, linkKey, signaturesOnly) => toldApart(bytes, () => openedAsSaid(bytes, asked, linkKey, signaturesOnly));

const openedAsSaid = async (bytes, asked, linkKey, signaturesOnly) => {
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
  if (manifest.sealed) await ownerAnswers(manifest, signature, manifestBytes, index);
  if (signaturesOnly) return { manifest, signature };
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
  if (manifest.sealed && asked.kind === "main") await ownerNameHolds(manifest, parts[0], linkKey);
  return { manifest, manifestBytes, index, parts, signature };
};

const asJson = (bytes) => JSON.parse(new TextDecoder().decode(bytes));
const atsOf = (list) => (list ?? []).map((one) => one.at);

// All a reader looks at once it is open: only it can, and a writer's word counts for nothing.
const documentIn = ({ manifest, parts }) => {
  const [first, rest] = parts.slice(0, 2).map(asJson);
  assert.ok(!("body" in first) && !("pages" in first), "the first part holds neither body nor pages");
  assert.ok(Object.keys(rest).every((one) => !(one in first)), "the rest repeats nothing of the first part");
  const held = { ...first, ...(rest.body === undefined ? {} : { body: rest.body }), ...(rest.pages === undefined ? {} : { pages: rest.pages }) };
  assert.deepEqual(check("document", held), []);
  assert.equal(held.id, manifest.id, "the document inside is the one the manifest says");
  assert.ok(manifest.sealed || held.owner_name === undefined, "what is public has nothing to answer for its owner's name");
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
  return { main, held: await toldApart(bytes, () => documentIn(main)) };
};

const filesFor = (held, bytes, asked, linkKey) => toldApart(bytes, () => filesAsSaid(held, bytes, asked, linkKey));

const filesAsSaid = async (held, bytes, asked, linkKey) => {
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

const hmacOf = async (key, ...pieces) => {
  const held = await subtle.importKey("raw", key, { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  return new Uint8Array(await subtle.sign("HMAC", held, joined(...pieces)));
};
const ofLanguage = (anchorKey, id) => hmacOf(anchorKey, words("tisty-language-review\0"), words(id));
const text = (said) => {
  assert.equal(typeof said, "string", "a review names what it keeps");
  return words(said);
};
const sized = (said) => [be(said.length), said];
const reviewBytes = (anchor, review) => {
  const title = review.title === undefined ? new Uint8Array() : text(review.title);
  const naming = review.kept ? [...sized(text(review.document)), ...sized(text(review.lang)), ...sized(title), b64(review.key), b64(review.print)] : [];
  const about = words("language-review");
  return joined(words("tisty-share.signed.v1\0"), b64(review.root), b64(review.machine), ...sized(about), ...sized(words(anchor)), b64(review.of), be(review.seq), new Uint8Array([review.kept ? 1 : 0]), ...naming);
};

const reviewHolds = async (anchor, owner, anchorKey, review) => {
  try {
    if (review.root !== owner) return false;
    await verified(await vouched(review), b64(review.signed), reviewBytes(anchor, review));
    return !review.kept || same(b64(review.of), await ofLanguage(anchorKey, review.document));
  } catch {
    return false;
  }
};

const reviewsHash = async (anchor, reviews) => {
  const each = await Promise.all(reviews.map((one) => sha(reviewBytes(anchor, one))));
  each.sort((one, other) => Buffer.compare(Buffer.from(one), Buffer.from(other)));
  return sha(words("tisty-language-reviews\0"), ...each);
};

const reviewedHolds = async (anchor, owner, reviews, head, served = {}) => {
  try {
    if (head?.anchor !== anchor || head.root !== owner) return false;
    const about = words("reviewed");
    const said = [words("tisty-share.signed.v1\0"), b64(head.root), b64(head.machine), ...sized(about), ...sized(words(anchor)), be(head.seq), b64(head.reviews)];
    await verified(await vouched(head), b64(head.signed), ...said);
    const weighed = await verdictOf(head, { received: served.reviewed_received, revocations: served.revocations });
    return same(b64(head.reviews), await reviewsHash(anchor, reviews)) && (weighed === "holds" || weighed === "retired");
  } catch {
    return false;
  }
};

const newerReview = (one, was) => !was || one.seq > was.seq || (one.seq === was.seq && Buffer.compare(Buffer.from(b64(one.machine)), Buffer.from(b64(was.machine))) > 0);

const languagesShown = async ({ anchor, owner, link_key, map, languages, reviews, served = {}, seen = 0 }) => {
  const anchorKey = b64(link_key);
  const counting = (await reviewedHolds(anchor, owner, reviews, served.reviewed, served)) && served.reviewed.seq >= seen;
  const latest = new Map();
  for (const one of counting ? reviews : []) {
    if ((await reviewHolds(anchor, owner, anchorKey, one)) && newerReview(one, latest.get(one.of))) latest.set(one.of, one);
  }
  const kept = [...latest.values()].filter((one) => one.kept);
  const reached = new Set([...languages.filter((one) => one.root === owner && map.includes(one.document)).map((one) => one.document), ...kept.map((one) => one.document)]);
  const shown = [];
  for (const one of languages.filter((held) => reached.has(held.document))) {
    const review = kept.find((keeps) => keeps.document === one.document);
    const keeps = review?.lang === one.lang && review.print === one.print;
    if (one.root === owner || keeps) shown.push(one.document);
  }
  return shown;
};

const printOf = async (parts) => sha(words("tisty-language-print\0"), ...(await Promise.all(parts.map((one) => sha(one)))));

for (const { says, holds, review } of VECTORS.reviews.cases) {
  test(`a review ${holds ? "holds" : "is left aside"}: ${says}`, async () => {
    const { anchor, owner, link_key } = VECTORS.reviews;
    assert.equal(await reviewHolds(anchor, owner, b64(link_key), review), holds);
    assert.equal(await reviewHolds(anchor, owner, b64(link_key), { ...review, since: "x" }), false);
  });
}

for (const { says, holds, head } of VECTORS.reviewed.cases) {
  test(`the word on the reviews ${holds ? "holds" : "is left aside"}: ${says}`, async () => {
    const { anchor, owner, reviews } = VECTORS.reviewed;
    assert.equal(await reviewedHolds(anchor, owner, reviews, head), holds);
  });
}

for (const one of VECTORS.languages_shown) {
  test(`a reader shows what its owner keeps: ${one.says}`, async () => {
    assert.deepEqual(await languagesShown(one), one.shown);
  });
}

const hexOf = (parts) => parts.map((one) => Buffer.from(one).toString("hex"));
const linkKey = b64(VECTORS.link_key);
const MAIN = { id: "k7Qm2xPa", kind: "main" };
const FILES = { id: "k7Qm2xPa", kind: "files" };
const PUBLIC = { id: "wordsOnly", kind: "main" };

const EDITING = { id: VECTORS.editing.id, kind: "main" };

for (const { says, signer, envelope } of VECTORS.editing.opens) {
  test(`a reader opens ${says}`, async () => {
    const { signature, manifest } = await opened(hex(envelope), EDITING, linkKey);
    assert.equal(signature.root, signer);
    assert.equal(manifest.owner.root, VECTORS.editing.owner);
  });
}

for (const { says, envelope } of VECTORS.editing.refused) {
  test(`a reader refuses ${says}`, async () => {
    await assert.rejects(reading(hex(envelope), EDITING, linkKey));
  });
}

for (const { says, envelope, without_the_key: withoutTheKey } of VECTORS.editing.refused) {
  if (withoutTheKey) {
    test(`a server refuses, with nothing open, ${says}`, async () => {
      await assert.rejects(opened(hex(envelope), EDITING, undefined, true));
    });
  }
}

test("what a rest says beyond its body and pages is left aside", async () => {
  const { held } = await reading(hex(VECTORS.editing.rest_left_aside), EDITING, linkKey);
  assert.equal(held.owner_name, undefined);
  assert.equal(held.title, undefined);
  assert.equal(held.body, "a");
});

for (const { says, envelope } of VECTORS.newer_than_this) {
  test(`under a later minor, ${says} reads as newer`, async () => {
    await assert.rejects(reading(hex(envelope), { id: "hostile", kind: "main" }), /newer/);
  });
}

test("under a later minor, files that do not fit read as newer", async () => {
  const { document, files } = VECTORS.newer_files;
  await assert.rejects(filesFor(document, hex(files), { id: "hostile", kind: "files" }), /newer/);
});

test("what comes before the first version shared is too old, never damaged", async () => {
  const { id, envelope } = VECTORS.before_the_first_shared;
  await assert.rejects(reading(hex(envelope), { id, kind: "main" }, linkKey), /version/);
});

test("no key of small order passes", () => {
  for (const one of VECTORS.small_order) {
    assert.notDeepEqual(check("revocation", { root: one, machine: VECTORS.signed_by.machine, at: "2026-11-03T12:00:00Z", signed: VECTORS.revocation.signed }), []);
  }
});

const playsAsItSays = async ({ document, files, is }, key) => {
  const { parts } = await filesFor(document, hex(files), FILES, key);
  for (const { at, says, is: said } of is) assert.deepEqual(playableAs(parts[at]), said, says);
};

for (const [version, { link_key: key, main, files, public: open, playing }] of Object.entries(VECTORS.frozen)) {
  test(`what ${version} wrote still opens`, async () => {
    const { held } = await reading(hex(main.envelope), MAIN, b64(key));
    assert.deepEqual(held, main.document);
    await filesFor(held, hex(files.envelope), FILES, b64(key));
    assert.deepEqual((await reading(hex(open.envelope), PUBLIC)).held, open.document);
    if (playing) await playsAsItSays(playing, b64(key));
  });
}

test("a file's first bytes say whether it plays", () => {
  for (const { says, bytes, format } of VECTORS.first_bytes) assert.equal(playableAs(hex(bytes))?.format ?? null, format, says);
});

test("a hostile widget travels as words that run nothing", () => {
  const { id, html, body } = VECTORS.hostile_widget;
  assert.deepEqual(check("document", { id, html, body }), []);
  assert.deepEqual(checkHtml(html), []);
  assert.doesNotMatch(html, /<script/i);
});

test("a revocation of its owner's machine weighs on what an editor signed", async () => {
  const { envelope, served, verdict } = VECTORS.editing.owner_revoked;
  const { manifest, signature } = await opened(hex(envelope), EDITING, linkKey);
  assert.equal(await verdictOf(signature, served), "holds");
  assert.equal(await verdictOf(manifest.owner, served), verdict);
});

test("the token follows the key after a point, and neither has one of its own", () => {
  const [, secret] = VECTORS.editing.link.split("#");
  assert.deepEqual(secret.split("."), [VECTORS.link_key, VECTORS.editing.token]);
});

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

// The standard refuses it and browsers follow, but Node 22 lets it through, and a reader can only see it through what its runtime does.
const refusesWhatFollows = await inflated(Uint8Array.of(3, 0, 1), 0).then(
  () => false,
  () => true,
);

test("a reader refuses a part with something past its end", { skip: !refusesWhatFollows }, async () => {
  await assert.rejects(reading(hex(VECTORS.past_its_end), { id: "hostile", kind: "main" }));
});

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

test("a language's print answers for its parts once open, whatever head or number sealed them", async () => {
  const { main } = await reading(hex(VECTORS.main.envelope), MAIN, linkKey);
  assert.equal(Buffer.from(await printOf(main.parts)).toString("base64url"), VECTORS.main.print);
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
