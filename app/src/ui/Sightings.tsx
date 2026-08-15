import type { Sighting } from "../core";
import { t } from "../locales";

type Props = {
  papers: Sighting[];
  onOpen: (id: string) => void;
};

export default function Sightings({ papers, onOpen }: Props) {
  if (!papers.length) return null;

  return (
    <div className="mt-6 border-t border-hair pt-4 first:mt-1 first:border-0 first:pt-0">
      <div className="mb-1 px-2.5 text-[11.5px] font-semibold tracking-[0.05em] text-faint uppercase">
        {t("foundPapers")}
        <span className="ml-1.5 tabular-nums opacity-70">{papers.length}</span>
      </div>

      <ul role="list">
        {papers.map((one) => (
          <li key={one.id}>
            <button
              type="button"
              onClick={() => onOpen(one.id)}
              className="grid w-full grid-cols-[16px_minmax(0,1fr)] items-start gap-2.5 rounded-lg px-2.5 py-2 text-left hover:bg-hover"
            >
              <span aria-hidden="true" className="text-center text-[13px] text-faint">
                {one.archived ? "▢" : "▣"}
              </span>
              <span className="min-w-0">
                <span className="block truncate text-sm">
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
    </div>
  );
}
