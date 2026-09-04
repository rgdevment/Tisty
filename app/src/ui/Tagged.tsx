import type { Filed } from "../core";
import { stamped } from "../format";
import { t } from "../locales";
import Glyph from "./Glyph";

interface Props {
  docs: Filed[];
  onOpen: (file: string) => void;
}

export default function Tagged({ docs, onOpen }: Props) {
  if (docs.length === 0) return null;

  return (
    <>
      <p className="mt-5 mb-1 flex items-center gap-2 px-2.5 text-[11.5px] font-semibold tracking-[0.05em] text-faint uppercase">
        {t("docs")}
        <span className="font-normal tracking-normal normal-case tabular-nums">{docs.length}</span>
      </p>
      {docs.map((doc) => (
        <button
          key={doc.id}
          type="button"
          onClick={() => onOpen(doc.file)}
          className="flex w-full items-baseline gap-2 rounded-lg px-2.5 py-1.5 text-left hover:bg-hover"
        >
          <Glyph name="page" className="h-[13px] w-[13px] shrink-0 self-center text-faint" />
          <span className="min-w-0 truncate text-sm">{doc.title || t("untitledDoc")}</span>
          <span className="min-w-0 shrink truncate text-[11.5px] text-faint">
            {(doc.tags ?? []).map((one) => `#${one}`).join(" ")}
          </span>
          {doc.wrote && (
            <span className="ml-auto shrink-0 text-[11.5px] text-faint">{stamped(doc.wrote)}</span>
          )}
        </button>
      ))}
    </>
  );
}
