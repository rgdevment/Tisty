import { type ReactNode, useCallback, useEffect, useRef, useState } from "react";
import { viewWidget } from "../core";
import { CATCHES, takesFiles } from "../dropped";
import type { Moved } from "../paging";
import { caretAt, type Editing, edited } from "./blocking";
import { movedCard, putCard } from "./carded";
import type { Carded } from "./carding";
import Page from "./Page";
import { charsAt, type Head, spanned } from "./paging";
import { closed, laid, parting } from "./parting";
import { ASIDE_MARGIN, settles } from "./seeking";
import { type Asking, putBlock } from "./slashing";

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

  const quiet = useRef(false);
  const left = useCallback(() => {
    const was = open.current;
    if (!was) return;
    open.current = null;
    was.leave();
  }, []);

  useEffect(() => {
    if (reading) left();
  }, [reading, left]);

  useEffect(
    () => () => {
      quiet.current = true;
      left();
      quiet.current = false;
    },
    [left],
  );

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
    const line = latest.current.includes("\r\n") ? "\r\n" : "\n";
    const ahead = fresh ? parting(latest.current.slice(0, from), false, line) : "";
    const behind = fresh ? parting(latest.current.slice(end), true, line) : "";
    const home = onFile.current;
    let mine = latest.current;
    const spliced = (text: string) => {
      const now = latest.current;
      const said = fresh && (text === seed || !text.trim()) ? "" : text;
      const [gap, rest] = said ? [ahead, behind] : ["", ""];
      const next = `${now.slice(0, from)}${gap}${said}${tail}${rest}${now.slice(end)}`;
      end = from + gap.length + said.length + tail.length + rest.length;
      mine = next;
      return next;
    };
    open.current = edited(
      element,
      piece || seed,
      (text) => written(spliced(text)),
      () => {
        open.current = null;
        // Only a block left by the person closes its gap: text from outside or another file is not its own.
        const own = !quiet.current && home === onFile.current && latest.current === mine;
        if (own && !fresh && !latest.current.slice(from, end).trim()) {
          written(closed(latest.current, from, end));
        }
        setDrawn(latest.current);
      },
      () => {
        const asking = said.current.asking;
        if (!asking) return undefined;
        const before = !latest.current.slice(0, from).trim();
        return {
          kinds: asking.kinds,
          tags: asking.tags,
          pasted: asking.pasted,
          lost: asking.lost,
          ask: (kind, at, put, first) => asking.ask(kind, at, put, first && before),
        };
      },
    );
    open.current.kept(said.current.kept ?? true);
    const editing = open.current.view;
    const caret = seed ? seed.length : caretAt(editing.state.sliceDoc(), read);
    editing.dispatch({ selection: { anchor: caret }, scrollIntoView: true });
  };

  const dropped = (text: string, at?: { left: number; top: number }) => {
    const editing = open.current;
    if (editing) return putBlock(editing.view, text);
    const now = latest.current;
    // A page still drawing names bytes of an older text, so the file then goes at the end.
    const fresh = showing.current === now;
    const page = shown.current?.document;
    const frame = shown.current?.frameElement?.getBoundingClientRect();
    const block =
      fresh && at && page && frame
        ? page
            .elementFromPoint(at.left - frame.left, at.top - frame.top)
            ?.closest<HTMLElement>(".page > [data-at]")
        : null;
    const span = spanned(block?.dataset.at ?? null);
    const next = laid(now, span ? charsAt(now, span[1]) : now.length, text);
    written(next);
    setDrawn(next);
  };
  const dropping = useRef(dropped);
  dropping.current = dropped;

  useEffect(() => {
    const box = room.current;
    if (!box || reading) return;
    return takesFiles(box, (text, at) => dropping.current(text, at));
  }, [reading]);

  const ticked = ([from, end]: [number, number]) => {
    // Like a click that opens a block, a box names bytes of the page on screen, which a block being written has moved.
    if (open.current) return left();
    if (reading || showing.current !== latest.current) return;
    const now = latest.current;
    const box = /^\s*(?:>\s?)*\s*(?:[-*+]|\d+[.)])\s+\[[ xX]\]/.exec(now.slice(from, end));
    if (!box) return;
    const at = from + box[0].length - 3;
    const next = `${now.slice(0, at)}${now.slice(at, at + 3) === "[ ]" ? "[x]" : "[ ]"}${now.slice(at + 3)}`;
    written(next);
    setDrawn(next);
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
      {...(reading ? {} : { [CATCHES]: "" })}
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
          onTick={reading ? undefined : ticked}
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
