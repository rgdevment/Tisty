import { useEffect, useMemo, useState } from "react";
import type { Filed, Folded } from "../core";
import { type DocFacts, docFacts, type Paper } from "../core";
import { fill, t } from "../locales";
import Glyph from "./Glyph";
import PaperFacts from "./PaperFacts";
import Properties from "./Properties";
import type { Head } from "./paging";
import { Knob } from "./Rows";
import { LEAVES, leafWord } from "./sheets";
import { often } from "./tagging";

export const trailed = (folders: Folded[], at: string | null): Folded[] => {
  const by = new Map(folders.map((one) => [one.id, one]));
  const walk: Folded[] = [];
  const seen = new Set<string>();
  for (let up = at; up && !seen.has(up); ) {
    seen.add(up);
    const one = by.get(up);
    if (!one) break;
    walk.unshift(one);
    up = one.parent;
  }
  return walk;
};

const mild =
  "rounded-[10px] border border-line px-2 py-1 text-[11.5px] text-soft hover:bg-hover hover:text-ink disabled:border-hair disabled:text-faint";

interface Props {
  title: string;
  paper: string;
  body: string;
  tags: string[];
  kept: number;
  heads: Head[];
  pages: Filed[];
  onPage?: (doc: Filed) => void;
  leaf: Paper;
  onLeaf: (leaf: Paper) => void;
  sheets?: boolean;
  landscape?: boolean;
  onSheets?: (on: boolean) => void;
  onLandscape?: (turned: boolean) => void;
  making: boolean;
  onPdf: () => void;
  onSee: () => void;
  onCopy: () => void;
  onTakeOut: () => void;
  onShut: () => void;
  onApart: () => void;
  trail?: Folded[];
  onFolder?: (id: string | null) => void;
}

export default function Beside({
  title,
  paper,
  body,
  tags,
  kept,
  heads,
  pages,
  onPage,
  leaf,
  onLeaf,
  sheets = false,
  landscape = false,
  onSheets,
  onLandscape,
  making,
  onPdf,
  onSee,
  onCopy,
  onTakeOut,
  onShut,
  onApart,
  trail = [],
  onFolder,
}: Props) {
  const [facts, setFacts] = useState<DocFacts | null>(null);

  useEffect(() => {
    let gone = false;
    docFacts(paper)
      .then((now) => {
        if (!gone) setFacts(now);
      })
      .catch(() => {});
    return () => {
      gone = true;
    };
  }, [paper, kept]);

  const times = useMemo(() => often(body), [body]);
  const outline = heads.filter((one) => one.level > 0);

  return (
    <aside
      data-beside=""
      aria-label={t("aboutPaper")}
      className="absolute top-11 right-3 bottom-3 flex w-[320px] flex-col overflow-hidden rounded-[10px] border border-hair bg-panel shadow-lift"
    >
      <div className="flex items-start gap-2 px-4 pt-3.5">
        <div className="min-w-0 flex-1">
          <h2 className="truncate text-[13px] font-semibold">{title || t("untitledDoc")}</h2>
          <p className="mt-0.5 truncate text-[11.5px] text-faint">
            <button
              type="button"
              onClick={() => onFolder?.(null)}
              className="hover:text-ink hover:underline"
            >
              {t("everyPaper")}
            </button>
            {trail.map((one) => (
              <span key={one.id}>
                <span aria-hidden="true"> / </span>
                <button
                  type="button"
                  onClick={() => onFolder?.(one.id)}
                  className="hover:text-ink hover:underline"
                >
                  {one.name}
                </button>
              </span>
            ))}
          </p>
        </div>
        <button
          type="button"
          onClick={onShut}
          title={t("besideShut")}
          aria-label={t("besideShut")}
          className="grid h-5.5 w-5.5 shrink-0 place-items-center rounded-md text-[11.5px] text-faint hover:bg-hover hover:text-ink"
        >
          <span aria-hidden="true">✕</span>
        </button>
      </div>

      <div className="scroller flex flex-1 flex-col gap-5 px-4 pt-3.5 pb-5">
        <section className="flex flex-col gap-2">
          <h3 className="text-[10.5px] tracking-[0.07em] text-faint uppercase">
            {t("aboutPaper")}
          </h3>
          <PaperFacts facts={facts} body={body} />
        </section>

        <Properties
          body={body}
          onEdit={heads.find((one) => one.level === 0)?.go}
          skip={["title", "tags", "tag"]}
        />

        {tags.length > 0 && (
          <section className="flex flex-col gap-2">
            <h3 className="text-[10.5px] tracking-[0.07em] text-faint uppercase">{t("tags")}</h3>
            <div className="flex flex-wrap gap-1">
              {tags.map((one) => {
                const many = times.get(one) ?? 0;
                return (
                  <span
                    key={one}
                    title={many > 0 ? fill("tagTimes", String(many)) : undefined}
                    className="inline-flex items-baseline gap-1 rounded-full bg-mark-tag px-2 py-0.5 text-[11.5px] text-ink"
                  >
                    #{one}
                    {many > 0 && <span className="tabular-nums text-faint">{many}</span>}
                  </span>
                );
              })}
            </div>
          </section>
        )}

        <section className="flex flex-col gap-2">
          <h3 className="text-[10.5px] tracking-[0.07em] text-faint uppercase">{t("leafIs")}</h3>
          <div className="flex gap-1">
            {LEAVES.map((one) => (
              <button
                key={one}
                type="button"
                aria-pressed={leaf === one}
                onClick={() => onLeaf(one)}
                className={`flex-1 rounded-[10px] border px-2 py-1 text-[11.5px] ${
                  leaf === one
                    ? "border-ink bg-ink text-bg"
                    : "border-line text-faint hover:text-soft"
                }`}
              >
                {t(leafWord(one))}
              </button>
            ))}
          </div>
          {onSheets && (
            <div className="flex items-center gap-2 text-[11.5px] text-faint">
              <Knob on={sheets} label={t("sheetsShown")} onPress={() => onSheets(!sheets)} />
              <span>{t("sheetsShown")}</span>
              {sheets && onLandscape && (
                <span className="ml-auto flex gap-2">
                  {[false, true].map((turned) => (
                    <button
                      key={String(turned)}
                      type="button"
                      aria-pressed={landscape === turned}
                      onClick={() => onLandscape(turned)}
                      className={landscape === turned ? "text-ink" : "hover:text-soft"}
                    >
                      {t(turned ? "sheetsAcross" : "sheetsUpright")}
                    </button>
                  ))}
                </span>
              )}
            </div>
          )}
        </section>

        <section className="flex flex-col gap-2">
          <h3 className="text-[10.5px] tracking-[0.07em] text-faint uppercase">{t("bandApart")}</h3>
          <button type="button" onClick={onApart} className={mild}>
            {t("viewDocApart")}
          </button>
        </section>

        <section className="flex flex-col gap-2">
          <h3 className="text-[10.5px] tracking-[0.07em] text-faint uppercase">{t("bandPdf")}</h3>
          <div className="flex gap-1">
            <button
              type="button"
              disabled={making}
              onClick={onSee}
              aria-busy={making}
              className={`${mild} flex-1`}
            >
              {making ? t("makingPdf") : t("seePdf")}
            </button>
            <button type="button" disabled={making} onClick={onPdf} className={`${mild} flex-1`}>
              {t("exportIt")}
            </button>
          </div>
        </section>

        <section className="flex flex-col gap-2">
          <h3 className="text-[10.5px] tracking-[0.07em] text-faint uppercase">{t("bandText")}</h3>
          <div className="flex gap-1">
            <button type="button" onClick={onCopy} className={`${mild} flex-1`}>
              {t("copyIt")}
            </button>
            <button type="button" onClick={onTakeOut} className={`${mild} flex-1`}>
              {t("saveCopy")}
            </button>
          </div>
        </section>

        <section className="flex flex-col gap-2">
          <h3 className="text-[10.5px] tracking-[0.07em] text-faint uppercase">{t("outline")}</h3>
          {outline.length === 0 && pages.length === 0 ? (
            <p className="text-[12.5px] text-faint">{t("outlineNone")}</p>
          ) : (
            <nav className="flex flex-col gap-px">
              {outline.map((one) => (
                <button
                  key={one.key}
                  type="button"
                  onClick={one.go}
                  className={`truncate rounded-md px-1.5 py-1 text-left text-[12.5px] hover:bg-hover hover:text-ink ${
                    one.level > 1 ? "pl-5 text-faint" : "text-soft"
                  }`}
                >
                  {one.text}
                </button>
              ))}
              {pages.length > 0 && (
                <h4 className="mt-2.5 px-1.5 text-[10.5px] tracking-[0.05em] text-faint uppercase">
                  {pages.length === 1 ? t("pageHeld") : fill("pagesHeld", String(pages.length))}
                </h4>
              )}
              {pages.map((one) => (
                <button
                  key={one.id}
                  type="button"
                  onClick={() => onPage?.(one)}
                  aria-label={
                    one.away
                      ? `${one.title || t("untitledDoc")} — ${t("isArchived")}`
                      : one.flagged
                        ? `${one.title || t("untitledDoc")} — ${t("docFlagged")}`
                        : undefined
                  }
                  aria-current={one.file === paper ? "true" : undefined}
                  className={`flex items-center gap-2 rounded-md px-1.5 py-1 text-left text-[12.5px] hover:bg-hover hover:text-ink ${
                    one.file === paper ? "bg-accent-soft text-accent" : ""
                  } ${one.away ? "text-faint" : "text-soft"}`}
                >
                  <Glyph
                    name={one.archived ? "archive" : "alignleft"}
                    className="h-[13px] w-[13px] shrink-0"
                  />
                  <span className="min-w-0 flex-1 truncate">{one.title || t("untitledDoc")}</span>
                  {one.flagged && !one.away && (
                    <span aria-hidden="true" className="shrink-0 text-[9px] text-hue-teal">
                      ◆
                    </span>
                  )}
                </button>
              ))}
            </nav>
          )}
        </section>
      </div>
    </aside>
  );
}
