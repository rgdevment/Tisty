import type { HelpEntry } from "../guide/calls";
import { t } from "../locales";

interface Props {
  readonly before?: HelpEntry;
  readonly after?: HelpEntry;
  readonly onOpen: (slug: string) => void;
}

const pill =
  "inline-flex max-w-[48%] items-center gap-2 rounded-full border border-line bg-sheet px-4 py-2 text-[13px] font-medium text-ink hover:border-accent hover:text-accent";

export default function HelpSteps({ before, after, onOpen }: Props) {
  return (
    <nav
      aria-label={`${t("helpBefore")} · ${t("helpAfter")}`}
      className="flex shrink-0 gap-3 border-t border-hair px-6 py-3"
    >
      {before && (
        <button
          type="button"
          title={t("helpBefore")}
          className={pill}
          onClick={() => onOpen(before.slug)}
        >
          <span aria-hidden="true" className="text-faint">
            ←
          </span>
          <span className="truncate">{before.title}</span>
        </button>
      )}
      {after && (
        <button
          type="button"
          title={t("helpAfter")}
          className={`${pill} ml-auto`}
          onClick={() => onOpen(after.slug)}
        >
          <span className="truncate">{after.title}</span>
          <span aria-hidden="true" className="text-faint">
            →
          </span>
        </button>
      )}
    </nav>
  );
}
