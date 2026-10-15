import { useEffect } from "react";

type Props = {
  body: string;
  onShown?: (page: Window | null) => void;
  onOutline?: (heads: never[]) => void;
};

// The real page lives in a frame jsdom never loads; this one lays the words out in the document.
export default function Page({ body, onShown, onOutline }: Props) {
  useEffect(() => {
    onShown?.(window);
    onOutline?.([]);
  }, [onShown, onOutline]);
  return (
    <article className="page">
      {body.split(/\n{2,}/).map((one, at) => {
        const card = /^!\[([^\]]*)\]\(tisty:doc\/([^)\s]+)\)/.exec(one.trim());
        return card ? (
          // biome-ignore lint/suspicious/noArrayIndexKey: a block of one text has no identity but its place
          <span key={at} className="card" data-doc={card[2]}>
            {card[1]}
          </span>
        ) : (
          // biome-ignore lint/suspicious/noArrayIndexKey: a block of one text has no identity but its place
          <p key={at}>{one.replace(/^#+ /, "")}</p>
        );
      })}
    </article>
  );
}
