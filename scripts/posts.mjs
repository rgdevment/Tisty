import { createHmac, randomBytes } from "node:crypto";
import { readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const POSTS = "docs/posts.json";
const TONGUES = ["es", "en"];
const OPENING = { es: "¿Sabías que", en: "Did you know" };
const X_MOST = 280;
const X_LINK = 23;
const BLUESKY_MOST = 300;
const BLUESKY_PICTURE_MOST = 976_560;
const BLUESKY_LOOKED_BACK = 50;
const SHOWS_MOST = 1000;
const LINK = /https?:\/\/[^\s]+[^\s.,;:!?)»"']/g;
// X links a bare domain too, and weighs it like any other link.
const X_LINKED = /https?:\/\/[^\s]+[^\s.,;:!?)»"']|(?<![\w@/.-])(?:[a-z0-9-]+\.)+[a-z]{2,}(?:\/[^\s]*[^\s.,;:!?)»"'])?/gi;
const X_LIGHT = [
  [0, 4351],
  [8192, 8205],
  [8208, 8223],
  [8242, 8247],
];
const PICTORIAL = /\p{Extended_Pictographic}/u;
const PICTURE_TYPES = { ".webp": "image/webp", ".png": "image/png", ".jpg": "image/jpeg", ".jpeg": "image/jpeg" };

export const stable = (version) => /^\d+\.\d+\.\d+$/.test(version);

export function readPosts(root = ROOT) {
  let posts;
  try {
    posts = JSON.parse(readFileSync(join(root, POSTS), "utf8"));
  } catch (why) {
    throw new Error(`${POSTS} could not be read: ${why.message}`);
  }
  if (!Array.isArray(posts)) throw new Error(`${POSTS} is not a list of versions`);
  return posts;
}

const graphemesOf = (text) => [...new Intl.Segmenter("en", { granularity: "grapheme" }).segment(text)].map((one) => one.segment);

export const blueskyLength = (text) => graphemesOf(text).length;

const xWeight = (grapheme) =>
  PICTORIAL.test(grapheme)
    ? 2
    : [...grapheme].reduce((sum, one) => {
        const at = one.codePointAt(0);
        return sum + (X_LIGHT.some(([from, to]) => at >= from && at <= to) ? 1 : 2);
      }, 0);

// X counts as twitter-text does: a link is 23, an emoji 2, and so is any character past its light ranges.
export function xLength(text) {
  const links = text.match(X_LINKED) ?? [];
  const rest = text.replace(X_LINKED, "");
  return links.length * X_LINK + graphemesOf(rest).reduce((sum, one) => sum + xWeight(one), 0);
}

const typeOf = (path) => PICTURE_TYPES[path.slice(path.lastIndexOf(".")).toLowerCase()];

function pictureWrong(picture, root) {
  if (!picture || !typeOf(picture)) return "the picture is a .webp, .png or .jpg in the repo";
  let size;
  try {
    size = statSync(join(root, picture)).size;
  } catch {
    return `${picture} is not in the repo`;
  }
  return size > BLUESKY_PICTURE_MOST ? `${picture} weighs ${size} bytes and Bluesky takes ${BLUESKY_PICTURE_MOST}` : undefined;
}

function saidWrong(said, tongue, root) {
  if (!said?.text) return ["no text"];
  const onX = xLength(said.text);
  const onBluesky = blueskyLength(said.text);
  return [
    !said.text.startsWith(OPENING[tongue]) && `it opens with «${OPENING[tongue]}»`,
    onX > X_MOST && `${onX} characters on X, which takes ${X_MOST}`,
    onBluesky > BLUESKY_MOST && `${onBluesky} characters on Bluesky, which takes ${BLUESKY_MOST}`,
    (!said.shows || said.shows.length > SHOWS_MOST) && `«shows» says what the picture shows, in ${SHOWS_MOST} characters at most`,
    pictureWrong(said.picture, root),
  ].filter(Boolean);
}

export function judged(posts, version, root = ROOT) {
  const entry = posts.find((one) => one?.version === version);
  if (!entry) {
    return [
      `nothing in ${POSTS} tells what to post for ${version}: add a «¿Sabías que…?» in Spanish` +
        " and a «Did you know…?» in English before the tag goes out",
    ];
  }
  return TONGUES.flatMap((tongue) => saidWrong(entry[tongue], tongue, root).map((one) => `${version} ${tongue}: ${one}`));
}

const encoded = (value) =>
  encodeURIComponent(value).replace(/[!'()*]/g, (one) => `%${one.charCodeAt(0).toString(16).toUpperCase()}`);

export function oauthSignature(method, url, params, consumerSecret, tokenSecret) {
  const pairs = Object.keys(params)
    .sort()
    .map((key) => `${encoded(key)}=${encoded(params[key])}`)
    .join("&");
  const base = [method.toUpperCase(), encoded(url), encoded(pairs)].join("&");
  return createHmac("sha1", `${encoded(consumerSecret)}&${encoded(tokenSecret)}`).update(base).digest("base64");
}

// A JSON body is not signed: OAuth 1.0a signs only the query and form-encoded fields.
function oauthHeader(method, url, keys) {
  const params = {
    oauth_consumer_key: keys.apiKey,
    oauth_nonce: randomBytes(16).toString("hex"),
    oauth_signature_method: "HMAC-SHA1",
    oauth_timestamp: String(Math.floor(Date.now() / 1000)),
    oauth_token: keys.accessToken,
    oauth_version: "1.0",
  };
  params.oauth_signature = oauthSignature(method, url, params, keys.apiSecret, keys.accessSecret);
  const header = Object.keys(params)
    .sort()
    .map((key) => `${encoded(key)}="${encoded(params[key])}"`)
    .join(", ");
  return `OAuth ${header}`;
}

class Refused extends Error {
  constructor(what, status, body) {
    super(`${what}: ${status} ${body.slice(0, 300)}`);
    this.body = body;
  }
}

async function answered(response, what) {
  const body = await response.text();
  if (!response.ok) throw new Refused(what, response.status, body);
  return body ? JSON.parse(body) : {};
}

export function linkFacets(text) {
  const bytes = (part) => Buffer.byteLength(part, "utf8");
  return [...text.matchAll(LINK)].map((found) => {
    const byteStart = bytes(text.slice(0, found.index));
    return {
      index: { byteStart, byteEnd: byteStart + bytes(found[0]) },
      features: [{ $type: "app.bsky.richtext.facet#link", uri: found[0] }],
    };
  });
}

export function webpSize(bytes) {
  if (bytes.length < 30 || bytes.toString("ascii", 0, 4) !== "RIFF" || bytes.toString("ascii", 8, 12) !== "WEBP") {
    return undefined;
  }
  const chunk = bytes.toString("ascii", 12, 16);
  if (chunk === "VP8X") return { width: 1 + bytes.readUIntLE(24, 3), height: 1 + bytes.readUIntLE(27, 3) };
  if (chunk === "VP8 ") return { width: bytes.readUInt16LE(26) & 0x3fff, height: bytes.readUInt16LE(28) & 0x3fff };
  if (chunk === "VP8L") {
    const bits = bytes.readUInt32LE(21);
    return { width: 1 + (bits & 0x3fff), height: 1 + ((bits >> 14) & 0x3fff) };
  }
  return undefined;
}

// X refuses a post it already holds, so a run tried again finds it out instead of posting twice.
const alreadyOnX = (why) => why instanceof Refused && /duplicate/i.test(why.body);

async function openX(keys) {
  const call = async (url, body, what) =>
    answered(
      await fetch(url, {
        method: "POST",
        headers: { Authorization: oauthHeader("POST", url, keys), "Content-Type": "application/json" },
        body: JSON.stringify(body),
      }),
      what,
    );
  return async ({ said, picture }) => {
    const media = await call(
      "https://api.x.com/2/media/upload",
      { media: picture.toString("base64"), media_category: "tweet_image" },
      "X took no picture",
    );
    const id = media.data.id;
    await call("https://api.x.com/2/media/metadata", { id, metadata: { alt_text: { text: said.shows } } }, "X took no alt text");
    try {
      const post = await call("https://api.x.com/2/tweets", { text: said.text, media: { media_ids: [id] } }, "X took no post");
      return `https://x.com/i/status/${post.data.id}`;
    } catch (why) {
      if (alreadyOnX(why)) return "already posted";
      throw why;
    }
  };
}

async function openBluesky(keys) {
  const host = "https://bsky.social/xrpc";
  const session = await answered(
    await fetch(`${host}/com.atproto.server.createSession`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ identifier: keys.handle, password: keys.password }),
    }),
    "Bluesky let nobody in",
  );
  const signedIn = { Authorization: `Bearer ${session.accessJwt}` };
  const linkOf = (uri) => `https://bsky.app/profile/${session.did}/post/${uri.split("/").pop()}`;
  const recent = await answered(
    await fetch(
      `${host}/com.atproto.repo.listRecords?repo=${encodeURIComponent(session.did)}` +
        `&collection=app.bsky.feed.post&limit=${BLUESKY_LOOKED_BACK}`,
      { headers: signedIn },
    ),
    "Bluesky showed no earlier posts",
  );
  return async ({ said, tongue, picture, type }) => {
    const earlier = recent.records?.find((one) => one.value?.text === said.text);
    if (earlier) return `already posted, ${linkOf(earlier.uri)}`;
    const blob = await answered(
      await fetch(`${host}/com.atproto.repo.uploadBlob`, {
        method: "POST",
        headers: { ...signedIn, "Content-Type": type },
        body: picture,
      }),
      "Bluesky took no picture",
    );
    const sized = webpSize(picture);
    const record = {
      $type: "app.bsky.feed.post",
      text: said.text,
      createdAt: new Date().toISOString(),
      langs: [tongue],
      facets: linkFacets(said.text),
      embed: {
        $type: "app.bsky.embed.images",
        images: [{ alt: said.shows, image: blob.blob, ...(sized ? { aspectRatio: sized } : {}) }],
      },
    };
    const made = await answered(
      await fetch(`${host}/com.atproto.repo.createRecord`, {
        method: "POST",
        headers: { ...signedIn, "Content-Type": "application/json" },
        body: JSON.stringify({ repo: session.did, collection: "app.bsky.feed.post", record }),
      }),
      "Bluesky took no post",
    );
    return linkOf(made.uri);
  };
}

function networks(env) {
  const x = { apiKey: env.X_API_KEY, apiSecret: env.X_API_SECRET, accessToken: env.X_ACCESS_TOKEN, accessSecret: env.X_ACCESS_SECRET };
  const bluesky = { handle: env.BLUESKY_HANDLE, password: env.BLUESKY_APP_PASSWORD };
  return [
    { name: "X", keys: x, open: openX },
    { name: "Bluesky", keys: bluesky, open: openBluesky },
  ];
}

export async function send(version, { env = process.env, root = ROOT, say = console.log } = {}) {
  const posts = readPosts(root);
  const wrong = judged(posts, version, root);
  if (wrong.length > 0) {
    for (const one of wrong) say(`::error::${one}`);
    say(`::error::nothing was posted for ${version}`);
    return 1;
  }
  const entry = posts.find((one) => one.version === version);
  const going = TONGUES.map((tongue) => ({
    tongue,
    said: entry[tongue],
    picture: readFileSync(join(root, entry[tongue].picture)),
    type: typeOf(entry[tongue].picture),
  }));
  let failed = false;
  for (const network of networks(env)) {
    if (Object.values(network.keys).some((one) => !one)) {
      say(`::notice::${network.name} has no keys here, so nothing was posted there`);
      continue;
    }
    let post;
    try {
      post = await network.open(network.keys);
    } catch (why) {
      failed = true;
      say(`::error::${network.name}: ${why.message}`);
      continue;
    }
    for (const one of going) {
      try {
        say(`ok ${network.name} ${one.tongue}: ${await post(one)}`);
      } catch (why) {
        failed = true;
        say(`::error::${network.name} ${one.tongue}: ${why.message}`);
      }
    }
  }
  return failed ? 1 : 0;
}

export function check(version, { root = ROOT, say = console.log, inActions = Boolean(process.env.GITHUB_ACTIONS) } = {}) {
  if (!stable(version)) {
    say(`ok ${version} is a candidate, and a candidate posts nothing`);
    return 0;
  }
  let wrong;
  try {
    wrong = judged(readPosts(root), version, root);
  } catch (why) {
    wrong = [why.message];
  }
  for (const one of wrong) say(inActions ? `::error::${one}` : `x  ${one}`);
  if (wrong.length === 0) say(`ok ${version} has its post in ${POSTS}, in both languages`);
  return wrong.length === 0 ? 0 : 1;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [what, version] = process.argv.slice(2);
  if (!version || !["check", "send"].includes(what)) {
    console.log("usage: node scripts/posts.mjs check|send <version, without the v>");
    process.exit(2);
  }
  try {
    process.exit(what === "check" ? check(version) : await send(version));
  } catch (why) {
    console.log(`::error::${why.message}`);
    process.exit(1);
  }
}
