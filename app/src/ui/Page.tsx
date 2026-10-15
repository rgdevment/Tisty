import { useEffect, useRef, useState } from "react";
import { docDrawn } from "../core";
import type { Reaching } from "./dressing";
import Frame from "./Frame";
import type { Head } from "./paging";
import { escaped, heads, shell } from "./paging";

type Props = Reaching & {
  readonly label: string;
  readonly stranger: boolean;
  readonly tall?: boolean;
  readonly wide?: number;
  readonly zoom?: number;
  readonly onOutline?: (heads: Head[]) => void;
  readonly onShown?: (page: Window | null) => void;
  readonly again?: string;
};

type Drawn = { html: string; turn: number; body: string };

// A new drawing waits unseen until it is ready, so a redraw never blinks or jumps.
export default function Page(props: Readonly<Props>) {
  const { body, label, stranger, wide, zoom, tall = false, again } = props;
  const reach = useRef(props);
  reach.current = props;
  const [frames, setFrames] = useState<Drawn[]>([]);
  const [shown, setShown] = useState(0);
  const showing = useRef(0);
  const held = useRef(new Map<number, HTMLIFrameElement>());
  const turns = useRef(0);

  useEffect(() => {
    let live = true;
    const drawn = (html: string) => {
      if (!live) return;
      turns.current += 1;
      const turn = turns.current;
      setFrames((were) => [
        ...were.filter((one) => one.turn === showing.current),
        { html: shell(html), turn, body },
      ]);
    };
    docDrawn(body)
      .then(drawn)
      .catch(() => drawn(`<pre>${escaped(body)}</pre>`));
    return () => {
      live = false;
    };
  }, [body, again]);

  useEffect(() => () => reach.current.onShown?.(null), []);

  const ready = (turn: number, frame: HTMLIFrameElement) => {
    held.current.set(turn, frame);
    const was = held.current.get(showing.current);
    if (!tall && was && was !== frame)
      frame.contentWindow?.scrollTo(0, was.contentWindow?.scrollY ?? 0);
    for (const one of held.current.keys()) if (one < turn) held.current.delete(one);
    showing.current = turn;
    setShown(turn);
    setFrames((were) => were.filter((one) => one.turn >= turn));
    const page = frame.contentDocument;
    if (page) reach.current.onOutline?.(heads(page));
    reach.current.onShown?.(frame.contentWindow);
  };

  return (
    <div className={tall ? "relative" : "relative flex min-h-0 flex-1 flex-col"}>
      {frames.map((one) => (
        <Frame
          key={one.turn}
          html={one.html}
          label={label}
          stranger={stranger}
          tall={tall}
          hidden={one.turn !== shown}
          wide={wide}
          zoom={zoom}
          reach={() => ({
            ...reach.current,
            body: one.body,
            // A page with a newer drawing on its way names bytes nobody is writing any more.
            onBlock: one.turn === turns.current ? reach.current.onBlock : undefined,
          })}
          onReady={(frame) => ready(one.turn, frame)}
        />
      ))}
    </div>
  );
}
