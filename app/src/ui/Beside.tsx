import { useEffect, useState } from "react";
import { type DocFacts, docFacts } from "../core";
import { stamped, weigh } from "../format";
import { fill, t } from "../locales";
import { DOC } from "../markdown";
import type { Block } from "./Slash";
import type { Head } from "./writing";

const SHAPES = ["h1", "h2", "bullets", "numbers", "todo", "quote", "code", "table", "rule"];

export const worded = (body: string): number =>
  body
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/[#>*_`~[\]()!|-]/g, " ")
    .split(/\s+/)
    .filter(Boolean).length;

export const counted = (body: string, needle: string): number => body.split(needle).length - 1;

const dated = (when: number | null): string =>
  when ? stamped(new Date(when * 1000).toISOString()) : "—";

interface Props {
  title: string;
  paper: string;
  body: string;
  kept: number;
  blocks: Block[];
  heads: Head[];
  onShut: () => void;
}

export default function Beside({ title, paper, body, kept, blocks, heads, onShut }: Props) {
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
        <h2 className="min-w-0 flex-1 truncate text-[13.5px] font-semibold">
          {title || t("untitledDoc")}
        </h2>
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
            {(papers > 0 || files > 0) && (
              <div className="flex items-baseline justify-between gap-2.5">
                <dt className="text-faint">{t("paperHolds")}</dt>
                <dd className="text-right tabular-nums text-soft">
                  {[
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
            <span aria-hidden="true" className="w-4 shrink-0 text-center text-[12.5px]">
              {one.icon}
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
