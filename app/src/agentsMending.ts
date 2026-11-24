import { landingSaid, leftAsIs, type Parts, skillsBehind } from "./agentsAmiss";
import { partsSaid } from "./agentsSaid";
import { AGENTS_CHANGED, type TistySkills, tistySkillsRound, tistySkillsTake } from "./agentsSync";
import { hooked, mendAgent, seenAgents, type Wired } from "./core";
import { t } from "./locales";
import { saidPlainly } from "./refusal";

export interface Outcome {
  id: string;
  name: string;
  done?: string;
  failed?: { what?: string; why: string }[];
  wrote?: boolean;
  brings?: boolean;
  recordUnread?: boolean;
}

const coded = (e: unknown, code: string): boolean =>
  (e as { code?: string } | null | undefined)?.code === code;

const byPlugin = (e: unknown): boolean => coded(e, "hooksByPlugin");

export const bringing = (all: readonly Outcome[], brings: boolean): string[] =>
  all.filter((one) => one.brings === brings).map((one) => one.id);

export interface Asked {
  one: Wired;
  parts: Parts;
}

const hooksGo = ({ one, parts }: Asked): boolean => parts.hooks.length > 0 && !!one.hooks?.plugged;

const stillStray = (after: Wired | undefined, parts: Parts): boolean =>
  !!after?.hooks?.stray?.some((it) => parts.hooks.includes(it.said));

const doneSaid = (one: Wired, parts: Parts): string => {
  if (!hooksGo({ one, parts })) return partsSaid(parts);
  const said = t("mendHooksOut");
  return parts.connection === undefined ? said : `${t("mendConnection")} · ${said}`;
};

export async function mendOne(asked: Asked): Promise<{ said: Outcome; seen: Wired[] | null }> {
  const { one, parts } = asked;
  const said: Outcome = { id: one.id, name: one.name };
  try {
    const answer = await mendAgent(one.id, {
      connection: parts.connection,
      hooks: parts.hooks,
      hooksOut: hooksGo(asked),
    });
    const after = answer.seen.find((it) => it.id === one.id);
    const failed: NonNullable<Outcome["failed"]> = [];
    const fixed: Parts = { ...parts };
    if (parts.connection !== undefined && (answer.connection || !after?.wired)) {
      const why = answer.connection ? saidPlainly(answer.connection) : t("mendStill");
      failed.push({ what: t("mendConnection"), why });
      fixed.connection = undefined;
    }
    if (parts.hooks.length > 0 && (answer.hooks || stillStray(after, parts))) {
      const why = answer.hooks ? saidPlainly(answer.hooks) : t("mendStill");
      failed.push({ what: t("mendHooks"), why });
      fixed.hooks = [];
    }
    const any = fixed.connection !== undefined || fixed.hooks.length > 0;
    if (byPlugin(answer.hooks)) said.brings = true;
    if (coded(answer.hooks, "hooksNotByPlugin")) said.brings = false;
    if (any) {
      said.done = doneSaid(one, fixed);
      said.wrote = true;
    }
    if (failed.length > 0) said.failed = failed;
    return { said, seen: answer.seen };
  } catch (e) {
    return { said: { ...said, failed: [{ why: saidPlainly(e) }] }, seen: null };
  }
}

const SKILLS = "tisty-skills";

const skillsDone = (done: string): Outcome => ({ id: SKILLS, name: t("tistySkills"), done });

const skillsFailed = (why: string): Outcome => ({
  id: SKILLS,
  name: t("tistySkills"),
  failed: [{ why }],
});

const changed = () => window.dispatchEvent(new Event(AGENTS_CHANGED));

export async function mendSkills(skills: TistySkills | null): Promise<Outcome[]> {
  if (!skillsBehind(skills)) return [];
  try {
    const now = await tistySkillsRound();
    changed();
    const behind = skillsBehind(now);
    return [behind ? skillsFailed(t("mendSkillsStill")) : skillsDone(t("mendSkillsDone"))];
  } catch (e) {
    return [skillsFailed(saidPlainly(e))];
  }
}

export async function mendAll(
  all: readonly Asked[],
  skills: TistySkills | null,
): Promise<Outcome[]> {
  const told: Outcome[] = [];
  for (const asked of all) told.push((await mendOne(asked)).said);
  return [...told, ...(await mendSkills(skills))];
}

async function skillsOn(): Promise<Outcome> {
  try {
    const now = await tistySkillsTake(true);
    const left = skillsBehind(now) ? "write" : leftAsIs(now);
    const done = skillsDone(t("extrasSkillsOn"));
    return left ? { ...done, failed: [{ why: landingSaid(left) }] } : done;
  } catch (e) {
    return skillsFailed(saidPlainly(e));
  } finally {
    changed();
  }
}

// The skills' yes lets the plugin's records be read, so the hooks come after it and never run twice.
export async function addExtras(hooks: readonly Wired[], skills: boolean): Promise<Outcome[]> {
  const taught = skills ? [await skillsOn()] : [];
  const now = skills ? await seenAgents().catch(() => hooks) : hooks;
  const told: Outcome[] = [];
  for (const one of hooks) {
    const said: Outcome = { id: one.id, name: one.name };
    if (now.find((it) => it.id === one.id)?.hooks?.plugged) {
      told.push({ ...said, done: t("extrasByPlugin") });
      continue;
    }
    await hooked(one.id, true)
      .then(() => told.push({ ...said, done: t("mendHooks"), wrote: true }))
      .catch((e) =>
        told.push(
          byPlugin(e)
            ? { ...said, done: t("extrasByPlugin"), brings: true }
            : {
                ...said,
                failed: [{ why: saidPlainly(e) }],
                recordUnread: coded(e, "hooksRecordUnread"),
              },
        ),
      );
  }
  return [...told, ...taught];
}
