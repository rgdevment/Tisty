import { useEffect, useMemo, useRef, useState } from "react";
import { brainNetwork, type Group, type Network, type Neuron, OPENED } from "../brainsty";
import { taskOf } from "../core";
import { type Desk, seeing } from "../desk";
import { numbered } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import type { Escaping } from "./Brainsty";
import { withGroupsAfter } from "./brainsty/grouping";
import Legend from "./brainsty/Legend";
import { brainstyLoaded, type LoadedBrainsty } from "./brainstyLoaded";
import Glyph from "./Glyph";

type Read = { network: Network | null } | { failed: string };

type Hands = Pick<Desk, "setChosen" | "choose" | "opening" | "setError" | "setHere">;

export const openedHere = (
  one: Neuron,
  { setChosen, choose, opening, setError, setHere }: Hands,
) => {
  if (one.kind === "doc" || one.kind === "page") setChosen({ named: "docs", doc: one.id });
  else if (one.kind === "folder" && one.group) {
    setHere(one.group);
    setChosen({ named: "docs" });
  } else if (one.kind === "list" && one.group) setChosen({ named: "lists", list: one.group });
  else if (one.kind === "tag") choose({ named: "tags", tags: [one.title] });
  else if (one.kind === "task")
    taskOf(one.id).then(
      (task) => {
        setChosen(seeing(task));
        opening(task);
      },
      (e: unknown) => setError(saidPlainly(e)),
    );
};

interface Props {
  readonly near?: string;
  readonly groups: readonly Group[];
  readonly onOpen: (neuron: Neuron) => void;
  readonly onLeave: () => void;
}

export default function BrainstyView({ near, groups, onOpen, onLeave }: Props) {
  const [read, setRead] = useState<Read | null>(null);
  const [Drawn, setDrawn] = useState<LoadedBrainsty | null>(null);
  const escaping: Escaping = useRef(null);
  const view = useRef<HTMLElement>(null);
  const leaving = useRef(onLeave);
  leaving.current = onLeave;

  useEffect(() => {
    performance.mark(OPENED);
    let live = true;
    brainNetwork(near).then(
      (network) => live && setRead({ network }),
      (e: unknown) => live && setRead({ failed: saidPlainly(e) }),
    );
    void brainstyLoaded().then((ready) => live && setDrawn(() => ready));
    return () => {
      live = false;
    };
  }, [near]);

  useEffect(() => {
    // Heard after the rest of the window, so a menu or a dialog open over the view shuts first.
    const leave = (e: KeyboardEvent) => {
      if (e.key !== "Escape" || e.defaultPrevented || !meantFor(view.current, e.target)) return;
      e.preventDefault();
      if (!escaping.current?.()) leaving.current();
    };
    window.addEventListener("keydown", leave);
    return () => window.removeEventListener("keydown", leave);
  }, []);

  const network = read && "network" in read ? read.network : null;
  const grown = useMemo(
    () => (network ? withGroupsAfter(network, groups) : null),
    [network, groups],
  );
  const drawn = network !== null && network.nodes.length > 0 && Drawn !== null;
  useEffect(() => {
    if (!drawn) return;
    // Opened from the sidebar, the focus sits on a button the arrows leave alone.
    view.current?.querySelector<HTMLElement>("figure[tabindex]")?.focus({ preventScroll: true });
  }, [drawn]);

  useEffect(() => {
    const room = view.current;
    if (!room || typeof ResizeObserver === "undefined") return;
    let frame = 0;
    // The panel can change width while the window keeps its own, and sigma redraws only on the window's resize.
    const watching = new ResizeObserver(() => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => window.dispatchEvent(new Event("resize")));
    });
    watching.observe(room);
    return () => {
      cancelAnimationFrame(frame);
      watching.disconnect();
    };
  }, []);

  return (
    <section
      ref={view}
      aria-label={t("brainsty")}
      className="flex min-h-0 min-w-0 flex-col bg-bg text-ink"
    >
      <header className="flex h-11 shrink-0 items-center gap-3 border-b border-hair px-4">
        <h1 className="flex items-center gap-2 text-[13px] font-semibold">
          <Glyph name="network" className="h-4 w-4 text-accent" />
          {t("brainsty")}
        </h1>
        {network && <Counted network={network} />}
      </header>
      <div className="relative min-h-0 flex-1">
        <Inside
          read={read}
          grown={grown}
          Drawn={Drawn}
          groups={groups}
          onOpen={onOpen}
          escaping={escaping}
        />
      </div>
    </section>
  );
}

// Escape typed into a field elsewhere in the window, such as the sidebar's search, belongs there.
const meantFor = (view: HTMLElement | null, target: EventTarget | null): boolean =>
  !(target instanceof Element) ||
  target.closest("input, textarea, select, [contenteditable]") === null ||
  (view?.contains(target) ?? false);

function Inside({
  read,
  grown,
  Drawn,
  groups,
  onOpen,
  escaping,
}: {
  readonly read: Read | null;
  readonly grown: Network | null;
  readonly Drawn: LoadedBrainsty | null;
  readonly groups: readonly Group[];
  readonly onOpen: (neuron: Neuron) => void;
  readonly escaping: Escaping;
}) {
  if (read === null) return <Said text={t("brainstyReading")} />;
  if ("failed" in read) return <Said text={read.failed} urgent />;
  if (grown === null) return <Said text={t("brainstyNotYet")} />;
  if (grown.nodes.length === 0) return <Said text={t("brainstyEmpty")} />;
  if (!Drawn) return <Said text={t("brainstyReading")} />;
  return (
    <>
      <Drawn network={grown} groups={groups} onOpen={onOpen} escaping={escaping} />
      <Legend network={grown} groups={groups} />
    </>
  );
}

function Counted({ network }: { readonly network: Network }) {
  const out = network.left_out;
  return (
    <p className="flex min-w-0 items-center gap-3 truncate text-[11.5px] text-faint tabular-nums">
      <span>
        {fill("brainstyCounted", numbered(network.nodes.length), numbered(network.edges.length))}
      </span>
      {out && (
        <span title={t("brainstyLeftOutWhy")}>
          {fill("brainstyLeftOut", numbered(out.nodes), numbered(out.edges))}
        </span>
      )}
    </p>
  );
}

function Said({ text, urgent = false }: { readonly text: string; readonly urgent?: boolean }) {
  return (
    <p
      role={urgent ? "alert" : "status"}
      className={`grid h-full place-items-center px-6 text-center text-[12.5px] ${
        urgent ? "text-urgent" : "text-faint"
      }`}
    >
      {text}
    </p>
  );
}
