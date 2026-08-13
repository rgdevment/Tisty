import { useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { opened, revealed, served } from "../core";
import { INSIDE } from "../markdown";
import { t } from "../locales";

interface Props {
  html: string;
  label?: string;
  /** Click, Enter or Space — never mere focus, or tabbing past would edit. */
  onEnter?: () => void;
  onError?: (problem: unknown) => void;
  /** Cuts a long body down to a preview and offers the full screen instead. */
  onWhole?: () => void;
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
  onWhole,
  className,
  tabIndex,
}: Props) {
  const box = useRef<HTMLDivElement>(null);
  const [long, setLong] = useState(false);
  const cut = onWhole !== undefined;

  // Not `dangerouslySetInnerHTML`: React compares it by object identity, so
  // every render rewrote the markup and threw away the resolved sources.
  useEffect(() => {
    const holder = box.current;
    if (!holder) return;
    holder.innerHTML = html;
    setLong(false);
    let live = true;

    // Latched, never lowered: the clamp changes the height it measures, so a
    // two-way answer oscillates and the text flickers.
    const measure = () => {
      if (live && holder.scrollHeight > holder.clientHeight + 4) setLong(true);
    };

    holder.querySelectorAll<HTMLImageElement>(`img[${INSIDE}]`).forEach((img) => {
      const reference = img.getAttribute(INSIDE);
      if (!reference) return;
      img.addEventListener("load", measure);

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

    measure();
    return () => {
      live = false;
    };
  }, [html, cut]);

  return (
    <div>
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
          const target = link?.getAttribute("href") ?? "";
          const away = () =>
            /^(https?|mailto|tel):/i.test(target) ? openUrl(target) : revealed(decodeURI(target));

          (inside ? opened(inside) : away()).catch((problem) => onError?.(problem));
        }}
        className={`${className} ${cut ? "clamped" : ""}`}
      />
      {cut && long && (
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            onWhole?.();
          }}
          className="mt-1 rounded-md px-1.5 py-0.5 text-xs text-accent hover:bg-hover"
        >
          {t("showWhole")}
        </button>
      )}
    </div>
  );
}

function missing(name: string): HTMLElement {
  const said = document.createElement("span");
  said.className = "text-faint";
  said.textContent = `⚠ ${name}`;
  return said;
}
