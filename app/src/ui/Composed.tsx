import { useEffect, useRef } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { served } from "../core";
import { INSIDE } from "../markdown";

interface Props {
  html: string;
  label?: string;
  /** Click, Enter or Space — never mere focus, or tabbing past would edit. */
  onEnter?: () => void;
  className: string;
  tabIndex?: number;
}

/**
 * Renders composed Markdown and does the two things a webview will not:
 * resolve a reference that lives under the data root, and hand a link to the
 * system instead of navigating away from the app.
 */
export default function Composed({ html, label, onEnter, className, tabIndex }: Props) {
  const box = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const holder = box.current;
    if (!holder) return;
    let live = true;

    holder.querySelectorAll<HTMLImageElement>(`img[${INSIDE}]`).forEach((img) => {
      const reference = img.getAttribute(INSIDE);
      if (!reference) return;
      served(reference)
        .then((at) => live && img.setAttribute("src", convertFileSrc(at)))
        .catch(() => live && img.setAttribute("alt", `⚠ ${img.alt}`));
    });
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
        const link = (e.target as HTMLElement).closest("a");
        if (!link) {
          onEnter?.();
          return;
        }
        e.preventDefault();
        const reference = link.getAttribute(INSIDE);
        if (reference) served(reference).then(openPath).catch(() => {});
        else void openUrl(link.getAttribute("href") ?? "");
      }}
      className={className}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
