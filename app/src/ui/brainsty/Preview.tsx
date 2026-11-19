import { useEffect, useState } from "react";
import {
  brainCard,
  type Card,
  type Near,
  type Neuron,
  type Tie,
  taskAddress,
} from "../../brainsty";
import { copied, docAddress } from "../../core";
import { numbered } from "../../format";
import { fill, t } from "../../locales";
import { CARDED, factsOf } from "./cardFacts";
import { type Alike, wordsSaid } from "./guessing";

export interface Guessed extends Alike {
  neuron: Neuron;
}

interface Props {
  readonly neuron: Neuron;
  readonly ties: number;
  readonly tied: readonly [Tie, number][];
  readonly zone?: string;
  readonly alike?: readonly Guessed[];
  readonly nodeOf?: (id: string) => number | undefined;
  readonly onOpen: () => void;
  readonly onClose: () => void;
  readonly onPick?: (node: number) => void;
}

const POINTING_SHOWN = 6;

const named = (one: { kind: string; title: string }): string =>
  one.kind === "tag" ? `#${one.title}` : one.title;

export default function Preview({
  neuron,
  ties,
  tied,
  zone,
  alike = [],
  nodeOf,
  onOpen,
  onClose,
  onPick,
}: Props) {
  const [card, setCard] = useState<Card | null>(null);

  useEffect(() => {
    let live = true;
    setCard(null);
    if (CARDED.has(neuron.kind)) {
      brainCard(neuron.id).then(
        (said) => live && setCard(said),
        () => undefined,
      );
    }
    return () => {
      live = false;
    };
  }, [neuron.id, neuron.kind]);

  const facts = card ? factsOf(neuron, card) : [];
  const pointing = [...(card?.cited_by ?? []), ...(card?.mentioned_by ?? [])];
  return (
    <aside
      aria-label={neuron.title}
      className="absolute top-3 right-3 z-10 flex max-h-[calc(100%-24px)] w-80 flex-col gap-3 overflow-hidden rounded-[10px] border border-hair bg-sheet p-4 shadow-lift"
    >
      <div className="flex flex-col gap-1">
        <div className="flex items-start gap-2">
          <h2 className="min-w-0 flex-1 text-[13px] leading-snug font-semibold break-words text-ink">
            {named(neuron)}
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
      </div>
      <div className="scroller flex min-h-0 flex-col gap-3">
        {facts.length > 0 && (
          <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-[12px]">
            {facts.map(([label, value]) => (
              <div key={label} className="contents">
                <dt className="text-faint">{label}</dt>
                <dd className="m-0 text-ink tabular-nums">{value}</dd>
              </div>
            ))}
          </dl>
        )}
        {card?.tags && card.tags.length > 0 && (
          <Part title={t("brainstyFactTags")}>
            <p className="flex flex-wrap gap-1">
              {card.tags.map((tag) => (
                <span
                  key={tag}
                  className="rounded-full bg-hover px-2 py-0.5 text-[11.5px] text-soft"
                >
                  #{tag}
                </span>
              ))}
            </p>
          </Part>
        )}
        {tied.length > 0 && (
          <Part title={t("brainstyTiesTitle")}>
            <p className="flex flex-wrap gap-x-3 gap-y-1 text-[12px] text-soft">
              {tied.map(([tie, many]) => (
                <span key={tie}>
                  <span className="text-ink tabular-nums">{numbered(many)}</span>{" "}
                  {t(`brainstyTies_${tie}`)}
                </span>
              ))}
            </p>
          </Part>
        )}
        {pointing.length > 0 && (
          <Part title={t("brainstyPointedHere")}>
            <Pointing all={pointing} nodeOf={nodeOf} onPick={onPick} />
          </Part>
        )}
        {alike.length > 0 && <Guesses alike={alike} onPick={onPick} />}
        <Address neuron={neuron} />
      </div>
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

function Part({ title, children }: { readonly title: string; readonly children: React.ReactNode }) {
  return (
    <section aria-label={title} className="flex flex-col gap-1">
      <h3 className="text-[11px] font-semibold tracking-wide text-faint uppercase">{title}</h3>
      {children}
    </section>
  );
}

const addressOf = (neuron: Neuron): Promise<string | null> => {
  if (neuron.kind === "task") return Promise.resolve(taskAddress(neuron.id));
  if (neuron.kind === "doc" || neuron.kind === "page") return docAddress(neuron.id);
  return Promise.resolve(null);
};

function Address({ neuron }: { readonly neuron: Neuron }) {
  const [address, setAddress] = useState<string | null>(null);
  const [taken, setTaken] = useState(false);

  useEffect(() => {
    let live = true;
    setAddress(null);
    setTaken(false);
    addressOf(neuron).then(
      (said) => live && setAddress(said),
      () => undefined,
    );
    return () => {
      live = false;
    };
  }, [neuron]);

  if (!address) return null;
  return (
    <Part title={t("brainstyFactLink")}>
      <p className="flex min-w-0 items-center gap-2">
        <code className="min-w-0 flex-1 truncate text-[11.5px] text-soft" title={address}>
          {address}
        </code>
        <button
          type="button"
          onClick={() =>
            copied(address).then(
              () => setTaken(true),
              () => undefined,
            )
          }
          className="shrink-0 rounded-md px-2 py-0.5 text-[11.5px] text-accent hover:bg-hover"
        >
          {taken ? t("linkCopied") : t("copyLink")}
        </button>
      </p>
    </Part>
  );
}

function Pointing({
  all,
  nodeOf,
  onPick,
}: {
  readonly all: readonly Near[];
  readonly nodeOf?: (id: string) => number | undefined;
  readonly onPick?: (node: number) => void;
}) {
  const shown = all.slice(0, POINTING_SHOWN);
  const more = all.length - shown.length;
  return (
    <ul className="flex flex-col gap-0.5 text-[12.5px]">
      {shown.map((one) => {
        const node = nodeOf?.(one.id);
        return (
          <li key={one.id} className="truncate">
            {node === undefined ? (
              <span className="text-soft">{named(one)}</span>
            ) : (
              <button
                type="button"
                onClick={() => onPick?.(node)}
                className="max-w-full truncate text-left text-ink hover:underline"
              >
                {named(one)}
              </button>
            )}
          </li>
        );
      })}
      {more > 0 && <li className="text-faint">{fill("brainstyLegendMore", numbered(more))}</li>}
    </ul>
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
