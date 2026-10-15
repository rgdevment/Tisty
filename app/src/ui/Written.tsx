import { type ComponentProps, useEffect, useMemo, useState } from "react";
import { docDrawn } from "../core";
import { dressed } from "../markdown";
import Composed from "./Composed";

type Props = Omit<ComponentProps<typeof Composed>, "html"> & {
  text: string;
  steps?: string[];
  hint?: string;
};

const drawn = new Map<string, string>();
const KEPT = 200;

const remember = (text: string, html: string) => {
  if (drawn.size >= KEPT) drawn.delete(drawn.keys().next().value ?? "");
  drawn.set(text, html);
};

export default function Written({ text, steps, hint, ...rest }: Props) {
  const [html, setHtml] = useState(() => drawn.get(text));

  useEffect(() => {
    let live = true;
    const known = drawn.get(text);
    setHtml(known);
    if (known !== undefined || !text.trim()) return;
    docDrawn(text)
      .then((page) => {
        if (!page.trim()) return;
        remember(text, page);
        if (live) setHtml(page);
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  }, [text]);

  const said = JSON.stringify(steps ?? []);
  // biome-ignore lint/correctness/useExhaustiveDependencies: the steps are compared by what they say
  const shown = useMemo(
    () => (!text.trim() && hint ? placeholder(hint) : dressed(html ?? plain(text), steps)),
    [text, hint, html, said],
  );
  return <Composed {...rest} html={shown} />;
}

// Until the core has drawn it, or if it cannot, the note still reads as the words it is.
const plain = (text: string): string =>
  text
    .split(/\n{2,}/)
    .map((block) => block.replace(/[<>&"]/g, (c) => `&#${c.charCodeAt(0)};`))
    .map((block) => `<p>${block.replace(/\n/g, "<br>")}</p>`)
    .join("");

const placeholder = (hint: string): string =>
  `<p class="text-faint">${hint.replace(/[<>&]/g, "")}</p>`;
