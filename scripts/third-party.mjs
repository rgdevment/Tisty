import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const out = join(root, "THIRD-PARTY-BUNDLED.md");

const shipped = () => {
  const lock = JSON.parse(readFileSync(join(root, "app", "package-lock.json"), "utf8"));
  const seen = new Map();
  for (const [at, one] of Object.entries(lock.packages ?? {})) {
    if (!at || one.dev || one.devOptional || one.extraneous) continue;
    const name = one.name ?? at.slice(at.lastIndexOf("node_modules/") + 13);
    if (!name || seen.has(name)) continue;
    seen.set(name, {
      version: one.version ?? "?",
      licence: one.license ?? "see the package",
      notice: noticed(join(root, "app", at)),
    });
  }
  return seen;
};

const told = (pkg) =>
  typeof pkg.license === "string"
    ? pkg.license
    : (pkg.license?.type ?? pkg.licenses?.map((one) => one.type).join(" OR ") ?? "see the package");

const noticed = (at) => {
  if (!existsSync(at)) return null;
  const named = readdirSync(at).find((one) => /^(licen[cs]e|copying)/i.test(one));
  if (!named) return null;
  const said = readFileSync(join(at, named), "utf8").trim();
  return said.length > 4000 ? `${said.slice(0, 4000)}\n…` : said;
};

const crates = () => {
  const said = execFileSync(
    "cargo",
    ["metadata", "--format-version", "1", "--filter-platform", process.env.TARGET ?? hostTriple()],
    { cwd: root, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  const meta = JSON.parse(said);
  const ours = new Set(meta.workspace_members);
  const seen = new Map();
  for (const one of meta.packages) {
    if (ours.has(one.id) || seen.has(one.name)) continue;
    seen.set(one.name, { version: one.version, licence: one.license ?? "see the crate" });
  }
  return seen;
};

const hostTriple = () => {
  const said = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  return /host: (.+)/.exec(said)?.[1]?.trim() ?? "";
};

const listed = (seen) =>
  [...seen.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([name, one]) => `| \`${name}\` | ${one.version} | ${one.licence} |`)
    .join("\n");

const js = shipped();
const rs = crates();

const kept = [...js.entries()]
  .filter(([, one]) => one.notice)
  .sort(([a], [b]) => a.localeCompare(b))
  .map(([name, one]) => `<details>\n<summary><code>${name}</code> — ${one.licence}</summary>\n\n\`\`\`text\n${one.notice}\n\`\`\`\n\n</details>`)
  .join("\n\n");

writeFileSync(
  out,
  `# Third-party notices — what ships inside Tisty

<!-- Written by \`npm run notices\`. Do not edit by hand. -->

Tisty is AGPL-3.0-only. The binary carries the work below, each under its own
licence. Anything copied into Tisty's own source rather than bundled is in
[THIRD-PARTY.md](THIRD-PARTY.md) instead.

## In the window (${js.size} packages)

| Package | Version | Licence |
| --- | --- | --- |
${listed(js)}

## In the core (${rs.size} crates)

| Crate | Version | Licence |
| --- | --- | --- |
${listed(rs)}

## The notices themselves

${kept}
`,
);

console.log(`${js.size} packages, ${rs.size} crates -> ${out}`);
