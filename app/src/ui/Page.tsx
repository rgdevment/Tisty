import { useEffect, useRef, useState } from "react";
import { docDrawn, type Mend, mendsIn } from "../core";
import { datedSteps, type Reaching } from "./dressing";
import Frame from "./Frame";
import { mendable } from "./mending";
import type { Head } from "./paging";
import { escaped, heads } from "./paging";
import { recarded } from "./partcarding";
import type { Sheet } from "./sheeting";

type Props = Reaching & {
  readonly label: string;
  readonly stranger: boolean;
  readonly tall?: boolean;
  readonly untitled?: boolean;
  readonly shaping?: boolean;
  readonly wide?: number;
  readonly zoom?: number;
  readonly onOutline?: (heads: Head[]) => void;
  readonly onShown?: (page: Window | null) => void;
  readonly sheet?: Sheet;
  readonly again?: string;
  readonly cards?: string;
  readonly draw?: (body: string) => Promise<string>;
  readonly onMend?: (one: Mend, body: string) => void;
};

type Drawn = { html: string; turn: number; body: string; mends: Promise<readonly Mend[]> };

// One page is kept and its words change in place, so a redraw never blinks, jumps or builds a new document.
export default function Page(props: Readonly<Props>) {
  const { body, label, stranger, wide, zoom, tall = false, again } = props;
  const reach = useRef(props);
  reach.current = props;
  const [drawing, setDrawing] = useState<Drawn | null>(null);
  const [seen, setSeen] = useState(false);
  const showing = useRef(0);
  const held = useRef<HTMLIFrameElement | null>(null);
  const turns = useRef(0);
  const found = useRef<{ turn: number; said: string; mends: readonly Mend[] } | null>(null);

  // Read when it is laid, so a page that turns read-only drops its fixes without being drawn again.
  const lay = () => {
    const page = held.current?.contentDocument;
    if (!page) return;
    const now = found.current?.turn === showing.current ? found.current : null;
    const mends = reach.current.onMend && now ? now.mends : [];
    mendable(page, mends, (one) => now && reach.current.onMend?.(one, now.said));
  };
  const dates = props.stepDates;
  useEffect(() => {
    const page = held.current?.contentDocument;
    if (page) datedSteps(page, dates);
  }, [dates]);
  const { cards } = props;
  useEffect(() => {
    const page = held.current?.contentDocument;
    if (page && !stranger) recarded(page, reach.current.part);
  }, [cards, stranger]);
  const mending = Boolean(props.onMend);
  useEffect(lay, [mending]);

  useEffect(() => {
    let live = true;
    const drawn = (html: string) => {
      if (!live) return;
      turns.current += 1;
      setDrawing({ html, turn: turns.current, body, mends });
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
    held.current = frame;
    showing.current = turn;
    setSeen(true);
    const page = frame.contentDocument;
    if (page) reach.current.onOutline?.(heads(page));
    mends
      .then((all) => {
        found.current = { turn, said, mends: all };
        if (showing.current === turn) lay();
      })
      .catch(() => {});
    reach.current.onShown?.(frame.contentWindow);
  };

  const current = <K extends keyof Reaching>(asked: K): Reaching[K] | undefined =>
    drawing && drawing.turn === turns.current ? reach.current[asked] : undefined;
  return (
    // Hidden until the first drawing is ready, then shown at once: a fade on every task opened is a wait.
    <div
      className={`${tall ? "relative" : "relative flex min-h-0 flex-1 flex-col"} ${seen ? "opacity-100" : "opacity-0"}`}
    >
      {drawing && (
        <Frame
          html={drawing.html}
          turn={drawing.turn}
          label={label}
          stranger={stranger}
          tall={tall}
          untitled={props.untitled}
          shaping={props.shaping}
          hidden={!seen}
          wide={wide}
          sheet={props.sheet}
          zoom={zoom}
          reach={() => ({
            ...reach.current,
            body: drawing.body,
            // A page with a newer drawing on its way names bytes nobody is writing any more.
            onBlock: current("onBlock"),
            onTick: current("onTick"),
            onPart: current("onPart"),
            onCut: current("onCut"),
            onWiden: current("onWiden"),
            onFormat: current("onFormat"),
            onChoose: current("onChoose"),
            onMenu: current("onMenu"),
            onMore: current("onMore"),
            onDiagram: current("onDiagram"),
          })}
          onReady={(frame) => ready(drawing, frame)}
        />
      )}
    </div>
  );
}
