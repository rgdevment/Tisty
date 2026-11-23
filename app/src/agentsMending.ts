import { amiss, hooksAmiss, landingSaid, leftAsIs, skillsBehind } from "./agentsAmiss";
import { AGENTS_CHANGED, type TistySkills, tistySkillsRound, tistySkillsTake } from "./agentsSync";
import { hooked, type Wired, wireAgent } from "./core";
import { t } from "./locales";
import { saidPlainly } from "./refusal";

export type Part = "connection" | "hooks";

export interface Mended {
  id: string;
  name: string;
  parts: Part[];
  state?: string;
  why?: string;
}

export const partsOf = (one: Wired): Part[] => [
  ...(one.astray ? (["connection"] as const) : []),
  ...(hooksAmiss(one) ? (["hooks"] as const) : []),
];

export const partsSaid = (parts: readonly Part[]): string => {
  if (parts.length === 2) return t("mendBoth");
  return parts.map((one) => t(one === "hooks" ? "mendHooks" : "mendConnection")).join("");
};

// Writing the connection carries its hooks along; ones the plugin brings too go, as they would run twice.
async function written(one: Wired): Promise<Wired[]> {
  const joined = one.astray ? await wireAgent(one.id) : null;
  if (hooksAmiss(one) && one.hooks?.plugged) return hooked(one.id, false);
  return joined ?? hooked(one.id, true);
}

export async function mendOne(one: Wired): Promise<{ said: Mended; seen: Wired[] | null }> {
  const said: Mended = { id: one.id, name: one.name, parts: partsOf(one) };
  try {
    const seen = await written(one);
    const after = seen.find((it) => it.id === one.id);
    return { said: after && amiss(after) ? { ...said, why: t("mendStill") } : said, seen };
  } catch (e) {
    return { said: { ...said, why: saidPlainly(e) }, seen: null };
  }
}

const skillsLine = (why?: string): Mended => ({
  id: "tisty-skills",
  name: t("tistySkills"),
  parts: [],
  why,
});

const changed = () => window.dispatchEvent(new Event(AGENTS_CHANGED));

export async function mendSkills(skills: TistySkills | null): Promise<Mended[]> {
  if (!skillsBehind(skills)) return [];
  try {
    const now = await tistySkillsRound();
    changed();
    return [skillsLine(skillsBehind(now) ? t("mendSkillsStill") : undefined)];
  } catch (e) {
    return [skillsLine(saidPlainly(e))];
  }
}

export async function mendAll(
  all: readonly Wired[],
  skills: TistySkills | null,
): Promise<Mended[]> {
  const told: Mended[] = [];
  for (const one of all) told.push((await mendOne(one)).said);
  return [...told, ...(await mendSkills(skills))];
}

export async function addExtras(hooks: readonly Wired[], skills: boolean): Promise<Mended[]> {
  const told: Mended[] = [];
  for (const one of hooks) {
    const said: Mended = { id: one.id, name: one.name, parts: ["hooks"] };
    await hooked(one.id, true)
      .then(() => told.push(said))
      .catch((e) => told.push({ ...said, why: saidPlainly(e) }));
  }
  if (!skills) return told;
  await tistySkillsTake(true)
    .then((now) => {
      const left = skillsBehind(now) ? "write" : leftAsIs(now);
      told.push(
        left ? skillsLine(landingSaid(left)) : { ...skillsLine(), state: t("extrasSkillsOn") },
      );
    })
    .catch((e) => told.push(skillsLine(saidPlainly(e))));
  changed();
  return told;
}
