import { useEffect, useState } from "react";
import { docHeads } from "../citing";
import type { Filed } from "../core";
import { fill, t } from "../locales";
import Section from "./Section";

interface Props {
  source?: string;
  papers?: readonly Filed[];
  onDoc?: (id: string) => void;
}

export const bornOf = (
  source: string | undefined,
  papers: readonly Filed[],
): { doc: Filed; at: string } | null => {
  const cut = source?.indexOf("#") ?? -1;
  if (!source || cut <= 0) return null;
  const named = source.slice(0, cut).trim();
  const doc = papers.find((one) => one.file === named);
  return doc ? { doc, at: source.slice(cut + 1) } : null;
};

const useHeading = (file?: string, at?: string): string | null => {
  const [heading, setHeading] = useState<{ asked: string; title: string | null } | null>(null);
  const asked = `${file}#${at}`;
  useEffect(() => {
    if (!file || !at) return;
    let live = true;
    docHeads(file)
      .then((heads) => {
        const found = Array.isArray(heads) ? heads.find((one) => one.id === at) : undefined;
        if (live) setHeading({ asked, title: found?.title ?? null });
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  }, [file, at, asked]);
  return heading?.asked === asked ? heading.title : null;
};

export default function Origin({ source, papers, onDoc }: Props) {
  const born = bornOf(source, papers ?? []);
  const heading = useHeading(born?.doc.file, born?.at);
  if (!born) return null;
  const { doc } = born;

  return (
    <>
      <Section label={t("bornIn")} />
      <button
        type="button"
        onClick={() => onDoc?.(doc.file)}
        className="grid w-full cursor-pointer grid-cols-[17px_minmax(0,1fr)] items-baseline gap-2.5 rounded-[10px] border border-hair px-2.5 py-2 text-left hover:border-line"
      >
        <span aria-hidden="true" className="text-center text-[11.5px] text-faint">
          {doc.away ? "▢" : "▤"}
        </span>
        <span className="min-w-0">
          <span className={`block truncate text-[12.5px] ${doc.away ? "text-faint" : "text-ink"}`}>
            {doc.title.trim() || t("untitledDoc")}
          </span>
          {heading && (
            <span className="block truncate text-[11.5px] text-faint">
              {fill("citedUnder", heading)}
            </span>
          )}
        </span>
      </button>
    </>
  );
}
