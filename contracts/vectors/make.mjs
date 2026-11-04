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
const HEAD = filled(0x22);
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

const revoked = await sign(root, words("tisty-share.revocation.v1\0"), root.public, machine.public, be8(Date.parse(SINCE) / 1000));

const seal = async ({ kind, id, version, sealed, parts }) => {
  const told = parts.map(({ name, compressed, bytes }) => {
    const squeezed = compressed ? deflateRawSync(bytes, { level: 9 }) : bytes;
    return { name, compressed, squeezed, open_size: bytes.length, sealed_size: squeezed.length + (sealed ? 16 : 0) };
  });
  const opening = sealed
    ? { head: b64(HEAD), commitment: b64(await hmac()) }
    : {};
  const manifest = words(
    JSON.stringify({
      contract: CONTRACT,
      kind,
      id,
      version,
      sealed,
      ...opening,
      parts: told.map(({ name, compressed, sealed_size, open_size }) => ({ name, compressed, sealed_size, open_size })),
    }),
  );
  const key = sealed ? await envelopeKey() : undefined;
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
  const signed = await sign(machine, words("tisty-share.signed.v1\0"), root.public, machine.public, be8(about.length), about, sha(manifest), index);
  const signature = words(JSON.stringify({ root: b64(root.public), machine: b64(machine.public), since: SINCE, certificate: b64(certificate), signed: b64(signed) }));
  const bytes = Buffer.concat([words("TENV"), u32(manifest.length), manifest, u32(signature.length), signature, index, ...travelling]);
  return { bytes, hash: b64(sha(manifest, index)) };
};

const nonce = (at) => {
  const out = Buffer.alloc(12);
  out.writeUInt32BE(at, 8);
  return out;
};

async function hmac() {
  const key = await subtle.importKey("raw", LINK_KEY, { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  return Buffer.from(await subtle.sign("HMAC", key, Buffer.concat([words("tisty-envelope-commit"), HEAD])));
}

async function envelopeKey() {
  const base = await subtle.importKey("raw", LINK_KEY, "HKDF", false, ["deriveKey"]);
  return subtle.deriveKey({ name: "HKDF", hash: "SHA-256", salt: HEAD, info: words("tisty-envelope-key") }, base, { name: "AES-GCM", length: 256 }, false, ["encrypt"]);
}

const textParts = ({ body, pages, ...first }) => [
  { name: "first", compressed: true, bytes: words(JSON.stringify(first)) },
  { name: "rest", compressed: true, bytes: words(JSON.stringify({ body, pages })) },
];

const PICTURES = [Buffer.from("89504e470d0a1a0a00", "hex")];
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
  pictures: [{ at: 0, name: "rina.png", kind: "image/png", size: PICTURES[0].length }],
  files: [{ at: 0, name: "cartilla.pdf", kind: "application/pdf", size: FILES[0].length }],
  files_hash: files.hash,
  html: '<h1 id="paseos" data-at="1">Paseos</h1><img src="tisty:picture/0" alt="Rina"><a href="tisty:page/1">Enero</a>',
  body: "# Paseos\n\n![Rina](rina.png)\n",
  pages: [{ id: "p1", page_of: "k7Qm2xPa", at: 1, title: "Enero", html: '<h1 id="enero" data-at="1">Enero</h1>', body: "# Enero\n" }],
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

const said = {
  link_key: b64(LINK_KEY),
  revocation: { root: b64(root.public), machine: b64(machine.public), at: SINCE, signed: b64(revoked) },
  main: { envelope: main.bytes.toString("hex"), document: whole, pictures: PICTURES.map((one) => one.toString("hex")) },
  files: { envelope: files.bytes.toString("hex"), files: FILES.map((one) => Buffer.from(one).toString("hex")) },
  public: { envelope: open.bytes.toString("hex"), document: plain },
};
writeFileSync(new URL("./vectors.json", import.meta.url), `${JSON.stringify(said, null, 2)}\n`);
