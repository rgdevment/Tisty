import { useState } from "react";
import type { Offered } from "../core";
import { fill, t } from "../locales";

export const AT_MOST = 5;

interface Props {
  offered: Offered[];
  onHang: (whole: string) => void;
}

export default function Hang({ offered, onHang }: Props) {
  const [looking, setLooking] = useState(false);
  const [query, setQuery] = useState("");
  const said = query.trim().toLowerCase();
  const found = offered.filter((one) => !said || one.title.toLowerCase().includes(said));

  return (
    <>
      <button
        type="button"
        aria-expanded={looking}
        onClick={() => {
          setLooking(!looking);
          setQuery("");
        }}
        className="ml-auto shrink-0 text-[12.5px] text-accent hover:underline"
      >
        ⌂ {t("partOfOther")}
      </button>
      {looking && (
        <div className="absolute inset-x-0 top-full z-10 mt-1 flex flex-col gap-0.5 rounded-[10px] border border-hair bg-panel p-1.5 text-[12.5px] shadow-lift">
          <input
            autoFocus
            value={query}
            placeholder={t("partOfFind")}
            aria-label={t("partOfFind")}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Escape") setLooking(false);
              if (e.key !== "Enter") return;
              e.preventDefault();
              if (found[0]) onHang(found[0].id);
            }}
            className="mb-0.5 border-hair border-b bg-transparent px-2 py-1.5 outline-none placeholder:text-faint"
          />
          {found.slice(0, AT_MOST).map((one) => (
            <button
              key={one.id}
              type="button"
              onClick={() => onHang(one.id)}
              className="truncate rounded-md px-2 py-1.5 text-left hover:bg-hover"
            >
              {one.title}
            </button>
          ))}
          {found.length > AT_MOST && (
            <span className="px-2 py-1 text-[11.5px] text-faint">
              {fill("partOfMore", String(found.length - AT_MOST))}
            </span>
          )}
          {found.length === 0 && (
            <span className="px-2 py-1.5 text-[12.5px] text-faint">{t("partOfNone")}</span>
          )}
        </div>
      )}
    </>
  );
}
