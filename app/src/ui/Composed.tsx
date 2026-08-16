import { useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { opened, revealed, served } from "../core";
import { INSIDE, docOf } from "../markdown";
import { ending, named, pictured } from "../previews";
import { t } from "../locales";

interface Props {
  html: string;
  label?: string;
  onEnter?: () => void;
  onError?: (problem: unknown) => void;
  onWhole?: () => void;
  onDoc?: (id: string) => void;
  className: string;
  tabIndex?: number;
}

const known = new Map<string, string>();

export default function Composed({
  html,
  label,
  onEnter,
  onError,
  onWhole,
  onDoc,
  className,
  tabIndex,
}: Props) {
  const box = useRef<HTMLDivElement>(null);
  const [long, setLong] = useState(false);
  const cut = onWhole !== undefined;

  useEffect(() => {
    const holder = box.current;
    if (!holder) return;
    holder.innerHTML = html;
    setLong(false);
    let live = true;

    const measure = () => {
      if (live && holder.scrollHeight > holder.clientHeight + 4) setLong(true);
    };

    holder.querySelectorAll<HTMLImageElement>(`img[${INSIDE}]`).forEach((img) => {
      const reference = img.getAttribute(INSIDE);
      if (!reference) return;
      if (!pictured(reference)) return img.replaceWith(chipped(img.alt, reference));
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
          const paper = docOf(target);
          const away = () =>
            /^(https?|mailto|tel):/i.test(target) ? openUrl(target) : revealed(decodeURI(target));

          if (paper) return onDoc?.(paper);
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

function chipped(label: string, reference: string): HTMLElement {
  const chip = document.createElement("a");
  chip.className = "chip";
  chip.setAttribute(INSIDE, reference);
  chip.href = reference;
  const kind = docOf(reference) ? "DOC" : ending(reference).toUpperCase();
  const badge = document.createElement("span");
  badge.className = "chip-badge";
  badge.textContent = kind.slice(0, 4) || "?";
  const name = document.createElement("span");
  name.textContent = label || named(reference);
  chip.append(badge, name);
  return chip;
}

function missing(name: string): HTMLElement {
  const said = document.createElement("span");
  said.className = "text-faint";
  said.textContent = `⚠ ${name}`;
  return said;
}
