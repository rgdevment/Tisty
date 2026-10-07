import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join } from "node:path";

const quick = process.argv.includes("--quick");
const windows = process.platform === "win32";

// On Windows a bare `bash` can be WSL's, which sees another machine's toolchain.
const gitBash = windows && join(process.env.ProgramFiles ?? "C:\\Program Files", "Git", "bin", "bash.exe");
const bash = gitBash && existsSync(gitBash) ? gitBash : "bash";

const steps = [
  ["the Rust is formatted", "cargo", ["fmt", "--all", "--check"]],
  ["clippy has nothing to say", "cargo", ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]],
  ["advisories and licences", "cargo", ["deny", "check"]],
  ["the window is linted", "npm", ["run", "lint"], "app"],
  ["the window builds", "npm", ["run", "build"], "app"],
  ["the documents are linted", "npm", ["run", "lint:md"]],
  ["the house rules hold", bash, ["scripts/rules.sh"]],
  ["the notices match the lockfiles", "node", ["scripts/third-party.mjs"]],
  ["the notices did not move", "git", ["diff", "--exit-code", "--", "THIRD-PARTY-LICENSES.md", "THIRD-PARTY-BUNDLED.md"]],
];
if (!quick) {
  steps.push(
    ["the Rust tests pass", "cargo", ["nextest", "run", "--workspace", "--no-tests=pass"]],
    ["the window tests pass", "npm", ["test"], "app"],
  );
}

for (const [what, command, args, cwd] of steps) {
  console.log(`\n== ${what}`);
  const ran = spawnSync(command, args, { cwd, stdio: "inherit", shell: windows && command !== bash });
  if (ran.error) {
    console.error(`${command} could not be started: ${ran.error.message}`);
    process.exit(1);
  }
  if (ran.status !== 0) {
    console.error(`\nfailed: ${what}`);
    process.exit(ran.status ?? 1);
  }
}
console.log(`\nall ${steps.length} checks passed${quick ? " (tests left to CI)" : ""}`);
