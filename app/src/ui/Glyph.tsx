import { useLayoutEffect, useRef, useState } from "react";
import { markup } from "../glyphs";
import { onlyMark } from "../leading";

interface Props {
  name: string;
  className?: string;
}

export { known, markup } from "../glyphs";

// The viewBox every icon shares; a single emoji at font-size 18 measures about 25 across it.
const ROOM = 24;
const PRESSED_PAST = 30;

export default function Glyph({ name, className = "h-4 w-4" }: Props) {
  const drawn = markup(name);
  if (!drawn) {
    if (!onlyMark(name)) return null;
    return <Mark name={name} className={className} />;
  }
  return (
    <svg
      viewBox={`0 0 ${ROOM} ${ROOM}`}
      aria-hidden="true"
      className={`glyph shrink-0 ${className}`}
      // biome-ignore lint/security/noDangerouslySetInnerHtml: the markup is ours, from a fixed table
      dangerouslySetInnerHTML={{ __html: drawn }}
    />
  );
}

// As plain text an emoji takes the font's size, and macOS draws it past the box it sits in.
function Mark({ name, className }: { name: string; className: string }) {
  const text = useRef<SVGTextElement>(null);
  const [pressed, setPressed] = useState(false);
  // A sequence the font cannot join is drawn as several emoji side by side: pressed in, not cut.
  useLayoutEffect(() => {
    const wide = text.current?.getBBox?.().width ?? 0;
    setPressed(wide > PRESSED_PAST);
  }, [name]);
  return (
    <svg
      viewBox={`0 0 ${ROOM} ${ROOM}`}
      aria-hidden="true"
      className={`glyph shrink-0 ${className}`}
    >
      <text
        ref={text}
        x={ROOM / 2}
        y={ROOM / 2}
        textAnchor="middle"
        dominantBaseline="central"
        fontSize="18"
        style={{ fill: "currentColor", stroke: "none" }}
        {...(pressed ? { textLength: ROOM, lengthAdjust: "spacingAndGlyphs" } : {})}
      >
        {name}
      </text>
    </svg>
  );
}
