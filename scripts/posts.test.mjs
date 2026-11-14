import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, test } from "node:test";
import {
  blueskyLength,
  check,
  forX,
  judged,
  linkFacets,
  newer,
  oauthSignature,
  pick,
  preview,
  readPosts,
  send,
  turnOf,
  webpSize,
  xLength,
} from "./posts.mjs";

const picture = "assets/screens/readme/en-01-archive.webp";
const pictured = (text) => ({ text, picture, shows: "The Archive" });
const post = (id, more = {}) => ({ id, en: pictured(`Tisty keeps what you finish (${id}). https://tisty.app`), ...more });
const plain = (id) => ({ id, en: { text: "No picture here" } });
const file = (more = {}) => ({ since: "9.0.0", queue: [post("first"), post("second")], ...more });
const RELEASED = ["9.0.0", "9.0.1", "9.1.0-rc.1"];
const released = () => RELEASED;
const DAY_MS = 24 * 60 * 60 * 1000;

const KEYS = {
  X_API_KEY: "k",
  X_API_SECRET: "s",
  X_ACCESS_TOKEN: "t",
  X_ACCESS_SECRET: "a",
  BLUESKY_HANDLE: "tisty.app",
  BLUESKY_APP_PASSWORD: "p",
};
const ONLY_X = { ...KEYS, BLUESKY_HANDLE: "" };
const ONLY_BLUESKY = { ...KEYS, X_API_KEY: "" };

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
  let made = 0;
  globalThis.fetch = async (url, init = {}) => {
    calls.push({ url, init, body: typeof init.body === "string" ? JSON.parse(init.body) : undefined });
    const json = (status, body) => new Response(JSON.stringify(body), { status });
    if (url.endsWith("/2/media/upload")) return json(200, { data: { id: "77" } });
    if (url.endsWith("/2/media/metadata")) return json(200, { data: { id: "77" } });
    if (url.endsWith("/2/tweets")) {
      return xDuplicate
        ? json(403, { detail: "You are not allowed to create a Tweet with duplicate content." })
        : json(201, { data: { id: `10${++made}` } });
    }
    if (url.endsWith("createSession")) return json(200, { accessJwt: "jwt", did: "did:plc:tisty" });
    if (url.includes("getAuthorFeed")) return json(200, { feed: earlier.map((one) => ({ post: one })) });
    if (url.endsWith("uploadBlob")) return json(200, { blob: { $type: "blob", ref: "b" } });
    if (url.endsWith("createRecord")) return json(200, { uri: `at://did:plc:tisty/app.bsky.feed.post/3a${++made}`, cid: `c${made}` });
    return json(404, {});
  };
  return calls;
}

const listening = () => {
  const lines = [];
  return { lines, say: (line) => lines.push(line) };
};
const made = (calls, ending) => calls.filter((one) => one.url.endsWith(ending)).map((one) => one.body);

test("the posts in the repo hold, and the queue has a next one", () => {
  const posts = readPosts();
  assert.deepEqual(judged(posts), []);
  assert.ok(posts.queue.length > 0);
});

test("versions compare by number, not by text", () => {
  assert.ok(newer("1.10.0", "1.9.9"));
  assert.ok(newer("1.47.1", "1.47.0"));
  assert.ok(!newer("1.47.0", "1.47.0"));
});

test("each stable release since «since» takes the next post in the queue", () => {
  assert.equal(turnOf("9.0.0", RELEASED, "9.0.0"), 0);
  assert.equal(turnOf("9.0.1", RELEASED, "9.0.0"), 1);
  assert.equal(turnOf("9.1.0", RELEASED, "9.0.0"), 2);
  const posts = file();
  assert.equal(pick(posts, "9.0.1", RELEASED).post.id, "first");
  assert.equal(pick(posts, "9.1.0", RELEASED).post.id, "second");
  assert.match(pick(posts, "9.2.0", [...RELEASED, "9.1.0"]).wrong, /release 3 since 9\.0\.0 and the queue .* holds 2/);
  assert.match(pick(posts, "9.1.0-rc.2", RELEASED).skip, /candidate/);
  assert.match(pick(posts, "8.9.0", RELEASED).skip, /not past 9\.0\.0/);
  assert.equal(pick(file({ pinned: post("pin") }), "pinned", RELEASED).post.id, "pin");
  assert.equal(pick(posts, "second", RELEASED).post.id, "second");
});

test("a post needs an id of its own, and its English text", () => {
  assert.match(judged(file({ queue: [post("same"), post("same")] })).join(), /same is in the queue twice/);
  assert.match(judged(file({ queue: [{ en: pictured("b") }] })).join(), /every post has an id/);
  assert.match(judged(file({ queue: [post("blank", { en: undefined })] })).join(), /blank en: no text/);
});

test("a picture is optional, but one that is named has to be there with its words", () => {
  assert.deepEqual(judged(file({ queue: [plain("plain")] })), []);
  const missing = { ...pictured("a"), picture: "assets/nowhere.webp" };
  assert.match(judged(file({ queue: [post("p", { en: missing })] })).join(), /not in the repo/);
  const gif = { ...pictured("a"), picture: "assets/a.gif" };
  assert.match(judged(file({ queue: [post("p", { en: gif })] })).join(), /\.webp, \.png or \.jpg/);
  assert.match(judged(file({ queue: [post("p", { en: { text: "a", picture } })] })).join(), /«shows» says what the picture shows/);
});

test("a post without a picture passes the check with a word that one is recommended", () => {
  const { lines, say } = listening();
  assert.equal(check("9.0.1", { root: home(file({ queue: [plain("plain")] })), say, released }), 0);
  assert.ok(lines.some((one) => /plain goes without a picture: one is recommended/.test(one)));
});

test("X gets the text without the link at its end, and nothing else is taken out", () => {
  assert.equal(forX("Tisty keeps what you finish. https://tisty.app"), "Tisty keeps what you finish.");
  assert.equal(forX("Node.js and NOTES.md stay?\n\nhttps://tisty.app"), "Node.js and NOTES.md stay?");
  assert.equal(xLength(forX("Is it? https://tisty.app")), "Is it?".length);
});

test("a link anywhere but the end, or a domain without its scheme, is refused", () => {
  const middle = post("m", { en: pictured("Try https://tisty.app today") });
  assert.match(judged(file({ queue: [middle] })).join(), /https:\/\/tisty\.app goes at the end/);
  const bare = post("b", { en: pictured("Try tisty.app today") });
  assert.match(judged(file({ queue: [bare] })).join(), /tisty\.app needs its https:\/\//);
  assert.deepEqual(judged(file({ queue: [post("ok", { en: pictured("winget install rgdevment.Tisty, NOTES.md") })] })), []);
});

test("a reply keeps its link on X, and counts it as 23", () => {
  const reply = { en: { text: "Try it:\n\nhttps://www.tisty.app" } };
  assert.deepEqual(judged(file({ pinned: post("pin", { reply }) })), []);
  assert.equal(xLength("Try it:\n\nhttps://www.tisty.app"), "Try it:\n\n".length + 23);
});

test("X weighs an emoji, an ellipsis and a CJK character as 2, and accents as 1", () => {
  assert.equal(xLength("¿Sabías?"), 8);
  assert.equal(xLength("👍🏽"), 2);
  assert.equal(xLength("…"), 2);
  assert.equal(xLength("漢"), 2);
  assert.equal(xLength("—"), 1);
  const heavy = post("heavy", { en: pictured("…".repeat(141)) });
  assert.match(judged(file({ queue: [heavy] })).join(), /on X, which takes 280/);
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

test("a broken or old-shaped file is told in one line, not a stack", () => {
  const { lines, say } = listening();
  assert.equal(check("9.0.1", { root: home("[{,]"), say, released }), 1);
  assert.match(lines[0], /^x {2}docs\/posts\.json could not be read/);
  assert.equal(check("9.0.1", { root: home([{ version: "9.0.1" }]), say, released }), 1);
  assert.match(lines[1], /holds «since»/);
});

test("the post goes to X with its picture and alt text, signed, and without the link", async () => {
  const calls = networksThatAnswer();
  const { lines, say } = listening();
  assert.equal(await send("9.0.1", { env: ONLY_X, root: home(file()), say, released }), 0);
  const [tweet] = made(calls, "/2/tweets");
  assert.equal(tweet.text, "Tisty keeps what you finish (first).");
  assert.deepEqual(tweet.media, { media_ids: ["77"] });
  assert.deepEqual(made(calls, "/2/media/metadata")[0], { id: "77", metadata: { alt_text: { text: "The Archive" } } });
  assert.ok(calls.every((one) => one.init.headers.Authorization.startsWith("OAuth ")));
  assert.ok(lines.some((one) => /Bluesky has no keys here/.test(one)));
});

test("a post without a picture goes to X with no upload at all", async () => {
  const calls = networksThatAnswer();
  assert.equal(await send("9.0.1", { env: ONLY_X, root: home(file({ queue: [plain("plain")] })), say: () => {}, released }), 0);
  assert.equal(calls.filter((one) => one.url.includes("/2/media/")).length, 0);
  assert.equal(made(calls, "/2/tweets")[0].media, undefined);
});

test("the pinned post goes out with its reply under it, on X and on Bluesky", async () => {
  const calls = networksThatAnswer();
  const reply = { en: { text: "Try it: https://www.tisty.app" } };
  const { lines, say } = listening();
  assert.equal(await send("pinned", { env: KEYS, root: home(file({ pinned: post("pin", { reply }) })), say, released }), 0);
  const tweets = made(calls, "/2/tweets");
  assert.equal(tweets.length, 2);
  assert.equal(tweets[1].text, "Try it: https://www.tisty.app");
  assert.deepEqual(tweets[1].reply, { in_reply_to_tweet_id: "101" });
  const records = made(calls, "createRecord").map((one) => one.record);
  assert.equal(records.length, 2);
  assert.deepEqual(records[1].reply.parent, records[1].reply.root);
  assert.equal(records[1].embed, undefined);
  assert.ok(lines.some((one) => /ok X en reply:/.test(one)));
});

test("X refusing a post it already holds counts as posted", async () => {
  networksThatAnswer({ xDuplicate: true });
  const { lines, say } = listening();
  assert.equal(await send("9.0.1", { env: ONLY_X, root: home(file()), say, released }), 0);
  assert.equal(lines.filter((one) => one.includes("already posted")).length, 1);
});

test("Bluesky skips a post made days ago, and posts again one told months ago", async () => {
  const told = (daysAgo) => [
    {
      uri: "at://did:plc:tisty/app.bsky.feed.post/old",
      cid: "c0",
      record: { text: post("first").en.text, createdAt: new Date(Date.now() - daysAgo * DAY_MS).toISOString() },
    },
  ];
  const lately = listening();
  const lastWeek = networksThatAnswer({ earlier: told(2) });
  assert.equal(await send("9.0.1", { env: ONLY_BLUESKY, root: home(file()), say: lately.say, released }), 0);
  assert.equal(made(lastWeek, "createRecord").length, 0);
  assert.ok(lately.lines.some((one) => /ok Bluesky en: already posted/.test(one)));

  const calls = networksThatAnswer({ earlier: told(90) });
  assert.equal(await send("9.0.1", { env: ONLY_BLUESKY, root: home(file()), say: () => {}, released }), 0);
  assert.equal(made(calls, "createSession").length, 1);
  const [record] = made(calls, "createRecord").map((one) => one.record);
  assert.equal(record.text, post("first").en.text);
  assert.deepEqual(record.langs, ["en"]);
  assert.equal(record.embed.images[0].alt, "The Archive");
  assert.ok(record.embed.images[0].aspectRatio.width > 0);
  assert.equal(record.facets[0].features[0].uri, "https://tisty.app");
});

test("a release with nothing left in the queue posts nothing anywhere", async () => {
  const calls = networksThatAnswer();
  const { lines, say } = listening();
  const later = () => [...RELEASED, "9.1.0"];
  assert.equal(await send("9.2.0", { env: KEYS, root: home(file()), say, released: later }), 1);
  assert.equal(calls.length, 0);
  assert.ok(lines.some((one) => /nothing was posted for 9\.2\.0/.test(one)));
});

test("a network that fails leaves the other one to post", async () => {
  const calls = networksThatAnswer();
  const answer = globalThis.fetch;
  globalThis.fetch = async (url, init) =>
    url.endsWith("/2/media/upload") ? new Response("down", { status: 503 }) : answer(url, init);
  const { lines, say } = listening();
  assert.equal(await send("9.0.1", { env: KEYS, root: home(file()), say, released }), 1);
  assert.equal(made(calls, "createRecord").length, 1);
  assert.ok(lines.some((one) => /::error::X en: X took no picture: 503/.test(one)));
});

test("a preview shows what each network gets, and posts nothing", () => {
  const calls = networksThatAnswer();
  const { lines, say } = listening();
  assert.equal(preview("9.0.1", { root: home(file()), say, released }), 0);
  const shown = lines.join("\n");
  assert.match(shown, /9\.0\.1 posts first, number 1 in the queue/);
  assert.match(shown, /X, \d+\/280:\n {4}Tisty keeps what you finish \(first\)\.\n/);
  assert.match(shown, /Bluesky, \d+\/300:\n {4}Tisty keeps what you finish \(first\)\. https:\/\/tisty\.app/);
  assert.equal(calls.length, 0);
});
