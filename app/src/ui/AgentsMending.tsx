import { useEffect, useRef, useState } from "react";
import {
  type Extras,
  extrasIds,
  filesOf,
  hush,
  hushedHere,
  inState,
  partsIn,
  skillsBehind,
  unheeded,
} from "../agentsAmiss";
import { addExtras, bringing, mendAll, type Outcome } from "../agentsMending";
import { hooksToken, markOffered, SKILLS_DECLINED } from "../agentsOffered";
import { partsSaid, pathsSaid, pendingSaid, placeNamed } from "../agentsSaid";
import type { TistySkills } from "../agentsSync";
import type { Wired } from "../core";
import type { KnownAgents } from "../knownAgents";
import { fill, t, type Word } from "../locales";
import type { Run } from "./Card";
import { Files, PluginSaid, Ticked } from "./MendAsk";
import { mild, strong } from "./Rows";
import { agentsTaught, foldersBehind, skillFolders } from "./TistySkills";
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
  all: Outcome[];
}

export default function AgentsMending({ known, busy, quietly, sentAt }: Readonly<Props>) {
  const [told, setTold] = useState<Told | null>(null);
  const { extras, decline } = useExtras(known);
  const quiet = hushedHere.use().said;
  const held = isHeld(busy, "wiring");
  const open = known.agent?.on === true;
  const lost = open ? inState(known.everywhere, "lost") : [];
  const elsewhere = open ? unheeded(known.everywhere, "elsewhere", quiet) : [];
  const behind = open && skillsBehind(known.skills) ? known.skills : null;
  const top = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!sentAt || !top.current?.childElementCount) return;
    top.current.scrollIntoView?.({ block: "nearest" });
    top.current.focus();
  }, [sentAt]);

  useEffect(() => {
    if (told) top.current?.focus();
  }, [told]);

  const write = (
    how: Told["how"],
    work: () => Promise<Outcome[]>,
    then?: (all: Outcome[]) => void,
  ) =>
    quietly("wiring", work, (all) => {
      then?.(all);
      known.learnPlugged(bringing(all, true), bringing(all, false));
      setTold({ how, all });
      known.lookAgents().catch(() => undefined);
      known.lookSkills();
    });
  const fix = () =>
    write("mended", () =>
      mendAll(
        lost.map((one) => ({ one, parts: partsIn(one, "lost") })),
        behind,
      ),
    );
  const add = (hooks: Wired[], skills: boolean) =>
    write(
      "added",
      () => addExtras(hooks, skills),
      (all) => markOffered(settled(extras, hooks, skills, all)),
    );

  return (
    <div ref={top} tabIndex={-1} className="outline-none">
      {told && <Result told={told} onClose={() => setTold(null)} />}
      {(lost.length > 0 || behind) && (
        <Broken lost={lost} behind={behind} held={held} onFix={fix} />
      )}
      {elsewhere.length > 0 && (
        <Elsewhere all={elsewhere} onQuiet={() => hush(elsewhere, "elsewhere")} />
      )}
      {extras && (
        <Offer extras={extras} skills={known.skills} held={held} onAdd={add} onDecline={decline} />
      )}
    </div>
  );
}

// One that failed can be offered again, unless the plugin's records stay unreadable: its row still has it.
const settled = (
  extras: Extras | null,
  hooks: readonly Wired[],
  skills: boolean,
  all: readonly Outcome[],
): string[] => {
  if (!extras) return [];
  const failed = new Set(all.filter((one) => one.failed && !one.recordUnread).map((one) => one.id));
  return extrasIds(extras).filter((token) => {
    if (token === SKILLS_DECLINED) return !skills;
    const asked = hooks.find((one) => hooksToken(one.id) === token);
    return !asked || !failed.has(asked.id);
  });
};

function Broken({
  lost,
  behind,
  held,
  onFix,
}: Readonly<{ lost: Wired[]; behind: TistySkills | null; held: boolean; onFix: () => void }>) {
  const some = lost.length === 1 ? t("mendOne") : fill("mendMany", String(lost.length));
  const title = lost.length > 0 ? some : t("mendSkillsTitle");
  return (
    <section
      aria-label={title}
      className="mt-3 rounded-[10px] border border-hue-amber/40 bg-hue-amber/10 px-3 py-2.5"
    >
      <h3 className="flex min-w-0 items-center gap-2 text-[13px] font-semibold">
        <span aria-hidden="true" className="size-[7px] shrink-0 rounded-full bg-hue-amber" />
        {title}
      </h3>
      <ul className="mt-1.5 flex flex-col gap-1.5 text-[12.5px] text-soft">
        {lost.map((one) => (
          <Lost key={one.id} one={one} />
        ))}
        {behind && (
          <li>
            <span className="font-medium text-ink">{t("tistySkills")}</span> · {t("mendSkillsWhy")}
            <span className="block">{t("mendWrites")}</span>
            <Files files={foldersBehind(behind)} />
          </li>
        )}
      </ul>
      {lost.length > 0 && <p className="mt-1.5 text-[11.5px] text-faint">{t("mendKept")}</p>}
      <button type="button" disabled={held} onClick={onFix} className={`mt-2 ${strong}`}>
        {t(lost.length + Number(behind !== null) > 1 ? "mendAll" : "wiringFix")}
      </button>
    </section>
  );
}

function Lost({ one }: Readonly<{ one: Wired }>) {
  const parts = partsIn(one, "lost");
  return (
    <li className="[overflow-wrap:anywhere]">
      <span className="font-medium text-ink">{placeNamed(one)}</span> · {partsSaid(parts)}
      {pathsSaid(one, parts).map((said) => (
        <span key={said} className="block text-[11.5px]">
          {said}
        </span>
      ))}
      <span className="block text-[11.5px]">{t("mendWrites")}</span>
      <Files files={filesOf(one, parts)} />
      <PluginSaid one={one} parts={parts} />
    </li>
  );
}

function Elsewhere({ all, onQuiet }: Readonly<{ all: Wired[]; onQuiet: () => void }>) {
  return (
    <section
      aria-label={pendingSaid(all, "elsewhere")}
      className="mt-3 rounded-[10px] border border-hair px-3 py-2.5 text-[12.5px]"
    >
      <h3 className="text-[13px] font-semibold">{pendingSaid(all, "elsewhere")}</h3>
      <p className="mt-0.5 leading-relaxed text-soft">{t("elsewhereWhy")}</p>
      <ul className="mt-1 text-soft">
        {all.map((one) => (
          <li key={one.id} className="[overflow-wrap:anywhere]">
            <span className="font-medium text-ink">{placeNamed(one)}</span> ·{" "}
            {pathsSaid(one, partsIn(one, "elsewhere")).join(" · ")}
          </li>
        ))}
      </ul>
      <button type="button" onClick={onQuiet} className={`mt-2 ${mild}`}>
        {t("agentsNotNow")}
      </button>
    </section>
  );
}

const outcomeSaid = ({ how, all }: Told) => {
  const failed = all.some((one) => one.failed);
  if (how === "added") return t(failed ? "extrasDonePartly" : "extrasDone");
  return t(failed ? "mendDonePartly" : "mendDone");
};

function Result({ told, onClose }: Readonly<{ told: Told; onClose: () => void }>) {
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
          <li key={one.id} className="[overflow-wrap:anywhere]">
            {one.done && (
              <span className="block">
                <span aria-hidden="true" className="text-hue-green">
                  ✓{" "}
                </span>
                <span className="font-medium">{one.name}</span> · {one.done}
              </span>
            )}
            {one.failed?.map(({ what, why }) => (
              <span key={what ?? why} className="block text-urgent">
                <span aria-hidden="true">✕ </span>
                <span className="font-medium">{one.name}</span>
                {what && ` · ${what}`}: {why}
              </span>
            ))}
          </li>
        ))}
      </ul>
      {told.all.some((one) => one.wrote) && <p className="mt-1.5 text-soft">{t("mendRestart")}</p>}
    </section>
  );
}

const named = (all: Wired[]) => all.map((one) => placeNamed(one)).join(", ");

const addSaid = (hooks: boolean, skills: boolean): Word => {
  if (hooks && skills) return "extrasAdd";
  return hooks ? "extrasAddHooks" : "extrasAddSkills";
};

function Offer({
  extras,
  skills,
  held,
  onAdd,
  onDecline,
}: Readonly<{
  extras: Extras;
  skills: TistySkills | null;
  held: boolean;
  onAdd: (hooks: Wired[], skills: boolean) => void;
  onDecline: () => void;
}>) {
  const { hooks } = extras;
  const teaches = extras.skills.length > 0 && skills !== null;
  const [asking, setAsking] = useState(false);
  const [withHooks, setWithHooks] = useState(true);
  const [withSkills, setWithSkills] = useState(true);
  const choices = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (asking) choices.current?.querySelector("input")?.focus();
  }, [asking]);
  const takesHooks = hooks.length > 0 && withHooks;
  const takesSkills = teaches && withSkills;
  const asked = addSaid(hooks.length > 0, teaches);
  const hooksSaid = fill("extrasHooks", named(hooks));
  const skillsSaid = teaches ? fill("extrasSkills", agentsTaught(skills).join(", ")) : "";
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
            <Ticked on={withHooks} onTurn={setWithHooks} label={hooksSaid}>
              {fill("hooksAsk", hooks.map((one) => one.hooks?.at ?? "").join(", "))}
              {hooks.some((one) => one.id === "claude-code") && (
                <span className="block">{t("hooksAskPlugin")}</span>
              )}
            </Ticked>
          )}
          {teaches && (
            <Ticked on={withSkills} onTurn={setWithSkills} label={skillsSaid}>
              <span className="block">{t("tistySkillsLead")}</span>
              <Files files={skillFolders(skills)} />
              <span className="block">{t("tistySkillsAsk")}</span>
            </Ticked>
          )}
        </div>
      ) : (
        <ul className="mt-1.5 text-soft">
          {hooks.length > 0 && <li>{hooksSaid}</li>}
          {teaches && <li>{skillsSaid}</li>}
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
