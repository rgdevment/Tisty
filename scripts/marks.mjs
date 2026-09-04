import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const TEST = "https://unicode.org/Public/emoji/15.0/emoji-test.txt";
const CLDR = "https://cdn.jsdelivr.net/npm";
const NOTES = [
  ["cldr-annotations-modern@45.0.0", "annotations"],
  ["cldr-annotations-derived-modern@45.0.0", "annotationsDerived"],
];

const FAMILIES = new Map([
  ["Smileys & Emotion", "faces"],
  ["People & Body", "folk"],
  ["Animals & Nature", "plants"],
  ["Food & Drink", "food"],
  ["Travel & Places", "travel"],
  ["Activities", "fun"],
  ["Objects", "stuff"],
  ["Symbols", "signs"],
  ["Flags", "flags"],
]);

const EXTRA = new Map([
  ["✅", ["hecho", "listo", "ok", "done"]],
  ["❌", ["error", "fallo", "mal", "fail"]],
  ["⚠️", ["aviso", "alerta", "warning"]],
  ["\u{1F5D1}️", ["basura", "borrar", "trash"]],
  ["\u{1F697}", ["coche", "carro"]],
]);

const fetched = async (url) => {
  const got = await fetch(url);
  if (!got.ok) throw new Error(`${got.status} ${url}`);
  return got;
};

const noted = async (lang) => {
  const all = new Map();
  for (const [pkg, field] of NOTES) {
    const said = await (await fetched(`${CLDR}/${pkg}/${field}/${lang}/annotations.json`)).json();
    for (const [key, one] of Object.entries(said[field].annotations)) {
      all.set(key, [...(all.get(key) ?? []), ...(one.tts ?? []), ...(one.default ?? [])]);
    }
  }
  return all;
};

const plain = (said) =>
  said
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .trim();

const regional = (cp) => cp >= 0x1f1e6 && cp <= 0x1f1ff;
const toned = (cp) => cp >= 0x1f3fb && cp <= 0x1f3ff;
const tagged = (cp) => cp >= 0xe0020 && cp <= 0xe007f;

const gathered = (raw) => {
  const kept = [];
  const out = { tones: 0, countries: 0, subdivisions: 0, components: 0 };
  let group = "";
  for (const line of raw.split("\n")) {
    const head = line.match(/^# group: (.+?)\r?$/);
    if (head) group = head[1];
    const one = line.match(/^([0-9A-F ]+?)\s*;\s*fully-qualified\s+#\s+\S+\s+E(\d+\.\d+)\s/);
    if (!one) continue;
    if (group === "Component") {
      out.components++;
      continue;
    }
    if (parseFloat(one[2]) > 15.0) continue;
    const cps = one[1].trim().split(/\s+/).map((hex) => parseInt(hex, 16));
    if (cps.some(toned)) {
      out.tones++;
      continue;
    }
    if (cps.every(regional)) {
      out.countries++;
      continue;
    }
    if (cps.some(tagged)) {
      out.subdivisions++;
      continue;
    }
    kept.push({ mark: String.fromCodePoint(...cps), family: FAMILIES.get(group) ?? group });
  }
  return { kept, out };
};

const worded = (mark, en, es) => {
  const bare = mark.replace(/️/g, "");
  const found = [
    ...(en.get(mark) ?? en.get(bare) ?? []),
    ...(es.get(mark) ?? es.get(bare) ?? []),
    ...(EXTRA.get(mark) ?? []),
  ];
  const seen = new Set();
  const held = [];
  for (const raw of found) {
    const word = plain(raw);
    if (!/[a-z]/.test(word) || seen.has(word)) continue;
    seen.add(word);
    held.push(word);
  }
  return held;
};

const run = async () => {
  const [raw, en, es] = await Promise.all([
    fetched(TEST).then((got) => got.text()),
    noted("en"),
    noted("es"),
  ]);
  const { kept, out } = gathered(raw);
  const bare = [];
  const rows = [];
  const counted = new Map();
  for (const name of FAMILIES.values()) counted.set(name, 0);
  for (const name of FAMILIES.values()) {
    for (const one of kept.filter((held) => held.family === name)) {
      const held = worded(one.mark, en, es);
      if (!held.length) {
        bare.push(one.mark);
        continue;
      }
      rows.push({ mark: one.mark, family: name, words: held });
      counted.set(name, counted.get(name) + 1);
    }
  }

  const rs = ["pub const MARKS: &[&str] = &["];
  for (const one of rows) rs.push(`    "${one.mark}",`);
  rs.push("];", "", "pub const MARK_FAMILIES: &[(&str, usize)] = &[");
  for (const [name, many] of counted) rs.push(`    ("${name}", ${many}),`);
  rs.push("];", "");
  writeFileSync(join(root, "crates", "tisty-core", "src", "model", "mark.rs"), rs.join("\n"));

  const ts = ["const WORDS: Record<string, string[]> = {"];
  for (const one of rows) {
    ts.push(`  ${JSON.stringify(one.mark)}: [${one.words.map((w) => JSON.stringify(w)).join(", ")}],`);
  }
  ts.push(
    "};",
    "",
    "export const MARKS: string[] = Object.keys(WORDS);",
    "",
    "export const isMark = (key: string): boolean => key in WORDS;",
    "",
    "export const markedAs = (said: string): string[] =>",
    "  MARKS.filter((one) => WORDS[one].some((word) => word.startsWith(said)));",
    "",
  );
  writeFileSync(join(root, "app", "src", "marks.ts"), ts.join("\n"));
  try {
    execFileSync("npx biome format --write src/marks.ts", {
      cwd: join(root, "app"),
      shell: true,
      stdio: "ignore",
    });
  } catch {}

  const total = rows.length;
  const spoken = rows.reduce((sum, one) => sum + one.words.length, 0);
  console.log(`marks: ${total}`);
  for (const [name, many] of counted) console.log(`  ${name}: ${many}`);
  console.log(`excluded: skin tones ${out.tones}, country flags ${out.countries}, subdivision flags ${out.subdivisions}, components ${out.components}`);
  console.log(`without annotations: ${bare.length}${bare.length ? ` (${bare.join(" ")})` : ""}`);
  console.log(`words per mark: ${(spoken / total).toFixed(1)}`);
};

await run();
