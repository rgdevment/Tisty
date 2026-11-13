import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, test } from "node:test";
import { blueskyLength, check, forX, judged, linkFacets, oauthSignature, readPosts, send, webpSize, xLength } from "./posts.mjs";

const picture = "assets/screens/readme/en-01-archive.webp";
const said = (text) => ({ text, picture, shows: "The Archive" });
const entry = (more = {}) => ({
  version: "9.9.0",
  es: said("¿Sabías que Tisty guarda lo que terminas? https://tisty.app"),
  en: said("Did you know Tisty keeps what you finish? https://tisty.app"),
  ...more,
});

const KEYS = {
  X_API_KEY: "k",
  X_API_SECRET: "s",
  X_ACCESS_TOKEN: "t",
  X_ACCESS_SECRET: "a",
  BLUESKY_HANDLE: "tisty.app",
  BLUESKY_APP_PASSWORD: "p",
};

function home(posts) {
  const root = mkdtempSync(join(tmpdir(), "posts-"));
  mkdirSync(join(root, "docs"));
  mkdirSync(join(root, "assets/screens/readme"), { recursive: true });
  writeFileSync(join(root, picture), readFileSync(new URL(`../${picture}`, import.meta.url)));
  writeFileSync(join(root, "docs/posts.json"), typeof posts === "string" ? posts : JSON.stringify(posts));
  return root;
}

const realFetch = globalThis.fetch;
afterEach(() => {
  globalThis.fetch = realFetch;
});

function networksThatAnswer({ xDuplicate = false, earlier = [] } = {}) {
  const calls = [];
  globalThis.fetch = async (url, init = {}) => {
    calls.push({ url, init });
    const json = (status, body) => new Response(JSON.stringify(body), { status });
    if (url.endsWith("/2/media/upload")) return json(200, { data: { id: "77" } });
    if (url.endsWith("/2/media/metadata")) return json(200, { data: { id: "77" } });
    if (url.endsWith("/2/tweets")) {
      return xDuplicate
        ? json(403, { detail: "You are not allowed to create a Tweet with duplicate content." })
        : json(201, { data: { id: "1001" } });
    }
    if (url.endsWith("createSession")) return json(200, { accessJwt: "jwt", did: "did:plc:tisty" });
    if (url.includes("getAuthorFeed")) return json(200, { feed: earlier.map((post) => ({ post })) });
    if (url.endsWith("uploadBlob")) return json(200, { blob: { $type: "blob", ref: "b" } });
    if (url.endsWith("createRecord")) return json(200, { uri: "at://did:plc:tisty/app.bsky.feed.post/3abc" });
    return json(404, {});
  };
  return calls;
}

const listening = () => {
  const lines = [];
  return { lines, say: (line) => lines.push(line) };
};

test("every post in the file holds for its own version", () => {
  for (const one of readPosts()) assert.deepEqual(judged(readPosts(), one.version), [], one.version);
});

test("a version with no post is told what to add", () => {
  assert.match(judged([entry()], "9.9.1")[0], /nothing in docs\/posts\.json tells what to post for 9\.9\.1/);
});

test("each language opens with its own question, and both are there", () => {
  assert.deepEqual(judged([entry()], "9.9.0"), []);
  assert.match(judged([entry({ en: said("Tisty keeps what you finish") })], "9.9.0").join(), /en: it opens with «Did you know»/);
  assert.match(judged([entry({ es: undefined })], "9.9.0").join(), /9\.9\.0 es: no text/);
});

test("a picture outside the repo or of another kind is refused", () => {
  const missing = { ...said("Did you know"), picture: "assets/nowhere.webp" };
  assert.match(judged([entry({ en: missing })], "9.9.0").join(), /not in the repo/);
  const gif = { ...said("Did you know"), picture: "assets/a.gif" };
  assert.match(judged([entry({ en: gif })], "9.9.0").join(), /\.webp, \.png or \.jpg/);
});

test("X gets the text without the link at its end, and nothing else is taken out", () => {
  assert.equal(forX("¿Sabías que Tisty guarda lo que terminas? https://tisty.app"), "¿Sabías que Tisty guarda lo que terminas?");
  assert.equal(forX("Did you know Node.js and NOTES.md stay? https://tisty.app"), "Did you know Node.js and NOTES.md stay?");
  assert.equal(xLength(forX("Did you know? https://tisty.app")), "Did you know?".length);
});

test("a link anywhere but the end, or a domain without its scheme, is refused", () => {
  const middle = said("Did you know https://tisty.app is free?");
  assert.match(judged([entry({ en: middle })], "9.9.0").join(), /https:\/\/tisty\.app goes at the end/);
  const bare = said("Did you know tisty.app is free?");
  assert.match(judged([entry({ en: bare })], "9.9.0").join(), /tisty\.app needs its https:\/\//);
  assert.deepEqual(judged([entry({ en: said("Did you know NOTES.md and Node.js stay? https://tisty.app") })], "9.9.0"), []);
});

test("X weighs an emoji, an ellipsis and a CJK character as 2, and accents as 1", () => {
  assert.equal(xLength("¿Sabías?"), 8);
  assert.equal(xLength("👍🏽"), 2);
  assert.equal(xLength("…"), 2);
  assert.equal(xLength("漢"), 2);
  assert.equal(xLength("—"), 1);
  const heavy = said(`Did you know ${"…".repeat(134)}`);
  assert.match(judged([entry({ en: heavy })], "9.9.0").join(), /on X, which takes 280/);
});

test("Bluesky counts what a reader sees as one character", () => {
  assert.equal(blueskyLength("¿Sabías?"), 8);
  assert.equal(blueskyLength("👍🏽"), 1);
});

test("a link is clickable on Bluesky at its place in bytes, after accents", () => {
  const text = "¿Sabías que sí? https://tisty.app.";
  const [facet] = linkFacets(text);
  const bytes = Buffer.from(text, "utf8");
  assert.equal(bytes.subarray(facet.index.byteStart, facet.index.byteEnd).toString("utf8"), "https://tisty.app");
});

test("the OAuth 1.0a signature matches the one X documents", () => {
  const params = {
    include_entities: "true",
    oauth_consumer_key: "xvz1evFS4wEEPTGEFPHBog",
    oauth_nonce: "kYjzVBB8Y0ZFabxSWbWovY3uYSQ2pTgmZeNu2VS4cg",
    oauth_signature_method: "HMAC-SHA1",
    oauth_timestamp: "1318622958",
    oauth_token: "370773112-GmHxMAgYyLbNEtIKZeRNFsMKPR9EyMZeS9weJAEb",
    oauth_version: "1.0",
    status: "Hello Ladies + Gentlemen, a signed OAuth request!",
  };
  const signed = oauthSignature(
    "POST",
    "https://api.twitter.com/1.1/statuses/update.json",
    params,
    "kAcSOqF21Fu85e7zjz7ZN2U4ZRhfV3WpwPAoE3Z7kBw",
    "LswwdoUaIvS8ltyTt5jkRh4J50vUPVVHtR2YPi5kE",
  );
  assert.equal(signed, "hCtSmYh+iHYCEqBWrE7C7hYmtUk=");
});

test("a picture's size is read from each kind of WebP header", () => {
  const size = webpSize(readFileSync(new URL(`../${picture}`, import.meta.url)));
  assert.ok(size.width > size.height && size.height > 0);
  const header = (chunk) => {
    const bytes = Buffer.alloc(30);
    bytes.write("RIFF", 0, "ascii");
    bytes.write("WEBP", 8, "ascii");
    bytes.write(chunk, 12, "ascii");
    return bytes;
  };
  const lossy = header("VP8 ");
  lossy.writeUInt16LE(1200, 26);
  lossy.writeUInt16LE(630, 28);
  assert.deepEqual(webpSize(lossy), { width: 1200, height: 630 });
  const lossless = header("VP8L");
  lossless.writeUInt32LE(1199 | (629 << 14), 21);
  assert.deepEqual(webpSize(lossless), { width: 1200, height: 630 });
  assert.equal(webpSize(Buffer.from("not a picture at all")), undefined);
});

test("a broken file is told in one line, not a stack", () => {
  const { lines, say } = listening();
  assert.equal(check("9.9.0", { root: home("[{,]"), say, inActions: false }), 1);
  assert.match(lines[0], /^x {2}docs\/posts\.json could not be read/);
});

test("a candidate is exempt, a stable version needs its post", () => {
  const root = home([entry()]);
  assert.equal(check("9.9.1-rc.1", { root, say: () => {} }), 0);
  assert.equal(check("9.9.0", { root, say: () => {} }), 0);
  assert.equal(check("9.9.1", { root, say: () => {} }), 1);
});

test("both languages go to X with their picture and alt text, signed", async () => {
  const calls = networksThatAnswer();
  const { lines, say } = listening();
  const env = { ...KEYS, BLUESKY_HANDLE: "" };
  assert.equal(await send("9.9.0", { env, root: home([entry()]), say }), 0);
  const posted = calls.filter((one) => one.url.endsWith("/2/tweets")).map((one) => JSON.parse(one.init.body));
  assert.deepEqual(
    posted.map((one) => one.text),
    ["¿Sabías que Tisty guarda lo que terminas?", "Did you know Tisty keeps what you finish?"],
  );
  assert.deepEqual(posted[0].media, { media_ids: ["77"] });
  const alt = JSON.parse(calls.find((one) => one.url.endsWith("/2/media/metadata")).init.body);
  assert.deepEqual(alt, { id: "77", metadata: { alt_text: { text: "The Archive" } } });
  assert.ok(calls.every((one) => one.init.headers.Authorization.startsWith("OAuth ")));
  assert.ok(lines.some((one) => /Bluesky has no keys here/.test(one)));
});

test("X refusing a post it already holds counts as posted", async () => {
  networksThatAnswer({ xDuplicate: true });
  const { lines, say } = listening();
  const env = { ...KEYS, BLUESKY_HANDLE: "" };
  assert.equal(await send("9.9.0", { env, root: home([entry()]), say }), 0);
  assert.equal(lines.filter((one) => one.includes("already posted")).length, 2);
});

test("Bluesky signs in once, skips a post made days ago, and posts again a tip told months ago", async () => {
  const lately = new Date().toISOString();
  const months = new Date(Date.now() - 90 * 24 * 60 * 60 * 1000).toISOString();
  const earlier = [
    { uri: "at://did:plc:tisty/app.bsky.feed.post/old", record: { text: entry().es.text, createdAt: lately } },
    { uri: "at://did:plc:tisty/app.bsky.feed.post/older", record: { text: entry().en.text, createdAt: months } },
  ];
  const calls = networksThatAnswer({ earlier });
  const { lines, say } = listening();
  const env = { ...KEYS, X_API_KEY: "" };
  assert.equal(await send("9.9.0", { env, root: home([entry()]), say }), 0);
  assert.equal(calls.filter((one) => one.url.endsWith("createSession")).length, 1);
  const records = calls.filter((one) => one.url.endsWith("createRecord")).map((one) => JSON.parse(one.init.body).record);
  assert.equal(records.length, 1);
  assert.equal(records[0].text, entry().en.text);
  assert.deepEqual(records[0].langs, ["en"]);
  assert.equal(records[0].embed.images[0].alt, "The Archive");
  assert.ok(records[0].embed.images[0].aspectRatio.width > 0);
  assert.equal(records[0].facets[0].features[0].uri, "https://tisty.app");
  assert.ok(lines.some((one) => /ok Bluesky es: already posted/.test(one)));
});

test("a version with no post posts nothing anywhere", async () => {
  const calls = networksThatAnswer();
  const { lines, say } = listening();
  assert.equal(await send("9.9.1", { env: KEYS, root: home([entry()]), say }), 1);
  assert.equal(calls.length, 0);
  assert.ok(lines.some((one) => /nothing was posted for 9\.9\.1/.test(one)));
});

test("a network that fails leaves the other one to post", async () => {
  const calls = networksThatAnswer();
  const answer = globalThis.fetch;
  globalThis.fetch = async (url, init) =>
    url.endsWith("/2/media/upload") ? new Response("down", { status: 503 }) : answer(url, init);
  const { lines, say } = listening();
  assert.equal(await send("9.9.0", { env: KEYS, root: home([entry()]), say }), 1);
  assert.equal(calls.filter((one) => one.url.endsWith("createRecord")).length, 2);
  assert.ok(lines.some((one) => /::error::X es: X took no picture: 503/.test(one)));
});
