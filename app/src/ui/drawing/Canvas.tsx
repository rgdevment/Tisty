import { useEffect, useState } from "react";
import { t } from "../../locales";
import { drawing } from "../diagrams";
import Board from "./Board";
import { type Size, sizeOf } from "./framing";

// Drawing on every key would lay out a half-typed diagram many times a second.
const SETTLE_MS = 250;

interface Drawn {
  svg: string;
  size: Size;
}

/** What Mermaid draws from the text, for every kind the window cannot edit by hand yet. */
export default function Canvas({ source, dark }: Readonly<{ source: string; dark: boolean }>) {
  const [drawn, setDrawn] = useState<Drawn | null>(null);
  const [broken, setBroken] = useState(false);

  useEffect(() => {
    let live = true;
    const timer = setTimeout(() => {
      void drawing(source, dark, () => live).then((svg) => {
        if (!live) return;
        const size = svg === null ? null : sizeOf(svg);
        setBroken(size === null);
        // The last good drawing stays while the text is half written.
        if (svg !== null && size) setDrawn({ svg, size });
      });
    }, SETTLE_MS);
    return () => {
      live = false;
      clearTimeout(timer);
    };
  }, [source, dark]);

  return (
    <Board
      size={drawn?.size ?? null}
      notice={
        broken && (
          <output className="absolute top-3 left-1/2 -translate-x-1/2 rounded-md bg-panel px-3 py-1.5 text-[12.5px] text-soft shadow">
            {t("diagramUnreadable")}
          </output>
        )
      }
    >
      {drawn && (
        <div
          className="h-full w-full"
          // biome-ignore lint/security/noDangerouslySetInnerHtml: Mermaid draws in strict mode, which leaves no script in the SVG.
          dangerouslySetInnerHTML={{ __html: drawn.svg }}
        />
      )}
    </Board>
  );
}
