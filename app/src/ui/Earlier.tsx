import { useCallback, useEffect, useId, useRef, useState } from "react";
import type { Carded } from "tisty-editor/read";
import {
  awayOf,
  docHistoryEmpty,
  docVersion,
  docVersions,
  type Shelf,
  saidOf,
  type Earlier as Version,
  whenOf,
  writerOf,
} from "../earlier";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { sure } from "../sure";
import { useHeldFocus } from "./heldFocus";
import Page from "./Page";
import { mild, risky, strong } from "./Rows";

interface Props {
  doc: string;
  title: string;
  /** A task's page: going back brings its words, and the task keeps its steps and their boxes. */
  page?: boolean;
  card?: (file: string) => Carded;
  /** Absent where nothing can be written, so its versions are only read. */
  onBack?: (one: Version) => Promise<void>;
  onEmptied?: () => void;
  onClose: () => void;
  onError: (problem: unknown) => void;
}

const ROW = "block w-full rounded-[10px] px-3 py-2 text-left";

function Listed({
  one,
  chosen,
  onPick,
}: Readonly<{ one: Version; chosen: boolean; onPick: () => void }>) {
  const facts = [writerOf(one), weigh(one.bytes)].filter(Boolean).join(" · ");
  if (!one.here)
    return (
      <div aria-disabled="true" className={`${ROW} text-faint`}>
        <span className="block text-[13px] tabular-nums">{whenOf(one)}</span>
        <span className="block text-[11.5px]">{facts}</span>
        <span className="block text-[11.5px] italic">{awayOf(one)}</span>
      </div>
    );
  return (
    <button
      type="button"
      aria-current={chosen}
      onClick={onPick}
      className={`${ROW} hover:bg-hover ${chosen ? "bg-active" : ""}`}
    >
      <span className="block text-[13px] tabular-nums">{whenOf(one)}</span>
      <span className="block text-[11.5px] text-faint">{facts}</span>
    </button>
  );
}

const NO_CARD = (): Carded => ({ kind: "doc", title: "" });

function Shown({
  doc,
  one,
  label,
  card,
  onError,
}: Readonly<{
  doc: string;
  one: Version;
  label: string;
  card?: (file: string) => Carded;
  onError: (problem: unknown) => void;
}>) {
  const [read, setRead] = useState<{ print: string; body: string } | null>(null);
  const failed = useRef(onError);
  failed.current = onError;
  useEffect(() => {
    let live = true;
    docVersion(doc, one.print)
      .then((body) => live && setRead({ print: one.print, body }))
      .catch((e) => live && failed.current(e));
    return () => {
      live = false;
    };
  }, [doc, one.print]);
  if (read?.print !== one.print) return null;
  return (
    <Page
      key={one.print}
      body={read.body}
      label={label}
      card={card ?? NO_CARD}
      onDoc={() => undefined}
    />
  );
}

export default function Earlier({
  doc,
  title,
  page,
  card,
  onBack,
  onEmptied,
  onClose,
  onError,
}: Readonly<Props>) {
  const box = useRef<HTMLDivElement>(null);
  const heading = useId();
  const { wrapped, held } = useHeldFocus(box);
  const [shelf, setShelf] = useState<Shelf | null>(null);
  const [chosen, setChosen] = useState<string | null>(null);
  const [working, setWorking] = useState(false);
  const failed = useRef(onError);
  failed.current = onError;

  const look = useCallback(() => {
    let live = true;
    docVersions(doc)
      .then((all) => {
        if (!live) return;
        setShelf(all);
        setChosen((was) => {
          const kept = all.versions.filter((one) => one.here);
          return kept.some((one) => one.print === was) ? was : (kept[0]?.print ?? null);
        });
      })
      .catch((e) => {
        if (!live) return;
        setShelf({ versions: [], kept: 0 });
        failed.current(e);
      });
    return () => {
      live = false;
    };
  }, [doc]);
  useEffect(look, [look]);

  const kept = (event: React.KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
      return;
    }
    wrapped(event);
  };

  const rows = shelf?.versions ?? null;
  const one = rows?.find((row) => row.print === chosen && row.here);
  const here = rows?.some((row) => row.here) ?? false;

  const back = async () => {
    if (!one || !onBack || working) return;
    const asked = [fill("earlierBackSure", whenOf(one)), page ? t("earlierPageSure") : ""];
    if (!(await sure(asked.filter(Boolean).join(" ")))) return;
    setWorking(true);
    try {
      await onBack(one);
    } catch (e) {
      onError(e);
    } finally {
      setWorking(false);
    }
  };

  const empty = async () => {
    if (working || !(await sure(t("earlierEmptySure")))) return;
    setWorking(true);
    try {
      await docHistoryEmpty(doc);
      onEmptied?.();
      look();
    } catch (e) {
      onError(e);
    } finally {
      setWorking(false);
    }
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby={heading}
      className="fixed inset-0 z-50 flex items-center justify-center bg-veil p-6"
      onKeyDown={kept}
      onBlur={held}
    >
      <div
        ref={box}
        className="flex h-full max-h-[860px] w-full max-w-[1120px] overflow-hidden rounded-[10px] border border-hair bg-bg shadow-lift-tall"
      >
        <div className="flex w-[290px] shrink-0 flex-col border-r border-hair">
          <header className="px-5 pt-5 pb-3">
            <h2 id={heading} className="text-[21px] font-semibold">
              {t(page ? "earlierOfPage" : "earlierTitle")}
            </h2>
            <p className="mt-0.5 truncate text-[12.5px] text-soft">{title}</p>
            <p role="note" className="mt-2.5 text-[11.5px] leading-relaxed text-faint">
              {t("earlierLocal")}
            </p>
          </header>
          {rows?.length === 0 && (
            <p className="px-5 text-[12.5px] leading-relaxed text-soft">{t("earlierNone")}</p>
          )}
          <ul aria-label={t("earlierList")} className="scroller min-h-0 flex-1 px-2 pb-2">
            {rows?.map((row) => (
              <li key={row.print}>
                <Listed
                  one={row}
                  chosen={row.print === chosen}
                  onPick={() => setChosen(row.print)}
                />
              </li>
            ))}
          </ul>
          <footer className="flex items-center justify-between gap-2 border-t border-hair px-3 py-2.5">
            <button
              type="button"
              disabled={!shelf?.kept || working}
              onClick={empty}
              className={risky}
            >
              {t("earlierEmpty")}
            </button>
            <button type="button" onClick={onClose} className={mild}>
              {t("close")}
            </button>
          </footer>
        </div>
        <section className="flex min-w-0 flex-1 flex-col bg-desk">
          <div className="flex min-h-[46px] items-center gap-3 border-b border-hair bg-bg px-5 py-2">
            <span className="min-w-0 flex-1 truncate text-[12.5px] text-soft">
              {one ? saidOf(one) : ""}
            </span>
            {one && onBack && (
              <button type="button" disabled={working} onClick={back} className={strong}>
                {t("earlierBack")}
              </button>
            )}
          </div>
          <div className="relative flex min-h-0 flex-1 flex-col">
            {one ? (
              <Shown doc={doc} one={one} label={title} card={card} onError={onError} />
            ) : (
              rows !== null &&
              rows.length > 0 &&
              !here && <p className="px-8 pt-8 text-[12.5px] text-faint">{t("earlierNoneHere")}</p>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
