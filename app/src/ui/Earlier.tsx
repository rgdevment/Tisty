import { useCallback, useEffect, useId, useRef, useState } from "react";
import { agentHistory, type Copy, copyKey, keyOf, timeline } from "../agentHistory";
import { type Signing, useSigning } from "../crediting";
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
  writerWhy,
} from "../earlier";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { sure } from "../sure";
import EarlierCompared from "./EarlierCompared";
import { CopyBar, CopyListed, CopyShown } from "./EarlierCopies";
import { useHeldFocus } from "./heldFocus";
import type { CardsShown } from "./liveCards";
import Page from "./Page";
import { mild, risky, strong } from "./Rows";

interface Props {
  doc: string;
  title: string;
  /** A task's page: going back brings its words, and the task keeps its steps and their boxes. */
  page?: boolean;
  /** Which live cards a version draws, as the page it belongs to draws them. */
  cardsShown?: CardsShown;
  /** Absent where nothing can be written, so its versions are only read. */
  onBack?: (one: Version) => Promise<void>;
  ready?: () => Promise<unknown>;
  onEmptied?: () => void;
  /** An agent's document: the copies Tisty kept of its files join its versions, and `restores` puts one back. */
  agentFile?: { restores: boolean };
  onClose: () => void;
  onError: (problem: unknown) => void;
}

const ROW = "block w-full rounded-[10px] px-3 py-2 text-left";

function Listed({
  one,
  chosen,
  against,
  onPick,
  onAgainst,
  signing,
}: Readonly<{
  one: Version;
  chosen: boolean;
  against: boolean;
  onPick: () => void;
  onAgainst?: () => void;
  signing?: Signing;
}>) {
  const facts = [writerOf(one, signing), weigh(one.bytes)].filter(Boolean).join(" · ");
  if (!one.here)
    return (
      <div aria-disabled="true" className={`${ROW} text-faint`}>
        <span className="block text-[13px] tabular-nums">{whenOf(one)}</span>
        <span className="block text-[11.5px]" title={writerWhy(one, signing)}>
          {facts}
        </span>
        <span className="block text-[11.5px] italic">{awayOf(one)}</span>
      </div>
    );
  return (
    <div className="flex items-center gap-1">
      <button
        type="button"
        aria-current={chosen}
        onClick={onPick}
        className={`${ROW} hover:bg-hover ${chosen ? "bg-active" : ""}`}
      >
        <span className="block text-[13px] tabular-nums">{whenOf(one)}</span>
        <span className="block text-[11.5px] text-faint" title={writerWhy(one, signing)}>
          {facts}
        </span>
      </button>
      {onAgainst && (
        <button
          type="button"
          aria-pressed={against}
          aria-label={fill("earlierCompareWith", whenOf(one))}
          onClick={onAgainst}
          className={`shrink-0 rounded-md px-2 py-1 text-[11.5px] ${against ? "bg-active text-ink" : "text-faint hover:bg-hover"}`}
        >
          {t("earlierCompareRow")}
        </button>
      )}
    </div>
  );
}

const NOW = "now";
const RIGHT_AWAY = () => Promise.resolve();

function Shown({
  doc,
  one,
  label,
  cardsShown,
  signing,
  onError,
}: Readonly<{
  doc: string;
  one: Version;
  label: string;
  cardsShown?: CardsShown;
  signing?: Signing;
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
      cardsShown={cardsShown}
      signing={signing}
      onDoc={() => undefined}
    />
  );
}

export default function Earlier({
  doc,
  title,
  page,
  cardsShown,
  onBack,
  ready = RIGHT_AWAY,
  onEmptied,
  agentFile,
  onClose,
  onError,
}: Readonly<Props>) {
  const box = useRef<HTMLDivElement>(null);
  const closing = useRef<HTMLButtonElement>(null);
  const comparing = useRef<HTMLButtonElement>(null);
  const heading = useId();
  const { wrapped, held } = useHeldFocus(box);
  const credited = useSigning("inside");
  const signing = page ? credited : undefined;
  const [shelf, setShelf] = useState<Shelf | null>(null);
  const [copies, setCopies] = useState<readonly Copy[]>([]);
  const [chosen, setChosen] = useState<string | null>(null);
  const [against, setAgainst] = useState<string | null>(null);
  const [working, setWorking] = useState(false);
  const failed = useRef(onError);
  failed.current = onError;

  const withCopies = agentFile !== undefined;
  const look = useCallback(() => {
    let live = true;
    const load = (): Promise<Shelf & { copies?: Copy[] }> =>
      withCopies ? agentHistory(doc) : docVersions(doc);
    load()
      .then((all) => {
        if (!live) return;
        const kept = all.versions.filter((one) => one.here);
        const stays = (print: string | null) => kept.some((one) => one.print === print);
        const found = all.copies ?? [];
        const open = timeline(kept, found).map(keyOf);
        setShelf(all);
        setCopies(found);
        setChosen((was) => (was !== null && open.includes(was) ? was : (open[0] ?? null)));
        setAgainst((was) => (was === null || was === NOW || stays(was) ? was : NOW));
      })
      .catch((e) => {
        if (!live) return;
        setShelf({ versions: [], kept: 0 });
        setCopies([]);
        failed.current(e);
      });
    return () => {
      live = false;
    };
  }, [doc, withCopies]);
  useEffect(look, [look]);
  // A version compared with the one read is no comparison: what it says now takes its place.
  useEffect(() => {
    if (against !== null && against === chosen) setAgainst(NOW);
  }, [against, chosen]);

  const kept = (event: React.KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
      return;
    }
    wrapped(event);
  };

  const rows = shelf?.versions ?? null;
  const listed = rows && timeline(rows, copies);
  const copy = copies.find((row) => copyKey(row) === chosen);
  const one = rows?.find((row) => row.print === chosen && row.here);
  const here = rows?.some((row) => row.here) ?? false;
  const other = rows?.find((row) => row.here && row.print === against && row.print !== chosen);
  const [older, newer] =
    one && other && rows && rows.indexOf(other) > rows.indexOf(one) ? [other, one] : [one, other];
  const between =
    older && newer
      ? fill("earlierBetween", whenOf(older), whenOf(newer))
      : one && fill("earlierToNow", whenOf(one));

  const pick = (print: string) => {
    setChosen(print);
    setAgainst((was) => (was === print ? NOW : was));
  };
  const turnAgainst = (print: string) => setAgainst((was) => (was === print ? NOW : print));

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
    const asked = [t("earlierEmptySure"), agentFile ? t("agentHistoryEmptyKeeps") : ""];
    if (working || !(await sure(asked.filter(Boolean).join(" ")))) return;
    setWorking(true);
    try {
      await docHistoryEmpty(doc);
      onEmptied?.();
      look();
      // The button pressed goes disabled, and a page whose focus is on nothing hears no Escape.
      closing.current?.focus();
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
            {agentFile && (
              <p className="mt-1.5 text-[11.5px] leading-relaxed text-faint">
                {t("agentHistoryCopies")}
              </p>
            )}
          </header>
          {listed?.length === 0 && (
            <p className="px-5 text-[12.5px] leading-relaxed text-soft">
              {t(agentFile ? "agentHistoryNone" : "earlierNone")}
            </p>
          )}
          <ul aria-label={t("earlierList")} className="scroller min-h-0 flex-1 px-2 pb-2">
            {listed?.map((row) => (
              <li key={keyOf(row)}>
                {row.kind === "copy" ? (
                  <CopyListed
                    one={row.one}
                    chosen={keyOf(row) === chosen}
                    onPick={() => pick(keyOf(row))}
                  />
                ) : (
                  <Listed
                    one={row.one}
                    chosen={row.one.print === chosen}
                    against={row.one.print === against}
                    onPick={() => pick(row.one.print)}
                    onAgainst={
                      row.one.print === chosen ? undefined : () => turnAgainst(row.one.print)
                    }
                    signing={signing}
                  />
                )}
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
            <button ref={closing} type="button" onClick={onClose} className={mild}>
              {t("close")}
            </button>
          </footer>
        </div>
        <section className="flex min-w-0 flex-1 flex-col bg-desk">
          <div className="flex min-h-[46px] items-center gap-3 border-b border-hair bg-bg px-5 py-2">
            {copy ? (
              <CopyBar
                doc={doc}
                one={copy}
                restores={agentFile?.restores ?? false}
                working={working}
                onWorking={setWorking}
                onBack={look}
                onError={onError}
              />
            ) : (
              <span
                className="min-w-0 flex-1 truncate text-[12.5px] text-soft"
                title={one ? writerWhy(one, signing) : undefined}
              >
                {one ? saidOf(one, signing) : ""}
              </span>
            )}
            {one && (
              <button
                ref={comparing}
                type="button"
                aria-pressed={against !== null}
                onClick={() => setAgainst((was) => (was === null ? NOW : null))}
                className={`${mild} ${against === null ? "" : "bg-active"}`}
              >
                {t("earlierCompare")}
              </button>
            )}
            {one && onBack && (
              <button type="button" disabled={working} onClick={back} className={strong}>
                {t("earlierBack")}
              </button>
            )}
          </div>
          <div className="relative flex min-h-0 flex-1 flex-col">
            {copy ? (
              <CopyShown doc={doc} one={copy} onError={onError} />
            ) : one && older && against !== null ? (
              <EarlierCompared
                doc={doc}
                was={older.print}
                now={newer?.print ?? null}
                between={between ?? ""}
                label={title}
                cardsShown={cardsShown}
                signing={signing}
                ready={ready}
                onWhole={() => {
                  setAgainst(null);
                  comparing.current?.focus();
                }}
                onError={onError}
              />
            ) : one ? (
              <Shown
                doc={doc}
                one={one}
                label={title}
                cardsShown={cardsShown}
                signing={signing}
                onError={onError}
              />
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
