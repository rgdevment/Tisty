import { useEffect, useState } from "react";
import {
  type Assistant,
  copied,
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

const ready = (one: Wired) => !one.wired && !one.astray && !one.byHand && !one.maybe;

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

function Row({
  one,
  hand,
  held,
  onJoin,
}: Readonly<{ one: Wired; hand?: string; held: boolean; onJoin: (one: Wired) => void }>) {
  const [open, setOpen] = useState(false);
  const state = one.byHand
    ? "wiringStateHand"
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
    : one.wired
      ? "wiringOut"
      : one.maybe
        ? "wiringAnyway"
        : "wiringJoin";
  const detail = one.astray
    ? fill(one.gone ? "wiringPointsGone" : "wiringPoints", one.points ?? "")
    : one.maybe && !one.wired
      ? t("wiringMaybe")
      : hand;
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
        {!one.byHand && (
          <button
            type="button"
            disabled={held}
            onClick={() => onJoin(one)}
            className={`shrink-0 rounded-md border px-2.5 py-1 text-[12.5px] disabled:text-faint ${
              one.wired && !one.astray
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
          ⋯
        </button>
      </div>
      {open && <Manual one={one} />}
    </div>
  );
}

function Rows({
  all,
  hands,
  held,
  onJoin,
}: Readonly<{
  all: Wired[];
  hands?: (one: Wired) => string | undefined;
  held: boolean;
  onJoin: (one: Wired) => void;
}>) {
  return (
    <div className="overflow-hidden rounded-[10px] border border-hair">
      {all.map((one) => (
        <Row key={one.id} one={one} hand={hands?.(one)} held={held} onJoin={onJoin} />
      ))}
    </div>
  );
}

interface Props {
  agents: Wired[] | null;
  setAgents: (all: Wired[]) => void;
  hands: Assistant[] | null;
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
        for (const name of all.filter((one) => looked().includes(one))) {
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
  }, []);

  const place = (one: Wired, now: Wired[]) => {
    if (one.place) {
      const at = one.place;
      setInside((was) => ({ ...was, [at]: { shut: was[at]?.shut ?? false, seen: now } }));
    } else setAgents(now);
  };

  const join = (one: Wired) => {
    const out = one.wired && !one.astray;
    quietly("wiring", out ? unwireAgent(one.id) : wireAgent(one.id), (now) => {
      place(one, now);
      tell({ card: "wiring", text: t(out ? "wiringGone" : "wiringFresh") });
    });
  };

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
        tell({ card: "wiring", text: t("wiringFresh") });
        for (const one of waiting) {
          if (one.place) look(one.place);
        }
        if (waiting.some((one) => !one.place)) {
          seenAgents()
            .then(setAgents)
            .catch((e) => fail({ card: "wiring", text: saidPlainly(e) }));
        }
      },
    );

  const astray = [
    ...(agents ?? []).filter((one) => one.astray),
    ...Object.values(inside).flatMap((one) => (one.shut ? [] : one.seen.filter((it) => it.astray))),
  ];
  // Assistants installed or moved while the window was open show up without closing it.
  const lookAgain = () => {
    seenAgents()
      .then(setAgents)
      .catch((e) => fail({ card: "wiring", text: saidPlainly(e) }));
    for (const name of Object.keys(inside)) look(name);
  };
  const repair = () =>
    quietly(
      "wiring",
      astray.reduce<Promise<Wired[]>>(
        (before, one) => before.then(() => wireAgent(one.id)),
        Promise.resolve([]),
      ),
      () => {
        tell({ card: "wiring", text: t("wiringRepaired") });
        lookAgain();
      },
    );

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
        {astray.length > 0 && (
          <button
            type="button"
            disabled={held}
            onClick={repair}
            className="shrink-0 rounded-md border border-hue-amber/40 px-2.5 py-1 text-[12.5px] text-hue-amber disabled:text-faint"
          >
            {fill("wiringRepair", String(astray.length))}
          </button>
        )}
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
          <Rows all={agents} hands={hand} held={held} onJoin={join} />
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
              <Rows all={inside[name].seen} held={held || inside[name].shut} onJoin={join} />
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
            <Rows all={found} held={held} onJoin={join} />
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
