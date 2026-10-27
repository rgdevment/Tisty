import { markup } from "../glyphs";
import { onlyMark } from "../leading";

interface Props {
  name: string;
  className?: string;
}

export { known, markup } from "../glyphs";

export default function Glyph({ name, className = "h-4 w-4" }: Props) {
  const drawn = markup(name);
  if (!drawn) {
    if (!onlyMark(name)) return null;
    // As plain text an emoji takes the font's size, and macOS draws it past the box it sits in.
    return (
      <svg viewBox="0 0 24 24" aria-hidden="true" className={`glyph shrink-0 ${className}`}>
        <text
          x="12"
          y="12"
          textAnchor="middle"
          dominantBaseline="central"
          fontSize="18"
          style={{ fill: "currentColor", stroke: "none" }}
        >
          {name}
        </text>
      </svg>
    );
  }
  return (
    <svg
      viewBox="0 0 24 24"
      aria-hidden="true"
      className={`glyph shrink-0 ${className}`}
      // biome-ignore lint/security/noDangerouslySetInnerHtml: the markup is ours, from a fixed table
      dangerouslySetInnerHTML={{ __html: drawn }}
    />
  );
}
