import { useEffect, useRef, useState } from "react";
import {
  type CardAsked,
  cardKey,
  cardsIn,
  cardsOn,
  cardsPlaced,
  type DrawnCards,
  escaped,
  type Head,
  heads,
  type Sheet,
} from "tisty-editor/read";
import { docDrawn, type Mend, mendsIn } from "../core";
import { datedSteps, type Reaching } from "./dressing";
import Frame from "./Frame";
import { type CardsShown, cardDays, drawnFor } from "./liveCards";
import { mendable } from "./mending";

type Props = Reaching & {
  readonly label: string;
  readonly tall?: boolean;
  readonly untitled?: boolean;
  readonly viewing?: boolean;
  readonly shaping?: boolean;
  readonly agent?: boolean;
  readonly wide?: number;
  readonly zoom?: number;
  readonly onOutline?: (heads: Head[]) => void;
  readonly onShown?: (page: Window | null, body?: string) => void;
  readonly sheet?: Sheet;
  readonly again?: string;
  readonly cardsShown?: CardsShown;
  readonly draw?: (body: string) => Promise<string>;
  readonly already?: string | null;
  readonly onDresser?: (dress: (blocks: readonly Element[]) => void) => void;
  readonly onMend?: (one: Mend, body: string) => void;
};

type Drawn = { html: string; turn: number; body: string; mends: Promise<readonly Mend[]> };

const drawnBy = (props: Readonly<Props>, asked: readonly CardAsked[]) =>
  drawnFor(asked, props.cardsShown, {
    here: props.cardsShown?.here,
    acting: Boolean(props.onTaskDone),
    more: Boolean(props.onMore),
    examples: Boolean(props.cardsShown?.examples),
  }).catch(() => new Map<string, string>());

// One page is kept and its words change in place, so a redraw never blinks, jumps or builds a new document.
export default function Page(props: Readonly<Props>) {
  const { body, label, wide, zoom, tall = false, again } = props;
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
  const cards = useRef<DrawnCards>(new Map());
  const seenAs = useRef("");
  // Cards come back after the page is laid, so only those whose drawing changed are laid again.
  const relaid = (asked: readonly CardAsked[]) => {
    const askedAs = seenAs.current;
    return drawnBy(reach.current, asked).then((fresh) => {
      if (fresh.size === 0 || seenAs.current !== askedAs) return;
      const was = cards.current;
      cards.current = new Map([...was, ...fresh]);
      const page = held.current?.contentDocument;
      if (page) cardsPlaced(page, cards.current, was, cardDays);
    });
  };
  const shown = props.cardsShown;
  const acting = Boolean(props.onTaskDone);
  const more = Boolean(props.onMore);
  // biome-ignore lint/correctness/useExhaustiveDependencies: what a card shows changes with what the window heard, never with `relaid` itself.
  useEffect(() => {
    // A card is drawn for its page and its controls, so another of either draws every card anew.
    const seen = `${shown?.here}|${acting}|${more}`;
    if (seenAs.current !== seen) cards.current = new Map();
    seenAs.current = seen;
    const page = held.current?.contentDocument;
    if (page && shown) void relaid(cardsOn(page));
  }, [shown?.heard, shown?.here, shown?.docs, shown?.tasks, acting, more]);
  const mending = Boolean(props.onMend);
  useEffect(lay, [mending]);

  const { draw } = props;
  useEffect(() => {
    let live = true;
    // The page written on in place already shows these words, so only what the handlers read moves on.
    if (reach.current.already === body && turns.current > 0) {
      setDrawing((was) => was && { ...was, body });
      return;
    }
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
    void (draw ?? docDrawn)(body)
      .catch(() => `<pre>${escaped(body)}</pre>`)
      .then((html) => {
        const askedAs = seenAs.current;
        const missing = cardsIn(html).filter((one) => !cards.current.has(cardKey(one)));
        return drawnBy(reach.current, missing).then((fresh) => {
          if (!live) return;
          if (seenAs.current === askedAs) cards.current = new Map([...cards.current, ...fresh]);
          drawn(html);
        });
      });
    return () => {
      live = false;
    };
  }, [body, again, draw]);

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
    reach.current.onShown?.(frame.contentWindow, said);
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
          tall={tall}
          untitled={props.untitled}
          viewing={props.viewing}
          shaping={props.shaping}
          agent={props.agent}
          hidden={!seen}
          wide={wide}
          sheet={props.sheet}
          zoom={zoom}
          reach={() => ({
            ...reach.current,
            body: drawing.body,
            // A page with a newer drawing on its way names bytes nobody is writing any more.
            onBegin: current("onBegin"),
            onHintWords: current("onHintWords"),
            onLanguage: current("onLanguage"),
            onTick: current("onTick"),
            onStick: current("onStick"),
            onPart: current("onPart"),
            onCut: current("onCut"),
            onWiden: current("onWiden"),
            onFormat: current("onFormat"),
            onMenu: current("onMenu"),
            onMore: current("onMore"),
            onDiagram: current("onDiagram"),
            onDiagramExport: current("onDiagramExport"),
            onTable: current("onTable"),
            drawnCards: reach.current.cardsShown ? cards.current : undefined,
            onCardsMissing: (asked) => void relaid(asked),
            cardDays,
          })}
          onDresser={(dress) => reach.current.onDresser?.(dress)}
          onReady={(frame) => ready(drawing, frame)}
        />
      )}
    </div>
  );
}
