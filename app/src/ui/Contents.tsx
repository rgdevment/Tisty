import { useEffect, useRef, useState } from "react";
import type { Filed } from "../core";
import { fill, t } from "../locales";
import Glyph from "./Glyph";
import { onMac } from "./WindowChrome";

interface Props {
  pages: Filed[];
  told: string[];
  onOpen: (page: Filed) => void;
  onPut?: (page: Filed) => void;
  onMove?: (page: Filed, before: string | null) => void;
}

export default function Contents({ pages, told, onOpen, onPut, onMove }: Props) {
  const [carried, setCarried] = useState<string | null>(null);
  const [over, setOver] = useState<string | null>(null);
  const [atEnd, setAtEnd] = useState(false);
  const [said, setSaid] = useState("");
  const wanted = useRef<string | null>(null);
  const box = useRef<HTMLElement | null>(null);

  useEffect(() => {
    const file = wanted.current;
    if (!file) return;
    wanted.current = null;
    box.current?.querySelector<HTMLElement>(`[data-leaf="${file}"]`)?.focus();
  }, [told]);

  if (pages.length === 0) return null;
  const held = new Set(told);
  const inside = told
    .map((file) => pages.find((one) => one.file === file))
    .filter((one): one is Filed => Boolean(one));
  const loose = pages.filter((one) => !held.has(one.file));
  const mod = onMac ? "⌥" : "Alt+";

  const named = (page: Filed) => page.title || t("untitledDoc");

  const drops = (page: Filed | null) => {
    if (!onMove || !carried || (page && carried === page.file)) return;
    const took = inside.find((one) => one.file === carried);
    if (!took) return;
    onMove(took, page ? page.file : null);
  };

  /// A chapter moves one place at a time, so where it lands is the row it swaps with. Going down
  /// means going before the one after that, and the last place means going before none.
  const moved = (at: number, by: -1 | 1) => {
    const page = inside[at];
    const to = at + by;
    if (!onMove || !page || to < 0 || to >= inside.length) return;
    const before = by < 0 ? inside[at - 1].file : (inside[at + 2]?.file ?? null);
    wanted.current = page.file;
    setSaid(fill("leafMoved", named(page), String(to + 1)));
    onMove(page, before);
  };

  const row = (page: Filed, at: number, movable: boolean) => (
    <li
      key={page.id}
      draggable={movable}
      onDragStart={() => setCarried(page.file)}
      onDragEnd={() => {
        setCarried(null);
        setOver(null);
        setAtEnd(false);
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
        data-leaf={page.file}
        onClick={() => onOpen(page)}
        onKeyDown={(e) => {
          if (!movable || !e.altKey) return;
          if (e.key !== "ArrowUp" && e.key !== "ArrowDown") return;
          e.preventDefault();
          moved(at, e.key === "ArrowUp" ? -1 : 1);
        }}
        aria-keyshortcuts={movable ? "Alt+ArrowUp Alt+ArrowDown" : undefined}
        aria-label={page.away ? `${named(page)} — ${t("isArchived")}` : undefined}
        className="leaf-open"
      >
        <span className="leaf-num">
          <span className="leaf-count">{at >= 0 ? String(at + 1).padStart(2, "0") : "—"}</span>
          {movable && <Glyph name="grip" className="leaf-grip" />}
        </span>
        <span className="leaf-name">{named(page)}</span>
        {page.archived && <Glyph name="archive" className="leaf-mark" />}
        {page.flagged && !page.away && (
          <span aria-hidden="true" className="leaf-flag">
            ◆
          </span>
        )}
      </button>
      {onPut && !held.has(page.file) && (
        <button type="button" onClick={() => onPut(page)} className="leaf-put">
          {t("putLeaf")}
        </button>
      )}
    </li>
  );

  return (
    <section ref={box} aria-label={t("theseLeaves")} className="leaves">
      <h2 className="leaves-head">
        {t("theseLeaves")}
        <span className="leaves-many">{pages.length}</span>
      </h2>
      <p className="leaves-why">{loose.length > 0 ? t("someLoose") : t("allInside")}</p>
      {onMove && inside.length > 1 && <p className="leaves-why">{fill("moveLeaves", mod, mod)}</p>}
      <p role="status" aria-live="polite" className="sr-only">
        {said}
      </p>
      <ul className="leaves-list">
        {inside.map((one, at) => row(one, at, Boolean(onMove)))}
        {onMove && inside.length > 1 && (
          <li
            aria-hidden="true"
            onDragOver={(e) => {
              if (!carried) return;
              e.preventDefault();
              setAtEnd(true);
            }}
            onDragLeave={() => setAtEnd(false)}
            onDrop={(e) => {
              e.preventDefault();
              drops(null);
              setCarried(null);
              setOver(null);
              setAtEnd(false);
            }}
            className={atEnd && carried ? "leaf-end leaf-over" : "leaf-end"}
          />
        )}
      </ul>
      {loose.length > 0 && (
        <>
          <div aria-hidden="true" className="leaves-split" />
          <ul className="leaves-list">{loose.map((one) => row(one, -1, false))}</ul>
        </>
      )}
    </section>
  );
}
