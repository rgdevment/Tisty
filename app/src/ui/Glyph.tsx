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
    return (
      <span aria-hidden="true" className={`glyph grid shrink-0 place-items-center ${className}`}>
        {name}
      </span>
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
