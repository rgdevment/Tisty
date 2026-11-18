import { useEffect, useRef, useState } from "react";
import { brainNetwork, type Network } from "../brainsty";
import { numbered } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { brainstyLoaded, type LoadedBrainsty } from "./brainstyLoaded";
import Glyph from "./Glyph";
import { onMac } from "./WindowChrome";

type Read = { network: Network | null } | { failed: string };

interface Props {
  near?: string;
  onClose: () => void;
}

export default function BrainstyScreen({ near, onClose }: Props) {
  const [read, setRead] = useState<Read | null>(null);
  const [Drawn, setDrawn] = useState<LoadedBrainsty | null>(null);
  const frame = useRef<HTMLElement>(null);
  const closing = useRef(onClose);
  closing.current = onClose;

  useEffect(() => {
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
    const before = document.activeElement;
    frame.current?.focus();
    const shut = (e: KeyboardEvent) => {
      if (e.key !== "Escape" || e.defaultPrevented) return;
      e.preventDefault();
      closing.current();
    };
    window.addEventListener("keydown", shut);
    return () => {
      window.removeEventListener("keydown", shut);
      if (before instanceof HTMLElement && before.isConnected) before.focus();
    };
  }, []);

  const network = read && "network" in read ? read.network : null;
  return (
    <section
      ref={frame}
      tabIndex={-1}
      aria-label={t("brainsty")}
      className="fixed inset-0 z-50 flex flex-col bg-bg font-sans text-ink outline-none"
    >
      <header
        data-tauri-drag-region
        className={`flex h-11 shrink-0 items-center gap-3 border-b border-hair pr-36 ${
          onMac ? "pl-[84px]" : "pl-3"
        }`}
      >
        <button
          type="button"
          onClick={onClose}
          title={`${t("goBack")} · Esc`}
          className="flex items-center gap-1.5 rounded-md px-2 py-1 text-[12.5px] text-soft hover:bg-hover hover:text-ink"
        >
          <Glyph name="left" className="h-3.5 w-3.5" />
          {t("goBack")}
        </button>
        <h1 className="flex items-center gap-2 text-[13px] font-semibold">
          <Glyph name="network" className="h-4 w-4 text-accent" />
          {t("brainsty")}
        </h1>
        {network && <Counted network={network} />}
      </header>
      <div className="relative min-h-0 flex-1">
        {read === null || (network && !Drawn) ? (
          <Said text={t("brainstyReading")} />
        ) : "failed" in read ? (
          <Said text={read.failed} urgent />
        ) : network === null ? (
          <Said text={t("brainstyNotYet")} />
        ) : (
          Drawn && <Drawn network={network} />
        )}
      </div>
    </section>
  );
}

function Counted({ network }: { network: Network }) {
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

function Said({ text, urgent = false }: { text: string; urgent?: boolean }) {
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
