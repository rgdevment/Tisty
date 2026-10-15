import { type ComponentProps, useEffect, useMemo, useRef, useState } from "react";
import { noteDrawn } from "../core";
import { dressed } from "../markdown";
import Composed from "./Composed";

type Props = Omit<ComponentProps<typeof Composed>, "html"> & {
  text: string;
  steps?: string[];
  hint?: string;
  typing?: boolean;
};

const drawn = new Map<string, string>();
const KEPT = 200;
const SETTLES = 150;

const remember = (text: string, html: string) => {
  if (drawn.size >= KEPT) drawn.delete(drawn.keys().next().value ?? "");
  drawn.set(text, html);
};

export default function Written({ text, steps, hint, typing, ...rest }: Props) {
  const [last, setLast] = useState(() => drawn.get(text));
  const waits = useRef(typing);
  waits.current = typing;

  useEffect(() => {
    if (!text.trim() || drawn.has(text)) return;
    let live = true;
    const ask = () =>
      noteDrawn(text)
        .then((html) => {
          if (!html.trim()) return;
          remember(text, html);
          if (live) setLast(html);
        })
        .catch(() => {});
    const wait = setTimeout(ask, waits.current ? SETTLES : 0);
    return () => {
      live = false;
      clearTimeout(wait);
    };
  }, [text]);

  // While a note is being typed the last drawing stays up, so the preview never flickers to bare words.
  const html = drawn.get(text) ?? (typing ? last : undefined) ?? plain(text);
  const said = JSON.stringify(steps ?? []);
  // biome-ignore lint/correctness/useExhaustiveDependencies: the steps are compared by what they say
  const shown = useMemo(
    () => (!text.trim() && hint ? placeholder(hint) : dressed(html, steps)),
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
