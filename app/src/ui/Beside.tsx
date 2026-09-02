import { useEffect, useState } from "react";
import type { Folded } from "../core";
import { type DocFacts, docFacts, type Paper } from "../core";
import { stamped, weigh } from "../format";
import { fill, t } from "../locales";
import { DOC } from "../markdown";
import Glyph, { known } from "./Glyph";
import type { Block } from "./Slash";
import type { Head } from "./writing";

export const SHAPES = [
  "h1",
  "h2",
  "bullets",
  "numbers",
  "todo",
  "quote",
  "callout",
  "code",
  "mermaid",
  "math",
  "table",
  "rule",
  "pen",
];

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

export const worded = (body: string): number =>
  body
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/[#>*_`~[\]()!|-]/g, " ")
    .split(/\s+/)
    .filter(Boolean).length;

export const counted = (body: string, needle: string): number => body.split(needle).length - 1;

const dated = (when: number | null): string =>
  when ? stamped(new Date(when * 1000).toISOString()) : "—";

const mild =
  "rounded-[7px] border border-line px-2 py-1 text-[11.5px] text-soft hover:bg-hover hover:text-ink disabled:border-hair disabled:text-faint";

const LEAVES: Paper[] = ["a4", "letter", "tabloid"];

interface Props {
  title: string;
  paper: string;
  body: string;
  kept: number;
  blocks: Block[];
  heads: Head[];
  leaf: Paper;
  onLeaf: (leaf: Paper) => void;
  making: boolean;
  onPdf: () => void;
  onSee: () => void;
  onCopy: () => void;
  onTakeOut: () => void;
  onShut: () => void;
  trail?: Folded[];
  onFolder?: (id: string | null) => void;
}

export default function Beside({
  title,
  paper,
  body,
  kept,
  blocks,
  heads,
  leaf,
  onLeaf,
  making,
  onPdf,
  onSee,
  onCopy,
  onTakeOut,
  onShut,
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

  const words = worded(body);
  const papers = counted(body, DOC);
  const files = counted(body, "attachments/");
  const shapes = blocks.filter((one) => SHAPES.includes(one.key));
  const puts = blocks.filter((one) => !SHAPES.includes(one.key));

  return (
    <aside
      aria-label={t("aboutPaper")}
      className="absolute top-11 right-3 bottom-3 flex w-[320px] flex-col overflow-hidden rounded-[11px] border border-hair bg-panel shadow-lift"
    >
      <div className="flex items-start gap-2 px-4 pt-3.5">
        <div className="min-w-0 flex-1">
          <h2 className="truncate text-[13.5px] font-semibold">{title || t("untitledDoc")}</h2>
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
          className="grid h-5.5 w-5.5 shrink-0 place-items-center rounded-md text-[11px] text-faint hover:bg-hover hover:text-ink"
        >
          <span aria-hidden="true">✕</span>
        </button>
      </div>

      <div className="scroller flex flex-1 flex-col gap-5 px-4 pt-3.5 pb-5">
        <section className="flex flex-col gap-2">
          <h3 className="text-[10.5px] tracking-[0.07em] text-faint uppercase">
            {t("aboutPaper")}
          </h3>
          <dl className="flex flex-col gap-1.5 text-[12px]">
            <div className="flex items-baseline justify-between gap-2.5">
              <dt className="text-faint">{t("paperMade")}</dt>
              <dd className="tabular-nums text-soft">{dated(facts?.made ?? null)}</dd>
            </div>
            <div className="flex items-baseline justify-between gap-2.5">
              <dt className="text-faint">{t("paperWrote")}</dt>
              <dd className="tabular-nums text-soft">{dated(facts?.wrote ?? null)}</dd>
            </div>
            <div className="flex items-baseline justify-between gap-2.5">
              <dt className="text-faint">{t("paperLong")}</dt>
              <dd className="tabular-nums text-soft">
                {words ? fill("paperWords", String(words)) : t("paperEmpty")}
              </dd>
            </div>
            <div className="flex items-baseline justify-between gap-2.5">
              <dt className="text-faint">{t("paperWeighs")}</dt>
              <dd className="tabular-nums text-soft">{facts ? weigh(facts.bytes) : "—"}</dd>
            </div>
            {(papers > 0 || files > 0 || (facts?.pages ?? 0) > 0) && (
              <div className="flex items-baseline justify-between gap-2.5">
                <dt className="text-faint">{t("paperHolds")}</dt>
                <dd className="text-right tabular-nums text-soft">
                  {[
                    facts?.pages
                      ? facts.pages === 1
                        ? t("pageHeld")
                        : fill("pagesHeld", String(facts.pages))
                      : "",
                    papers > 0 ? fill("paperDocs", String(papers)) : "",
                    files > 0 ? fill("paperFiles", String(files)) : "",
                  ]
                    .filter(Boolean)
                    .join(" · ")}
                </dd>
              </div>
            )}
          </dl>
        </section>

        <section className="flex flex-col gap-2">
          <h3 className="text-[10.5px] tracking-[0.07em] text-faint uppercase">{t("leafIs")}</h3>
          <div className="flex gap-1">
            {LEAVES.map((one) => (
              <button
                key={one}
                type="button"
                aria-pressed={leaf === one}
                onClick={() => onLeaf(one)}
                className={`flex-1 rounded-[7px] border px-2 py-1 text-[11.5px] ${
                  leaf === one
                    ? "border-ink bg-ink text-bg"
                    : "border-line text-faint hover:text-soft"
                }`}
              >
                {t(one === "a4" ? "leafA4" : one === "letter" ? "leafLetter" : "leafTabloid")}
              </button>
            ))}
          </div>
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

        <Band name={t("shaping")} blocks={shapes} />
        <Band name={t("putting")} blocks={puts} />

        <section className="flex flex-col gap-2">
          <h3 className="text-[10.5px] tracking-[0.07em] text-faint uppercase">{t("outline")}</h3>
          {heads.length === 0 ? (
            <p className="text-[12px] text-faint">{t("outlineNone")}</p>
          ) : (
            <nav className="flex flex-col gap-px">
              {heads.map((one) => (
                <button
                  key={one.key}
                  type="button"
                  onClick={one.go}
                  className={`truncate rounded-md px-1.5 py-1 text-left text-[12px] hover:bg-hover hover:text-ink ${
                    one.level > 1 ? "pl-5 text-faint" : "text-soft"
                  }`}
                >
                  {one.text}
                </button>
              ))}
            </nav>
          )}
        </section>
      </div>
    </aside>
  );
}

function Band({ name, blocks }: { name: string; blocks: Block[] }) {
  if (blocks.length === 0) return null;
  return (
    <section className="flex flex-col gap-2">
      <h3 className="text-[10.5px] tracking-[0.07em] text-faint uppercase">{name}</h3>
      <div className="flex flex-col gap-px">
        {blocks.map((one) => (
          <button
            key={one.key}
            type="button"
            onClick={one.run}
            className="flex w-full items-center gap-2.5 rounded-md px-1.5 py-1 text-left text-[12.5px] text-soft hover:bg-hover hover:text-ink"
          >
            <span aria-hidden="true" className="flex w-4 shrink-0 justify-center text-[12.5px]">
              {known(one.icon) ? <Glyph name={one.icon} className="h-[15px] w-[15px]" /> : one.icon}
            </span>
            <span className="min-w-0 flex-1 truncate">{one.label}</span>
            {one.hint && (
              <span className="shrink-0 rounded-[4px] bg-hover px-1.5 py-px font-mono text-[10.5px] text-faint">
                {one.hint}
              </span>
            )}
          </button>
        ))}
      </div>
    </section>
  );
}
