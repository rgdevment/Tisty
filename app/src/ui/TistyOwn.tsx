import { type ButtonHTMLAttributes, type ReactNode, useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { addExtras, bringing } from "../agentsMending";
import { AGENTS_CHANGED, type TistySkills } from "../agentsSync";
import type { Wired } from "../core";
import type { KnownAgents } from "../knownAgents";
import { fill, t, type Word } from "../locales";
import { saidPlainly } from "../refusal";
import { adds, type Dot, type Own, ownOf, shown } from "../tistyOwn";
import { Result, type Told } from "./AgentsMending";
import { button, primary } from "./AgentsReview";
import Glyph from "./Glyph";
import { Files, Ticked } from "./MendAsk";
import Modal from "./Modal";
import { skillFolders, TistySkillsCard, useTistySkills } from "./TistySkills";

const DOT: Record<Dot, string> = {
  on: "border-hue-green bg-hue-green",
  partly: "border-hue-amber bg-hue-amber",
  off: "border-faint bg-transparent",
};

export function OwnDot({ dot }: Readonly<{ dot: Dot }>) {
  return (
    <span
      aria-hidden="true"
      data-dot={dot}
      className={`inline-block h-2 w-2 shrink-0 rounded-full border-[1.5px] ${DOT[dot]}`}
    />
  );
}

const names = (all: readonly { name: string }[]) => all.map((one) => one.name).join(", ");

const hooksDot = (own: Own): Dot => {
  const hooks = own.hooks;
  if (!hooks || hooks.muted) return "off";
  const on = hooks.lines.filter((one) => one.on).length;
  if (on === hooks.lines.length) return "on";
  return on > 0 ? "partly" : "off";
};

function hooksDetail(own: Own): string | undefined {
  const hooks = own.hooks;
  if (!hooks) return undefined;
  if (hooks.muted) return t("tistySkillsHooksMuted");
  const on = hooks.lines.filter((one) => one.on);
  if (on.length === hooks.lines.length) return undefined;
  if (on.length > 0) return fill("tistySkillsHooksHooksIn", names(on));
  return fill("tistySkillsHooksHooksOff", names(hooks.lines));
}

export function ownSaid(own: Own): string {
  if (own.dot === "on") return t("tistySkillsHooksOn");
  if (own.dot === "off") return t("tistySkillsHooksOff");
  const lacking: string[] = [];
  if (own.skills && !own.skills.on) lacking.push(t("tistySkillsHooksLackSkills"));
  const hooks = own.hooks;
  if (hooks?.muted) lacking.push(t("tistySkillsHooksMuted"));
  else if (hooks && hooksDot(own) === "off") lacking.push(t("tistySkillsHooksLackHooks"));
  else if (hooks && hooksDot(own) === "partly")
    lacking.push(fill("tistySkillsHooksOnlyIn", names(hooks.lines.filter((one) => one.on))));
  return lacking.join(" · ");
}

interface Seeing {
  known?: KnownAgents;
  agents?: readonly Wired[] | null;
  skillsShown?: boolean;
}

function useOwn({ known, agents, skillsShown = true }: Seeing) {
  const [skills, setSkills] = useTistySkills();
  const own = ownOf({
    agent: known?.agent,
    agents: known?.agents ?? agents ?? null,
    skills: skillsShown ? skills : null,
  });
  return { own, skills, setSkills };
}

function Part({
  glyph,
  label,
  dot,
  detail,
  onAdjust,
}: Readonly<{
  glyph: string;
  label: string;
  dot: Dot;
  detail?: string;
  onAdjust?: () => void;
}>) {
  return (
    <li className="flex items-center gap-2 rounded-[10px] border border-hair px-3 py-2 text-[12.5px]">
      <Glyph name={glyph} className="h-[13px] w-[13px] text-faint" />
      <span className="text-ink">{label}</span>
      <OwnDot dot={dot} />
      {detail && <span className="min-w-0 truncate text-soft">{detail}</span>}
      <span className="flex-1" />
      {onAdjust && (
        <button
          type="button"
          onClick={onAdjust}
          aria-label={`${t("tistySkillsHooksAdjust")} · ${label}`}
          className="shrink-0 text-[12.5px] text-accent hover:underline"
        >
          {t("tistySkillsHooksAdjust")}
        </button>
      )}
    </li>
  );
}

const whatSaid = (own: Own): Word => {
  if (own.skills && own.hooks) return "tistySkillsHooksWhat";
  return own.skills ? "tistySkillsHooksWhatSkills" : "tistySkillsHooksWhatHooks";
};

function Asked({
  hooks,
  skills,
  withHooks,
  onHooks,
  withSkills,
  onSkills,
}: Readonly<{
  hooks: readonly Wired[];
  skills: TistySkills | null;
  withHooks: boolean;
  onHooks: (on: boolean) => void;
  withSkills: boolean;
  onSkills: (on: boolean) => void;
}>) {
  const files = hooks.map((one) => one.hooks?.at ?? "");
  return (
    <div className="flex flex-col gap-2.5 text-[12.5px]">
      <p className="text-[13px] leading-relaxed text-soft">{t("tistySkillsHooksWrites")}</p>
      {hooks.length > 0 && (
        <Ticked
          on={withHooks}
          onTurn={onHooks}
          label={fill("tistySkillsHooksTickHooks", names(hooks))}
        >
          <Files files={files} />
        </Ticked>
      )}
      {skills && (
        <Ticked on={withSkills} onTurn={onSkills} label={t("tistySkillsHooksTickSkills")}>
          <Files files={skillFolders(skills)} />
        </Ticked>
      )}
      <details>
        <summary className="cursor-pointer text-[12.5px] text-accent hover:underline">
          {t("tistySkillsHooksMore")}
        </summary>
        <div className="mt-1.5 flex flex-col gap-1.5 leading-relaxed text-soft [overflow-wrap:anywhere]">
          {hooks.length > 0 && <p>{fill("hooksAsk", files.join(", "))}</p>}
          {hooks.some((one) => one.id === "claude-code") && <p>{t("hooksAskPlugin")}</p>}
          {skills && <p>{t("tistySkillsLead")}</p>}
          {skills && <p>{t("tistySkillsAsk")}</p>}
        </div>
      </details>
    </div>
  );
}

const codexTrusted = (own: Own) =>
  !own.hooks?.muted && !!own.hooks?.lines.some((one) => one.on && one.id.endsWith("codex"));

interface CardProps {
  own: Own;
  skills: TistySkills | undefined;
  known?: KnownAgents;
  onSkills: (now: TistySkills) => void;
  onHooks?: () => void;
  onClose: () => void;
}

function useTurnOn(known?: KnownAgents) {
  const [busy, setBusy] = useState(false);
  const [told, setTold] = useState<Told | null>(null);
  const [trouble, setTrouble] = useState<string>();
  const run = (hooks: readonly Wired[], skills: boolean, after: () => void) => {
    setBusy(true);
    setTrouble(undefined);
    addExtras(hooks, skills)
      .then((all) => {
        known?.learnPlugged(bringing(all, true), bringing(all, false));
        known?.lookAgents().catch(() => undefined);
        known?.lookSkills();
        window.dispatchEvent(new Event(AGENTS_CHANGED));
        setTold({ how: "added", all });
        after();
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };
  return { busy, told, trouble, run, seen: () => setTold(null) };
}

function useLookOnOpen(known?: KnownAgents) {
  const look = known?.lookAgents;
  useEffect(() => {
    look?.().catch(() => undefined);
  }, [look]);
}

function Overview({
  own,
  skills,
  told,
  onSeen,
  onSkills,
  onHooks,
}: Readonly<{
  own: Own;
  skills: TistySkills | undefined;
  told: Told | null;
  onSeen: () => void;
  onSkills?: () => void;
  onHooks?: () => void;
}>) {
  return (
    <>
      <div className="flex flex-col gap-1.5 text-[13px] leading-relaxed text-soft">
        <p>{t(whatSaid(own))}</p>
        <p>{t(own.skills && own.hooks ? "tistySkillsHooksWhy" : "tistySkillsHooksWhyOne")}</p>
      </div>
      <ul className="mt-3 flex flex-col gap-1.5">
        {own.skills && (
          <Part
            glyph="puzzle"
            label={t("tistySkillsHooksPartSkills")}
            dot={own.skills.on ? "on" : "off"}
            detail={own.skills.on && !skills?.on ? t("tistySkillsRowPlugged") : undefined}
            onAdjust={onSkills}
          />
        )}
        {own.hooks && (
          <Part
            glyph="bolt"
            label={t("tistySkillsHooksPartHooks")}
            dot={hooksDot(own)}
            detail={hooksDetail(own)}
            onAdjust={onHooks}
          />
        )}
      </ul>
      {own.dot !== "on" && !adds(own) && (
        <p className="mt-3 text-[12.5px] text-soft">{t("tistySkillsHooksByHand")}</p>
      )}
      {codexTrusted(own) && <p className="mt-3 text-[12.5px] text-soft">{t("hooksCodexTrust")}</p>}
      {told && <Result told={told} onClose={onSeen} />}
    </>
  );
}

function Footer({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <div className="sticky -bottom-6 z-10 -mx-6 mt-4 -mb-6 flex items-center justify-end gap-2 border-t border-hair bg-bg px-6 py-3 text-[12.5px]">
      {children}
    </div>
  );
}

function AskButtons({
  held,
  onBack,
  onYes,
}: Readonly<{ held: boolean; onBack: () => void; onYes: () => void }>) {
  return (
    <>
      <button type="button" onClick={onBack} className={button}>
        {t("tistySkillsHooksBack")}
      </button>
      <button type="button" disabled={held} onClick={onYes} className={primary}>
        {t("tistySkillsHooksYes")}
      </button>
    </>
  );
}

function OfferButtons({
  offers,
  both,
  busy,
  onClose,
  onAsk,
}: Readonly<{
  offers: boolean;
  both: boolean;
  busy: boolean;
  onClose: () => void;
  onAsk: () => void;
}>) {
  return (
    <>
      <button type="button" onClick={onClose} className={button}>
        {t(offers ? "agentsNotNow" : "close")}
      </button>
      {offers && (
        <button type="button" disabled={busy} onClick={onAsk} className={primary}>
          {t(both ? "tistySkillsHooksBoth" : "tistySkillsHooksRest")}
        </button>
      )}
    </>
  );
}

export function TistyOwnCard({
  own,
  skills,
  known,
  onSkills,
  onHooks,
  onClose,
}: Readonly<CardProps>) {
  const [asking, setAsking] = useState(false);
  const [withHooks, setWithHooks] = useState(true);
  const [withSkills, setWithSkills] = useState(true);
  const [skillsOpen, setSkillsOpen] = useState(false);
  const turning = useTurnOn(known);
  useLookOnOpen(known);
  const hooksAdd = own.hooks?.adds ?? [];
  const skillsAdd = own.skills?.adds && skills ? skills : null;
  const takesHooks = hooksAdd.length > 0 && withHooks;
  const takesSkills = !!skillsAdd && withSkills;

  if (skillsOpen && skills)
    return (
      <TistySkillsCard skills={skills} onTaken={onSkills} onClose={() => setSkillsOpen(false)} />
    );

  const turnOn = () => turning.run(takesHooks ? hooksAdd : [], takesSkills, () => setAsking(false));
  const adjustHooks =
    onHooks &&
    (() => {
      onClose();
      onHooks();
    });

  return (
    <Modal title={t("tistySkillsHooksTitle")} wide closable onClose={onClose}>
      {asking ? (
        <Asked
          hooks={hooksAdd}
          skills={skillsAdd}
          withHooks={withHooks}
          onHooks={setWithHooks}
          withSkills={withSkills}
          onSkills={setWithSkills}
        />
      ) : (
        <Overview
          own={own}
          skills={skills}
          told={turning.told}
          onSeen={turning.seen}
          onSkills={skills ? () => setSkillsOpen(true) : undefined}
          onHooks={adjustHooks}
        />
      )}
      {turning.trouble && <p className="mt-2.5 text-[11.5px] text-urgent">{turning.trouble}</p>}
      <Footer>
        {asking ? (
          <AskButtons
            held={turning.busy || !(takesHooks || takesSkills)}
            onBack={() => setAsking(false)}
            onYes={turnOn}
          />
        ) : (
          <OfferButtons
            offers={adds(own)}
            both={!!skillsAdd && hooksAdd.length > 0 && own.dot === "off"}
            busy={turning.busy}
            onClose={onClose}
            onAsk={() => setAsking(true)}
          />
        )}
      </Footer>
    </Modal>
  );
}

function useOwnOpener({ onHooks, ...seeing }: Readonly<Seeing & { onHooks?: () => void }>) {
  const { own, skills, setSkills } = useOwn(seeing);
  const [open, setOpen] = useState(false);
  const card = open
    ? createPortal(
        <TistyOwnCard
          own={own}
          skills={skills}
          known={seeing.known}
          onSkills={setSkills}
          onHooks={onHooks}
          onClose={() => setOpen(false)}
        />,
        document.body,
      )
    : null;
  return {
    own,
    said: fill("tistySkillsHooksSaid", ownSaid(own)),
    card,
    show: () => setOpen(true),
  };
}

type Opened = ReturnType<typeof useOwnOpener>;

function OwnButton({
  opened,
  className,
  row,
  label,
  children,
}: Readonly<{
  opened: Opened;
  className: string;
  row?: ButtonHTMLAttributes<HTMLButtonElement>;
  label?: string;
  children: ReactNode;
}>) {
  return (
    <button
      type="button"
      data-row="tisty-own"
      data-dot={opened.own.dot}
      {...row}
      onClick={opened.show}
      aria-label={label ?? opened.said}
      title={opened.said}
      className={className}
    >
      {children}
    </button>
  );
}

export function TistyOwnRow({
  depth,
  step,
  rowOf,
  lead,
  ...rest
}: Readonly<
  Seeing & {
    depth: number;
    step: number;
    rowOf?: (key: string) => ButtonHTMLAttributes<HTMLButtonElement>;
    lead?: ReactNode;
    onHooks?: () => void;
  }
>) {
  const opened = useOwnOpener(rest);
  if (!shown(opened.own)) return null;
  return (
    <>
      {lead}
      <li className="relative">
        <div className="relative flex items-center rounded-md has-[:focus-visible]:bg-hover">
          <span
            aria-hidden="true"
            className="grid h-5 w-[18px] shrink-0 place-items-center text-faint"
            style={{ marginLeft: `${8 + depth * step}px` }}
          >
            <Glyph name="plug" className="h-[13px] w-[13px]" />
          </span>
          <OwnButton
            opened={opened}
            row={rowOf?.("tisty-own")}
            className="flex min-w-0 flex-1 items-center gap-1.5 rounded-md py-1 pl-1.5 pr-2 text-left text-[13px] text-soft hover:bg-hover"
          >
            <span className="truncate">{t("tistySkillsHooks")}</span>
            <span className="ml-auto grid h-4 w-4 place-items-center">
              <OwnDot dot={opened.own.dot} />
            </span>
          </OwnButton>
        </div>
      </li>
      {opened.card}
    </>
  );
}

export function TistyOwnEntry({
  className,
  ...rest
}: Readonly<Seeing & { className: string; onHooks?: () => void }>) {
  const opened = useOwnOpener(rest);
  if (!shown(opened.own)) return null;
  return (
    <>
      <li className="flex break-inside-avoid items-center">
        <OwnButton opened={opened} className={className}>
          <span className="shrink-0 text-faint">
            <Glyph name="plug" className="h-3.5 w-3.5" />
          </span>
          <span className="truncate text-ink">{t("tistySkillsHooks")}</span>
          <span className="ml-auto grid h-4 w-4 place-items-center">
            <OwnDot dot={opened.own.dot} />
          </span>
        </OwnButton>
      </li>
      {opened.card}
    </>
  );
}

export function TistyOwnSettingsRow(props: Readonly<Seeing>) {
  const opened = useOwnOpener(props);
  if (!shown(opened.own)) return null;
  return (
    <>
      <section
        aria-label={t("tistySkillsHooksTitle")}
        className="mt-3 flex items-center gap-3 rounded-[10px] border border-hair px-3 py-2.5"
      >
        <span className="min-w-0 flex-1">
          <span className="block text-[13px] font-semibold">{t("tistySkillsHooksTitle")}</span>
          <span className="block text-[12.5px] text-soft">{t("tistySkillsHooksLine")}</span>
        </span>
        <OwnButton
          opened={opened}
          label={`${t("agentSee")} · ${opened.said}`}
          className={`${button} flex shrink-0 items-center gap-1.5 border border-line`}
        >
          <OwnDot dot={opened.own.dot} />
          {t("agentSee")}
        </OwnButton>
      </section>
      {opened.card}
    </>
  );
}
