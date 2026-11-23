import { type ReactNode, useEffect, useRef, useState } from "react";
import { amiss, type Extras, extrasIds, hooksAmiss, skillsBehind } from "../agentsAmiss";
import { addExtras, type Mended, mendAll, partsOf, partsSaid } from "../agentsMending";
import { markOffered } from "../agentsOffered";
import type { Wired } from "../core";
import type { KnownAgents } from "../knownAgents";
import { fill, t, type Word } from "../locales";
import type { Run } from "./Card";
import { mild, strong } from "./Rows";
import { skillFolders } from "./TistySkills";
import { useExtras } from "./useExtras";
import { held as isHeld, type Working } from "./working";

interface Props {
  known: KnownAgents;
  busy: Working;
  quietly: Run;
  sentAt?: number;
}

interface Told {
  how: "mended" | "added";
  all: Mended[];
}

const pathSaid = (one: Wired): string | undefined => {
  const hooks = !one.astray && hooksAmiss(one);
  const points = hooks ? one.hooks?.points : one.points;
  const gone = hooks ? one.hooks?.gone : one.gone;
  return points ? fill(gone ? "wiringPointsGone" : "wiringPoints", points) : undefined;
};

export default function AgentsMending({ known, busy, quietly, sentAt }: Readonly<Props>) {
  const [told, setTold] = useState<Told | null>(null);
  const { extras, decline } = useExtras(known);
  const held = isHeld(busy, "wiring");
  const broken = known.agents?.filter(amiss) ?? [];
  const behind = skillsBehind(known.skills);
  const top = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!sentAt || !top.current?.childElementCount) return;
    top.current.scrollIntoView?.({ block: "nearest" });
    top.current.focus();
  }, [sentAt]);

  useEffect(() => {
    if (told) top.current?.focus();
  }, [told]);

  const write = (how: Told["how"], work: () => Promise<Mended[]>) =>
    quietly("wiring", work, (all) => {
      setTold({ how, all });
      known.lookAgents().catch(() => undefined);
      known.lookSkills();
    });
  const fix = () => write("mended", () => mendAll(broken, known.skills));
  const add = (hooks: Wired[], skills: boolean) => {
    if (extras) markOffered(extrasIds(extras));
    write("added", () => addExtras(hooks, skills));
  };

  return (
    <div ref={top} tabIndex={-1} className="outline-none">
      {told && <Outcome told={told} onClose={() => setTold(null)} />}
      {broken.length + Number(behind) > 0 && (
        <Broken broken={broken} behind={behind} held={held} onFix={fix} />
      )}
      {extras && (
        <Offer
          extras={extras}
          folders={known.skills ? skillFolders(known.skills) : []}
          held={held}
          onAdd={add}
          onDecline={decline}
        />
      )}
    </div>
  );
}

function Broken({
  broken,
  behind,
  held,
  onFix,
}: Readonly<{ broken: Wired[]; behind: boolean; held: boolean; onFix: () => void }>) {
  const [one] = broken;
  const some = broken.length === 1 ? t("mendOne") : fill("mendMany", String(broken.length));
  const title = one ? some : t("mendSkillsTitle");
  return (
    <section
      aria-label={title}
      className="mt-3 rounded-[10px] border border-hue-amber/40 bg-hue-amber/10 px-3 py-2.5"
    >
      <div className="flex flex-wrap items-center gap-3">
        <h3 className="flex min-w-0 flex-1 items-center gap-2 text-[13px] font-semibold">
          <span aria-hidden="true" className="size-[7px] shrink-0 rounded-full bg-hue-amber" />
          {title}
        </h3>
        <button type="button" disabled={held} onClick={onFix} className={strong}>
          {t(broken.length + Number(behind) > 1 ? "mendAll" : "wiringFix")}
        </button>
      </div>
      <ul className="mt-1.5 flex flex-col gap-1 text-[12.5px] text-soft">
        {broken.map((it) => (
          <li key={it.id} className="[overflow-wrap:anywhere]">
            <span className="font-medium text-ink">{it.name}</span> · {partsSaid(partsOf(it))}
            {pathSaid(it) && <span className="block text-[11.5px]">{pathSaid(it)}</span>}
          </li>
        ))}
        {behind && (
          <li>
            {one && <span className="font-medium text-ink">{t("tistySkills")} · </span>}
            {t("mendSkillsWhy")}
          </li>
        )}
      </ul>
      {one && <p className="mt-1.5 text-[11.5px] text-faint">{t("mendKept")}</p>}
    </section>
  );
}

const outcomeSaid = ({ how, all }: Told) => {
  const failed = all.some((one) => one.why);
  if (how === "added") return t(failed ? "extrasDonePartly" : "extrasDone");
  return t(failed ? "mendDonePartly" : "mendDone");
};

function Outcome({ told, onClose }: Readonly<{ told: Told; onClose: () => void }>) {
  const well = told.all.some((one) => !one.why);
  return (
    <section
      role="status"
      className="mt-3 rounded-[10px] border border-hair bg-panel px-3 py-2.5 text-[12.5px]"
    >
      <div className="flex items-center gap-3">
        <h3 className="min-w-0 flex-1 text-[13px] font-semibold">{outcomeSaid(told)}</h3>
        <button type="button" onClick={onClose} className={mild}>
          {t("viewGotIt")}
        </button>
      </div>
      <ul className="mt-1.5 flex flex-col gap-1">
        {told.all.map((one) => (
          <li key={one.id} className="flex gap-2 [overflow-wrap:anywhere]">
            <span aria-hidden="true" className={one.why ? "text-urgent" : "text-hue-green"}>
              {one.why ? "✕" : "✓"}
            </span>
            <span className="min-w-0">
              <span className="font-medium">{one.name}</span>
              {one.parts.length > 0 && ` · ${partsSaid(one.parts)}`}
              {one.state && ` · ${one.state}`}
              {one.why && <span className="block text-urgent">{one.why}</span>}
            </span>
          </li>
        ))}
      </ul>
      {well && <p className="mt-1.5 text-soft">{t("mendRestart")}</p>}
    </section>
  );
}

const named = (all: Wired[]) => all.map((one) => one.name).join(", ");

const addSaid = (hooks: boolean, skills: boolean): Word => {
  if (hooks && skills) return "extrasAdd";
  return hooks ? "extrasAddHooks" : "extrasAddSkills";
};

function Offer({
  extras,
  folders,
  held,
  onAdd,
  onDecline,
}: Readonly<{
  extras: Extras;
  folders: string[];
  held: boolean;
  onAdd: (hooks: Wired[], skills: boolean) => void;
  onDecline: () => void;
}>) {
  const { hooks, skills } = extras;
  const [asking, setAsking] = useState(false);
  const [withHooks, setWithHooks] = useState(true);
  const [withSkills, setWithSkills] = useState(true);
  const choices = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (asking) choices.current?.querySelector("input")?.focus();
  }, [asking]);
  const takesHooks = hooks.length > 0 && withHooks;
  const takesSkills = skills.length > 0 && withSkills;
  const asked = addSaid(hooks.length > 0, skills.length > 0);
  return (
    <section
      aria-label={t("extrasTitle")}
      className="mt-3 rounded-[10px] border border-hair px-3 py-2.5 text-[12.5px]"
    >
      <h3 className="text-[13px] font-semibold">{t("extrasTitle")}</h3>
      <p className="mt-0.5 leading-relaxed text-soft">{t("extrasWhy")}</p>
      {asking ? (
        <div ref={choices} className="mt-2 flex flex-col gap-2.5">
          {hooks.length > 0 && (
            <Ticked on={withHooks} onTurn={setWithHooks} label={fill("extrasHooks", named(hooks))}>
              {fill("hooksAsk", hooks.map((one) => one.hooks?.at ?? "").join(", "))}
            </Ticked>
          )}
          {skills.length > 0 && (
            <Ticked
              on={withSkills}
              onTurn={setWithSkills}
              label={fill("extrasSkills", named(skills))}
            >
              <span className="block">{t("tistySkillsLead")}</span>
              {folders.map((at) => (
                <span key={at} className="block font-mono text-[11.5px] break-all text-faint">
                  {at}
                </span>
              ))}
              <span className="block">{t("tistySkillsAsk")}</span>
            </Ticked>
          )}
        </div>
      ) : (
        <ul className="mt-1.5 text-soft">
          {hooks.length > 0 && <li>{fill("extrasHooks", named(hooks))}</li>}
          {skills.length > 0 && <li>{fill("extrasSkills", named(skills))}</li>}
        </ul>
      )}
      <div className="mt-2.5 flex flex-wrap gap-2">
        {asking ? (
          <button
            key="add"
            type="button"
            disabled={held || !(takesHooks || takesSkills)}
            onClick={() => onAdd(takesHooks ? hooks : [], takesSkills)}
            className={strong}
          >
            {t(takesHooks || takesSkills ? addSaid(takesHooks, takesSkills) : asked)}
          </button>
        ) : (
          <button
            key="ask"
            type="button"
            disabled={held}
            onClick={() => setAsking(true)}
            className={strong}
          >
            {t(asked)}
          </button>
        )}
        <button type="button" onClick={onDecline} className={mild}>
          {t("agentsNotNow")}
        </button>
      </div>
    </section>
  );
}

function Ticked({
  on,
  onTurn,
  label,
  children,
}: Readonly<{
  on: boolean;
  onTurn: (on: boolean) => void;
  label: string;
  children: ReactNode;
}>) {
  return (
    <div>
      <label className="flex cursor-pointer items-start gap-2 font-medium">
        <input
          type="checkbox"
          checked={on}
          onChange={(event) => onTurn(event.target.checked)}
          className="mt-0.5"
        />
        <span className="min-w-0 [overflow-wrap:anywhere]">{label}</span>
      </label>
      <p className="mt-1 pl-[22px] leading-relaxed text-soft [overflow-wrap:anywhere]">
        {children}
      </p>
    </div>
  );
}
