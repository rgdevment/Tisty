import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { steps as stepsOf } from "./check-steps.mjs";

const quick = process.argv.includes("--quick");
const windows = process.platform === "win32";

// On Windows a bare `bash` can be WSL's, which sees another machine's toolchain.
const gitBash = windows && join(process.env.ProgramFiles ?? "C:\\Program Files", "Git", "bin", "bash.exe");
const bash = gitBash && existsSync(gitBash) ? gitBash : "bash";

const steps = stepsOf({ quick, bash });

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
