import { convertFileSrc } from "@tauri-apps/api/core";
import { useCallback } from "react";
import { type HelpEntry, type HelpPictures, helpDrawn } from "../guide/calls";
import { aimedAt, dressed } from "../guide/help";
import { fill } from "../locales";
import type { Carded } from "./carding";
import Page from "./Page";

interface Props {
  readonly entry: HelpEntry;
  readonly body: string;
  readonly pictures: HelpPictures | null;
  readonly heading?: string;
  readonly anchor: string | null;
  readonly onOpen: (slug: string, heading?: string, anchor?: string | null) => void;
}

const card = (): Carded => ({ kind: "doc", title: "" });

const headed = (page: Document, words: string): Element | undefined =>
  [...page.querySelectorAll("h1, h2, h3, h4")].find((one) => one.textContent?.trim() === words);

export default function HelpArticle({ entry, body, pictures, heading, anchor, onOpen }: Props) {
  const draw = useCallback(
    (text: string) =>
      helpDrawn(text).then((html) =>
        dressed(html, pictures, convertFileSrc, (alt) => fill("helpPictureAway", alt)),
      ),
    [pictures],
  );
  const kept = Object.keys(pictures?.kept ?? {}).length;

  return (
    <Page
      key={`${entry.slug}:${kept}:${heading ?? ""}:${anchor ?? ""}`}
      body={`# ${entry.title}\n\n${body}`}
      draw={draw}
      label={entry.title}
      stranger={false}
      card={card}
      onDoc={() => undefined}
      onHere={(id) => {
        const aimed = aimedAt(id);
        if (aimed) onOpen(aimed.slug, undefined, aimed.anchor);
      }}
      onShown={(seen) => {
        const page = seen?.document;
        if (!page) return;
        const aimed = heading ? headed(page, heading) : anchor ? page.getElementById(anchor) : null;
        aimed?.scrollIntoView({ block: "start" });
      }}
    />
  );
}
