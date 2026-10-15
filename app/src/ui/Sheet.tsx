import { type ReactNode, useCallback, useEffect, useRef, useState } from "react";
import type { Moved } from "../paging";
import { type Editing, edited } from "./blocking";
import { movedCard, putCard } from "./carded";
import Page from "./Page";
import { ASIDE_MARGIN, settles } from "./seeking";
import type { Head } from "./writing";

type Props = {
  readonly value: string;
  readonly label: string;
  readonly reading?: boolean;
  readonly seek?: number;
  readonly anchor?: string;
  readonly above?: ReactNode;
  readonly below?: ReactNode;
  readonly onSeen?: (at: number) => void;
  readonly onWrite: (text: string) => void;
  readonly onDoc?: (file: string) => void;
  readonly onTag?: (tag: string) => void;
  readonly onOpen?: (reference: string) => void;
  readonly onOutline?: (heads: Head[]) => void;
  readonly onInsert?: (put: ((file: string, title: string) => boolean) | null) => void;
  readonly onOrder?: (move: ((file: string, before: string | null) => Moved) | null) => void;
};

export default function Sheet(props: Readonly<Props>) {
  const { value, label, reading = false, above, below } = props;
  const room = useRef<HTMLDivElement>(null);
  const said = useRef(props);
  said.current = props;
  const [drawn, setDrawn] = useState(value);
  const open = useRef<Editing | null>(null);
  const showing = useRef(value);
  showing.current = drawn;
  const latest = useRef(value);
  const shown = useRef<Window | null>(null);
  const putting = useRef(-1);

  useEffect(() => {
    latest.current = value;
    if (!open.current) setDrawn(value);
  }, [value]);

  const written = useCallback((text: string) => {
    latest.current = text;
    said.current.onWrite(text);
  }, []);

  const left = useCallback(() => {
    const was = open.current;
    if (!was) return;
    open.current = null;
    was.leave();
  }, []);

  useEffect(() => {
    if (reading) left();
  }, [reading, left]);

  useEffect(() => () => left(), [left]);

  const block = (at: [number, number], element: HTMLElement) => {
    if (open.current) return left();
    // A click on the page still on screen while the next one is drawn names bytes of an older text.
    if (reading || showing.current !== latest.current) return;
    const base = latest.current;
    const [from, to] = at;
    const piece = base.slice(from, to);
    const tail = piece.endsWith("\n") ? "\n" : "";
    const spliced = (text: string) => `${base.slice(0, from)}${text}${tail}${base.slice(to)}`;
    open.current = edited(
      element,
      piece,
      (text) => written(spliced(text)),
      () => {
        open.current = null;
        setDrawn(latest.current);
      },
    );
  };

  useEffect(() => {
    said.current.onInsert?.(
      reading
        ? null
        : (file, title) => {
            const put = putCard(latest.current, file, title);
            if (put === null) return false;
            written(put);
            setDrawn(put);
            return true;
          },
    );
    said.current.onOrder?.(
      reading
        ? null
        : (file, before) => {
            const done = movedCard(latest.current, file, before);
            if (done.body !== latest.current) {
              written(done.body);
              setDrawn(done.body);
            }
            return done.moved;
          },
    );
    return () => {
      said.current.onInsert?.(null);
      said.current.onOrder?.(null);
    };
  }, [reading, written]);

  const wanted = useRef(props.seek);
  const marked = useRef(props.anchor);
  useEffect(() => {
    const want = wanted.current;
    const mark = marked.current;
    if (!room.current || (!want && !mark)) return;
    const spot = () => {
      const sheet = room.current;
      if (!sheet) return null;
      const page = shown.current?.document;
      const found = mark ? page?.querySelector<HTMLElement>(`[data-doc="${mark}"]`) : null;
      const frame = shown.current?.frameElement as HTMLElement | null | undefined;
      if (!found || !frame) return want ?? null;
      const above = found.getBoundingClientRect().top + frame.getBoundingClientRect().top;
      return Math.max(
        0,
        Math.round(above - sheet.getBoundingClientRect().top + sheet.scrollTop - ASIDE_MARGIN),
      );
    };
    putting.current = 0;
    return settles({
      at: () => room.current,
      want: spot,
      done: () => {
        putting.current = -1;
        const sheet = room.current;
        if (!sheet || sheet.scrollHeight <= sheet.clientHeight) return;
        said.current.onSeen?.(sheet.scrollTop);
      },
      put: (down) => {
        putting.current = down;
      },
      watch: () => room.current?.querySelector<HTMLElement>("iframe") ?? null,
    });
  }, []);

  return (
    <div
      ref={room}
      tabIndex={-1}
      onScroll={(one) => {
        const sheet = one.currentTarget;
        if (putting.current >= 0) return;
        if (sheet.scrollHeight <= sheet.clientHeight) return;
        said.current.onSeen?.(sheet.scrollTop);
      }}
      className={`scroller gutter flex min-h-0 flex-1 flex-col${above || below ? " leafed" : ""}`}
    >
      {above}
      <div className="tisty-sheet shrink-0 overflow-hidden rounded-[10px] bg-sheet">
        <Page
          body={drawn}
          label={label}
          stranger={false}
          tall
          onDoc={(file) => said.current.onDoc?.(file)}
          onOpen={(reference) => said.current.onOpen?.(reference)}
          onTag={(tag) => said.current.onTag?.(tag)}
          onBlock={block}
          onOutline={(heads) => said.current.onOutline?.(heads)}
          onShown={(page) => {
            shown.current = page;
          }}
        />
      </div>
      {below}
      {!below && <div aria-hidden="true" className="h-[40vh] shrink-0" />}
    </div>
  );
}
