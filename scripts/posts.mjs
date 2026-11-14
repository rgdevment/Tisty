import { execFileSync } from "node:child_process";
import { createHmac, randomBytes } from "node:crypto";
import { readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const POSTS = "docs/posts.json";
// X and Bluesky translate a post for whoever reads it, so English alone reaches everyone.
const TONGUES = ["en"];
const PINNED = "pinned";
const X_MOST = 280;
const X_LINK = 23;
const BLUESKY_MOST = 300;
const BLUESKY_PICTURE_MOST = 976_560;
const BLUESKY_LOOKED_BACK = 50;
const RERUN_WITHIN_MS = 7 * 24 * 60 * 60 * 1000;
const SHOWS_MOST = 1000;
const LINK = /https?:\/\/[^\s]+[^\s.,;:!?)»"']/g;
const LAST_LINK = /\s*https?:\/\/\S+$/;
// X turns a bare domain into a link it charges for, so the text may only name one with its scheme, at the end.
const BARE_DOMAIN = /(?<![\w@/.:-])(?:[a-z0-9-]+\.)+(?:app|com|org|net|io|dev|ai|co|cl)\b/gi;
const X_LIGHT = [
  [0, 4351],
  [8192, 8205],
  [8208, 8223],
  [8242, 8247],
];
const PICTORIAL = /\p{Extended_Pictographic}/u;
const PICTURE_TYPES = { ".webp": "image/webp", ".png": "image/png", ".jpg": "image/jpeg", ".jpeg": "image/jpeg" };

export const stable = (version) => /^\d+\.\d+\.\d+$/.test(version);

const parts = (version) => version.split(".").map(Number);

export function newer(one, other) {
  const [a, b] = [parts(one), parts(other)];
  for (let at = 0; at < 3; at++) if (a[at] !== b[at]) return a[at] > b[at];
  return false;
}

export function readPosts(root = ROOT) {
  let posts;
  try {
    posts = JSON.parse(readFileSync(join(root, POSTS), "utf8"));
  } catch (why) {
    throw new Error(`${POSTS} could not be read: ${why.message}`);
  }
  if (!posts || !stable(posts.since ?? "") || !Array.isArray(posts.queue)) {
    throw new Error(`${POSTS} holds «since», the version the queue starts after, and «queue», the posts in order`);
  }
  return posts;
}

export function releasedVersions(root = ROOT) {
  const listed = execFileSync("git", ["ls-remote", "--tags", "origin", "v*"], { cwd: root, encoding: "utf8" });
  return [...listed.matchAll(/refs\/tags\/v(\d+\.\d+\.\d+)$/gm)].map((found) => found[1]);
}

// The queue keeps no record of what went out: the stable releases since «since» say whose turn it is.
export function turnOf(version, released, since) {
  if (!newer(version, since)) return 0;
  return 1 + new Set(released.filter((one) => stable(one) && newer(one, since) && newer(version, one))).size;
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

export const xLength = (text) => {
  const links = text.match(LINK) ?? [];
  return links.length * X_LINK + graphemesOf(text.replace(LINK, "")).reduce((sum, one) => sum + xWeight(one), 0);
};

// A post with a link costs X's API about thirteen times more, and tisty.app is already on the profile.
export const forX = (text) => text.replace(LAST_LINK, "");

const typeOf = (path) => PICTURE_TYPES[path.slice(path.lastIndexOf(".")).toLowerCase()];

function pictureWrong(picture, root) {
  if (!typeOf(picture)) return "the picture is a .webp, .png or .jpg in the repo";
  let size;
  try {
    size = statSync(join(root, picture)).size;
  } catch {
    return `${picture} is not in the repo`;
  }
  return size > BLUESKY_PICTURE_MOST ? `${picture} weighs ${size} bytes and Bluesky takes ${BLUESKY_PICTURE_MOST}` : undefined;
}

function saidWrong(said, root, { linkStays = false } = {}) {
  if (!said?.text) return ["no text"];
  const onX = linkStays ? said.text : forX(said.text);
  const xCount = xLength(onX);
  const blueskyCount = blueskyLength(said.text);
  const inside = linkStays ? [] : (onX.match(LINK) ?? []);
  const bare = said.text.replace(LINK, "").match(BARE_DOMAIN) ?? [];
  return [
    inside.length > 0 && `${inside.join(", ")} goes at the end, the one place X leaves a link out`,
    bare.length > 0 && `${bare.join(", ")} needs its https:// and the end of the text, or X makes it a paid link`,
    xCount > X_MOST && `${xCount} characters on X, which takes ${X_MOST}`,
    blueskyCount > BLUESKY_MOST && `${blueskyCount} characters on Bluesky, which takes ${BLUESKY_MOST}`,
    said.picture && pictureWrong(said.picture, root),
    said.picture && (!said.shows || said.shows.length > SHOWS_MOST) && `«shows» says what the picture shows, in ${SHOWS_MOST} characters at most`,
  ].filter(Boolean);
}

function postWrong(post, root) {
  const name = post?.id ?? "a post with no id";
  const wrong = TONGUES.flatMap((tongue) => saidWrong(post?.[tongue], root).map((one) => `${name} ${tongue}: ${one}`));
  const reply = post?.reply
    ? TONGUES.flatMap((tongue) => saidWrong(post.reply[tongue], root, { linkStays: true }).map((one) => `${name} reply ${tongue}: ${one}`))
    : [];
  return [...(post?.id ? [] : [`${name}: every post has an id`]), ...wrong, ...reply];
}

export function judged(posts, root = ROOT) {
  const ids = posts.queue.map((one) => one?.id);
  const twice = ids.filter((one, at) => one && ids.indexOf(one) !== at);
  return [
    ...(posts.pinned ? postWrong(posts.pinned, root) : []),
    ...posts.queue.flatMap((one) => postWrong(one, root)),
    ...twice.map((one) => `${one} is in the queue twice`),
  ];
}

export const withoutPicture = (post) => (TONGUES.some((tongue) => !post[tongue]?.picture) ? post.id : undefined);

export function pick(posts, what, released) {
  if (what === PINNED) {
    if (!posts.pinned) return { wrong: `${POSTS} has no pinned post` };
    return { post: posts.pinned, said: `the pinned post, ${posts.pinned.id}` };
  }
  const named = posts.queue.find((one) => one?.id === what);
  if (named) return { post: named, said: `${what}, asked for by name` };
  if (!stable(what)) return { skip: `${what} is a candidate, and a candidate posts nothing` };
  const turn = turnOf(what, released, posts.since);
  if (turn === 0) return { skip: `${what} is not past ${posts.since}, where the queue starts` };
  const post = posts.queue[turn - 1];
  if (!post) {
    return {
      wrong:
        `${what} is release ${turn} since ${posts.since} and the queue in ${POSTS} holds ${posts.queue.length}:` +
        " add the next post at its end before the tag goes out",
    };
  }
  return { post, said: `${what} posts ${post.id}, number ${turn} in the queue` };
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
    this.status = status;
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

const X_KEYS_HINT =
  "X_API_KEY and X_API_SECRET are the Consumer Key and its secret under «OAuth 1.0 Keys», not the OAuth 2.0 Client ID," +
  " and X_ACCESS_TOKEN with X_ACCESS_SECRET were generated after the app was set to Read and write";

// A 401 says nothing of which half failed, so one look at who the keys belong to tells the keys from the endpoint.
async function whyRefused(keys, why) {
  if (!(why instanceof Refused) || why.status !== 401) return why;
  const url = "https://api.x.com/2/users/me";
  const me = await fetch(url, { headers: { Authorization: oauthHeader("GET", url, keys) } });
  const said = me.ok
    ? `these keys sign in as ${(await me.json()).data?.username}, so it is this endpoint that refuses them`
    : `X does not take these keys at all: ${X_KEYS_HINT}`;
  return new Error(`${why.message.split(":")[0]}: ${said}`);
}

async function openX(keys) {
  const call = async (url, body, what) =>
    answered(
      await fetch(url, {
        method: "POST",
        headers: { Authorization: oauthHeader("POST", url, keys), "Content-Type": "application/json" },
        body: JSON.stringify(body),
      }),
      what,
    ).catch(async (why) => {
      throw await whyRefused(keys, why);
    });
  return async ({ text, said, picture, under }) => {
    const post = { text };
    if (picture) {
      const media = await call(
        "https://api.x.com/2/media/upload",
        { media: picture.toString("base64"), media_category: "tweet_image" },
        "X took no picture",
      );
      await call("https://api.x.com/2/media/metadata", { id: media.data.id, metadata: { alt_text: { text: said.shows } } }, "X took no alt text");
      post.media = { media_ids: [media.data.id] };
    }
    if (under) post.reply = { in_reply_to_tweet_id: under };
    try {
      const made = await call("https://api.x.com/2/tweets", post, "X took no post");
      return { link: `https://x.com/i/status/${made.data.id}`, ref: made.data.id };
    } catch (why) {
      if (alreadyOnX(why)) return { link: "already posted" };
      throw why;
    }
  };
}

async function openBluesky(keys, say) {
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
  let recent;
  // The public AppView answers a feed without the PDS proxying it, which is what failed upstream.
  const feed = async () => {
    const url = `https://public.api.bsky.app/xrpc/app.bsky.feed.getAuthorFeed?actor=${encodeURIComponent(session.did)}&limit=${BLUESKY_LOOKED_BACK}`;
    for (let tried = 1; ; tried++) {
      try {
        return (await answered(await fetch(url), "Bluesky showed no earlier posts")).feed ?? [];
      } catch (why) {
        if (tried >= 2) {
          say(`::warning::${why.message}, so this goes out without looking for a twin`);
          return [];
        }
      }
    }
  };
  const lately = async () => {
    recent ??= feed();
    const since = Date.now() - RERUN_WITHIN_MS;
    return (await recent).map((one) => one.post).filter((one) => Date.parse(one?.record?.createdAt) >= since);
  };
  return async ({ text, said, tongue, picture, type, under }) => {
    const earlier = (await lately()).find((one) => one.record.text === text);
    if (earlier) return { link: `already posted, ${linkOf(earlier.uri)}`, ref: { uri: earlier.uri, cid: earlier.cid } };
    const record = {
      $type: "app.bsky.feed.post",
      text,
      createdAt: new Date().toISOString(),
      langs: [tongue],
      facets: linkFacets(text),
    };
    if (picture) {
      const blob = await answered(
        await fetch(`${host}/com.atproto.repo.uploadBlob`, {
          method: "POST",
          headers: { ...signedIn, "Content-Type": type },
          body: picture,
        }),
        "Bluesky took no picture",
      );
      const sized = webpSize(picture);
      record.embed = {
        $type: "app.bsky.embed.images",
        images: [{ alt: said.shows, image: blob.blob, ...(sized ? { aspectRatio: sized } : {}) }],
      };
    }
    if (under) record.reply = { root: under, parent: under };
    const made = await answered(
      await fetch(`${host}/com.atproto.repo.createRecord`, {
        method: "POST",
        headers: { ...signedIn, "Content-Type": "application/json" },
        body: JSON.stringify({ repo: session.did, collection: "app.bsky.feed.post", record }),
      }),
      "Bluesky took no post",
    );
    return { link: linkOf(made.uri), ref: { uri: made.uri, cid: made.cid } };
  };
}

function networks(env) {
  const x = { apiKey: env.X_API_KEY, apiSecret: env.X_API_SECRET, accessToken: env.X_ACCESS_TOKEN, accessSecret: env.X_ACCESS_SECRET };
  const bluesky = { handle: env.BLUESKY_HANDLE, password: env.BLUESKY_APP_PASSWORD };
  return [
    { name: "X", keys: x, open: openX, shaped: forX },
    { name: "Bluesky", keys: bluesky, open: openBluesky, shaped: (text) => text },
  ];
}

function going(post, root) {
  return TONGUES.map((tongue) => {
    const said = post[tongue];
    return {
      tongue,
      said,
      picture: said.picture ? readFileSync(join(root, said.picture)) : undefined,
      type: said.picture ? typeOf(said.picture) : undefined,
      reply: post.reply?.[tongue],
    };
  });
}

function opened(root, what, released, say) {
  const posts = readPosts(root);
  const wrong = judged(posts, root);
  const chosen = wrong.length > 0 ? { wrong: wrong.join("\n") } : pick(posts, what, released());
  if (chosen.wrong) for (const one of chosen.wrong.split("\n")) say(`::error::${one}`);
  if (chosen.skip) say(`ok ${chosen.skip}`);
  return chosen;
}

export async function send(what, { env = process.env, root = ROOT, say = console.log, released = () => releasedVersions(root) } = {}) {
  const chosen = opened(root, what, released, say);
  if (chosen.wrong) {
    say(`::error::nothing was posted for ${what}`);
    return 1;
  }
  if (chosen.skip) return 0;
  say(`ok ${chosen.said}`);
  let failed = false;
  for (const network of networks(env)) {
    if (Object.values(network.keys).some((one) => !one)) {
      say(`::notice::${network.name} has no keys here, so nothing was posted there`);
      continue;
    }
    let post;
    try {
      post = await network.open(network.keys, say);
    } catch (why) {
      failed = true;
      say(`::error::${network.name}: ${why.message}`);
      continue;
    }
    for (const one of going(chosen.post, root)) {
      try {
        const made = await post({ ...one, text: network.shaped(one.said.text) });
        say(`ok ${network.name} ${one.tongue}: ${made.link}`);
        if (!one.reply) continue;
        if (!made.ref) throw new Error("the post was already there and its id is unknown here, so post the reply by hand");
        const answer = await post({ tongue: one.tongue, text: one.reply.text, under: made.ref });
        say(`ok ${network.name} ${one.tongue} reply: ${answer.link}`);
      } catch (why) {
        failed = true;
        say(`::error::${network.name} ${one.tongue}: ${why.message}`);
      }
    }
  }
  return failed ? 1 : 0;
}

export function check(what, { root = ROOT, say = console.log, released = () => releasedVersions(root) } = {}) {
  let chosen;
  try {
    chosen = opened(root, what, released, (line) => say(line.replace("::error::", "x  ")));
  } catch (why) {
    say(`x  ${why.message}`);
    return 1;
  }
  if (chosen.wrong) return 1;
  if (chosen.post) {
    say(`ok ${chosen.said}`);
    const bare = withoutPicture(chosen.post);
    if (bare) say(`!  ${bare} goes without a picture: one is recommended`);
  }
  return 0;
}

export function preview(what, { root = ROOT, say = console.log, released = () => releasedVersions(root) } = {}) {
  const posts = readPosts(root);
  const chosen = pick(posts, what, released());
  if (!chosen.post) {
    say(chosen.wrong ? `x  ${chosen.wrong}` : `ok ${chosen.skip}`);
    return chosen.wrong ? 1 : 0;
  }
  say(chosen.said);
  for (const tongue of TONGUES) {
    const said = chosen.post[tongue];
    say(`\n${tongue} · ${said.picture ? `picture ${said.picture}\n  alt: ${said.shows}` : "no picture"}`);
    say(`  X, ${xLength(forX(said.text))}/${X_MOST}:\n${forX(said.text).replace(/^/gm, "    ")}`);
    say(`  Bluesky, ${blueskyLength(said.text)}/${BLUESKY_MOST}:\n${said.text.replace(/^/gm, "    ")}`);
    const reply = chosen.post.reply?.[tongue];
    if (reply) say(`  reply, ${xLength(reply.text)}/${X_MOST} on X:\n${reply.text.replace(/^/gm, "    ")}`);
  }
  return check(what, { root, say: (line) => !line.startsWith("ok ") && say(line), released });
}

const COMMANDS = { check, preview, send };

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [what, version] = process.argv.slice(2);
  if (!version || !COMMANDS[what]) {
    console.log("usage: node scripts/posts.mjs check|preview|send <version without the v, pinned, or a post id>");
    process.exit(2);
  }
  try {
    process.exit(await COMMANDS[what](version));
  } catch (why) {
    console.log(`::error::${why.message}`);
    process.exit(1);
  }
}
