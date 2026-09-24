import { useState } from "react";
import type { Filed } from "../core";
import { t } from "../locales";
import Glyph from "./Glyph";

interface Props {
  pages: Filed[];
  told: Set<string>;
  onOpen: (page: Filed) => void;
  onPut?: (page: Filed) => void;
  onMove?: (page: Filed, before: string | null) => void;
}

export default function Contents({ pages, told, onOpen, onPut, onMove }: Props) {
  const [carried, setCarried] = useState<string | null>(null);
  const [over, setOver] = useState<string | null>(null);

  if (pages.length === 0) return null;
  const inside = pages.filter((one) => told.has(one.file));
  const loose = pages.filter((one) => !told.has(one.file));

  const drops = (page: Filed) => {
    if (!onMove || !carried || carried === page.file) return;
    onMove(
      inside.find((one) => one.file === carried) ?? page,
      page.file === carried ? null : page.file,
    );
  };

  const row = (page: Filed, at: string, movable: boolean) => (
    <li
      key={page.id}
      draggable={movable}
      onDragStart={() => setCarried(page.file)}
      onDragEnd={() => {
        setCarried(null);
        setOver(null);
      }}
      onDragOver={(e) => {
        if (!movable || !carried) return;
        e.preventDefault();
        setOver(page.file);
      }}
      onDragLeave={() => setOver((one) => (one === page.file ? null : one))}
      onDrop={(e) => {
        e.preventDefault();
        drops(page);
        setCarried(null);
        setOver(null);
      }}
      className={[
        page.away ? "leaf-row leaf-away" : "leaf-row",
        over === page.file && carried && over !== carried ? "leaf-over" : "",
      ]
        .filter(Boolean)
        .join(" ")}
    >
      <button
        type="button"
        onClick={() => onOpen(page)}
        aria-label={
          page.away ? `${page.title || t("untitledDoc")} — ${t("isArchived")}` : undefined
        }
        className="leaf-open"
      >
        <span className="leaf-num">{at}</span>
        <span className="leaf-name">{page.title || t("untitledDoc")}</span>
        {page.archived && <Glyph name="archive" className="leaf-mark" />}
        {page.flagged && !page.away && (
          <span aria-hidden="true" className="leaf-flag">
            ◆
          </span>
        )}
      </button>
      {onPut && !told.has(page.file) && (
        <button type="button" onClick={() => onPut(page)} className="leaf-put">
          {t("putLeaf")}
        </button>
      )}
    </li>
  );

  return (
    <section aria-label={t("theseLeaves")} className="leaves">
      <h2 className="leaves-head">
        {t("theseLeaves")}
        <span className="leaves-many">{pages.length}</span>
      </h2>
      <p className="leaves-why">{loose.length > 0 ? t("someLoose") : t("allInside")}</p>
      {onMove && inside.length > 1 && <p className="leaves-why">{t("dragLeaves")}</p>}
      <ul className="leaves-list">
        {inside.map((one, at) => row(one, String(at + 1).padStart(2, "0"), Boolean(onMove)))}
      </ul>
      {loose.length > 0 && (
        <>
          <div aria-hidden="true" className="leaves-split" />
          <ul className="leaves-list">{loose.map((one) => row(one, "—", false))}</ul>
        </>
      )}
    </section>
  );
}
