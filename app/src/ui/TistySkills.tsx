import { type ReactNode, useCallback, useEffect, useState } from "react";
import { createPortal } from "react-dom";
import {
  AGENTS_CHANGED,
  type TistySkills,
  tistySkills,
  tistySkillsLeftover,
  tistySkillsTake,
} from "../agentsSync";
import { hide, show, useHidden } from "../hiding";
import { fill, t, type Word } from "../locales";
import { saidPlainly } from "../refusal";
import { button, primary } from "./AgentsReview";
import { AssistantMark } from "./AssistantMark";
import Glyph from "./Glyph";
import Modal from "./Modal";

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

const landingSaid = (standing: string): string => {
  const key = `tistyLanding_${standing}` as Word;
  const said = t(key);
  return said === key ? standing : said;
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
  const [busy, setBusy] = useState(false);
  const [trouble, setTrouble] = useState<string>();
  const none = skills.skills.length === 0;
  const take = (on: boolean) => done(tistySkillsTake(on));
  const away = (at: string) => done(tistySkillsLeftover(at));
  const done = (work: Promise<TistySkills>) => {
    setBusy(true);
    setTrouble(undefined);
    work
      .then((now) => {
        onTaken(now);
        window.dispatchEvent(new Event(AGENTS_CHANGED));
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };
  return (
    <Modal title={t("tistySkills")} wide closable onClose={onClose}>
      <p className="text-[13px] leading-relaxed text-soft">{t("tistySkillsLead")}</p>
      {none ? (
        <p className="mt-3 text-[12.5px] text-soft">{t("tistySkillsNone")}</p>
      ) : (
        <ul className="mt-3 flex flex-col gap-2 rounded-[10px] border border-hair bg-panel px-3 py-2">
          {skills.skills.map((one) => (
            <li key={one.name} className="text-[12.5px]">
              <span className="flex items-center gap-1.5 font-mono text-ink">
                <Glyph name="puzzle" className="h-[13px] w-[13px] text-faint" />
                {one.name}
              </span>
              <span className="block pl-[19px] text-soft">
                {one.yours ? fill("tistySkillsYours", one.name) : skillSaid(one.name, one.about)}
              </span>
              {!one.yours && sayings(one.name).length > 0 && (
                <span className="block pl-[19px] text-faint">
                  {t("tistySkillsSay")}{" "}
                  <span className="text-ink">
                    {sayings(one.name)
                      .map((said) => fill("tistySkillsQuote", said))
                      .join(" · ")}
                  </span>
                </span>
              )}
            </li>
          ))}
        </ul>
      )}
      {!none && skills.landing.length > 0 && (
        <>
          <p className="mt-4 text-[12.5px] font-semibold">{t("tistySkillsWhere")}</p>
          <Where skills={skills} />
        </>
      )}
      <ul className="mt-3 flex flex-col gap-1 text-[12.5px] text-soft">
        <li>↻ {t("tistySkillsUpdate")}</li>
        <li>🔒 {t("tistySkillsHere")}</li>
        {skills.plugged.map((id) => (
          <li key={id}>◇ {fill("tistySkillsPlugged", NAMES[id] ?? id)}</li>
        ))}
        <li>◇ {t("tistySkillsCursor")}</li>
        {skills.on && <li>⧉ {t("tistySkillsRemoveSaid")}</li>}
      </ul>
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
      <div className="mt-5 flex items-center gap-2 text-[12.5px]">
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

const standingOf = (skills: TistySkills) => t(skills.on ? "tistySkillsOn" : "tistySkillsOff");

// Hiding is a way of looking, kept on this computer alone, and turns nothing off.
export const TISTY_SKILLS_ROW = "tisty-skills";

function HideRow() {
  return (
    <button
      type="button"
      onClick={() => hide(TISTY_SKILLS_ROW)}
      aria-label={t("tistySkillsHideRow")}
      title={t("tistySkillsHideRow")}
      className="shrink-0 rounded-md px-1.5 py-0.5 text-[11.5px] text-faint opacity-0 group-hover/row:opacity-100 focus-visible:opacity-100 hover:text-ink"
    >
      {t("agentsHide")}
    </button>
  );
}

export function TistySkillsRow({ depth, step }: Readonly<{ depth: number; step: number }>) {
  const [skills, setSkills] = useTistySkills();
  const hidden = useHidden().includes(TISTY_SKILLS_ROW);
  if (!skills || hidden) return null;
  return (
    <li className="relative">
      <div className="group/row relative flex items-center rounded-md has-[:focus-visible]:bg-hover">
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
          <span className="ml-auto shrink-0 pl-2 text-[11.5px] text-faint">
            {standingOf(skills)}
          </span>
        </Opener>
        <HideRow />
      </div>
    </li>
  );
}

export function TistySkillsEntry({ className }: Readonly<{ className: string }>) {
  const [skills, setSkills] = useTistySkills();
  const hidden = useHidden().includes(TISTY_SKILLS_ROW);
  if (!skills || hidden) return null;
  return (
    <li className="group/row flex break-inside-avoid items-center">
      <Opener skills={skills} onTaken={setSkills} className={className}>
        <span className="shrink-0 text-faint">
          <Glyph name="puzzle" className="h-3.5 w-3.5" />
        </span>
        <span className="truncate text-ink">{t("tistySkills")}</span>
        <span className="ml-auto shrink-0 text-[11.5px] text-faint">{standingOf(skills)}</span>
      </Opener>
      <HideRow />
    </li>
  );
}

export function TistySkillsSettingsRow() {
  const [skills, setSkills] = useTistySkills();
  const hidden = useHidden().includes(TISTY_SKILLS_ROW);
  if (!skills) return null;
  return (
    <section
      aria-label={t("tistySkills")}
      className="mt-3 flex items-center gap-3 rounded-[10px] border border-hair px-3 py-2.5"
    >
      <span className="min-w-0 flex-1">
        <span className="block text-[13px] font-semibold">{t("tistySkills")}</span>
        <span className="block text-[12.5px] text-soft">{t("tistySkillsSettingsLine")}</span>
        {hidden && (
          <span className="block text-[12.5px] text-faint">
            {t("tistySkillsHidden")}{" "}
            <button
              type="button"
              onClick={() => show(TISTY_SKILLS_ROW)}
              className="cursor-pointer text-accent underline-offset-2 hover:underline"
            >
              {t("tistySkillsShowRow")}
            </button>
          </span>
        )}
      </span>
      <span className="shrink-0 rounded-full border border-line px-2 py-0.5 text-[11.5px] whitespace-nowrap text-soft">
        {standingOf(skills)}
      </span>
      <Opener skills={skills} onTaken={setSkills} className={button}>
        {t("agentSee")}
      </Opener>
    </section>
  );
}
