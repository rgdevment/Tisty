import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const LICENCE = "AGPL-3.0-only";

const cargoSaysIt = (text) => /^license(\.workspace\s*=\s*true|\s*=\s*"AGPL-3\.0-only")\s*$/m.test(text);

export const saysItsLicence = (path, text) =>
  path.endsWith("Cargo.toml") ? cargoSaysIt(text) : JSON.parse(text).license === LICENCE;

export const undeclared = (manifests, readOne) => manifests.filter((path) => !saysItsLicence(path, readOne(path)));

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const tracked = execFileSync("git", ["ls-files", "package.json", "Cargo.toml", "*/package.json", "*/Cargo.toml"], {
    encoding: "utf8",
  });
  const manifests = tracked.split(/\r?\n/).filter(Boolean);
  if (manifests.length === 0) throw new Error("git named no manifests");
  const missing = undeclared(manifests, (path) => readFileSync(path, "utf8"));
  console.log(missing.join(", "));
  process.exitCode = missing.length ? 1 : 0;
}
