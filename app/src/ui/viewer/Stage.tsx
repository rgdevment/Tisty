import { useEffect, useRef, useState } from "react";
import type { Lent } from "../../core";
import { fill, t } from "../../locales";
import { type Lending, lend, lendKept, type Span } from "../widgeting";

export type Preset = "fit" | "390" | "820" | "1280";

export const PRESETS: { key: Preset; word: "viewFit" | "viewPhone" | "viewTablet" | "viewDesk" }[] =
  [
    { key: "fit", word: "viewFit" },
    { key: "390", word: "viewPhone" },
    { key: "820", word: "viewTablet" },
    { key: "1280", word: "viewDesk" },
  ];

const GUTTER = 32;

// Content that fills its frame reaches the edge; anything narrower was drawn at a width of its own.
export const natural = (span: Span): number | null => {
  const wide = span.right + span.left;
  return span.right >= span.view - 0.5 || wide >= span.view ? null : Math.max(320, wide);
};

interface Props {
  lent: Lent;
  preset: Preset;
  actual: boolean;
  dark: boolean | null;
  again: number;
}

export default function Stage({ lent, preset, actual, dark, again }: Props) {
  const stage = useRef<HTMLDivElement>(null);
  const drawn = useRef<HTMLDivElement>(null);
  const lending = useRef<Lending | null>(null);
  const [room, setRoom] = useState({ wide: 800, tall: 600 });
  const [own, setOwn] = useState<number | null>(null);

  useEffect(() => {
    const at = stage.current;
    if (!at) return;
    const seen = new ResizeObserver(() =>
      setRoom({
        wide: Math.max(at.clientWidth - GUTTER, 1),
        tall: Math.max(at.clientHeight - GUTTER, 1),
      }),
    );
    seen.observe(at);
    return () => seen.disconnect();
  }, []);

  useEffect(() => {
    const at = drawn.current;
    if (!at || again < 0) return;
    setOwn(null);
    const spanned = (span: Span) => setOwn(natural(span));
    const made =
      "reference" in lent ? lendKept(at, lent.reference, spanned) : lend(at, lent.source, spanned);
    made.shade(shading.current);
    lending.current = made;
    return () => made.drop();
  }, [lent, again]);

  const shading = useRef(dark);
  shading.current = dark;
  useEffect(() => {
    lending.current?.shade(dark);
  }, [dark]);

  const wide = preset === "fit" ? (own ?? room.wide) : Number(preset);
  const scale = actual ? 1 : Math.min(1, room.wide / wide);
  const name = t(PRESETS.find((one) => one.key === preset)?.word ?? "viewFit");
  const said = [
    `${name} · ${Math.round(wide)} px`,
    scale < 1 ? fill("viewAt", String(Math.round(scale * 100))) : "",
    preset === "fit" && own !== null ? t("viewCentred") : "",
  ].filter(Boolean);

  return (
    <>
      <div ref={stage} className="flex min-h-0 min-w-0 flex-1 justify-center overflow-auto p-4">
        <div
          className="shrink-0 overflow-hidden rounded-[10px] shadow-[0_1px_2px_rgb(0_0_0/0.06),0_8px_24px_rgb(0_0_0/0.08)] transition-[width]"
          style={{ width: Math.round(wide * scale), height: room.tall }}
        >
          <div
            ref={drawn}
            className="viewed-widget origin-top-left bg-sheet"
            style={{
              width: wide,
              height: Math.round(room.tall / scale),
              transform: `scale(${scale})`,
            }}
          />
        </div>
      </div>
      <div className="flex h-7 shrink-0 items-center gap-3.5 border-t border-hair bg-panel px-3.5 text-[11.5px] text-faint">
        <span>{said.join(" · ")}</span>
        <span className="ml-auto">{t("viewSealed")}</span>
      </div>
    </>
  );
}
