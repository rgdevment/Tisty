import { useEffect, useState } from "react";
import { allParts, skillsBehind, some } from "../agentsAmiss";
import { bringing, mendOne, mendSkills, type Outcome } from "../agentsMending";
import { hooksToken, markOffered, offered, offers, SKILLED } from "../agentsOffered";
import type { TistySkills } from "../agentsSync";
import {
  type Assistant,
  copied,
  hooked,
  seenAgents,
  unwireAgent,
  type Wired,
  wireAgent,
  wiringKnown,
  wslDistros,
  wslSeen,
} from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import type { Run, Word } from "./Card";
import ConnectOffer from "./ConnectOffer";
import Hooking from "./Hooking";
import MendAsk, { OutAsk } from "./MendAsk";
import { foldersBehind, TistySkillsAgentLine } from "./TistySkills";

const LOOKED = "tisty.wsl.looked";

const looked = (): string[] => {
  try {
    return JSON.parse(window.localStorage.getItem(LOOKED) ?? "[]");
  } catch {
    return [];
  }
};

const remember = (all: string[]) => {
  try {
    window.localStorage.setItem(LOOKED, JSON.stringify(all));
  } catch {
    // A private window keeps nothing, and the distro is asked again next time.
  }
};

const ready = (one: Wired) =>
  !one.wired && !one.astray && !one.apart && !one.unread && !one.byHand && !one.maybe;

function Manual({ one }: Readonly<{ one: Wired }>) {
  const [done, setDone] = useState<string>();
  const copy = (text: string) =>
    void copied(text).then(() => {
      setDone(text);
      window.setTimeout(() => setDone(undefined), 1500);
    });
  return (
    <div className="mt-2 flex min-w-0 flex-col gap-2 rounded-[10px] border border-hair bg-bg px-3 py-2.5">
      {one.manual.cli && (
        <>
          <span className="text-[12.5px] text-soft">{t("wiringByCli")}</span>
          {/* Wrapped, not scrolled: on a Mac that always shows scroll bars, the bar ate the one line. */}
          <pre className="shrink-0 rounded-md border border-hair px-2.5 py-1.5 font-mono text-[11.5px] whitespace-pre-wrap text-soft [overflow-wrap:anywhere]">
            {one.manual.cli}
          </pre>
          <button type="button" onClick={() => copy(one.manual.cli ?? "")} className={small}>
            {done === one.manual.cli ? t("agentsCopied") : t("agentsCopy")}
          </button>
        </>
      )}
      <span className="text-[12.5px] text-soft [overflow-wrap:anywhere]">
        {fill("wiringByPaste", one.manual.file)}
      </span>
      <pre className="shrink-0 overflow-x-auto rounded-md border border-hair px-2.5 py-1.5 font-mono text-[11.5px] text-soft">
        {one.manual.snippet}
      </pre>
      <button type="button" onClick={() => copy(one.manual.snippet)} className={small}>
        {done === one.manual.snippet ? t("agentsCopied") : t("agentsCopy")}
      </button>
    </div>
  );
}

const small =
  "self-start rounded-md border border-line px-2.5 py-0.5 text-[12.5px] text-soft hover:border-accent hover:text-accent";

interface RowProps {
  one: Wired;
  hand?: string;
  held: boolean;
  offering?: boolean;
  onJoin: (one: Wired) => void;
  onHook: (one: Wired, on: boolean) => void;
  onFix: (one: Wired) => void;
  onOffered?: (one: Wired) => void;
  skills?: TistySkills | null;
  onSkills?: (now: TistySkills) => void;
  behind?: string[];
}

function Extras({ one, held, onHook, onFix, skills, onSkills }: Readonly<RowProps>) {
  const joined = one.wired && !one.astray;
  const hooks = one.hooks && (one.hooks.on || one.hooks.apart || one.hooks.unread || joined);
  const agent = one.place ? undefined : SKILLED[one.id];
  const taught = agent && joined && skills && skills.skills.length > 0 && onSkills;
  if (!hooks && !taught) return null;
  return (
    <div className="mt-2 divide-y divide-hair rounded-[10px] border border-hair">
      {hooks && (
        <Hooking one={one} held={held} onHook={onHook} onFix={one.astray ? undefined : onFix} />
      )}
      {taught && <TistySkillsAgentLine agent={agent} skills={skills} onTaken={onSkills} />}
    </div>
  );
}

const heldBack = (one: Wired): string | undefined => {
  if (one.unread) return fill("wiringUnread", one.at);
  if (!one.apart) return undefined;
  return one.points ? fill("wiringApart", one.points) : t("wiringApartPlain");
};

function Row(props: Readonly<RowProps>) {
  const { one, hand, held, offering, onJoin, onHook, onFix, onOffered } = props;
  const [open, setOpen] = useState(false);
  const [asking, setAsking] = useState(false);
  const [leaving, setLeaving] = useState(false);
  const fixing = asking && some(allParts(one));
  const holds = (one.wired || !!one.apart) && !one.astray;
  const press = () => {
    if (one.astray) setAsking(!asking);
    else if (holds) setLeaving(!leaving);
    else onJoin(one);
  };
  const state = one.byHand
    ? "wiringStateHand"
    : one.unread
      ? "wiringStateUnread"
      : one.apart
        ? "wiringStateApart"
        : one.astray
          ? "wiringStateAstray"
          : one.wired
            ? "wiringOn"
            : one.maybe
              ? "wiringStateMaybe"
              : "wiringStateOff";
  const chip = one.wired
    ? "border-hue-green/40 text-hue-green"
    : one.astray
      ? "border-hue-amber/40 text-hue-amber"
      : one.maybe
        ? "border-dashed border-line text-soft"
        : "border-line text-soft";
  const action = one.astray
    ? "wiringFix"
    : one.wired || one.apart
      ? "wiringOut"
      : one.maybe
        ? "wiringAnyway"
        : "wiringJoin";
  const detail = one.astray
    ? fill(one.gone ? "wiringPointsGone" : "wiringPoints", one.points ?? "")
    : (heldBack(one) ?? (one.maybe && !one.wired ? t("wiringMaybe") : hand));
  return (
    <div className="border-t border-hair px-3 py-2.5 first:border-t-0">
      <div className="flex items-center gap-3">
        <span className="min-w-0 flex-1">
          <span className="block text-[13px] font-semibold">
            {one.name}
            {one.also && (
              <span className="font-normal text-soft"> · {fill("wiringAlso", one.also)}</span>
            )}
          </span>
          {/* A path has no spaces to break at, so without this it runs under the state beside it. */}
          {detail && (
            <span className="block text-[12.5px] text-soft [overflow-wrap:anywhere]">{detail}</span>
          )}
          <span className="block truncate font-mono text-[10.5px] text-faint">{one.at}</span>
        </span>
        <span
          className={`shrink-0 rounded-full border px-2 py-0.5 text-[11.5px] whitespace-nowrap ${chip}`}
        >
          {t(state)}
        </span>
        {!one.byHand && !one.unread && (
          <button
            type="button"
            disabled={held}
            aria-expanded={one.astray ? fixing : holds ? leaving : undefined}
            onClick={press}
            className={`shrink-0 rounded-md border px-2.5 py-1 text-[12.5px] disabled:text-faint ${
              holds
                ? "border-line text-soft hover:border-urgent hover:text-urgent"
                : "border-accent text-accent"
            }`}
          >
            {t(action)}
          </button>
        )}
        <button
          type="button"
          aria-label={t("wiringHow")}
          aria-expanded={open}
          onClick={() => setOpen(!open)}
          className="shrink-0 rounded-md border border-line px-2 py-1 text-[12.5px] text-soft hover:border-accent hover:text-accent"
        >
          {t(one.byHand || one.unread || one.apart ? "wiringHowShort" : "wiringStateHand")}
        </button>
      </div>
      {offering && one.wired && !one.astray ? (
        <ConnectOffer one={one} held={held} onHook={onHook} onDone={() => onOffered?.(one)} />
      ) : (
        <Extras {...props} onFix={() => setAsking(!asking)} />
      )}
      {fixing && (
        <MendAsk
          one={one}
          folders={props.behind ?? []}
          held={held}
          onYes={() => {
            setAsking(false);
            onFix(one);
          }}
          onNo={() => setAsking(false)}
        />
      )}
      {leaving && holds && (
        <OutAsk
          one={one}
          held={held}
          onYes={() => {
            setLeaving(false);
            onJoin(one);
          }}
          onNo={() => setLeaving(false)}
        />
      )}
      {open && <Manual one={one} />}
    </div>
  );
}

function Rows({
  all,
  hands,
  fresh,
  ...rest
}: Readonly<
  Omit<RowProps, "one" | "hand" | "offering"> & {
    all: Wired[];
    hands?: (one: Wired) => string | undefined;
    fresh?: readonly string[];
  }
>) {
  return (
    <div className="overflow-hidden rounded-[10px] border border-hair">
      {all.map((one) => (
        <Row
          key={one.id}
          one={one}
          hand={hands?.(one)}
          offering={fresh?.includes(one.id)}
          {...rest}
        />
      ))}
    </div>
  );
}

interface Props {
  agents: Wired[] | null;
  setAgents: (all: Wired[]) => void;
  hands: Assistant[] | null;
  skills?: TistySkills | null;
  setSkills?: (now: TistySkills) => void;
  learnPlugged?: (ids: string[], gone: string[]) => void;
  wakes?: boolean;
  held: boolean;
  quietly: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  wroteSaid: (hand: Assistant | undefined) => string;
}

export default function Wiring({
  agents,
  setAgents,
  hands,
  skills,
  setSkills,
  learnPlugged,
  wakes = true,
  held,
  quietly,
  tell,
  fail,
  wroteSaid,
}: Props) {
  const [distros, setDistros] = useState<string[]>([]);
  const [inside, setInside] = useState<Record<string, { shut: boolean; seen: Wired[] }>>({});
  const [known, setKnown] = useState<Wired[] | null>(null);
  const [asked, setAsked] = useState("");
  const [fresh, setFresh] = useState<string[]>([]);
  // Right after its first connection an agent is offered its hooks and Tisty's skills, once.
  const offer = (joined: readonly Wired[]) => {
    const now = joined
      .filter((one) => offers(one) && !offered().includes(one.id))
      .map((one) => one.id);
    if (now.length === 0) return;
    markOffered(now);
    setFresh((was) => [...new Set([...was, ...now])]);
  };
  const offeredNow = (one: Wired) => setFresh((was) => was.filter((id) => id !== one.id));

  const lookHere = () => {
    seenAgents()
      .then(setAgents)
      .catch((e) => fail({ card: "wiring", text: saidPlainly(e) }));
  };

  const look = (name: string) =>
    quietly("wiring", wslSeen(name), (found) => {
      setInside((was) => ({ ...was, [name]: { shut: found.distro.shut, seen: found.seen } }));
      remember([...new Set([...looked(), name])]);
    });

  useEffect(() => {
    wslDistros()
      .then((found) => {
        const all = Array.isArray(found) ? found : [];
        setDistros(all);
        for (const name of wakes ? all.filter((one) => looked().includes(one)) : []) {
          wslSeen(name)
            .then((found) =>
              setInside((was) => ({
                ...was,
                [name]: { shut: found.distro.shut, seen: found.seen },
              })),
            )
            .catch(() => undefined);
        }
      })
      .catch(() => undefined);
  }, [wakes]);

  const place = (one: Wired, now: Wired[]) => {
    if (one.place) {
      const at = one.place;
      setInside((was) => ({ ...was, [at]: { shut: was[at]?.shut ?? false, seen: now } }));
    } else setAgents(now);
  };

  const join = (one: Wired) => {
    const out = one.wired || !!one.apart;
    const work = async () => {
      try {
        return await (out ? unwireAgent(one.id) : wireAgent(one.id));
      } catch (e) {
        if (one.place) look(one.place);
        else lookHere();
        throw e;
      }
    };
    quietly("wiring", work(), (now) => {
      place(one, now);
      if (!out) offer(now.filter((it) => it.id === one.id && it.wired));
      tell({ card: "wiring", text: t(out ? "wiringGone" : "wiringFresh") });
    });
  };

  const said = (all: Outcome[]) => {
    const failed = all.flatMap((one) =>
      (one.failed ?? []).map(({ what, why }) =>
        fill("mendRowFailed", one.name, what ? `${what}: ${why}` : why),
      ),
    );
    const done = all
      .filter((one) => one.done)
      .map((one) => fill(one.wrote ? "mendRowDone" : "mendRowSkills", one.name, one.done));
    const card = "wiring" as const;
    if (failed.length > 0) fail({ card, text: [...done, ...failed].join(" ") });
    else tell({ card, text: done.join(" ") });
  };

  const fix = (one: Wired) =>
    quietly(
      "wiring",
      async () => {
        const mended = await mendOne({ one, parts: allParts(one) });
        const taught = one.place ? [] : await mendSkills(skills ?? null);
        return { seen: mended.seen, told: [mended.said, ...taught] };
      },
      ({ seen, told }) => {
        if (seen) place(one, seen);
        else if (one.place) look(one.place);
        else lookHere();
        learnPlugged?.(bringing(told, true), bringing(told, false));
        said(told);
      },
    );

  const hook = (one: Wired, on: boolean) =>
    quietly("wiring", hooked(one.id, on), (now) => {
      place(one, now);
      if (on) {
        tell({ card: "wiring", text: t("hooksFresh") });
        return;
      }
      markOffered([hooksToken(one.id)]);
      const left = now.find((it) => it.id === one.id)?.hooks;
      if (left?.on) fail({ card: "wiring", text: fill("hooksPuzzling", left.at) });
      else tell({ card: "wiring", text: t(left?.apart ? "hooksGoneApart" : "hooksGone") });
    });

  const waiting = [
    ...(agents ?? []).filter(ready),
    ...Object.values(inside).flatMap((one) => (one.shut ? [] : one.seen.filter(ready))),
  ];
  const all = () =>
    quietly(
      "wiring",
      waiting.reduce<Promise<Wired[]>>(
        (before, one) => before.then(() => wireAgent(one.id)),
        Promise.resolve([]),
      ),
      () => {
        offer(waiting.filter((one) => !one.place));
        tell({ card: "wiring", text: t("wiringFresh") });
        for (const one of waiting) {
          if (one.place) look(one.place);
        }
        if (waiting.some((one) => !one.place)) lookHere();
      },
    );

  // Assistants installed or moved while the window was open show up without closing it.
  const lookAgain = () => {
    lookHere();
    for (const name of Object.keys(inside)) look(name);
  };
  const hand = (one: Wired) => wroteSaid(hands?.find((it) => it.via === one.id));
  const others = hands?.filter((it) => !agents?.some((one) => one.id === it.via)) ?? [];
  const found = (known ?? []).filter(
    (one) => asked.trim().length > 0 && one.name.toLowerCase().includes(asked.trim().toLowerCase()),
  );

  return (
    <>
      <p className="text-[12.5px] leading-relaxed text-soft">{t("wiringWhat")}</p>
      <div className="mt-2 flex flex-wrap items-center gap-2">
        <button
          type="button"
          disabled={held}
          onClick={lookAgain}
          className="shrink-0 rounded-md border border-line px-2.5 py-1 text-[12.5px] hover:bg-hover disabled:text-faint"
        >
          {t("wiringLookAgain")}
        </button>
        {waiting.length > 0 && (
          <button
            type="button"
            disabled={held}
            onClick={all}
            className="shrink-0 rounded-md bg-accent px-2.5 py-1 text-[12.5px] text-bg disabled:bg-hair disabled:text-soft"
          >
            {fill("wiringAll", String(waiting.length))}
          </button>
        )}
      </div>

      {agents?.length === 0 && (
        <p className="mt-3 text-[12.5px] leading-relaxed text-faint">{t("wiringNone")}</p>
      )}
      {agents && agents.length > 0 && (
        <div className="mt-3">
          <Rows
            all={agents}
            hands={hand}
            held={held}
            fresh={fresh}
            onJoin={join}
            onHook={hook}
            onFix={fix}
            onOffered={offeredNow}
            skills={skills}
            onSkills={setSkills}
            behind={skills && skillsBehind(skills) ? foldersBehind(skills) : undefined}
          />
        </div>
      )}

      {distros.map((name) => (
        <section key={name} className="mt-4">
          <span className="block text-[11.5px] font-semibold tracking-[0.06em] text-faint uppercase">
            {fill("wslTitle", name)}
          </span>
          {!inside[name] && (
            <div className="mt-1.5 flex items-center gap-3 rounded-[10px] border border-hair px-3 py-2.5">
              <span className="min-w-0 flex-1 text-[12.5px] text-soft">{t("wslLookWhy")}</span>
              <button type="button" disabled={held} onClick={() => look(name)} className={small}>
                {fill("wslLook", name)}
              </button>
            </div>
          )}
          {inside[name]?.shut && (
            <p className="mt-1.5 text-[12.5px] leading-relaxed text-high">
              {fill("wslShutSaid", name)}
            </p>
          )}
          {inside[name] && inside[name].seen.length === 0 && (
            <p className="mt-1.5 text-[12.5px] text-faint">{fill("wslNone", name)}</p>
          )}
          {inside[name] && inside[name].seen.length > 0 && (
            <div className="mt-1.5">
              <Rows
                all={inside[name].seen}
                held={held || inside[name].shut}
                onJoin={join}
                onHook={hook}
                onFix={fix}
              />
            </div>
          )}
        </section>
      ))}

      <section className="mt-4">
        <label
          htmlFor="wiring-search"
          className="block text-[11.5px] font-semibold tracking-[0.06em] text-faint uppercase"
        >
          {t("wiringSearch")}
        </label>
        <input
          id="wiring-search"
          type="search"
          value={asked}
          placeholder={t("wiringSearchHint")}
          onFocus={() => {
            if (known === null)
              wiringKnown()
                .then(setKnown)
                .catch(() => setKnown([]));
          }}
          onChange={(e) => setAsked(e.target.value)}
          className="mt-1.5 w-full rounded-md border border-line bg-bg px-2.5 py-1.5 text-[12.5px]"
        />
        {found.length > 0 && (
          <div className="mt-2">
            <Rows all={found} held={held} onJoin={join} onHook={hook} onFix={fix} />
          </div>
        )}
      </section>

      {others.length > 0 && (
        <section className="mt-4">
          <span className="block text-[11.5px] font-semibold tracking-[0.06em] text-faint uppercase">
            {t("wiringOthers")}
          </span>
          {others.map((it) => (
            <div
              key={it.via ?? "unnamed"}
              className="mt-1.5 flex items-center gap-3 rounded-[10px] border border-hair px-3 py-2.5"
              title={it.via ? undefined : t("assistantUnnamedWhy")}
            >
              <span className="min-w-0 flex-1">
                <span className="block text-[13px] font-semibold">
                  {it.named || t("assistantUnnamed")}
                </span>
                <span className="block text-[12.5px] text-soft">{wroteSaid(it)}</span>
              </span>
            </div>
          ))}
        </section>
      )}
    </>
  );
}
