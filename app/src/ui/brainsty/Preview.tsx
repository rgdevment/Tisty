import { useEffect, useState } from "react";
import type { Neuron } from "../../brainsty";
import { type Cited, docGlance, type Glance, taskCards } from "../../citing";
import { numbered } from "../../format";
import { fill, t } from "../../locales";

interface Props {
  readonly neuron: Neuron;
  readonly ties: number;
  readonly zone?: string;
  readonly onOpen: () => void;
  readonly onClose: () => void;
}

// Its ids and places name another document, and the page beneath finds things by both.
const unmarked = (html: string): string => {
  const held = document.createElement("template");
  held.innerHTML = html;
  for (const one of held.content.querySelectorAll("[id], [data-at]")) {
    one.removeAttribute("id");
    one.removeAttribute("data-at");
  }
  return held.innerHTML;
};

export default function Preview({ neuron, ties, zone, onOpen, onClose }: Props) {
  const [glance, setGlance] = useState<Glance | null>(null);
  const [card, setCard] = useState<Cited | null>(null);

  useEffect(() => {
    let live = true;
    setGlance(null);
    setCard(null);
    if (neuron.kind === "doc" || neuron.kind === "page") {
      docGlance(neuron.id).then(
        (said) => live && setGlance({ ...said, html: unmarked(said.html) }),
        () => undefined,
      );
    }
    if (neuron.kind === "task") {
      taskCards([neuron.id]).then(
        ([said]) => live && setCard(said ?? null),
        () => undefined,
      );
    }
    return () => {
      live = false;
    };
  }, [neuron.id, neuron.kind]);

  const steps = card?.steps ?? [];
  return (
    <aside
      aria-label={neuron.title}
      className="absolute top-3 right-3 z-10 flex max-h-[calc(100%-24px)] w-80 flex-col gap-2 overflow-hidden rounded-[10px] border border-hair bg-bg p-4 shadow-lift"
    >
      <div className="flex items-start gap-2">
        <h2 className="min-w-0 flex-1 text-[14px] leading-snug font-semibold break-words text-ink">
          {neuron.kind === "tag" ? `#${neuron.title}` : neuron.title}
        </h2>
        <button
          type="button"
          onClick={onClose}
          aria-label={t("brainstyClosePreview")}
          title={t("brainstyClosePreview")}
          className="grid size-6 shrink-0 place-items-center rounded-md text-faint hover:bg-hover hover:text-ink"
        >
          ✕
        </button>
      </div>
      <p className="text-[11.5px] text-faint">
        {[t(`brainstyOne_${neuron.kind}`), zone, fill("brainstyTies", numbered(ties))]
          .filter(Boolean)
          .join(" · ")}
      </p>
      {steps.length > 0 && (
        <p className="text-[12px] text-soft">
          {fill(
            "brainstySteps",
            numbered(steps.filter((one) => one.done).length),
            numbered(steps.length),
          )}
        </p>
      )}
      {glance && (
        <div
          className="scroller min-h-0 flex-1 text-[12.5px] leading-relaxed text-soft"
          // biome-ignore lint/security/noDangerouslySetInnerHtml: the core's translator drew it, as every page
          dangerouslySetInnerHTML={{ __html: glance.html }}
        />
      )}
      {neuron.kind !== "file" && (
        <button
          type="button"
          onClick={onOpen}
          className="self-start rounded-md bg-accent px-3 py-1.5 text-[12.5px] font-medium text-white hover:opacity-90"
        >
          {t("brainstyOpen")}
        </button>
      )}
    </aside>
  );
}
