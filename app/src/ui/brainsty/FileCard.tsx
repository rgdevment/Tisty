import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { ending, family, KINDS } from "tisty-editor/read";
import { brainFile, type FileFacts, type Neuron } from "../../brainsty";
import { opened, shownInFolder } from "../../core";
import { dayOf, weigh } from "../../format";
import { serve } from "../../inUse";
import { t, type Word } from "../../locales";
import { pictured } from "../../previews";
import { saidPlainly } from "../../refusal";
import Glyph from "../Glyph";
import type { Fact } from "./cardFacts";

const GLYPH: Record<string, string> = {
  image: "file-image",
  video: "file-video",
  audio: "file-audio",
  archive: "file-archive",
  sheet: "sheet",
  data: "table",
  slides: "book",
  plain: "clip",
};

export const fileGlyphOf = (id: string): string => GLYPH[family(ending(id))] ?? "page";

export const fileKindOf = (id: string): string => {
  const kind = ending(id).toUpperCase();
  const word = KINDS[kind.toLowerCase()];
  const said = word ? t(word as Word) : t("brainstyOne_file");
  return kind && kind !== said.toUpperCase() ? `${said} · ${kind}` : said;
};

export const fileFactsOf = (id: string, facts: FileFacts): Fact[] => [
  [t("brainstyFactKind"), fileKindOf(id)],
  [t("brainstyFactSize"), weigh(facts.bytes)],
  ...(facts.changed ? [[t("brainstyFactChanged"), dayOf(facts.changed)] as Fact] : []),
];

// Null until the core answers, and for a file nothing in sight uses: then the card says no more than its name.
export function useFileFacts(neuron: Neuron): FileFacts | null {
  const [told, setTold] = useState<{ id: string; facts: FileFacts | null } | null>(null);
  useEffect(() => {
    let live = true;
    if (neuron.kind === "file") {
      brainFile(neuron.id).then(
        (facts) => live && setTold({ id: neuron.id, facts }),
        () => undefined,
      );
    }
    return () => {
      live = false;
    };
  }, [neuron.id, neuron.kind]);
  // Another node picked meanwhile never wears the facts of the file before it.
  return neuron.kind === "file" && told?.id === neuron.id ? told.facts : null;
}

export function FileLooks({ id }: { readonly id: string }) {
  const [seen, setSeen] = useState<{ id: string; at: string } | null>(null);
  useEffect(() => {
    let live = true;
    if (pictured(id)) {
      serve(id).then(
        (at) => live && setSeen({ id, at: convertFileSrc(at) }),
        () => undefined,
      );
    }
    return () => {
      live = false;
    };
  }, [id]);
  const picture = seen?.id === id ? seen.at : null;
  return picture ? (
    <img
      src={picture}
      alt=""
      className="max-h-40 w-full shrink-0 rounded-md bg-hover object-contain"
    />
  ) : (
    <p className="grid h-20 shrink-0 place-items-center rounded-md bg-hover text-faint">
      <Glyph name={fileGlyphOf(id)} className="h-8 w-8" />
    </p>
  );
}

export function FileActions({ id }: { readonly id: string }) {
  const [slip, setSlip] = useState<string | null>(null);
  const doing = (act: (reference: string) => Promise<void>) => () => {
    setSlip(null);
    act(id).catch((e: unknown) => setSlip(saidPlainly(e)));
  };
  return (
    <div className="flex flex-col gap-1.5">
      <p className="flex flex-wrap items-center gap-2">
        <button
          type="button"
          onClick={doing(opened)}
          className="rounded-md bg-accent px-3 py-1.5 text-[12.5px] font-medium text-white hover:opacity-90"
        >
          {t("brainstyOpen")}
        </button>
        <button
          type="button"
          onClick={doing(shownInFolder)}
          className="rounded-md border border-hair px-3 py-1.5 text-[12.5px] text-soft hover:bg-hover hover:text-ink"
        >
          {t("brainstyShowInFolder")}
        </button>
      </p>
      {slip && (
        <p role="alert" className="text-[11.5px] text-urgent">
          {slip}
        </p>
      )}
    </div>
  );
}
