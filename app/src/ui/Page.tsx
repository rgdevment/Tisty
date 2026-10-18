import { useEffect, useRef, useState } from "react";
import { docDrawn, type Mend, mendsIn } from "../core";
import type { Reaching } from "./dressing";
import Frame from "./Frame";
import { mendable } from "./mending";
import type { Head } from "./paging";
import { escaped, heads, shell } from "./paging";
import type { Sheet } from "./sheeting";

type Props = Reaching & {
  readonly label: string;
  readonly stranger: boolean;
  readonly tall?: boolean;
  readonly untitled?: boolean;
  readonly wide?: number;
  readonly zoom?: number;
  readonly onOutline?: (heads: Head[]) => void;
  readonly onShown?: (page: Window | null) => void;
  readonly sheet?: Sheet;
  readonly again?: string;
  readonly draw?: (body: string) => Promise<string>;
  readonly onMend?: (one: Mend, body: string) => void;
};

type Drawn = { html: string; turn: number; body: string; mends: Promise<readonly Mend[]> };

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
  const found = useRef<{ turn: number; said: string; mends: readonly Mend[] } | null>(null);

  // Read when it is laid, so a page that turns read-only drops its fixes without being drawn again.
  const lay = () => {
    const page = held.current.get(showing.current)?.contentDocument;
    if (!page) return;
    const now = found.current?.turn === showing.current ? found.current : null;
    const mends = reach.current.onMend && now ? now.mends : [];
    mendable(page, mends, (one) => now && reach.current.onMend?.(one, now.said));
  };
  const mending = Boolean(props.onMend);
  useEffect(lay, [mending]);

  useEffect(() => {
    let live = true;
    const drawn = (html: string) => {
      if (!live) return;
      turns.current += 1;
      const turn = turns.current;
      setFrames((were) => [
        ...were.filter((one) => one.turn === showing.current),
        { html: shell(html), turn, body, mends },
      ]);
    };
    const mends: Promise<readonly Mend[]> = reach.current.onMend
      ? mendsIn(body)
          .then((said) => said ?? [])
          .catch(() => [])
      : Promise.resolve([]);
    (reach.current.draw ?? docDrawn)(body)
      .catch(() => `<pre>${escaped(body)}</pre>`)
      .then(drawn);
    return () => {
      live = false;
    };
  }, [body, again]);

  useEffect(() => () => reach.current.onShown?.(null), []);

  const ready = ({ turn, body: said, mends }: Drawn, frame: HTMLIFrameElement) => {
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
    mends
      .then((all) => {
        found.current = { turn, said, mends: all };
        lay();
      })
      .catch(() => {});
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
          untitled={props.untitled}
          hidden={one.turn !== shown}
          wide={wide}
          sheet={props.sheet}
          zoom={zoom}
          reach={() => ({
            ...reach.current,
            body: one.body,
            // A page with a newer drawing on its way names bytes nobody is writing any more.
            onBlock: one.turn === turns.current ? reach.current.onBlock : undefined,
            onTick: one.turn === turns.current ? reach.current.onTick : undefined,
            onWiden: one.turn === turns.current ? reach.current.onWiden : undefined,
            onFormat: one.turn === turns.current ? reach.current.onFormat : undefined,
            onMenu: one.turn === turns.current ? reach.current.onMenu : undefined,
            onMore: one.turn === turns.current ? reach.current.onMore : undefined,
          })}
          onReady={(frame) => ready(one, frame)}
        />
      ))}
    </div>
  );
}
