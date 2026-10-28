import { useEffect, useRef, useState } from "react";
import rina from "../../src-tauri/resources/guide/en/rina.jpg";
import { helpAnchors, helpDrawn } from "../guide/calls";
import { aimedAt, linkedHere } from "../guide/help";
import { examplesIn, shownBeside, type Topic } from "../guide/reference";
import { t } from "../locales";
import { pictured } from "../previews";
import type { Carded } from "./carding";
import Page from "./Page";
import type { Head } from "./paging";

const titles = new Map<string, string>();
const drawings = new Map<string, Promise<string>>();

// The examples name files only a store holds, so a picture shows the guide's own photo.
const drawn = async (body: string): Promise<string> => {
  const page = new DOMParser().parseFromString(
    `<article>${await helpDrawn(body)}</article>`,
    "text/html",
  );
  const examples = examplesIn(page);
  const looks = await Promise.all(examples.map((code) => helpDrawn(code.textContent ?? "")));
  const words = { written: t("helpWritten"), drawn: t("helpDrawn") };
  examples.forEach((code, at) => {
    shownBeside(code, looks[at], words);
  });
  const photo = new URL(rina, location.href).href;
  for (const img of page.querySelectorAll("img[src]")) {
    const at = img.getAttribute("src") ?? "";
    if (!/^[a-z][a-z0-9+.-]*:/i.test(at) && pictured(at)) img.setAttribute("src", photo);
  }
  for (const card of page.querySelectorAll<HTMLElement>("a.card[data-doc]"))
    titles.set(card.dataset.doc ?? "", card.textContent ?? "");
  return page.querySelector("article")?.innerHTML ?? "";
};

const drawGuide = (body: string): Promise<string> => {
  const known = drawings.get(body);
  if (known) return known;
  const made = drawn(body);
  drawings.set(body, made);
  made.catch(() => drawings.delete(body));
  return made;
};

const card = (file: string): Carded => ({ kind: "doc", title: titles.get(file) ?? "" });

interface Props {
  readonly topic: Topic;
  readonly topics: Topic[];
  readonly anchor: string | null;
  readonly onOpen: (slug: string, anchor: string | null) => void;
  // A link to a help page, which the window opens in place of the guide.
  readonly onHelp: (slug: string, anchor: string | null) => void;
  readonly spanish: boolean;
  readonly onOutline: (heads: Head[]) => void;
}

export default function MarkdownArticle({
  topic,
  topics,
  anchor,
  onOpen,
  onHelp,
  spanish,
  onOutline,
}: Props) {
  const [anchors, setAnchors] = useState<string[][]>([]);
  const view = useRef<Window | null>(null);

  useEffect(() => {
    helpAnchors(topics.map((one) => one.body))
      .then(setAnchors)
      .catch(() => undefined);
  }, [topics]);

  const onHere = (id: string) => {
    const aimed = aimedAt(id);
    if (aimed) {
      onHelp(aimed.slug, aimed.anchor);
      return;
    }
    const owner = topics.findIndex((_, one) => anchors[one]?.includes(id));
    if (owner < 0) return;
    if (topics[owner].slug === topic.slug) {
      view.current?.document.getElementById(id)?.scrollIntoView({ block: "start" });
      return;
    }
    onOpen(topics[owner].slug, anchors[owner][0] === id ? null : id);
  };

  return (
    <Page
      key={`${topic.slug}:${anchor ?? ""}`}
      body={linkedHere(topic.body, spanish)}
      draw={drawGuide}
      label={topic.heading ?? t("helpMarkdown")}
      stranger={false}
      card={card}
      onDoc={() => undefined}
      onHere={onHere}
      onOutline={onOutline}
      onShown={(seen) => {
        view.current = seen;
        if (seen && anchor)
          seen.document.getElementById(anchor)?.scrollIntoView({ block: "start" });
      }}
    />
  );
}
