import { useEffect, useRef, useState } from "react";
import { agentHeld, agentRestore, fencedFile } from "../agentDocs";
import type { Copy } from "../agentHistory";
import { AGENTS_CHANGED } from "../agentsSync";
import { stamped, weigh } from "../format";
import { fill, t } from "../locales";
import { sure } from "../sure";
import Page from "./Page";
import { strong } from "./Rows";

const ROW = "block w-full rounded-[10px] px-3 py-2 text-left";

export const copySaid = (one: Copy): string =>
  fill(one.pinned ? "agentCopyOriginal" : "agentCopyOf", one.named);

const factsOf = (one: Copy): string =>
  [
    one.when ? stamped(one.when) : fill("agentCopyNumber", String(one.n)),
    weigh(one.bytes),
    one.pinned && t("agentCopyStays"),
  ]
    .filter(Boolean)
    .join(" · ");

export function CopyListed({
  one,
  chosen,
  onPick,
}: Readonly<{ one: Copy; chosen: boolean; onPick: () => void }>) {
  return (
    <button
      type="button"
      aria-current={chosen}
      onClick={onPick}
      className={`${ROW} hover:bg-hover ${chosen ? "bg-active" : ""}`}
    >
      <span className="block text-[13px]">{copySaid(one)}</span>
      <span className="block text-[11.5px] text-faint tabular-nums">{factsOf(one)}</span>
    </button>
  );
}

export function CopyBar({
  doc,
  one,
  restores,
  working,
  onWorking,
  onBack,
  onError,
}: Readonly<{
  doc: string;
  one: Copy;
  restores: boolean;
  working: boolean;
  onWorking: (working: boolean) => void;
  onBack: () => void;
  onError: (problem: unknown) => void;
}>) {
  const back = async () => {
    if (working || !(await sure(fill("agentRestoreSure", one.at)))) return;
    onWorking(true);
    try {
      await agentRestore(doc, one.at, one.n);
      window.dispatchEvent(new Event(AGENTS_CHANGED));
      onBack();
    } catch (e) {
      onError(e);
    } finally {
      onWorking(false);
    }
  };
  return (
    <>
      <span className="min-w-0 flex-1 truncate text-[12.5px] text-soft">{factsOf(one)}</span>
      {restores && (
        <button type="button" disabled={working} onClick={back} className={strong}>
          {t("agentCopyBack")}
        </button>
      )}
    </>
  );
}

export function CopyShown({
  doc,
  one,
  onError,
}: Readonly<{ doc: string; one: Copy; onError: (problem: unknown) => void }>) {
  const key = `${one.at}#${one.n}`;
  const [read, setRead] = useState<{ key: string; text: string } | null>(null);
  const failed = useRef(onError);
  failed.current = onError;
  useEffect(() => {
    let live = true;
    agentHeld(doc, one.at, one.n)
      .then((text) => live && setRead({ key, text }))
      .catch((e) => live && failed.current(e));
    return () => {
      live = false;
    };
  }, [doc, one.at, one.n, key]);
  if (read?.key !== key) return null;
  return (
    <Page key={key} body={fencedFile(read.text, one.at)} label={one.at} onDoc={() => undefined} />
  );
}
