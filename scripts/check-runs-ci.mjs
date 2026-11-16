import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { load } from "js-yaml";
import { steps } from "./check-steps.mjs";

const MIRRORED_JOBS = ["window"];
const ONLY_IN_CI = new Set(["npm ci"]);

const where = (cwd, run) => `${cwd ?? "."}: ${run}`;

export const missingFromCheck = (workflow, local) => {
  const ran = new Set(local.map(([, command, args, cwd]) => where(cwd, [command, ...args].join(" "))));
  const missing = [];
  let compared = 0;
  for (const id of MIRRORED_JOBS) {
    const job = workflow.jobs?.[id];
    if (!job) throw new Error(`the workflow has no job ${id}`);
    const fallback = job.defaults?.run?.["working-directory"];
    for (const step of job.steps.filter((one) => one.run)) {
      const run = step.run.trim();
      if (ONLY_IN_CI.has(run)) continue;
      compared++;
      const at = where(step["working-directory"] ?? fallback, run);
      if (!ran.has(at)) missing.push(at);
    }
  }
  if (compared === 0) throw new Error("no step of CI was compared: this check no longer knows what to look for");
  return missing;
};

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const workflow = load(readFileSync(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8"));
  const missing = missingFromCheck(workflow, steps({ quick: false, bash: "bash" }));
  console.log(missing.join("\n"));
  process.exitCode = missing.length ? 1 : 0;
}
