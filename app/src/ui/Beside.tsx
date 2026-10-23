import { useEffect, useMemo, useRef, useState } from "react";
import type { Filed, Folded } from "../core";
import { type DocFacts, docFacts, type Paper } from "../core";
import { weigh } from "../format";
import { waysOut } from "../leaving";
import { fill, t } from "../locales";
import Glyph from "./Glyph";
import Menu, { type Choice } from "./Menu";
import PaperFacts, { held, worded } from "./PaperFacts";
import Properties from "./Properties";
import type { Head } from "./paging";
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

// An outline this short leaves room, so the details open with the document.
const SHORT_OUTLINE = 6;

const card = "flex flex-col overflow-hidden rounded-[10px] border border-hair bg-panel shadow-lift";
const tool =
  "grid h-6 w-6 shrink-0 place-items-center rounded-md text-faint hover:bg-hover hover:text-ink";
const control =
  "flex h-7 items-center rounded-md border border-line text-[11.5px] text-faint hover:text-ink";
const label = "text-[10.5px] tracking-[0.07em] text-faint uppercase";
const row =
  "flex shrink-0 items-center gap-2 rounded-md px-1.5 py-1 text-left text-[12.5px] hover:bg-hover hover:text-ink";

// A page put away or marked for deletion says so to a screen reader, where the eye sees its glyph.
const marked = (one: Filed): string | undefined => {
  const named = one.title || t("untitledDoc");
  if (one.away) return `${named} — ${t("isArchived")}`;
  if (one.flagged) return `${named} — ${t("docFlagged")}`;
  return undefined;
};

type Part = "details" | "shared";

interface Props {
  title: string;
  paper: string;
  body: string;
  tags: string[];
  kept: number;
  heads: Head[];
  pages: Filed[];
  /** The document a page belongs to, so its column can take the person back. */
  up?: Filed;
  onPage?: (doc: Filed) => void;
  leaf: Paper;
  onLeaf: (leaf: Paper) => void;
  sheets?: boolean;
  landscape?: boolean;
  onSheets?: (on: boolean) => void;
  onLandscape?: (turned: boolean) => void;
  making: boolean;
  onCopy: () => void;
  onTakeOut: () => void;
  onShut: () => void;
  onApart: () => void;
  trail?: Folded[];
  onFolder?: (id: string | null) => void;
}

function Fold({
  title,
  gist,
  open,
  onToggle,
  children,
}: Readonly<{
  title: string;
  gist: string;
  open: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}>) {
  return (
    <section className="flex min-h-0 flex-col border-hair not-first:border-t">
      <button
        type="button"
        aria-expanded={open}
        onClick={onToggle}
        className="flex items-center gap-2 px-3.5 py-2.5 text-left text-[12.5px] hover:bg-hover"
      >
        <span className="font-semibold text-ink">{title}</span>
        {!open && <span className="min-w-0 truncate text-[11.5px] text-faint">{gist}</span>}
        <Glyph
          name="chevron"
          className={`ml-auto h-3 w-3 shrink-0 text-faint transition-transform motion-reduce:transition-none ${
            open ? "" : "-rotate-90"
          }`}
        />
      </button>
      {open && (
        <div className="scroller flex max-h-[52vh] flex-col gap-3.5 px-4 pt-0.5 pb-4">
          {children}
        </div>
      )}
    </section>
  );
}

function Controls({
  leaf,
  onLeaf,
  sheets,
  landscape,
  onSheets,
  onLandscape,
  making,
  onCopy,
  onTakeOut,
}: Readonly<
  Pick<
    Props,
    "leaf" | "onLeaf" | "onSheets" | "onLandscape" | "making" | "onCopy" | "onTakeOut"
  > & { sheets: boolean; landscape: boolean }
>) {
  const [out, setOut] = useState<{ x: number; y: number } | null>(null);
  const ways: Choice[] = [
    ...waysOut(false, making),
    { key: "copy", icon: "⧉", label: t("copyPlain"), apart: true, onPick: onCopy },
    { key: "save", icon: "⇩", label: t("saveAsMarkdown"), onPick: onTakeOut },
  ];
  const toggled = (box: DOMRect) => setOut(out ? null : { x: box.right - 230, y: box.bottom + 4 });

  return (
    <>
      <div className="mt-2.5 flex items-stretch gap-1.5">
        <fieldset
          aria-label={t("leafIs")}
          title={t("leafIs")}
          className="m-0 flex h-7 min-w-0 flex-1 gap-0.5 rounded-md border border-line p-0.5"
        >
          {LEAVES.map((one) => (
            <button
              key={one}
              type="button"
              aria-pressed={leaf === one}
              onClick={() => onLeaf(one)}
              className={`flex-1 rounded-md text-[11.5px] ${
                leaf === one ? "bg-active text-ink" : "text-faint hover:text-soft"
              }`}
            >
              {t(leafWord(one))}
            </button>
          ))}
        </fieldset>
        {onSheets && (
          <button
            type="button"
            role="switch"
            aria-checked={sheets}
            aria-label={t("sheetsShown")}
            title={t("sheetsShown")}
            onClick={() => onSheets(!sheets)}
            className={`${control} w-7 justify-center ${
              sheets ? "border-accent/40 bg-active text-ink" : "hover:bg-hover"
            }`}
          >
            <Glyph name="pages" className={`h-3.5 w-3.5 ${sheets ? "text-accent" : ""}`} />
          </button>
        )}
        <button
          type="button"
          aria-haspopup="menu"
          aria-expanded={out !== null}
          aria-busy={making}
          aria-label={t("exportIt")}
          title={making ? t("makingPdf") : t("exportIt")}
          // The menu shuts on any mousedown outside it, so the mouse toggles here and onClick serves the keyboard.
          onMouseDown={(e) => {
            if (e.button !== 0) return;
            e.preventDefault();
            toggled(e.currentTarget.getBoundingClientRect());
          }}
          onClick={(e) => {
            if (e.detail === 0) toggled(e.currentTarget.getBoundingClientRect());
          }}
          className={`${control} gap-0.5 px-2 hover:bg-hover`}
        >
          <Glyph name="download" className="h-3.5 w-3.5" />
          <Glyph name="chevron" className="h-2.5 w-2.5" />
        </button>
      </div>
      {sheets && onLandscape && (
        <div className="mt-1.5 flex justify-end gap-3 text-[11.5px] text-faint">
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
        </div>
      )}
      {out && <Menu at={out} choices={ways} label={t("exportIt")} onClose={() => setOut(null)} />}
    </>
  );
}

function Outline({
  outline,
  pages,
  paper,
  up,
  onPage,
}: Readonly<{ outline: Head[] } & Pick<Props, "pages" | "paper" | "up" | "onPage">>) {
  return (
    <nav
      aria-label={t("outline")}
      className="scroller flex min-h-0 flex-1 flex-col gap-px border-t border-hair px-2.5 pt-2.5 pb-3"
    >
      <h3 className={`${label} shrink-0 px-1.5 pb-1`}>{t("outline")}</h3>
      {outline.length === 0 && pages.length === 0 && !up && (
        <p className="px-1.5 text-[12.5px] text-faint">{t("outlineNone")}</p>
      )}
      {outline.map((one) => (
        <button
          key={one.key}
          type="button"
          onClick={one.go}
          className={`${row} block truncate ${one.level > 1 ? "pl-5 text-faint" : "text-soft"}`}
        >
          {one.text}
        </button>
      ))}
      {up && (
        <>
          <h4 className={`${label} mt-2.5 shrink-0 px-1.5`}>{t("besidePageOf")}</h4>
          <button type="button" onClick={() => onPage?.(up)} className={`${row} text-soft`}>
            <Glyph name="chevron" className="h-[13px] w-[13px] shrink-0 rotate-90" />
            <span className="min-w-0 flex-1 truncate">{up.title || t("untitledDoc")}</span>
          </button>
        </>
      )}
      {pages.length > 0 && (
        <h4 className={`${label} mt-2.5 shrink-0 px-1.5`}>{held(pages.length)}</h4>
      )}
      {pages.map((one) => (
        <button
          key={one.id}
          type="button"
          onClick={() => onPage?.(one)}
          aria-label={marked(one)}
          aria-current={one.file === paper ? "true" : undefined}
          className={`${row} ${one.file === paper ? "bg-accent-soft text-accent" : ""} ${
            one.away ? "text-faint" : "text-soft"
          }`}
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
  );
}

function Tags({ tags, body }: Readonly<{ tags: string[]; body: string }>) {
  const times = useMemo(() => often(body), [body]);
  if (tags.length === 0) return null;
  return (
    <section className="flex flex-col gap-2">
      <h3 className={label}>{t("tags")}</h3>
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
  );
}

export default function Beside(props: Props) {
  const { title, paper, body, kept, heads, pages, onShut, onApart, trail = [], onFolder } = props;
  const outline = useMemo(() => heads.filter((one) => one.level > 0), [heads]);
  const short = outline.length + pages.length <= SHORT_OUTLINE;
  const [facts, setFacts] = useState<DocFacts | null>(null);
  const [open, setOpen] = useState<Part | null>(short ? "details" : null);

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

  // The outline arrives after the page is drawn, so the choice waits for it until the person makes one.
  const chosen = useRef(false);
  const showing = useRef(paper);
  useEffect(() => {
    if (showing.current !== paper) {
      showing.current = paper;
      chosen.current = false;
    }
    if (!chosen.current) setOpen(short ? "details" : null);
  }, [paper, short]);

  const words = useMemo(() => worded(body), [body]);
  const gist = [
    words ? fill("paperWords", String(words)) : t("paperEmpty"),
    facts ? weigh(facts.bytes) : "",
    facts?.pages ? held(facts.pages) : "",
  ]
    .filter(Boolean)
    .join(" · ");
  const toggle = (part: Part) => {
    chosen.current = true;
    setOpen((now) => (now === part ? null : part));
  };

  return (
    <aside
      data-beside=""
      aria-label={t("aboutPaper")}
      className="absolute top-11 right-3 bottom-3 flex w-[320px] flex-col gap-2.5"
    >
      <div className={`${card} min-h-[180px] flex-1`}>
        <div className="px-4 pt-3.5 pb-3">
          <div className="flex items-start gap-1.5">
            <h2 className="min-w-0 flex-1 truncate text-[13px] font-semibold">
              {title || t("untitledDoc")}
            </h2>
            <button
              type="button"
              onClick={onApart}
              title={t("viewDocApart")}
              aria-label={t("viewDocApart")}
              className={tool}
            >
              <Glyph name="window" className="h-3.5 w-3.5" />
            </button>
            <button
              type="button"
              onClick={onShut}
              title={t("besideShut")}
              aria-label={t("besideShut")}
              className={`${tool} text-[11.5px]`}
            >
              <span aria-hidden="true">✕</span>
            </button>
          </div>
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
          <Controls
            {...props}
            sheets={props.sheets ?? false}
            landscape={props.landscape ?? false}
          />
        </div>
        <Outline
          outline={outline}
          pages={pages}
          paper={paper}
          up={props.up}
          onPage={props.onPage}
        />
      </div>

      <div className={card}>
        <Fold
          title={t("besideDetails")}
          gist={gist}
          open={open === "details"}
          onToggle={() => toggle("details")}
        >
          <PaperFacts facts={facts} body={body} />
          <Properties
            body={body}
            onEdit={heads.find((one) => one.level === 0)?.go}
            skip={["title", "tags", "tag"]}
          />
          <Tags tags={props.tags} body={body} />
        </Fold>
        <Fold
          title={t("besideShared")}
          gist={t("sharedSoon")}
          open={open === "shared"}
          onToggle={() => toggle("shared")}
        >
          <p className="text-[12.5px] leading-relaxed text-soft">{t("sharedWill")}</p>
        </Fold>
      </div>
    </aside>
  );
}
