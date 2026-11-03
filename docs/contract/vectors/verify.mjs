// Opens every vector the way the contract says, with only what a browser has: WebCrypto and DecompressionStream.
import { readdirSync, readFileSync } from "node:fs";

const here = new URL(".", import.meta.url);
const json = (name) => JSON.parse(readFileSync(new URL(name, here)));
const allowed = json("../allowed.json");

const fromB64 = (said) => new Uint8Array(Buffer.from(said, "base64url"));
const b64 = (bytes) => Buffer.from(bytes).toString("base64url");
const encoder = new TextEncoder();
const utf8 = new TextDecoder("utf-8", { fatal: true });
const joined = (...parts) => {
  const out = new Uint8Array(parts.reduce((all, one) => all + one.length, 0));
  let at = 0;
  for (const part of parts) {
    out.set(part, at);
    at += part.length;
  }
  return out;
};
const u32 = (n) => {
  const out = new Uint8Array(4);
  new DataView(out.buffer).setUint32(0, n);
  return out;
};
const u64 = (n) => {
  const out = new Uint8Array(8);
  new DataView(out.buffer).setBigUint64(0, BigInt(n));
  return out;
};
const i64 = (n) => {
  const out = new Uint8Array(8);
  new DataView(out.buffer).setBigInt64(0, BigInt(n));
  return out;
};
const text = (said) => encoder.encode(said);
const sha = async (...parts) => new Uint8Array(await crypto.subtle.digest("SHA-256", joined(...parts)));
const same = (one, other) => one.length === other.length && one.every((byte, at) => byte === other[at]);

class Refused extends Error {}
const refuse = (why) => {
  throw new Refused(why);
};

const framed = (bytes, at, most) => {
  if (at.at + 4 > bytes.length) refuse("damaged");
  const len = new DataView(bytes.buffer, bytes.byteOffset).getUint32(at.at);
  const start = at.at + 4;
  if (len > most || bytes.length - start < len) refuse("damaged");
  at.at = start + len;
  return bytes.subarray(start, start + len);
};

const DEFLATED = new Set(["config", "cited", "html", "text"]);
const PICTURES = new Set(["png", "jpeg", "webp", "gif"]);
const named = (id) => /^[A-Za-z0-9_-]{1,64}$/.test(id);

const manifestHolds = (m) => {
  if (m.v > 5) refuse("newer");
  if (m.v !== 5) refuse("damaged");
  const ordered = Number.isInteger(m.version) && m.version >= 1 && (m.from === null || m.from < m.version);
  let keyed = false;
  try {
    keyed = fromB64(m.salt).length === 32 && fromB64(m.commit).length === 32;
  } catch {
    keyed = false;
  }
  if (!named(m.id) || !ordered || !keyed || !Array.isArray(m.parts)) refuse("damaged");
  for (const part of m.parts) {
    const sealed = part.size - 16;
    if (sealed < 0) refuse("damaged");
    if (DEFLATED.has(part.kind)) {
      if (sealed > part.len + Math.floor(part.len / 16) + 64) refuse("damaged");
    } else if (sealed !== part.len) refuse("damaged");
  }
  const kinds = m.parts.map((part) => part.kind);
  if (kinds[0] !== "config" || kinds[1] !== "signature" || kinds.at(-1) !== "text") refuse("damaged");
  let at = 2;
  if (kinds[at] === "cited") at += 1;
  if (kinds[at] !== "html") refuse("damaged");
  const firstPicture = at + 1;
  for (const kind of kinds.slice(firstPicture, -1)) if (!PICTURES.has(kind)) refuse("damaged");
  if (kinds.length < firstPicture + 1) refuse("damaged");
  if (m.parts[0].len > 16384) refuse("damaged");
  return { html: at, cited: at === 3 ? 2 : null, firstPicture };
};

const inflate = async (bytes, len) => {
  const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("deflate-raw"));
  const reader = stream.getReader();
  const out = new Uint8Array(len);
  let at = 0;
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      if (at + value.length > len) refuse("damaged");
      out.set(value, at);
      at += value.length;
    }
  } catch (error) {
    if (error instanceof Refused) throw error;
    refuse("damaged");
  }
  if (at !== len) refuse("damaged");
  return out;
};

const pictureKind = (bytes) => {
  const starts = (...said) => said.every((byte, at) => bytes[at] === byte);
  if (starts(0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a)) return "png";
  if (starts(0xff, 0xd8, 0xff)) return "jpeg";
  if (bytes.length >= 12 && starts(0x52, 0x49, 0x46, 0x46) && bytes[8] === 0x57 && bytes[9] === 0x45) return "webp";
  if (starts(0x47, 0x49, 0x46, 0x38)) return "gif";
  return null;
};

const MEDIA = { png: "image/png", jpeg: "image/jpeg", webp: "image/webp", gif: "image/gif" };

const edVerify = async (open, sig, message) => {
  // The one rule WebCrypto does not promise on every engine: a key of small order proves nothing.
  if (open.every((byte, at) => byte === (at === 0 ? 1 : 0))) return false;
  try {
    const key = await crypto.subtle.importKey("raw", open, { name: "Ed25519" }, false, ["verify"]);
    return await crypto.subtle.verify({ name: "Ed25519" }, key, sig, message);
  } catch {
    return false;
  }
};

const seconds = (iso) => {
  if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/.test(iso)) return null;
  return Date.parse(iso) / 1000;
};

const attested = async (label, root, share, when, sig) => {
  const at = seconds(when);
  if (at === null) return false;
  const rootBytes = fromB64(root);
  return edVerify(rootBytes, fromB64(sig), joined(text(label), rootBytes, fromB64(share), i64(at)));
};

const certificateHolds = (c) => attested("tisty-share.certificate.v1\0", c.root, c.share, c.since, c.sig);
const revocationHolds = (r) => attested("tisty-share.revocation.v1\0", r.root, r.share, r.at, r.sig);

const check = async (signed, about, said, revoked) => {
  const { certificate } = signed;
  if (!(await certificateHolds(certificate))) return "refused";
  const message = joined(
    text("tisty-share.signed.v1\0"),
    fromB64(certificate.root),
    fromB64(certificate.share),
    u64(text(about).length),
    text(about),
    said,
  );
  if (!(await edVerify(fromB64(certificate.share), fromB64(signed.sig), message))) return "refused";
  for (const one of revoked) {
    if (one.root === certificate.root && one.share === certificate.share && (await revocationHolds(one))) return "revoked";
  }
  return "holds";
};

const canonicalSigned = (said) => {
  let parsed;
  try {
    parsed = JSON.parse(utf8.decode(said));
  } catch {
    refuse("damaged");
  }
  const c = parsed?.certificate;
  if (!c) refuse("damaged");
  const again = JSON.stringify({
    certificate: { root: c.root, share: c.share, since: c.since, sig: c.sig },
    sig: parsed.sig,
  });
  if (again !== utf8.decode(said)) refuse("damaged");
  return parsed;
};

const open = async (keyText, envelopeText, revoked) => {
  const bytes = fromB64(envelopeText);
  const magic = [0xff, 0x46, 0x4f, 0x4c];
  if (!magic.every((byte, at) => bytes[at] === byte)) refuse("foreign");
  if (bytes[4] === undefined) refuse("damaged");
  if (bytes[4] > 1) refuse("newer");
  if (bytes[4] !== 1) refuse("damaged");
  const at = { at: 5 };
  const written = framed(bytes, at, 16384);
  let manifest;
  try {
    manifest = JSON.parse(utf8.decode(written));
  } catch {
    refuse("damaged");
  }
  const places = manifestHolds(manifest);
  const index = framed(bytes, at, 32 * manifest.parts.length);
  if (index.length !== 32 * manifest.parts.length) refuse("damaged");
  const trailer = framed(bytes, at, 2048);
  const outer = canonicalSigned(trailer);
  const sealed = [];
  for (const [n, part] of manifest.parts.entries()) {
    const end = at.at + part.size;
    if (end > bytes.length) refuse("damaged");
    const piece = bytes.subarray(at.at, end);
    if (!same(await sha(piece), index.subarray(n * 32, (n + 1) * 32))) refuse("damaged");
    sealed.push(piece);
    at.at = end;
  }
  if (at.at !== bytes.length) refuse("damaged");

  const digest = await sha(text("folio.indexed.v1\0"), u64(written.length), written, u64(index.length), index);
  const answered = joined(u64(text(manifest.id).length), text(manifest.id), u64(manifest.version), digest);
  const sender = await check(outer, "folio.envelope.v1", answered, revoked);

  const link = fromB64(keyText);
  if (link.length !== 32) refuse("damaged");
  const salt = fromB64(manifest.salt);
  const hmac = await crypto.subtle.importKey("raw", link, { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  const commit = new Uint8Array(await crypto.subtle.sign("HMAC", hmac, joined(text("folio.commit.v1\0"), salt)));
  if (!same(commit, fromB64(manifest.commit))) refuse("locked");
  const hkdf = await crypto.subtle.importKey("raw", link, "HKDF", false, ["deriveKey"]);
  const versionKey = await crypto.subtle.deriveKey(
    { name: "HKDF", hash: "SHA-256", salt, info: text("folio.version.v1\0") },
    hkdf,
    { name: "AES-GCM", length: 256 },
    false,
    ["decrypt"],
  );
  const additionalData = joined(text("folio.part.v1\0"), await sha(written));
  const plain = [];
  for (const [n, part] of manifest.parts.entries()) {
    const iv = joined(new Uint8Array(8), u32(n));
    let opened;
    try {
      opened = new Uint8Array(await crypto.subtle.decrypt({ name: "AES-GCM", iv, additionalData }, versionKey, sealed[n]));
    } catch {
      refuse("locked");
    }
    plain.push(DEFLATED.has(part.kind) ? await inflate(opened, part.len) : opened);
  }

  const parsed = (bytes_) => {
    try {
      return JSON.parse(utf8.decode(bytes_));
    } catch {
      return refuse("damaged");
    }
  };
  const config = parsed(plain[0]);
  const wrote = parsed(plain[1]);
  const citedBytes = places.cited === null ? new Uint8Array(0) : plain[places.cited];
  const cited = places.cited === null ? [] : parsed(citedBytes);
  const htmlBytes = plain[places.html];
  const html = utf8.decode(htmlBytes);

  const pictures = [];
  const hashes = wrote.pictures ?? [];
  const listed = manifest.parts.slice(places.firstPicture, -1);
  for (const [n, part] of listed.entries()) {
    const picture = plain[places.firstPicture + n];
    if (pictureKind(picture) !== part.kind) refuse("not_a_picture");
    const hash = b64(await sha(picture));
    if (hash !== hashes[n]) refuse("damaged");
    pictures.push({ kind: part.kind, sha256: hash });
  }
  const words = utf8.decode(plain.at(-1));
  if (b64(await sha(plain.at(-1))) !== wrote.text) refuse("damaged");

  const pieces = [text("folio.written.v1\0"), text("drawn")];
  for (const [name, said] of [
    ["text", text(wrote.text)],
    ["cited", citedBytes],
    ["html", htmlBytes],
    ["config", plain[0]],
  ]) {
    pieces.push(text(name), u64(said.length), said);
  }
  pieces.push(u64(listed.length));
  for (const [n, part] of listed.entries()) {
    for (const said of [MEDIA[part.kind], hashes[n]]) pieces.push(u64(text(said).length), text(said));
  }
  const name = wrote.name === undefined ? [new Uint8Array([0])] : [new Uint8Array([1]), u64(text(wrote.name).length), text(wrote.name)];
  const vouched = joined(await sha(...pieces), u64(text(wrote.id).length), text(wrote.id), i64(seconds(wrote.at)), ...name);
  const sameKey = wrote.signed.certificate.root === outer.certificate.root && wrote.signed.certificate.share === outer.certificate.share;
  const writer = wrote.id === manifest.id && sameKey ? await check(wrote.signed, "folio.text.v1", vouched, revoked) : "refused";

  return {
    sender,
    writer,
    config,
    cited,
    html,
    text: words,
    pictures,
    name: wrote.name ?? null,
    at: wrote.at,
    person: wrote.signed.certificate.root,
    id: manifest.id,
    version: manifest.version,
    from: manifest.from,
  };
};

let wrong = 0;
const amiss = (where, said) => {
  wrong += 1;
  console.error(`x  ${where}: ${said}`);
};

for (const one of json("envelopes.json").envelopes) {
  let got;
  try {
    got = await open(one.key, one.envelope, one.revoked);
  } catch (error) {
    if (!(error instanceof Refused)) throw error;
    got = { refused: error.message };
  }
  const sorted = (value) => JSON.stringify(value, Object.keys(value).sort());
  if (sorted(got) !== sorted(one.expect)) {
    const differs = Object.keys({ ...got, ...one.expect }).filter((key) => JSON.stringify(got[key]) !== JSON.stringify(one.expect[key]));
    amiss(`envelope ${one.name}`, `${differs.join(", ")}: got ${JSON.stringify(differs.map((key) => got[key]))}`);
  }
}

const identity = json("identity.json");
for (const one of identity.certificates) {
  if ((await certificateHolds(one.certificate)) !== one.holds) amiss(`certificate ${one.name}`, `expected ${one.holds}`);
}
for (const one of identity.revocations) {
  if ((await revocationHolds(one.revocation)) !== one.holds) amiss(`revocation ${one.name}`, `expected ${one.holds}`);
}
for (const one of identity.signatures) {
  const verdict = await check(one.signed, one.about, fromB64(one.said), []);
  if ((verdict === "holds") !== one.holds) amiss(`signature ${one.name}`, `got ${verdict}`);
}

const digest = await sha(text("tisty-share.person.v1 "), fromB64(identity.fingerprint.person));
const wide = BigInt(`0x${Buffer.from(digest.subarray(0, 9)).toString("hex")}`) >> 6n;
const shown = wide.toString().padStart(20, "0").match(/.{5}/g).join(" ");
if (shown !== identity.fingerprint.shown) amiss("fingerprint", `got ${shown}`);

// Every real sheet stays inside the list, so the translator and the list cannot drift apart unseen.
const tags = new Set(allowed.tags);
const attributes = new Set(allowed.attributes);
for (const file of readdirSync(new URL("sheets/", here))) {
  const sheet = readFileSync(new URL(`sheets/${file}`, here), "utf8");
  for (const [, tag, rest] of sheet.matchAll(/<([a-zA-Z][a-zA-Z0-9]*)((?:\s[^<>]*)?)\/?>/g)) {
    if (!tags.has(tag.toLowerCase())) amiss(`sheet ${file}`, `<${tag}> is not in allowed.json`);
    for (const [, attribute] of rest.replace(/="[^"]*"/g, "=").matchAll(/\s([a-zA-Z_:-]+)/g)) {
      if (!attributes.has(attribute.toLowerCase())) amiss(`sheet ${file}`, `${attribute} on <${tag}> is not in allowed.json`);
    }
  }
}

// The colours Tisty draws with and the colours the contract promises are one list.
const css = readFileSync(new URL("../../../packages/tisty-editor/src/tokens.css", here), "utf8");
const [light, dark] = css.split('[data-theme="dark"]');
const tokens = json("../tokens.json");
for (const [theme, block] of [["light", light], ["dark", dark]]) {
  for (const [, name, value] of block.matchAll(/--tisty-([a-z-]+):\s*([^;]+);/g)) {
    const promised = tokens.color[theme]?.[name]?.$value;
    if (promised !== value) amiss(`tokens ${theme}.${name}`, `tokens.css says ${value}, tokens.json says ${promised}`);
  }
}

if (wrong) process.exit(1);
console.log("ok the contract's vectors hold");
