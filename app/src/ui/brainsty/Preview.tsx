import { useEffect, useRef, useState } from "react";
import type { Neuron } from "../../brainsty";
import { docGlance, type Glance } from "../../citing";
import { numbered } from "../../format";
import { fill, t } from "../../locales";
import { taskCard } from "../../markdown";
import { picturesWithin } from "../dressing";
import Page from "../Page";
import { type Alike, wordsSaid } from "./guessing";

export interface Guessed extends Alike {
  neuron: Neuron;
}

interface Props {
  readonly neuron: Neuron;
  readonly ties: number;
  readonly zone?: string;
  readonly alike?: readonly Guessed[];
  readonly onOpen: () => void;
  readonly onClose: () => void;
  readonly onPick?: (node: number) => void;
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

const TASKS = { tasks: true };

export default function Preview({
  neuron,
  ties,
  zone,
  alike = [],
  onOpen,
  onClose,
  onPick,
}: Props) {
  const [glance, setGlance] = useState<Glance | null>(null);
  const words = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let live = true;
    setGlance(null);
    if (neuron.kind === "doc" || neuron.kind === "page") {
      docGlance(neuron.id).then(
        (said) => live && setGlance({ ...said, html: unmarked(said.html) }),
        () => undefined,
      );
    }
    return () => {
      live = false;
    };
  }, [neuron.id, neuron.kind]);

  useEffect(() => {
    if (!glance || !words.current) return;
    let live = true;
    picturesWithin(words.current, () => live);
    return () => {
      live = false;
    };
  }, [glance]);

  return (
    <aside
      aria-label={neuron.title}
      className="absolute top-3 right-3 z-10 flex max-h-[calc(100%-24px)] w-80 flex-col gap-2 overflow-hidden rounded-[10px] border border-hair bg-sheet p-4 shadow-lift"
    >
      <div className="flex items-start gap-2">
        <h2 className="min-w-0 flex-1 text-[13px] leading-snug font-semibold break-words text-ink">
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
      {neuron.kind === "task" && (
        <div className="scroller max-h-56 shrink-0">
          <Page
            body={`${taskCard(neuron.id, neuron.title)}\n`}
            label={neuron.title}
            tall
            untitled
            cardsShown={TASKS}
            onDoc={() => undefined}
            onTask={onOpen}
            onShown={(page) => page?.document.documentElement.classList.add("glanced")}
          />
        </div>
      )}
      {glance && (
        <div
          ref={words}
          className="scroller min-h-0 flex-1 text-[12.5px] leading-relaxed text-soft [&_img]:h-auto [&_img]:max-w-full [&_img]:rounded-md [&_.media-again]:ml-2 [&_.media-again]:text-accent [&_.media-again]:hover:underline [&_.picture-away]:block [&_.picture-away]:text-faint"
          // biome-ignore lint/security/noDangerouslySetInnerHtml: the core's translator drew it, as every page
          dangerouslySetInnerHTML={{ __html: glance.html }}
        />
      )}
      {alike.length > 0 && <Guesses alike={alike} onPick={onPick} />}
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

function Guesses({
  alike,
  onPick,
}: {
  readonly alike: readonly Guessed[];
  readonly onPick?: (node: number) => void;
}) {
  return (
    <section
      aria-label={t("brainstyAlike")}
      className="flex shrink-0 flex-col gap-2 border-t border-hair pt-2"
    >
      <h3 className="text-[11.5px] font-semibold text-hue-amber">{t("brainstyAlike")}</h3>
      <ul className="flex flex-col gap-1.5">
        {alike.map((one) => (
          <li key={one.node} className="flex flex-col">
            <button
              type="button"
              onClick={() => onPick?.(one.node)}
              className="self-start text-left text-[12.5px] break-words text-ink hover:underline"
            >
              {one.neuron.kind === "tag" ? `#${one.neuron.title}` : one.neuron.title}
            </button>
            <Why words={one.words} />
          </li>
        ))}
      </ul>
    </section>
  );
}

function Why({ words }: { readonly words: readonly string[] }) {
  if (words.length === 0) return null;
  const [before, after = ""] = t("brainstyAlikeWords").split("{name}");
  return (
    <p className="text-[11.5px] text-faint">
      {before}
      {wordsSaid(words, t("brainstyAnd"), t("brainstyAndBeforeI")).map((part, n) =>
        part.word ? (
          // biome-ignore lint/suspicious/noArrayIndexKey: the parts never move
          <em key={n} className="text-soft">
            {part.text}
          </em>
        ) : (
          part.text
        ),
      )}
      {after}
    </p>
  );
}
