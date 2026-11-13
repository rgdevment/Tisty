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
const SHOWS_MOST = 1000;
const LINK = /https?:\/\/[^\s]+[^\s.,;:!?)»"']/g;
const PICTURE_TYPES = { ".webp": "image/webp", ".png": "image/png", ".jpg": "image/jpeg", ".jpeg": "image/jpeg" };

export const stable = (version) => /^\d+\.\d+\.\d+$/.test(version);

export function readPosts(root = ROOT) {
  return JSON.parse(readFileSync(join(root, POSTS), "utf8"));
}

const graphemes = (text) => [...new Intl.Segmenter("en", { granularity: "grapheme" }).segment(text)].length;

// X weighs every link as 23 characters, whatever its length.
export const xLength = (text) => [...text.replace(LINK, "x".repeat(X_LINK))].length;

export const blueskyLength = graphemes;

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
  if (!Array.isArray(posts)) return [`${POSTS} is not a list of versions`];
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
  return `OAuth ${Object.keys(params).sort().map((key) => `${encoded(key)}="${encoded(params[key])}"`).join(", ")}`;
}

async function answered(response, what) {
  const body = await response.text();
  if (!response.ok) throw new Error(`${what}: ${response.status} ${body.slice(0, 300)}`);
  return body ? JSON.parse(body) : {};
}

async function toX(said, picture, keys) {
  const call = async (url, body, what) =>
    answered(
      await fetch(url, {
        method: "POST",
        headers: { Authorization: oauthHeader("POST", url, keys), "Content-Type": "application/json" },
        body: JSON.stringify(body),
      }),
      what,
    );
  const media = await call(
    "https://api.x.com/2/media/upload",
    { media: picture.toString("base64"), media_category: "tweet_image" },
    "X took no picture",
  );
  const id = media.data.id;
  await call("https://api.x.com/2/media/metadata", { id, metadata: { alt_text: { text: said.shows } } }, "X took no alt text");
  const post = await call("https://api.x.com/2/tweets", { text: said.text, media: { media_ids: [id] } }, "X took no post");
  return `https://x.com/i/status/${post.data.id}`;
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
  if (bytes.toString("ascii", 0, 4) !== "RIFF" || bytes.toString("ascii", 8, 12) !== "WEBP") return undefined;
  const chunk = bytes.toString("ascii", 12, 16);
  if (chunk === "VP8X") return { width: 1 + bytes.readUIntLE(24, 3), height: 1 + bytes.readUIntLE(27, 3) };
  if (chunk === "VP8 ") return { width: bytes.readUInt16LE(26) & 0x3fff, height: bytes.readUInt16LE(28) & 0x3fff };
  if (chunk === "VP8L") {
    const bits = bytes.readUInt32LE(21);
    return { width: 1 + (bits & 0x3fff), height: 1 + ((bits >> 14) & 0x3fff) };
  }
  return undefined;
}

async function toBluesky(said, tongue, picture, type, keys) {
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
  return `https://bsky.app/profile/${session.did}/post/${made.uri.split("/").pop()}`;
}

function networks(env) {
  const x = {
    apiKey: env.X_API_KEY,
    apiSecret: env.X_API_SECRET,
    accessToken: env.X_ACCESS_TOKEN,
    accessSecret: env.X_ACCESS_SECRET,
  };
  const bluesky = { handle: env.BLUESKY_HANDLE, password: env.BLUESKY_APP_PASSWORD };
  return [
    { name: "X", keys: x, send: (said, tongue, picture) => toX(said, picture, x) },
    { name: "Bluesky", keys: bluesky, send: (said, tongue, picture, type) => toBluesky(said, tongue, picture, type, bluesky) },
  ];
}

async function send(version) {
  const entry = readPosts().find((one) => one.version === version);
  if (!entry) {
    console.log(`::error::nothing in ${POSTS} for ${version}, so nothing was posted`);
    return 1;
  }
  let failed = false;
  for (const network of networks(process.env)) {
    if (Object.values(network.keys).some((one) => !one)) {
      console.log(`::notice::${network.name} has no keys here, so nothing was posted there`);
      continue;
    }
    for (const tongue of TONGUES) {
      const said = entry[tongue];
      try {
        const link = await network.send(said, tongue, readFileSync(join(ROOT, said.picture)), typeOf(said.picture));
        console.log(`ok ${network.name} ${tongue}: ${link}`);
      } catch (why) {
        failed = true;
        console.log(`::error::${network.name} ${tongue}: ${why.message}`);
      }
    }
  }
  return failed ? 1 : 0;
}

function check(version) {
  if (!stable(version)) {
    console.log(`ok ${version} is a candidate, and a candidate posts nothing`);
    return 0;
  }
  const wrong = judged(readPosts(), version);
  for (const one of wrong) console.log(process.env.GITHUB_ACTIONS ? `::error::${one}` : `x  ${one}`);
  if (wrong.length === 0) console.log(`ok ${version} has its post in ${POSTS}, in both languages`);
  return wrong.length === 0 ? 0 : 1;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [what, version] = process.argv.slice(2);
  if (!version || !["check", "send"].includes(what)) {
    console.log("usage: node scripts/posts.mjs check|send <version, without the v>");
    process.exit(2);
  }
  process.exit(what === "check" ? check(version) : await send(version));
}
