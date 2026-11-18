import { type Ref, useState } from "react";
import type { Network, NeuronKind } from "../../brainsty";
import { t } from "../../locales";
import Glyph from "../Glyph";
import type { Standing } from "./looks";
import { EVERYTHING, KINDS, PERIODS, type Period, type Sift, STANDINGS, sifting } from "./sifting";

interface Props {
  readonly network: Network;
  readonly query: string;
  readonly onQuery: (query: string) => void;
  readonly hits: readonly number[];
  readonly onChoose: (node: number) => void;
  readonly sift: Sift;
  readonly onSift: (sift: Sift) => void;
  readonly tags: readonly number[];
  readonly finder?: Ref<HTMLInputElement>;
}

const chip = (on: boolean) =>
  `rounded-full border px-2.5 py-1 text-[11.5px] transition-colors duration-150 ${
    on ? "border-accent bg-accent-soft text-accent" : "border-hair text-soft hover:bg-hover"
  }`;

const toggled = <T,>(all: ReadonlySet<T>, one: T): Set<T> => {
  const next = new Set(all);
  if (next.has(one)) next.delete(one);
  else next.add(one);
  return next;
};

export default function Toolbar({
  network,
  query,
  onQuery,
  hits,
  onChoose,
  sift,
  onSift,
  tags,
  finder,
}: Props) {
  const [open, setOpen] = useState(false);
  const asked = query.trim() !== "";
  return (
    <div className="pointer-events-none absolute top-3 left-3 z-10 flex w-80 flex-col gap-2">
      <div className="pointer-events-auto flex items-center gap-1.5 rounded-[10px] border border-hair bg-bg px-2.5 shadow-lift">
        <Glyph name="search" className="h-3.5 w-3.5 shrink-0 text-faint" />
        <input
          ref={finder}
          type="search"
          value={query}
          onChange={(e) => onQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key !== "Enter" || hits[0] === undefined) return;
            e.preventDefault();
            onChoose(hits[0]);
          }}
          placeholder={t("brainstySearch")}
          aria-label={t("brainstySearch")}
          className="h-8 min-w-0 flex-1 bg-transparent text-[12.5px] text-ink outline-none placeholder:text-faint"
        />
        <button
          type="button"
          onClick={() => setOpen(!open)}
          aria-expanded={open}
          aria-label={t("brainstyFilters")}
          title={t("brainstyFilters")}
          className={`grid size-7 shrink-0 place-items-center rounded-md hover:bg-hover ${
            sifting(sift) ? "text-accent" : "text-faint"
          }`}
        >
          <Glyph name="filter" className="h-3.5 w-3.5" />
        </button>
      </div>
      {asked && (
        <ul className="pointer-events-auto max-h-72 overflow-auto rounded-[10px] border border-hair bg-bg py-1 shadow-lift">
          {hits.length === 0 && (
            <li className="px-3 py-1.5 text-[12.5px] text-faint">{t("brainstyNothingFound")}</li>
          )}
          {hits.map((node) => {
            const one = network.nodes[node];
            return (
              <li key={one.id}>
                <button
                  type="button"
                  onClick={() => onChoose(node)}
                  className="flex w-full items-baseline gap-2 px-3 py-1.5 text-left text-[12.5px] hover:bg-hover"
                >
                  <span className="min-w-0 flex-1 truncate text-ink">{one.title}</span>
                  <span className="shrink-0 text-[11.5px] text-faint">
                    {t(`brainstyOne_${one.kind}`)}
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
      {open && <Filters network={network} sift={sift} onSift={onSift} tags={tags} />}
    </div>
  );
}

function Filters({
  network,
  sift,
  onSift,
  tags,
}: Pick<Props, "network" | "sift" | "onSift" | "tags">) {
  return (
    <fieldset className="pointer-events-auto flex flex-col gap-2.5 rounded-[10px] border border-hair bg-bg p-3 shadow-lift">
      <legend className="sr-only">{t("brainstyFilters")}</legend>
      <Row said={t("brainstyShowing")}>
        {KINDS.map((kind: NeuronKind) => (
          <button
            key={kind}
            type="button"
            aria-pressed={sift.kinds.has(kind)}
            onClick={() => onSift({ ...sift, kinds: toggled(sift.kinds, kind) })}
            className={chip(sift.kinds.has(kind))}
          >
            {t(`brainstyKind_${kind}`)}
          </button>
        ))}
      </Row>
      <Row>
        {STANDINGS.map((standing: Standing) => (
          <button
            key={standing}
            type="button"
            aria-pressed={sift.standings.has(standing)}
            onClick={() => onSift({ ...sift, standings: toggled(sift.standings, standing) })}
            className={chip(sift.standings.has(standing))}
          >
            {t(`brainstyStanding_${standing}`)}
          </button>
        ))}
      </Row>
      <Row said={t("brainstyPeriod")}>
        {PERIODS.map((days: Period) => (
          <button
            key={days}
            type="button"
            aria-pressed={sift.days === days}
            onClick={() => onSift({ ...sift, days })}
            className={chip(sift.days === days)}
          >
            {t(`brainstyPeriod_${days}`)}
          </button>
        ))}
      </Row>
      <label className="flex items-center gap-2 text-[11.5px] text-faint">
        {t("brainstyTag")}
        <select
          value={sift.tag ?? ""}
          onChange={(e) =>
            onSift({ ...sift, tag: e.target.value === "" ? null : Number(e.target.value) })
          }
          className="min-w-0 flex-1 rounded-md border border-hair bg-bg px-2 py-1 text-[12.5px] text-ink"
        >
          <option value="">{t("brainstyAnyTag")}</option>
          {tags.map((node) => (
            <option key={node} value={node}>
              #{network.nodes[node].title}
            </option>
          ))}
        </select>
      </label>
      {sifting(sift) && (
        <button
          type="button"
          onClick={() => onSift(EVERYTHING)}
          className="self-start text-[11.5px] text-accent hover:underline"
        >
          {t("brainstyEverything")}
        </button>
      )}
    </fieldset>
  );
}

function Row({ said, children }: { readonly said?: string; readonly children: React.ReactNode }) {
  return (
    <div className="flex flex-wrap items-center gap-1.5">
      {said && <span className="mr-1 text-[11.5px] text-faint">{said}</span>}
      {children}
    </div>
  );
}
