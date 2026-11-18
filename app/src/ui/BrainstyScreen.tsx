import { useEffect, useRef, useState } from "react";
import { brainNetwork, type Network, OPENED } from "../brainsty";
import { numbered } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { brainstyLoaded, type LoadedBrainsty } from "./brainstyLoaded";
import Glyph from "./Glyph";
import { onMac } from "./WindowChrome";

type Read = { network: Network | null } | { failed: string };

interface Props {
  readonly near?: string;
  readonly onClose: () => void;
}

export default function BrainstyScreen({ near, onClose }: Props) {
  const [read, setRead] = useState<Read | null>(null);
  const [Drawn, setDrawn] = useState<LoadedBrainsty | null>(null);
  const frame = useRef<HTMLDialogElement>(null);
  const closing = useRef(onClose);
  closing.current = onClose;

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
    const before = document.activeElement;
    frame.current?.focus();
    // Heard first, so the view this layer covers never acts on a key meant for it.
    const kept = (e: KeyboardEvent) => {
      const topmost = [...document.querySelectorAll('[aria-modal="true"]')].pop();
      if (topmost !== frame.current) return;
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopImmediatePropagation();
        closing.current();
      } else if (e.ctrlKey || e.metaKey) {
        e.stopImmediatePropagation();
      }
    };
    window.addEventListener("keydown", kept, true);
    return () => {
      window.removeEventListener("keydown", kept, true);
      if (before instanceof HTMLElement && before.isConnected) before.focus();
    };
  }, []);

  const network = read && "network" in read ? read.network : null;
  return (
    <dialog
      open
      ref={frame}
      tabIndex={-1}
      aria-modal="true"
      aria-label={t("brainsty")}
      className="fixed inset-0 z-50 m-0 flex h-full max-h-none w-full max-w-none flex-col border-0 bg-bg p-0 font-sans text-ink outline-none"
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
        <Inside read={read} Drawn={Drawn} />
      </div>
    </dialog>
  );
}

function Inside({
  read,
  Drawn,
}: {
  readonly read: Read | null;
  readonly Drawn: LoadedBrainsty | null;
}) {
  if (read === null) return <Said text={t("brainstyReading")} />;
  if ("failed" in read) return <Said text={read.failed} urgent />;
  const { network } = read;
  if (network === null) return <Said text={t("brainstyNotYet")} />;
  if (network.nodes.length === 0) return <Said text={t("brainstyEmpty")} />;
  if (!Drawn) return <Said text={t("brainstyReading")} />;
  return <Drawn network={network} />;
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
