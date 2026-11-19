import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";
import {
  brainNetwork,
  brainstyGroups,
  type Group,
  type Network,
  type Neuron,
  OPENED,
  openInMain,
  SPOKEN,
} from "./brainsty";
import { numbered } from "./format";
import { adopt, fill, locale, t } from "./locales";
import { saidPlainly } from "./refusal";
import type { Escaping } from "./ui/Brainsty";
import { brainstyLoaded, type LoadedBrainsty } from "./ui/brainstyLoaded";
import Glyph from "./ui/Glyph";

type Read = { network: Network | null; groups: readonly Group[] } | { failed: string };

const spoken = (code?: string) => {
  adopt(code);
  document.documentElement.lang = locale();
};

export default function BrainstyWindow() {
  const [asked] = useState(() => new URLSearchParams(window.location.search));
  useState(() => spoken(asked.get("locale") ?? undefined));
  const [, setRespoken] = useState(0);
  const [read, setRead] = useState<Read | null>(null);
  const [Drawn, setDrawn] = useState<LoadedBrainsty | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);
  const escaping: Escaping = useRef(null);
  const open = (one: Neuron) => {
    setTrouble(null);
    openInMain(one).catch((e: unknown) => setTrouble(saidPlainly(e)));
  };

  useEffect(
    () =>
      void getCurrentWindow()
        .show()
        .catch(() => {}),
    [],
  );

  useEffect(() => {
    const heard = listen<string | null>(SPOKEN, ({ payload }) => {
      spoken(payload ?? undefined);
      setRespoken((times) => times + 1);
    });
    return () => {
      heard.then((off) => off()).catch(() => {});
    };
  }, []);

  useEffect(() => {
    performance.mark(OPENED);
    let live = true;
    Promise.all([brainNetwork(asked.get("near") ?? undefined), brainstyGroups()]).then(
      ([network, groups]) => live && setRead({ network, groups }),
      (e: unknown) => live && setRead({ failed: saidPlainly(e) }),
    );
    void brainstyLoaded().then((ready) => live && setDrawn(() => ready));
    return () => {
      live = false;
    };
  }, [asked]);

  useEffect(() => {
    const leave = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      e.preventDefault();
      e.stopImmediatePropagation();
      if (!escaping.current?.()) void getCurrentWindow().close();
    };
    window.addEventListener("keydown", leave, true);
    return () => window.removeEventListener("keydown", leave, true);
  }, []);

  useEffect(() => {
    const stay = (e: MouseEvent) => {
      if (e.target instanceof Element && e.target.closest("a[href]")) e.preventDefault();
    };
    for (const kind of ["click", "auxclick"] as const) document.addEventListener(kind, stay, true);
    return () => {
      for (const kind of ["click", "auxclick"] as const)
        document.removeEventListener(kind, stay, true);
    };
  }, []);

  const network = read && "network" in read ? read.network : null;
  return (
    <main
      aria-label={t("brainsty")}
      className="flex h-full flex-col bg-bg font-sans text-ink outline-none"
    >
      <header className="flex h-11 shrink-0 items-center gap-3 border-b border-hair px-3">
        <h1 className="flex items-center gap-2 text-[13px] font-semibold">
          <Glyph name="network" className="h-4 w-4 text-accent" />
          {t("brainsty")}
        </h1>
        {network && <Counted network={network} />}
        {trouble && (
          <p role="alert" className="ml-auto truncate text-[11.5px] text-urgent">
            {trouble}
          </p>
        )}
      </header>
      <div className="relative min-h-0 flex-1">
        <Inside read={read} Drawn={Drawn} onOpen={open} escaping={escaping} />
      </div>
    </main>
  );
}

function Inside({
  read,
  Drawn,
  onOpen,
  escaping,
}: {
  readonly read: Read | null;
  readonly Drawn: LoadedBrainsty | null;
  readonly onOpen: (neuron: Neuron) => void;
  readonly escaping: Escaping;
}) {
  if (read === null) return <Said text={t("brainstyReading")} />;
  if ("failed" in read) return <Said text={read.failed} urgent />;
  const { network, groups } = read;
  if (network === null) return <Said text={t("brainstyNotYet")} />;
  if (network.nodes.length === 0) return <Said text={t("brainstyEmpty")} />;
  if (!Drawn) return <Said text={t("brainstyReading")} />;
  return <Drawn network={network} groups={groups} onOpen={onOpen} escaping={escaping} />;
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
