import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { blueskyLength, judged, linkFacets, oauthSignature, readPosts, webpSize, xLength } from "./posts.mjs";

const picture = "assets/screens/readme/en-01-archive.webp";
const said = (tongue, text) => ({ text, picture, shows: "The Archive" });
const entry = (more = {}) => ({
  version: "9.9.0",
  es: said("es", "¿Sabías que Tisty guarda lo que terminas? https://tisty.app"),
  en: said("en", "Did you know Tisty keeps what you finish? https://tisty.app"),
  ...more,
});

test("every post in the file holds for its own version", () => {
  for (const one of readPosts()) assert.deepEqual(judged(readPosts(), one.version), [], one.version);
});

test("a version with no post is told what to add", () => {
  assert.match(judged([entry()], "9.9.1")[0], /nothing in docs\/posts\.json tells what to post for 9\.9\.1/);
});

test("each language opens with its own question", () => {
  assert.deepEqual(judged([entry()], "9.9.0"), []);
  const wrong = judged([entry({ en: said("en", "Tisty keeps what you finish") })], "9.9.0");
  assert.match(wrong.join(), /en: it opens with «Did you know»/);
});

test("both languages are there", () => {
  assert.match(judged([entry({ es: undefined })], "9.9.0").join(), /9.9.0 es: no text/);
});

test("a picture outside the repo or of another kind is refused", () => {
  const missing = { ...said("en", "Did you know"), picture: "assets/nowhere.webp" };
  assert.match(judged([entry({ en: missing })], "9.9.0").join(), /not in the repo/);
  const gif = { ...said("en", "Did you know"), picture: "assets/a.gif" };
  assert.match(judged([entry({ en: gif })], "9.9.0").join(), /\.webp, \.png or \.jpg/);
});

test("X weighs a link as 23 characters, and a text past 280 is refused", () => {
  assert.equal(xLength(`Did you know https://tisty.app/${"a".repeat(100)}`), "Did you know ".length + 23);
  const long = said("en", `Did you know ${"a".repeat(280)}`);
  assert.match(judged([entry({ en: long })], "9.9.0").join(), /on X, which takes 280/);
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
  assert.equal(facet.features[0].uri, "https://tisty.app");
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

test("a capture's size is read from its WebP header", () => {
  const size = webpSize(readFileSync(new URL(`../${picture}`, import.meta.url)));
  assert.ok(size.width > size.height && size.height > 0);
  assert.equal(webpSize(Buffer.from("not a picture at all")), undefined);
});
