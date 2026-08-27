import { markup } from "../glyphs";

interface Props {
  name: string;
  className?: string;
}

export { known, markup } from "../glyphs";

export default function Glyph({ name, className = "h-4 w-4" }: Props) {
  const drawn = markup(name);
  if (!drawn) return null;
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
