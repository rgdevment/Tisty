import { useEffect, useRef } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { opened, served } from "../core";
import { INSIDE } from "../markdown";

interface Props {
  html: string;
  label?: string;
  /** Click, Enter or Space — never mere focus, or tabbing past would edit. */
  onEnter?: () => void;
  onError?: (problem: unknown) => void;
  className: string;
  tabIndex?: number;
}

/** The source column re-composes on every keystroke, and resolving hits the disk. */
const known = new Map<string, string>();

export default function Composed({
  html,
  label,
  onEnter,
  onError,
  className,
  tabIndex,
}: Props) {
  const box = useRef<HTMLDivElement>(null);

  // Not `dangerouslySetInnerHTML`: React compares it by object identity, so
  // every render rewrote the markup and threw away the resolved sources.
  useEffect(() => {
    const holder = box.current;
    if (!holder) return;
    holder.innerHTML = html;
    let live = true;

    holder.querySelectorAll<HTMLImageElement>(`img[${INSIDE}]`).forEach((img) => {
      const reference = img.getAttribute(INSIDE);
      if (!reference) return;

      const cached = known.get(reference);
      if (cached) {
        img.setAttribute("src", cached);
        return;
      }
      served(reference)
        .then((at) => {
          const url = convertFileSrc(at);
          known.set(reference, url);
          if (live) img.setAttribute("src", url);
        })
        .catch(() => live && img.replaceWith(missing(img.alt || reference)));
    });

    return () => {
      live = false;
    };
  }, [html]);

  return (
    <div
      ref={box}
      tabIndex={tabIndex}
      aria-label={label}
      onKeyDown={(e) => {
        if (onEnter && (e.key === "Enter" || e.key === " ")) {
          e.preventDefault();
          onEnter();
        }
      }}
      onClick={(e) => {
        const spot = e.target as HTMLElement;
        const link = spot.closest("a");
        const picture = spot.closest(`img[${INSIDE}]`);

        if (!link && !picture) {
          onEnter?.();
          return;
        }
        e.preventDefault();

        const inside = (picture ?? link)?.getAttribute(INSIDE);
        if (inside) opened(inside).catch((problem) => onError?.(problem));
        else openUrl(link?.getAttribute("href") ?? "").catch((problem) => onError?.(problem));
      }}
      className={className}
    />
  );
}

function missing(name: string): HTMLElement {
  const said = document.createElement("span");
  said.className = "text-faint";
  said.textContent = `⚠ ${name}`;
  return said;
}
