import { type ReactNode, useCallback, useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { landingSaid, leftAsIs } from "../agentsAmiss";
import { markOffered, SKILLED, SKILLS_DECLINED } from "../agentsOffered";
import {
  AGENTS_CHANGED,
  type TistySkills,
  tistySkills,
  tistySkillsLeftover,
  tistySkillsTake,
} from "../agentsSync";
import type { Wired } from "../core";
import { fill, t, type Word } from "../locales";
import { saidPlainly } from "../refusal";
import { button, primary } from "./AgentsReview";
import { AssistantMark } from "./AssistantMark";
import Glyph from "./Glyph";
import Modal from "./Modal";
import { Knob } from "./Rows";

const NAMES: Record<string, string> = {
  claude: "Claude Code",
  codex: "Codex",
  copilot: "Copilot",
  gemini: "Gemini",
  cursor: "Cursor",
};

const SHARED = new Set(["codex", "copilot", "gemini"]);

const skillSaid = (name: string, about: string | null): string => {
  const key = `tistySkill_${name}` as Word;
  const said = t(key);
  return said === key ? (about ?? "") : said;
};

// Each skill's own lines to say to the agent, numbered from 1 in the words.
export const sayings = (name: string): string[] => {
  const all: string[] = [];
  for (let at = 1; ; at++) {
    const key = `tistySkillSay_${name}_${at}` as Word;
    const said = t(key);
    if (said === key) return all;
    all.push(said);
  }
};

const SKILLS_DIR = "/skills/";

const folderOf = (at: string): string => {
  const cut = at.replace(/\\/g, "/").lastIndexOf(SKILLS_DIR);
  return cut < 0 ? at : at.slice(0, cut + SKILLS_DIR.length - 1);
};

export const skillFolders = (skills: TistySkills): string[] => [
  ...new Set(skills.landing.map((one) => folderOf(one.at))),
];

export const foldersBehind = (skills: TistySkills): string[] => [
  ...new Set(
    skills.landing.filter((one) => one.standing === "write").map((one) => folderOf(one.at)),
  ),
];

export const teaches = (all: readonly Wired[], skills: TistySkills | null | undefined): boolean =>
  !!skills &&
  !skills.on &&
  skills.skills.length > 0 &&
  skills.landing.length > 0 &&
  all.some((one) => {
    const agent = one.place ? undefined : SKILLED[one.id];
    return !!agent && !skills.plugged.includes(agent);
  });

export const agentsTaught = (skills: TistySkills): string[] => [
  ...new Set(
    skills.landing.map((one) =>
      SHARED.has(one.assistant)
        ? "Codex · Copilot · Gemini"
        : (NAMES[one.assistant] ?? one.assistant),
    ),
  ),
];

export function useTistySkills() {
  const [held, setHeld] = useState<TistySkills>();
  const ask = useCallback(() => {
    tistySkills()
      .then(setHeld)
      .catch(() => undefined);
  }, []);
  useEffect(() => {
    ask();
    window.addEventListener(AGENTS_CHANGED, ask);
    return () => window.removeEventListener(AGENTS_CHANGED, ask);
  }, [ask]);
  return [held, setHeld] as const;
}

const turnedOff = () => {
  markOffered([SKILLS_DECLINED]);
  return tistySkillsTake(false);
};

function useTaking(onTaken: (now: TistySkills) => void) {
  const [busy, setBusy] = useState(false);
  const [trouble, setTrouble] = useState<string>();
  const done = (work: Promise<TistySkills>, after?: (now: TistySkills) => void) => {
    setBusy(true);
    setTrouble(undefined);
    work
      .then((now) => {
        onTaken(now);
        window.dispatchEvent(new Event(AGENTS_CHANGED));
        after?.(now);
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };
  return { busy, trouble, done };
}

function Folded({ title, children }: Readonly<{ title: string; children: ReactNode }>) {
  return (
    <details className="mt-3">
      <summary className="cursor-pointer text-[12.5px] font-semibold text-accent hover:underline">
        {title}
      </summary>
      <div className="mt-1.5">{children}</div>
    </details>
  );
}

function HowToUse({ skills }: Readonly<{ skills: TistySkills }>) {
  return (
    <Folded title={t("tistySkillsHow")}>
      <p className="text-[12.5px] text-soft">{t("tistySkillsHowLine")}</p>
      <ul className="mt-1.5 flex flex-col gap-2.5 rounded-[10px] border border-hair bg-panel px-3 py-2.5">
        {skills.skills.map((one) => (
          <li key={one.name} className="text-[12.5px]">
            {!one.yours &&
              sayings(one.name).map((said) => (
                <span key={said} className="block text-[13px] font-medium text-ink">
                  {fill("tistySkillsQuote", said)}
                </span>
              ))}
            <span className="block text-soft">
              {one.yours ? fill("tistySkillsYours", one.name) : skillSaid(one.name, one.about)}
            </span>
            <span className="flex items-center gap-1 font-mono text-[11.5px] text-faint">
              <Glyph name="puzzle" className="h-[11px] w-[11px]" />
              {one.name}
            </span>
          </li>
        ))}
      </ul>
    </Folded>
  );
}

function Where({ skills }: Readonly<{ skills: TistySkills }>) {
  const folders = new Map<string, { at: string; standing: string[] }>();
  for (const one of skills.landing) {
    const at = folderOf(one.at);
    const seen = folders.get(one.assistant) ?? { at, standing: [] };
    seen.standing.push(one.standing);
    folders.set(one.assistant, seen);
  }
  return (
    <ul className="mt-1.5 flex flex-col gap-1.5 rounded-[10px] border border-hair bg-panel px-3 py-2">
      {[...folders]
        .sort(([a], [b]) => Number(SHARED.has(a)) - Number(SHARED.has(b)))
        .map(([id, seen]) => {
          const worst = seen.standing.find((one) => one !== "upToDate" && one !== "wrote");
          const named = SHARED.has(id) ? "Codex · Copilot · Gemini" : NAMES[id];
          return (
            <li key={id} className="text-[12.5px]">
              <div className="flex items-center gap-2">
                <AssistantMark id={id} className="h-[14px] w-[14px]" />
                <span className="text-ink">{named}</span>
                <span className="ml-auto text-[11.5px] text-faint">
                  {skills.on || worst === "foreign" ? landingSaid(worst ?? "upToDate") : ""}
                </span>
              </div>
              <span className="block pl-[22px] font-mono text-[11.5px] break-all text-faint">
                {seen.at}
              </span>
              {SHARED.has(id) && (
                <span className="block pl-[22px] text-[11.5px] text-soft">
                  {t("tistySkillsShared")}
                </span>
              )}
            </li>
          );
        })}
    </ul>
  );
}

export function TistySkillsCard({
  skills,
  onTaken,
  onClose,
}: Readonly<{ skills: TistySkills; onTaken: (now: TistySkills) => void; onClose: () => void }>) {
  const { busy, trouble, done } = useTaking(onTaken);
  const none = skills.skills.length === 0;
  // What was left behind when they were turned off is dealt with here, so the card stays for it.
  const take = (on: boolean) =>
    done(on ? tistySkillsTake(true) : turnedOff(), (now) => {
      if (on || now.leftover.length === 0) onClose();
    });
  const away = (at: string) => done(tistySkillsLeftover(at));
  return (
    <Modal title={t("tistySkills")} wide closable onClose={onClose}>
      <p className="text-[13px] leading-relaxed text-soft">{t("tistySkillsLead")}</p>
      {none && <p className="mt-3 text-[12.5px] text-soft">{t("tistySkillsNone")}</p>}
      {!none && skills.landing.length > 0 && (
        <>
          <p className="mt-3 text-[12.5px] font-semibold">{t("tistySkillsWhere")}</p>
          <Where skills={skills} />
        </>
      )}
      {!none && <HowToUse skills={skills} />}
      <Folded title={t("tistySkillsMore")}>
        <ul className="flex flex-col gap-1 text-[12.5px] text-soft">
          <li>↻ {t("tistySkillsUpdate")}</li>
          <li>🔒 {t("tistySkillsHere")}</li>
          {skills.plugged.map((id) => (
            <li key={id}>◇ {fill("tistySkillsPlugged", NAMES[id] ?? id)}</li>
          ))}
          <li>◇ {t("tistySkillsCursor")}</li>
          {skills.on && <li>⧉ {t("tistySkillsRemoveSaid")}</li>}
        </ul>
      </Folded>
      {skills.leftover.length > 0 && (
        <>
          <p className="mt-4 text-[12.5px] font-semibold">{t("tistySkillsLeftover")}</p>
          <ul className="mt-1.5 flex flex-col gap-1.5 rounded-[10px] border border-hair bg-panel px-3 py-2">
            {skills.leftover.map((one) => (
              <li key={one.at} className="flex items-center gap-2 text-[12.5px]">
                <span className="min-w-0 flex-1">
                  <span className="block font-mono text-[11.5px] break-all text-faint">
                    {one.at}
                  </span>
                  {one.changed && (
                    <span className="block text-[11.5px] text-soft">
                      {t("tistySkillsLeftoverChanged")}
                    </span>
                  )}
                </span>
                <button
                  type="button"
                  disabled={busy}
                  onClick={() => away(one.at)}
                  aria-label={fill("tistySkillsLeftoverAway", one.at)}
                  className={button}
                >
                  {t("agentsLooseAway")}
                </button>
              </li>
            ))}
          </ul>
        </>
      )}
      {!skills.on && !none && skills.landing.length > 0 && (
        <p className="mt-3 text-[12.5px] leading-relaxed text-soft">{t("tistySkillsAsk")}</p>
      )}
      {trouble && <p className="mt-2.5 text-[11.5px] text-urgent">{trouble}</p>}
      <div className="sticky -bottom-6 z-10 -mx-6 mt-4 -mb-6 flex items-center gap-2 border-t border-hair bg-bg px-6 py-3 text-[12.5px]">
        <span className="text-[11.5px] text-faint">
          {fill("tistySkillsVersion", skills.version)}
        </span>
        <span className="flex-1" />
        <button type="button" onClick={onClose} className={button}>
          {t("agentsNotNow")}
        </button>
        <button
          type="button"
          disabled={busy || none}
          onClick={() => take(!skills.on)}
          className={skills.on ? button : primary}
        >
          {t(skills.on ? "tistySkillsRemove" : "tistySkillsAdd")}
        </button>
      </div>
    </Modal>
  );
}

function Opener({
  skills,
  onTaken,
  className,
  children,
}: Readonly<{
  skills: TistySkills;
  onTaken: (now: TistySkills) => void;
  className: string;
  children: ReactNode;
}>) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <button
        type="button"
        data-row="tisty-skills"
        onClick={() => setOpen(true)}
        className={className}
      >
        {children}
      </button>
      {open &&
        createPortal(
          <TistySkillsCard skills={skills} onTaken={onTaken} onClose={() => setOpen(false)} />,
          document.body,
        )}
    </>
  );
}

function Standing({ skills, className }: Readonly<{ skills: TistySkills; className: string }>) {
  const color = skills.on ? "text-hue-green" : "text-faint";
  return (
    <span className={`shrink-0 whitespace-nowrap text-[11.5px] ${color} ${className}`}>
      {t(skills.on ? "tistySkillsTurnedOn" : "tistySkillsOff")}
    </span>
  );
}

export function TistySkillsAgentLine({
  agent,
  name,
  skills,
  onTaken,
}: Readonly<{
  agent: string;
  name: string;
  skills: TistySkills;
  onTaken: (now: TistySkills) => void;
}>) {
  const [open, setOpen] = useState(false);
  const { busy, trouble, done } = useTaking(onTaken);
  const said = lineSaid(agent, skills);
  const left = leftSaid(agent, skills, said);
  const pill = said && LINE[said];
  return (
    <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5 px-3 py-2">
      <span className="min-w-[9rem] flex-1 text-[12.5px] text-soft">
        {t("tistySkills")}
        <span className="text-faint"> · {t("tistySkillsWhole")}</span>
      </span>
      {pill && (
        <span
          className={`shrink-0 rounded-full border px-2 py-0.5 text-[11.5px] whitespace-nowrap ${pill}`}
        >
          {t(said)}
        </span>
      )}
      {said !== "tistySkillsRowPlugged" && (
        <Knob
          on={skills.on}
          label={`${t("tistySkills")} · ${name}`}
          disabled={busy}
          onPress={() => (skills.on ? done(turnedOff()) : setOpen(true))}
        />
      )}
      {left && <p className="w-full text-[12.5px] text-soft">{left}</p>}
      {trouble && <p className="w-full text-[11.5px] text-urgent">{trouble}</p>}
      {open &&
        createPortal(
          <TistySkillsCard skills={skills} onTaken={onTaken} onClose={() => setOpen(false)} />,
          document.body,
        )}
    </div>
  );
}

const LINE = {
  tistySkillsRowPlugged: "border-hue-green/40 text-hue-green",
  tistySkillsRowBehind: "border-hue-amber/40 text-hue-amber",
  tistySkillsRowPartly: "border-line text-soft",
  tistySkillsRowNone: "border-line text-soft",
} as const;

const landsFor = (agent: string) => (assistant: string) =>
  SHARED.has(agent) ? SHARED.has(assistant) : assistant === agent;

function lineSaid(agent: string, skills: TistySkills): keyof typeof LINE | undefined {
  if (skills.plugged.includes(agent)) return "tistySkillsRowPlugged";
  if (!skills.on) return undefined;
  const here = skills.landing.filter((one) => landsFor(agent)(one.assistant));
  if (here.length === 0) return "tistySkillsRowNone";
  if (here.some((one) => one.standing === "write")) return "tistySkillsRowBehind";
  const whole = !leftAsIs(skills, landsFor(agent)) && !yours(skills);
  return whole ? undefined : "tistySkillsRowPartly";
}

const yours = (skills: TistySkills) => skills.skills.some((one) => one.yours);

function leftSaid(agent: string, skills: TistySkills, said: keyof typeof LINE | undefined) {
  if (said === "tistySkillsRowNone" && !yours(skills)) return t("tistySkillsRowNoFolder");
  if (said !== "tistySkillsRowNone" && said !== "tistySkillsRowPartly") return undefined;
  const left = leftAsIs(skills, landsFor(agent));
  return left ? landingSaid(left) : t("tistySkillsRowYours");
}

export function TistySkillsRow({ depth, step }: Readonly<{ depth: number; step: number }>) {
  const [skills, setSkills] = useTistySkills();
  if (!skills) return null;
  return (
    <li className="relative">
      <div className="relative flex items-center rounded-md has-[:focus-visible]:bg-hover">
        <span
          aria-hidden="true"
          className="grid h-5 w-[18px] shrink-0 place-items-center text-faint"
          style={{ marginLeft: `${8 + depth * step}px` }}
        >
          <Glyph name="puzzle" className="h-[13px] w-[13px]" />
        </span>
        <Opener
          skills={skills}
          onTaken={setSkills}
          className="flex min-w-0 flex-1 items-center gap-1.5 rounded-md py-1 pl-1.5 pr-2 text-left text-[13px] text-soft hover:bg-hover"
        >
          <span className="truncate">{t("tistySkills")}</span>
          <Standing skills={skills} className="ml-auto pl-2" />
        </Opener>
      </div>
    </li>
  );
}

export function TistySkillsEntry({ className }: Readonly<{ className: string }>) {
  const [skills, setSkills] = useTistySkills();
  if (!skills) return null;
  return (
    <li className="flex break-inside-avoid items-center">
      <Opener skills={skills} onTaken={setSkills} className={className}>
        <span className="shrink-0 text-faint">
          <Glyph name="puzzle" className="h-3.5 w-3.5" />
        </span>
        <span className="truncate text-ink">{t("tistySkills")}</span>
        <Standing skills={skills} className="ml-auto" />
      </Opener>
    </li>
  );
}

export function TistySkillsSettingsRow() {
  const [skills, setSkills] = useTistySkills();
  if (!skills) return null;
  return (
    <section
      aria-label={t("tistySkills")}
      className="mt-3 flex items-center gap-3 rounded-[10px] border border-hair px-3 py-2.5"
    >
      <span className="min-w-0 flex-1">
        <span className="block text-[13px] font-semibold">{t("tistySkills")}</span>
        <span className="block text-[12.5px] text-soft">{t("tistySkillsSettingsLine")}</span>
      </span>
      <Standing skills={skills} className="rounded-full border border-line px-2 py-0.5" />
      <Opener skills={skills} onTaken={setSkills} className={button}>
        {t("agentSee")}
      </Opener>
    </section>
  );
}
