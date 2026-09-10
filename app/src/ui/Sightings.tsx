import type { Sighting } from "../core";
import { t } from "../locales";

type Props = {
  papers: Sighting[];
  onOpen: (id: string) => void;
};

export default function Sightings({ papers, onOpen }: Props) {
  if (!papers.length) return null;

  return (
    <section className="rounded-[10px] border border-hair bg-sheet px-2.5 py-2 shadow-lift">
      <p className="mb-1 flex items-baseline gap-2 border-b border-hair px-2.5 pb-1.5 text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase">
        {t("foundPapers")}
        <span className="ml-auto font-normal tracking-normal normal-case tabular-nums">
          {papers.length}
        </span>
      </p>

      <ul>
        {papers.map((one) => (
          <li key={one.id}>
            <button
              type="button"
              onClick={() => onOpen(one.id)}
              className="grid w-full grid-cols-[16px_minmax(0,1fr)] items-start gap-2.5 rounded-[10px] px-2.5 py-2 text-left hover:bg-hover"
            >
              <span aria-hidden="true" className="text-center text-[13px] text-faint">
                {one.archived ? "▢" : "▣"}
              </span>
              <span className="min-w-0">
                <span className="block truncate text-[13px]">
                  {one.title.trim() || t("untitledDoc")}
                  {one.archived ? <span className="sr-only"> ({t("scopeArchived")})</span> : null}
                </span>
                <span className="mt-0.5 block truncate text-[11.5px] text-faint">
                  {one.line || t("foundInTitle")}
                </span>
              </span>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}
