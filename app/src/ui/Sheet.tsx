import { type ReactNode, useCallback, useEffect, useRef, useState } from "react";
import { viewWidget } from "../core";
import type { Moved } from "../paging";
import { caretAt, type Editing, edited } from "./blocking";
import { movedCard, putCard } from "./carded";
import type { Carded } from "./carding";
import Page from "./Page";
import type { Head } from "./paging";
import { ASIDE_MARGIN, settles } from "./seeking";
import type { Asking } from "./slashing";

type Props = {
  readonly file?: string;
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
  readonly card?: (file: string) => Carded;
  readonly blurb?: (file: string) => Promise<string>;
  readonly again?: string;
  readonly onInsert?: (put: ((file: string, title: string) => boolean) | null) => void;
  readonly onOrder?: (move: ((file: string, before: string | null) => Moved) | null) => void;
  readonly asking?: Asking;
  readonly kept?: boolean;
};

export default function Sheet(props: Readonly<Props>) {
  const { file, value, label, reading = false, above, below } = props;
  const room = useRef<HTMLDivElement>(null);
  const said = useRef(props);
  said.current = props;
  const [drawn, setDrawn] = useState(value);
  const open = useRef<Editing | null>(null);
  const showing = useRef(value);
  showing.current = drawn;
  const latest = useRef(value);
  const echoes = useRef<string[]>([]);
  const shown = useRef<Window | null>(null);
  const putting = useRef(-1);
  const [ready, setReady] = useState(false);
  const [stuck, setStuck] = useState(false);

  const written = useCallback((text: string) => {
    latest.current = text;
    echoes.current.push(text);
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

  useEffect(() => {
    open.current?.kept(props.kept ?? true);
  }, [props.kept]);

  useEffect(() => {
    const outside = (event: MouseEvent) => {
      const target = event.target as Element | null;
      if (!open.current || target?.closest?.("[data-keeps-writing]")) return;
      // The scrollbar beside the page belongs to what holds it, and dragging it is still reading along.
      if (target === room.current) return;
      left();
    };
    window.addEventListener("mousedown", outside, true);
    return () => window.removeEventListener("mousedown", outside, true);
  }, [left]);

  const wanted = useRef(props.seek);
  const marked = useRef(props.anchor);
  const onFile = useRef(file);
  useEffect(() => {
    if (file === onFile.current) return;
    // Another document keeps this sheet, so its page stays in view until the next one is drawn.
    onFile.current = file;
    left();
    latest.current = value;
    echoes.current = [];
    wanted.current = said.current.seek;
    marked.current = said.current.anchor;
    putting.current = 0;
    setReady(false);
    setDrawn(value);
  }, [file, value, left]);

  useEffect(() => {
    if (value === latest.current) {
      echoes.current = [];
      return;
    }
    // Keys typed faster than the window draws come back late, in order; an older text of ours is not news.
    const late = echoes.current.indexOf(value);
    if (late >= 0) {
      echoes.current = echoes.current.slice(late + 1);
      return;
    }
    echoes.current = [];
    // Words from outside (a reload, a step back) end the block being written: it spoke of the old ones.
    latest.current = value;
    if (open.current) left();
    else setDrawn(value);
  }, [value, left]);

  const block = (at: [number, number], element: HTMLElement, seed = "", read = "") => {
    if (open.current) return left();
    // A click on the page still on screen while the next one is drawn names bytes of an older text.
    if (reading || showing.current !== latest.current) return;
    const [from] = at;
    let end = at[1];
    const piece = latest.current.slice(from, end);
    const tail = /\r?\n$/.exec(piece)?.[0] ?? "";
    // A new block is kept apart from the ones around it by a blank line, as Markdown needs.
    const fresh = from === end;
    const before = latest.current.slice(0, from);
    const ahead =
      !fresh || !before.trim() || before.endsWith("\n\n")
        ? ""
        : before.endsWith("\n")
          ? "\n"
          : "\n\n";
    const after = latest.current.slice(end);
    const behind =
      !fresh || !after.trim() || after.startsWith("\n\n")
        ? ""
        : after.startsWith("\n")
          ? "\n"
          : "\n\n";
    const spliced = (text: string) => {
      const now = latest.current;
      const said = fresh && (text === seed || !text.trim()) ? "" : text;
      const [gap, rest] = said ? [ahead, behind] : ["", ""];
      const next = `${now.slice(0, from)}${gap}${said}${tail}${rest}${now.slice(end)}`;
      end = from + gap.length + said.length + tail.length + rest.length;
      return next;
    };
    open.current = edited(
      element,
      piece || seed,
      (text) => written(spliced(text)),
      () => {
        open.current = null;
        setDrawn(latest.current);
      },
      () => {
        const asking = said.current.asking;
        if (!asking) return undefined;
        const before = !latest.current.slice(0, from).trim();
        return {
          kinds: asking.kinds,
          ask: (kind, at, put, first) => asking.ask(kind, at, put, first && before),
        };
      },
    );
    open.current.kept(said.current.kept ?? true);
    const editing = open.current.view;
    const caret = seed ? seed.length : caretAt(editing.state.sliceDoc(), read);
    editing.dispatch({ selection: { anchor: caret }, scrollIntoView: true });
  };

  useEffect(() => {
    said.current.onInsert?.(
      reading
        ? null
        : (file, title) => {
            left();
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
            left();
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
  }, [reading, written, left]);

  // The sheet takes the keys without a caret, so nothing pulls the view to where one would be.
  useEffect(() => room.current?.focus({ preventScroll: true }), []);

  useEffect(() => {
    const want = wanted.current;
    const mark = marked.current;
    if (!ready || !room.current) return;
    if (!want && !mark) {
      if (putting.current < 0) return;
      room.current.scrollTop = 0;
      putting.current = -1;
      return;
    }
    const spot = () => {
      const sheet = room.current;
      if (!sheet) return null;
      const page = shown.current?.document;
      const found = mark ? page?.querySelector<HTMLElement>(`[data-doc="${mark}"]`) : null;
      const box = found?.getBoundingClientRect();
      if (!box || (!box.height && !box.top)) return want ?? null;
      const frame = shown.current?.frameElement as HTMLElement | null | undefined;
      const above = box.top + (frame?.getBoundingClientRect().top ?? 0);
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
      watch: () => room.current?.querySelector<HTMLElement>("iframe, .page") ?? null,
    });
  }, [ready]);

  return (
    <div
      ref={room}
      tabIndex={-1}
      onScroll={(one) => {
        const sheet = one.currentTarget;
        setStuck(sheet.scrollTop > 8);
        if (putting.current >= 0) return;
        if (sheet.scrollHeight <= sheet.clientHeight) return;
        said.current.onSeen?.(sheet.scrollTop);
      }}
      className={`scroller gutter flex min-h-0 flex-1 flex-col outline-none${above || below ? " leafed" : ""}${stuck ? " stuck" : ""}`}
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
          onBlock={reading ? undefined : block}
          onAway={left}
          card={props.card}
          blurb={props.blurb}
          again={props.again}
          onApart={(source) => {
            const from = said.current.file;
            viewWidget(
              { source },
              from ? { id: "", file: from, title: said.current.label, place: "" } : undefined,
            ).catch(() => {});
          }}
          onOutline={(heads) => said.current.onOutline?.(heads)}
          onShown={(page) => {
            shown.current = page;
            if (page) setReady(true);
          }}
        />
      </div>
      {below}
      {!below && <div aria-hidden="true" className="h-[40vh] shrink-0" />}
    </div>
  );
}
