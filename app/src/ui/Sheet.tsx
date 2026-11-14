import { EditorSelection, Transaction } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { save as intoFile } from "@tauri-apps/plugin-dialog";
import {
  type CSSProperties,
  type ReactNode,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import {
  type Asking,
  BAR_CSS,
  type Crossing,
  caretAt,
  type Editing,
  enteredAt,
  FORMATS,
  flipped,
  formatButtons,
  MARKED_FOR,
  markLately,
  pressed,
  putBlock,
  SLASHED,
  stagesIn,
  type Widths,
  WRITTEN,
  widened,
  wrapped,
} from "tisty-editor/edit";
import type { Carded, Head, PartCarded, Sheet as SheetSize } from "tisty-editor/read";
import { bytesAt, charsAt, spanned, stepLine, steppedTitle } from "tisty-editor/read";
import { copied, copyAttached, type Mend, opened, viewWidget } from "../core";
import { viewDiagram } from "../drawing";
import { CATCHES, takesFiles } from "../dropped";
import { t } from "../locales";
import { DOC, partCard } from "../markdown";
import type { Moved } from "../paging";
import { docPlaced, docPlacedNear } from "../placing";
import { named } from "../previews";
import { inEditor, reanchored, spanOf, touches } from "./anchoring";
import { cardedAt, movedCard, putCard } from "./carded";
import { exportChoices, fileName } from "./drawing/exporting";
import { useKeptHeight } from "./heights";
import { edited, HOST } from "./hosting";
import { ALT, livelyChoices } from "./livelyMenu";
import { LivelyBelow, useFrameScroll, type Viewing, ViewSwitch } from "./liveSheet";
import Menu, { type Choice } from "./Menu";
import { mended as mendedText } from "./mending";
import Page from "./Page";
import type { Act } from "./pageMoving";
import { type Hands, type PageWriting, writtenOn } from "./pageWriting";
import type { Went } from "./paging";
import { closed, joined, laid, parting } from "./parting";
import type { Chosen, Formatting, Menued } from "./quickly";
import { type Reshape, reshaped } from "./recarding";
import Skeleton from "./Skeleton";
import { ASIDE_MARGIN, settles } from "./seeking";
import { carried, firstInSight } from "./sighted";
import { movedBlock } from "./sticking";
import { useTables } from "./tableSheet";
import { darkNow } from "./theming";
import { usePictures } from "./usePictures";

const BAR = { buttons: formatButtons, css: BAR_CSS };

type Props = {
  readonly file?: string;
  readonly value: string;
  readonly label: string;
  readonly reading?: boolean;
  readonly seek?: number;
  readonly anchor?: string;
  readonly above?: ReactNode;
  readonly below?: ReactNode;
  readonly belowFolds?: boolean;
  readonly onSeen?: (at: number) => void;
  readonly onWrite: (text: string) => void;
  readonly onSettle?: (before: string, now: string, renamed?: readonly [string, string][]) => void;
  readonly onOwn?: (file: string) => void;
  readonly onFail?: (problem: unknown) => void;
  readonly onNote?: (text: string) => void;
  readonly onDoc?: (file: string) => void;
  readonly onTag?: (tag: string) => void;
  readonly onOpen?: (reference: string) => void;
  readonly onOutline?: (heads: Head[]) => void;
  readonly card?: (file: string) => Carded;
  readonly part?: (id: string) => PartCarded;
  readonly onTask?: (id: string) => void;
  readonly onPart?: (title: string) => Promise<string | null>;
  readonly blurb?: (file: string) => Promise<string>;
  readonly again?: string;
  readonly cards?: string;
  readonly onInsert?: (put: ((file: string, title: string) => boolean) | null) => void;
  readonly onOrder?: (move: ((file: string, before: string | null) => Moved) | null) => void;
  readonly asking?: Asking;
  readonly stepDates?: Readonly<Record<string, string>>;
  readonly kept?: boolean;
  readonly inline?: boolean;
  readonly shaping?: boolean;
  // Whoever holds the page turns its view from beside it; left out, the page wears its own switch.
  readonly view?: Viewing;
  readonly sheet?: SheetSize;
  /** The page as it is drawn, for whoever prints it the way it is seen. */
  readonly onPage?: (page: Window | null) => void;
  readonly lately?: readonly [number, number] | null;
  readonly looking?: number;
  // Only a document is written whole; a task's page keeps the sections of its template.
  readonly wholeable?: boolean;
  // The folder the document sits in, for the windows it opens.
  readonly place?: string;
  // Each new count opens the whole document, or leaves it when it is open.
  readonly wholeAsked?: number;
  // Each new count chooses every word: the block being written, or the page as read.
  readonly allAsked?: number;
};

type Arrival = Crossing & { at: number };

// A block named by its first byte in the new words, and where on the window's screen it should stand.
type Landing = { bytes: number; y: number; turn: number };

// Blocks whose drawing takes the place of their words, so they are written in the Markdown view.
const DRAWN_OVER = new Set(["math", "widget", "mermaid"]);

const cameTo = (view: EditorView | undefined) => {
  if (!view) return;
  view.focus();
  view.dispatch({
    effects: EditorView.scrollIntoView(view.state.selection.main.head, { y: "center" }),
  });
};

const fenceInside = (text: string, at: number) => {
  const line = text.indexOf("\n", at);
  return line < 0 ? at : line + 1;
};

// Pictures, diagrams and the sheets they push down can arrive this long after a page is drawn.
const HELD_FOR = 4000;

export default function Sheet(props: Readonly<Props>) {
  const { file, value, label, reading = false, above, below } = props;
  const fixed = Boolean(props.inline && props.shaping);
  const room = useRef<HTMLDivElement>(null);
  const said = useRef(props);
  said.current = props;
  const lively = !props.inline && Boolean(props.wholeable);
  const widthOf = lively && !props.sheet ? "w-[85%] max-w-[1100px]" : "w-full";
  const diagram = (at: number, nth: number, source: string) => {
    const named = said.current.file;
    // A page not written yet has no file the diagram could be saved back into.
    if (!named || named.startsWith("unwritten:")) return;
    const paper = {
      id: "",
      file: named,
      title: said.current.label,
      place: said.current.place ?? "",
    };
    viewDiagram(paper, at, nth, source).catch((e) => said.current.onFail?.(e));
  };
  const diagramOut = (nth: number, source: string, at: { x: number; y: number }) =>
    setMenu({
      ...at,
      label: t("diagramExport"),
      choices: exportChoices({
        source,
        name: () => Promise.resolve(fileName(said.current.label, nth)),
        dark: darkNow(),
        said: (text) => said.current.onNote?.(text),
        failed: (e) => said.current.onFail?.(e),
      }),
    });
  const [drawn, setDrawn] = useState(value);
  const open = useRef<Editing | null>(null);
  const livelyOpen = useRef(false);
  // Follows the open block through words written around it from outside; false when they reach it.
  const follows = useRef<((was: string, now: string) => boolean) | null>(null);
  // The blocks the whole-document editor covers, shown again as it is left.
  const veiled = useRef<(() => void) | null>(null);
  // Asked while the page was still being drawn again, so it opens once the new one is shown.
  const pending = useRef<
    | ({ turn: number } & ({ caret: number; all: boolean; lively: boolean } | { arrival: Arrival }))
    | null
  >(null);
  // Where the caret stands in the whole text, so the whole document opens where the block was.
  const caretNow = useRef<(() => number) | null>(null);
  // One fading at a time: a mark laid again outlives the timer of the one before.
  const fading = useRef<ReturnType<typeof setTimeout> | null>(null);
  const markIn = (writing: Editing["view"], at: { from: number; to: number }) => {
    writing.dispatch({ effects: markLately.of(at) });
    if (fading.current) clearTimeout(fading.current);
    fading.current = setTimeout(() => {
      if (open.current?.view === writing) writing.dispatch({ effects: markLately.of(null) });
    }, MARKED_FOR);
  };
  const showing = useRef(value);
  showing.current = drawn;
  // The whole-document editor scrolls inside its frame, so that is where the place read is kept.
  const framed = () => (shown.current?.frameElement ? shown.current : null);
  const scroller = (): HTMLElement | null => {
    const inner = lively ? framed() : null;
    return inner ? (inner.document.scrollingElement as HTMLElement | null) : room.current;
  };
  const latest = useRef(value);
  const echoes = useRef<string[]>([]);
  const shown = useRef<Window | null>(null);
  const putting = useRef(-1);
  const hanging = useRef(false);
  const [ready, setReady] = useState(false);
  const [ownMarkdown, setOwnMarkdown] = useState(false);
  const markdown = props.view?.markdown ?? ownMarkdown;
  const setMarkdown = props.view?.onTurn ?? setOwnMarkdown;
  const markdownAt = useRef<number | null>(null);
  const markdownOn = (at: number) => {
    markdownAt.current = at;
    setMarkdown(true);
  };
  // Preview is the core's own page written on in place; only the Markdown view is an editor of plain words.
  // A document and a task's page are both written on as the core draws them.
  const writable = !reading && (lively || Boolean(props.inline));
  const onPage = writable && !markdown;
  const pageWriter = useRef<{ page: Document; hands: PageWriting } | null>(null);
  const alreadyShown = useRef<string | null>(null);
  const shownBody = useRef<string | null>(null);
  const dresser = useRef<((blocks: readonly Element[]) => void) | null>(null);
  const [turned, setTurned] = useState(0);
  const turnedNow = useRef(0);
  turnedNow.current = turned;
  const kept = useKeptHeight<HTMLDivElement>(file ?? label);
  const [more, setMore] = useState<Extract<Went, { kind: "more" }> | null>(null);
  const [menu, setMenu] = useState<{
    x: number;
    y: number;
    choices: Choice[];
    label?: string;
    find?: string;
    foot?: string;
    unpicked?: () => void;
  } | null>(null);
  const askedOn = useRef("");
  const landing = useRef<Landing | null>(null);
  const holdingNow = useRef(() => {});
  const [stuck, setStuck] = useState(false);

  const written = useCallback((text: string) => {
    latest.current = text;
    echoes.current.push(text);
    said.current.onWrite(text);
  }, []);

  const settled = useCallback(
    (before: string, now: string, renamed?: readonly [string, string][]) => {
      if (before !== now) said.current.onSettle?.(before, now, renamed);
    },
    [],
  );

  // The first block in sight, found again in the new words, so a page drawn anew keeps it where it was.
  const inSight = useCallback((before: string, next: string, moving?: number): Landing | null => {
    const page = shown.current?.document;
    const frame = shown.current?.frameElement?.getBoundingClientRect();
    const sheet = room.current;
    if (!page || !frame || !sheet || showing.current !== before) return null;
    const top = sheet.getBoundingClientRect().top - frame.top;
    const blocks = Array.from(
      page.querySelectorAll<HTMLElement>(".page > [data-at]:not(.front)"),
    ).filter((one) => moving === undefined || spanned(one.dataset.at ?? null)?.[0] !== moving);
    const block = firstInSight(blocks, top);
    const at = spanned(block?.dataset.at ?? null);
    if (!block || !at) return null;
    const from = charsAt(before, at[0]);
    const start = reanchored(before, next, from, charsAt(before, at[1]))?.[0] ?? from;
    const y = block.getBoundingClientRect().top + frame.top;
    return { bytes: bytesAt(next, start), y, turn: turnedNow.current };
  }, []);

  // `moving` is the first byte of a block on its way elsewhere, which cannot hold the view in place.
  const whole = useCallback(
    (next: string, moving?: number) => {
      const before = latest.current;
      const waiting = landing.current;
      // A page not drawn yet cannot be looked at, so the place still waiting is carried into the new words.
      landing.current =
        inSight(before, next, moving) ??
        (waiting && { ...waiting, bytes: carried(before, next, waiting.bytes) });
      written(next);
      setDrawn(next);
      settled(before, next);
    },
    [written, settled, inSight],
  );

  const quiet = useRef(false);
  const left = useCallback(() => {
    const was = open.current;
    if (!was) return;
    open.current = null;
    follows.current = null;
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
      if (!open.current || livelyOpen.current || target?.closest?.("[data-keeps-writing]")) return;
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
    pending.current = null;
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
    open.current?.flush?.();
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
    // Words from outside end the block being written only when they reach it: it spoke of the old ones.
    const was = latest.current;
    latest.current = value;
    if (!open.current) setDrawn(value);
    else if (!follows.current?.(was, value)) left();
  }, [value, left]);

  const block = (
    at: [number, number],
    element: HTMLElement,
    seed = "",
    read = "",
    asked?: Pick<Formatting, "said" | "open" | "close">,
    whole?: { caret: number; all?: boolean; lively?: boolean },
    entering?: Arrival,
  ) => {
    if (open.current) return left();
    if (pending.current && "arrival" in pending.current) pending.current = null;
    // A click on the page still on screen while the next one is drawn names bytes of an older text.
    if (reading || showing.current !== latest.current) return;
    let [from, end] = at;
    const piece = latest.current.slice(from, end);
    const tail = /(\r?\n)+$/.exec(piece)?.[0] ?? "";
    // A new block is kept apart from the ones around it by a blank line, as Markdown needs.
    const fresh = from === end;
    const line = latest.current.includes("\r\n") ? "\r\n" : "\n";
    const upto = latest.current.slice(0, from);
    // A box added right under a list joins it, where a blank line would split it in two.
    const mark = /^([-*+]|\d+[.)]) \[ \] /.exec(seed)?.[1];
    const lead = mark && (/^\d/.test(mark) ? `\\d+\\${mark.slice(-1)}` : `\\${mark}`);
    const joins =
      lead !== undefined && new RegExp(`(^|\\n)[ \\t>]*${lead} \\[[ xX]\\][^\\n]*\\n?$`).test(upto);
    const ahead = !fresh
      ? ""
      : joins
        ? upto.endsWith("\n")
          ? ""
          : line
        : parting(upto, false, line);
    const behind = fresh ? parting(latest.current.slice(end), true, line) : "";
    const home = onFile.current;
    let before = latest.current;
    let mine = latest.current;
    let going: Crossing | null = null;
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
      (_, renamed) => {
        open.current = null;
        livelyOpen.current = false;
        follows.current = null;
        veiled.current?.();
        veiled.current = null;
        // Only a block left by the person closes its gap: text from outside or another file is not its own.
        const ours = home === onFile.current && latest.current === mine;
        let at = going?.forward ? end : from;
        if (ours && !quiet.current && !fresh && !latest.current.slice(from, end).trim()) {
          written(closed(latest.current, from, end));
          at = from;
        }
        if (ours && going?.join) {
          const merged = joined(latest.current, at);
          written(merged.text);
          at = merged.at;
        }
        setDrawn(latest.current);
        if (ours) settled(before, latest.current, renamed);
        if (!ours || !going) return;
        const arrival = { ...going, at };
        if (showing.current === latest.current) arrive(arrival);
        else pending.current = { arrival, turn: turnedNow.current };
      },
      () => {
        const asking = said.current.asking;
        const before = !latest.current.slice(0, from).trim();
        return {
          kinds: asking?.kinds ?? [],
          tags: asking?.tags,
          pasted: asking?.pasted,
          lost: asking?.lost,
          failed: (problem) => said.current.onFail?.(problem),
          // A table made from the `/` menu is written from the page at once, its first title open.
          tabled: whole
            ? undefined
            : (at) => {
                const start = from + ahead.length + at;
                tables.freshAt(new TextEncoder().encode(latest.current.slice(0, start)).length);
                queueMicrotask(left);
              },
          ask: (kind, at, put, first) => asking?.ask(kind, at, put, first && before),
          steps: () => stagesIn(latest.current),
        };
      },
      Boolean(said.current.inline && said.current.shaping),
      whole?.lively ? "live" : Boolean(whole),
      fixed || !said.current.wholeable
        ? undefined
        : (all) => queueMicrotask(() => (whole && !all ? left() : wholly(undefined, all))),
      whole || said.current.inline
        ? undefined
        : (one) => {
            going = one;
            left();
          },
    );
    follows.current = (was, now) => {
      // The whole document takes the words in place, the caret carried along, instead of ending.
      const writing = (open.current as Editing | null)?.view;
      if (whole && writing) {
        const kept = writing.state.sliceDoc();
        // The editor keeps the line ends it opened with, whatever the agent wrote with.
        const theirs = now.replace(/\r?\n$/, "").replace(/\r?\n/g, writing.state.lineBreak);
        const span = spanOf(kept, theirs);
        const changes = {
          from: inEditor(kept, span.from),
          to: inEditor(kept, span.to),
          insert: span.insert,
        };
        end = now.length;
        before = now;
        mine = now;
        // Not the person's to undo: Ctrl+Z takes back their words, never the agent's.
        if (changes.from !== changes.to || changes.insert) {
          const to = changes.from + inEditor(span.insert, span.insert.length);
          writing.dispatch({
            changes,
            userEvent: "outside",
            annotations: Transaction.addToHistory.of(false),
          });
          markIn(writing, { from: changes.from, to });
        }
        return true;
      }
      const moved = reanchored(was, now, from, end);
      if (!moved) return false;
      [from, end] = moved;
      // What the person changed is still told against the block as it opened, now among the new words.
      before = `${now.slice(0, from)}${piece}${now.slice(end)}`;
      mine = now;
      return true;
    };
    caretNow.current = () => {
      const writing = open.current?.view;
      const head = writing?.state.selection.main.head ?? 0;
      // Back to the text's own count, where a Windows line end takes two places.
      const ends =
        writing?.state.lineBreak === "\r\n" ? writing.state.doc.lineAt(head).number - 1 : 0;
      return from + head + ends;
    };
    livelyOpen.current = Boolean(whole?.lively);
    open.current.kept(said.current.kept ?? true);
    const editing = open.current.view;
    const caret = whole
      ? inEditor(latest.current, whole.caret)
      : seed
        ? seed.length
        : entering
          ? entering.join
            ? inEditor(editing.state.sliceDoc(), entering.at - from)
            : enteredAt(editing, entering.forward, entering.x)
          : inEditor(editing.state.sliceDoc(), caretAt(editing.state.sliceDoc(), read));
    const length = editing.state.doc.length;
    editing.dispatch({
      selection: whole?.all
        ? { anchor: 0, head: length }
        : entering
          ? EditorSelection.create([EditorSelection.cursor(Math.min(caret, length), 1)])
          : { anchor: Math.min(caret, length) },
      scrollIntoView: !whole?.lively || (whole?.caret ?? 0) > 0,
    });
    if (asked) formatted(editing, read, asked);
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: `wholly` reads only refs, so each drawing alone decides.
  useEffect(() => {
    if (!writable || !markdown || !ready || open.current || !framed()) return;
    if (drawn !== latest.current) return;
    // A page drawn anew wipes whatever was laid in it, so the view waits for the drawing to land.
    if (alreadyShown.current !== latest.current && shownBody.current !== latest.current) return;
    const at = markdownAt.current;
    markdownAt.current = null;
    wholly(at ?? 0, false, true);
    if (at !== null) requestAnimationFrame(() => cameTo(open.current?.view));
  }, [writable, markdown, ready, turned, drawn]);

  const [viewedFile, setViewedFile] = useState(file);
  if (viewedFile !== file) {
    setViewedFile(file);
    setOwnMarkdown(false);
  }

  // biome-ignore lint/correctness/useExhaustiveDependencies: `pageWritten` reads only refs, so the view alone decides.
  useEffect(() => {
    if (!markdown && livelyOpen.current) left();
    if (onPage) {
      pageWritten(shown.current?.document ?? null);
      return;
    }
    pageWriter.current?.hands.stop();
    pageWriter.current = null;
  }, [markdown, onPage, left]);

  const { wholeAsked, allAsked } = props;
  // biome-ignore lint/correctness/useExhaustiveDependencies: `wholly` and `left` read only refs, so the count alone decides.
  useEffect(() => {
    if (!wholeAsked) return;
    if (writable) {
      setMarkdown(!markdown);
      return;
    }
    if (veiled.current) left();
    else wholly();
  }, [wholeAsked]);

  useEffect(() => {
    if (!allAsked) return;
    const editing = open.current?.view;
    if (editing) {
      editing.focus();
      editing.dispatch({ selection: { anchor: 0, head: editing.state.doc.length } });
      return;
    }
    const page = shown.current;
    const sheet = page?.document.querySelector(".page");
    if (!page || !sheet) return;
    page.focus();
    const range = page.document.createRange();
    range.selectNodeContents(sheet);
    const chosen = page.getSelection();
    chosen?.removeAllRanges();
    chosen?.addRange(range);
  }, [allAsked]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: each new drawing of the page may be the one a pending ask waits for.
  useEffect(() => {
    const waiting = pending.current;
    // Only a drawing made after the ask holds the words; the old page is about to be thrown away.
    if (!waiting || drawn !== latest.current || turned === waiting.turn) return;
    pending.current = null;
    if ("arrival" in waiting) arrive(waiting.arrival);
    else wholly(waiting.caret, waiting.all, waiting.lively);
  }, [turned, drawn]);

  // One editor over every word, laid where the blocks were, which are only hidden meanwhile.
  const wholly = (asked?: number, all = false, lively = false) => {
    if (reading || fixed || !(said.current.wholeable || said.current.inline)) return;
    const caret = asked ?? (open.current ? caretNow.current?.() : undefined) ?? 0;
    if (open.current) left();
    const sheet = shown.current?.document.querySelector<HTMLElement>(".page");
    // A block just left draws the page again, and an editor laid in the old one would go with it.
    if (!sheet || showing.current !== latest.current) {
      pending.current = { caret, all, lively, turn: turnedNow.current };
      return;
    }
    const fresh = sheet.ownerDocument.createElement("div");
    sheet.prepend(fresh);
    sheet.classList.add("whole");
    veiled.current = () => {
      sheet.classList.remove("whole");
      fresh.remove();
    };
    block([0, latest.current.length], fresh, "", "", undefined, { caret, all, lively });
    if (open.current) return;
    veiled.current();
    veiled.current = null;
  };

  const arrive = (one: Arrival) => {
    const text = latest.current;
    const bytes = new TextEncoder().encode(text.slice(0, one.at)).length;
    const blocks = Array.from(
      shown.current?.document.querySelectorAll<HTMLElement>(WRITTEN) ?? [],
    ).flatMap((element) => {
      const span = spanned(element.dataset.at ?? null);
      return span ? [{ element, span }] : [];
    });
    const target = one.join
      ? blocks.find(({ span }) => span[0] <= bytes && bytes <= span[1])
      : one.forward
        ? blocks.find(({ span }) => span[0] >= bytes)
        : blocks.filter(({ span }) => span[1] <= bytes).pop();
    if (!target) return;
    const at: [number, number] = [charsAt(text, target.span[0]), charsAt(text, target.span[1])];
    block(at, target.element, "", "", undefined, undefined, one);
  };

  const formatted = (
    editing: Editing["view"],
    read: string,
    { said, open: mark, close }: Pick<Formatting, "said" | "open" | "close">,
  ) => {
    const source = editing.state.sliceDoc();
    const end = caretAt(source, read + said);
    // Words drawn across marks are not in the Markdown as read, so the block stays open for a hand.
    if (source.slice(end - said.length, end) !== said) return;
    editing.dispatch({ selection: { anchor: end - said.length, head: end } });
    wrapped(editing, mark, close);
    if (!close.startsWith("](")) open.current?.leave();
  };

  const format = (asked: Formatting) => {
    if (!open.current) block(asked.at, asked.block, "", asked.read, asked);
  };

  // The blocks the `/` menu offers, put where the caret stands on the page; a slash left unanswered is written.
  const blockMenu = (at: { x: number; y: number }, slash: boolean) => {
    const writer = pageWriter.current?.hands;
    if (!writer) return;
    const kinds = said.current.asking?.kinds ?? [];
    const steps = stagesIn(latest.current).length > 0;
    let picked = false;
    const back = () => shown.current?.focus();
    const choices: Choice[] = SLASHED.filter(
      (one) =>
        !("wrap" in one.put) &&
        (!("ask" in one.put) || kinds.includes(one.put.ask)) &&
        (one.key !== "step" || steps),
    ).map((one) => ({
      key: one.key,
      label: t(one.label),
      glyph: one.icon,
      mark: one.hint,
      onPick: () => {
        picked = true;
        back();
        // A table is written from its cells, so the page is drawn whole and its first head opens.
        const table = one.key === "table";
        const laid = writer.slashed(one, table);
        if (table && laid)
          tables.freshAt(bytesAt(latest.current, laid.from + laid.insert.indexOf("|")));
        if (laid && DRAWN_OVER.has(one.key))
          markdownOn(fenceInside(latest.current, laid.from + laid.insert.indexOf("```")));
      },
    }));
    let closed = false;
    // The menu closes before it hands over what was picked, so the slash waits for that to land.
    const unpicked = () =>
      setTimeout(() => {
        if (picked || closed) return;
        closed = true;
        back();
        if (slash) writer.typed("/");
      });
    setMenu({
      ...at,
      label: t("blocksOf"),
      choices,
      find: t("blockFind"),
      foot: t("slashKeys"),
      unpicked,
    });
  };

  // What the grip beside a block offers: moving it, a copy under it, or taking it away.
  const blockActs = (at: { x: number; y: number }, top: Element) => {
    const writer = pageWriter.current?.hands;
    if (!writer) return;
    const act = (kind: Act) => () => {
      shown.current?.focus();
      writer.unitActed(top, kind);
    };
    setMenu({
      ...at,
      label: t("blocksOf"),
      choices: [
        { key: "up", label: t("blockUp"), keys: `${ALT} ↑`, onPick: act("up") },
        { key: "down", label: t("blockDown"), keys: `${ALT} ↓`, onPick: act("down") },
        { key: "double", label: t("blockDouble"), onPick: act("double") },
        { key: "drop", label: t("blockDrop"), danger: true, apart: true, onPick: act("drop") },
      ],
    });
  };

  const pageFormat = (asked: Formatting) =>
    pageWriter.current?.hands.format(asked.open, asked.close);

  // On the page written in place, cutting and copying carry the Tisty Markdown of the words chosen.
  const pageMenu = (at: { x: number; y: number }, asked: Menued) => {
    const writer = pageWriter.current?.hands;
    const words = writer?.chosen() ?? null;
    const back = () => shown.current?.focus();
    const choices: Choice[] = [...pictures.offered(shown.current?.document, asked.picture)];
    if (writer && words) {
      choices.push(
        {
          key: "cut",
          label: t("cutIt"),
          apart: choices.length > 0,
          onPick: () =>
            void copied(words).then(() => {
              writer.cut();
              back();
            }),
        },
        { key: "copy", label: t("copyIt"), onPick: () => void copied(words).then(back) },
        ...(writer.formattable() ? FORMATS : []).map(([word, , mark, close], index) => ({
          key: `format-${word}`,
          label: t(word),
          apart: index === 0,
          onPick: () => {
            back();
            writer.format(mark, close);
          },
        })),
      );
    }
    if (choices.length > 0) setMenu({ ...at, choices });
  };

  const chose = ({ at, block: element, read, said: words, typed }: Chosen) => {
    if (open.current) return;
    block(at, element, "", read);
    const editing = (open.current as Editing | null)?.view;
    if (!editing) return;
    const text = editing.state.sliceDoc();
    const end = caretAt(text, read + words);
    // Words drawn across marks are not in the Markdown as read: the key still lands, at the caret.
    if (text.slice(end - words.length, end) === words)
      editing.dispatch({
        selection: { anchor: inEditor(text, end - words.length), head: inEditor(text, end) },
      });
    if (typed !== null)
      editing.dispatch(editing.state.replaceSelection(typed), { userEvent: "input.type" });
  };

  const writingChoices = (editing: Editing["view"]): Choice[] => {
    if (livelyOpen.current) return livelyChoices(editing);
    const { from, to } = editing.state.selection.main;
    const chosen = editing.state.sliceDoc(from, to);
    const back = () => editing.focus();
    return [
      {
        key: "cut",
        label: t("cutIt"),
        off: from === to,
        onPick: () => {
          const was = editing.state.doc;
          void copied(chosen).then(() => {
            if (editing.state.doc === was) {
              editing.dispatch({ changes: { from, to }, userEvent: "delete.cut" });
            }
            back();
          });
        },
      },
      {
        key: "copy",
        label: t("copyIt"),
        off: from === to,
        onPick: () => void copied(chosen).then(back),
      },
      ...FORMATS.map(([said, , mark, close], at) => ({
        key: `format-${said}`,
        label: t(said),
        apart: at === 0,
        onPick: () => {
          wrapped(editing, mark, close);
          back();
        },
      })),
      {
        key: "marks",
        label: t("marksShown"),
        apart: true,
        onPick: () => {
          flipped(editing);
          back();
        },
      },
    ];
  };

  const menued = (asked: Menued) => {
    const frame = shown.current?.frameElement?.getBoundingClientRect();
    const at = { x: (frame?.left ?? 0) + asked.x, y: (frame?.top ?? 0) + asked.y };
    if (onPage) return pageMenu(at, asked);
    const editing = open.current?.view;
    if (asked.writing) return editing && setMenu({ ...at, choices: writingChoices(editing) });
    const { picked, block: under, picture } = asked;
    const choices: Choice[] = [];
    choices.push(...pictures.offered(shown.current?.document, picture));
    if (picked)
      choices.push({
        key: "copy",
        label: t("copyIt"),
        apart: choices.length > 0,
        onPick: () => void copied(picked.said),
      });
    const free = !reading && !editing;
    if (free && picked) {
      const before = choices.length;
      for (const [said, , mark, close] of FORMATS) {
        choices.push({
          key: `format-${said}`,
          label: t(said),
          apart: choices.length === before && before > 0,
          onPick: () => format({ ...picked, open: mark, close }),
        });
      }
    }
    if (free && under) {
      choices.push({
        key: "edit",
        label: t("editBlock"),
        apart: true,
        onPick: () => block(under.at, under.element),
      });
    }
    if (free && !fixed && said.current.wholeable) {
      choices.push({
        key: "whole",
        label: t("editWholeDoc"),
        hint: pressed("E"),
        apart: !under,
        onPick: () => wholly(under ? under.at[0] : 0),
      });
    }
    if (choices.length > 0) setMenu({ ...at, choices });
  };

  const dropped = (text: string, at?: { left: number; top: number }) => {
    if (said.current.inline && said.current.shaping) return;
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
    whole(next);
  };
  const dropping = useRef(dropped);
  dropping.current = dropped;

  useEffect(() => {
    const box = room.current;
    if (!box || reading) return;
    return takesFiles(box, (text, at) => dropping.current(text, at));
  }, [reading]);

  // A block the translator would say otherwise is mended only on the page it was found on.
  const mended = (one: Mend, body: string) => {
    if (open.current) return left();
    if (reading || latest.current !== body || showing.current !== body) return;
    const next = mendedText(body, [one]);
    whole(next);
  };

  const ticked = ([from, end]: [number, number]) => {
    // Like a click that opens a block, a box names bytes of the page on screen, which a block being written has moved.
    if (open.current) return left();
    if (reading || showing.current !== latest.current) return;
    const now = latest.current;
    const said = now.slice(from, end);
    const bracket = said.indexOf("[");
    if (bracket < 0 || !/^[\s>]*(?:[-*+]|\d+[.)])\s+$/.test(said.slice(0, bracket))) return;
    if (!/^\[[ xX]\]/.test(said.slice(bracket))) return;
    const at = from + bracket;
    const next = `${now.slice(0, at)}${now.slice(at, at + 3) === "[ ]" ? "[x]" : "[ ]"}${now.slice(at + 3)}`;
    whole(next);
  };

  const restuck = (note: [number, number], to: number) => {
    if (open.current) return left();
    if (reading || showing.current !== latest.current) return;
    const now = latest.current;
    const next = movedBlock(now, note, to);
    if (next !== now) whole(next, bytesAt(now, note[0]));
  };

  const parted = (at: [number, number]) => {
    if (open.current) return left();
    if (reading || hanging.current || showing.current !== latest.current) return;
    const asked = said.current.onPart;
    const line = stepLine(latest.current, at);
    const title = line && steppedTitle(line);
    if (!asked || !line || !title) return;
    hanging.current = true;
    asked(title)
      .then((id) => {
        // The part is there either way; the step makes room for it only where it still stands untouched.
        const now = latest.current;
        const after = now.slice(at[0] + line.length);
        const still = now.slice(at[0], at[0] + line.length) === line && /^(\r?\n|$)/.test(after);
        if (!id || !still || open.current || showing.current !== now) return;
        whole(cardedAt(now, at[0], at[0] + line.length, partCard(id, title)));
      })
      .catch((e) => said.current.onFail?.(e))
      .finally(() => {
        hanging.current = false;
      });
  };

  const cut = ([from, end]: [number, number]) => {
    if (open.current) return left();
    if (reading || showing.current !== latest.current) return;
    const now = latest.current;
    // Only a label goes: bytes from a page drawn before the last cut would name something else.
    if (!/^#{1,6}[ \t]/.test(now.slice(from, end))) return;
    const line = now.includes("\r\n") ? "\r\n" : "\n";
    const before = now.slice(0, from).replace(/(\r?\n)+$/, "");
    const after = now.slice(end).replace(/^(\r?\n)+/, "");
    whole(`${before}${before && after ? line + line : before ? line : ""}${after}`);
  };

  // On the page written in place a change from beside it is laid block by block, so nothing above it moves.
  const rewritten = (next: string) => {
    const writer = pageWriter.current?.hands;
    if (onPage && writer) writer.rewrite(next);
    else whole(next);
  };

  const tables = useTables({
    latest,
    drawn: showing,
    page: () => shown.current?.document ?? null,
    frame: () => shown.current?.frameElement?.getBoundingClientRect(),
    whole: rewritten,
    writing: () => Boolean(open.current),
    leave: left,
    markdown: (at, box) =>
      block([charsAt(latest.current, at[0]), charsAt(latest.current, at[1])], box),
    fail: (problem) => said.current.onFail?.(problem),
    turned,
  });

  const widen = (at: [number, number], widths: Widths) => {
    if (open.current) return left();
    if (reading || showing.current !== latest.current) return;
    const next = widened(latest.current, at, widths);
    if (next === null) return;
    rewritten(next);
  };

  const asked = (one: Extract<Went, { kind: "more" }>) => {
    if (open.current) return left();
    if (reading || showing.current !== latest.current) return;
    askedOn.current = latest.current;
    const frame = shown.current?.frameElement?.getBoundingClientRect();
    setMore({ ...one, x: (frame?.left ?? 0) + one.x, y: (frame?.top ?? 0) + one.y });
  };

  const reshape = (one: Extract<Went, { kind: "more" }>, how: Reshape) => {
    // The card's place was read from the page as it stood when the menu opened.
    if (askedOn.current !== latest.current) return;
    const next = reshaped(latest.current, one.at, one.target, how);
    if (next === null) return;
    whole(next);
  };

  const keepCopy = (reference: string) =>
    intoFile({ defaultPath: named(reference) })
      .then((where) => (where ? copyAttached(reference, where) : undefined))
      .catch((error_) => said.current.onFail?.(error_));

  const openKept = (reference: string) =>
    opened(reference).catch((error_) => said.current.onFail?.(error_));
  const pageHands: Hands = {
    text: () => latest.current,
    write: written,
    shown: (now) => {
      alreadyShown.current = now;
      setDrawn(now);
      setTurned((many) => many + 1);
    },
    redraw: (now) => {
      alreadyShown.current = null;
      setDrawn(now);
    },
    near: docPlacedNear,
    dressed: (_sheet, laid) => dresser.current?.(laid),
    ask: (kind, at, put, first) => said.current.asking?.ask(kind, at, put, first),
    blocks: (at, slash) => blockMenu(at, slash),
    acts: blockActs,
    translated: (html, plain) =>
      html ? HOST.fromHtml(html) : HOST.fromText(plain).then((said) => said ?? plain),
  };
  // The page is drawn anew after every key, so its listeners stay and only the caret is laid again.
  const pageWritten = (page: Document | null) => {
    if (pageWriter.current && pageWriter.current.page !== page) {
      pageWriter.current.hands.stop();
      pageWriter.current = null;
    }
    if (!page || !onPage) return;
    pageWriter.current ??= { page, hands: writtenOn(page, pageHands) };
    pageWriter.current.hands.drawn();
  };

  const pictures = usePictures({
    save: (kept) => void keepCopy(kept),
    open: (kept) => void openKept(kept),
  });

  const offered = (one: Extract<Went, { kind: "more" }>): Choice[] => {
    const file = one.target.startsWith(DOC) ? one.target.slice(DOC.length) : null;
    const plain = [
      { key: "link", label: t("cardAsLink"), icon: "↩", onPick: () => reshape(one, "link") },
      {
        key: "remove",
        label: t("cardRemove"),
        icon: "✕",
        danger: true,
        apart: true,
        onPick: () => reshape(one, "remove"),
      },
    ];
    if (file) {
      const kind = said.current.card?.(file).kind;
      return [
        {
          key: "own",
          label: t("cardOwn"),
          icon: "⇤",
          off: kind !== "page" || !said.current.onOwn,
          onPick: () => said.current.onOwn?.(file),
        },
        ...plain,
      ];
    }
    return [
      { key: "keep", label: t("cardKeepCopy"), icon: "↧", onPick: () => void keepCopy(one.target) },
      ...plain,
    ];
  };

  useEffect(() => {
    said.current.onInsert?.(
      reading
        ? null
        : (file, title) => {
            left();
            const put = putCard(latest.current, file, title);
            if (put === null) return false;
            whole(put);
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
              whole(done.body);
            }
            return done.moved;
          },
    );
    return () => {
      said.current.onInsert?.(null);
      said.current.onOrder?.(null);
    };
  }, [reading, whole, left]);

  // The sheet takes the keys without a caret, so nothing pulls the view to where one would be.
  useEffect(() => {
    // A task's page sits under its title and fields, which keep the keys the person is typing in.
    if (!said.current.inline) room.current?.focus({ preventScroll: true });
  }, []);

  // A block opened in the other mode keeps that mode's keys, so turning the mode closes it.
  useEffect(() => {
    if (props.shaping !== undefined) left();
  }, [props.shaping, left]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: `scroller` reads only refs and the props of this render, so readiness alone decides.
  useEffect(() => {
    const want = wanted.current;
    const mark = marked.current;
    const rolled = scroller();
    if (!ready || !rolled) return;
    const inside = rolled !== room.current;
    if (!want && !mark) {
      if (putting.current < 0) return;
      rolled.scrollTop = 0;
      putting.current = -1;
      return;
    }
    const spot = () => {
      const sheet = scroller();
      if (!sheet) return null;
      const page = shown.current?.document;
      const found = mark ? page?.querySelector<HTMLElement>(`[data-doc="${mark}"]`) : null;
      const box = found?.getBoundingClientRect();
      if (!box || (!box.height && !box.top)) return want ?? null;
      const frame = shown.current?.frameElement as HTMLElement | null | undefined;
      const above = inside ? box.top : box.top + (frame?.getBoundingClientRect().top ?? 0);
      const top = inside ? 0 : sheet.getBoundingClientRect().top;
      return Math.max(0, Math.round(above - top + sheet.scrollTop - ASIDE_MARGIN));
    };
    putting.current = 0;
    return settles({
      at: scroller,
      want: spot,
      done: () => {
        putting.current = -1;
        const sheet = scroller();
        if (!sheet || sheet.scrollHeight <= sheet.clientHeight) return;
        said.current.onSeen?.(sheet.scrollTop);
      },
      put: (down) => {
        putting.current = down;
      },
      watch: () =>
        (inside
          ? shown.current?.document.querySelector<HTMLElement>(".page")
          : room.current?.querySelector<HTMLElement>("iframe, .page")) ?? null,
    });
  }, [ready]);

  useFrameScroll({
    lively,
    frame: framed,
    putting,
    onStuck: setStuck,
    onSeen: (at) => said.current.onSeen?.(at),
    ready,
    turned,
  });

  const { lately, looking } = props;
  // `turned` is each new drawing of the page, which comes without the marks.
  useEffect(() => {
    const page = shown.current?.document;
    // A page drawn before the open block followed new words counts bytes the marks do not.
    if (!page || drawn !== latest.current) return;
    for (const one of page.querySelectorAll<HTMLElement>(".page > [data-at]")) {
      const at = spanned(one.dataset.at ?? null);
      one.classList.toggle("lately", Boolean(lately && at && touches(at, lately)));
    }
  }, [lately, turned, drawn]);

  // A page drawn anew lays its sheets and its height again, so the block it was kept by goes back where it was.
  useEffect(() => {
    const wanted = landing.current;
    if (!wanted || turned === wanted.turn) return;
    landing.current = null;
    // Words that moved on before this drawing count bytes this page does not hold.
    if (drawn !== latest.current) return;
    const sheet = room.current;
    const held = shown.current?.frameElement as HTMLElement | null | undefined;
    if (!sheet || !held) return;
    holdingNow.current();
    holdingNow.current = settles({
      at: () => room.current,
      want: () => {
        const block = shown.current?.document.querySelector<HTMLElement>(
          `.page > [data-at^="${wanted.bytes}-"]`,
        );
        if (!block) return null;
        const top = block.getBoundingClientRect().top + held.getBoundingClientRect().top;
        return sheet.scrollTop + top - wanted.y;
      },
      done: () => {},
      put: () => {},
      keeps: { waits: HELD_FOR, inside: () => shown.current?.document ?? null },
    });
  }, [turned, drawn]);

  useEffect(() => () => holdingNow.current(), []);

  // biome-ignore lint/correctness/useExhaustiveDependencies: `markIn` reads only refs, so the count alone decides.
  useEffect(() => {
    if (!looking) return;
    const writing = open.current?.view;
    const asked = said.current.lately;
    if (veiled.current && writing && asked) {
      // The page's mark counts bytes of the words as they are now; the editor counts its own places.
      const text = latest.current;
      const place = (bytes: number) =>
        Math.min(inEditor(text, charsAt(text, bytes)), writing.state.doc.length);
      markIn(writing, { from: place(asked[0]), to: place(asked[1]) });
    }
    const sheet = room.current;
    const found = shown.current?.document.querySelector<HTMLElement>(".page > .lately, .cm-lately");
    const frame = shown.current?.frameElement?.getBoundingClientRect();
    if (!sheet || !found || !frame) return;
    const above = found.getBoundingClientRect().top + frame.top;
    sheet.scrollTo({
      top: Math.max(0, above - sheet.getBoundingClientRect().top + sheet.scrollTop - ASIDE_MARGIN),
      behavior: "smooth",
    });
  }, [looking]);

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
      className={`scroller gutter flex min-h-0 flex-1 flex-col outline-none${above || below ? " leafed" : ""}${stuck ? " stuck" : ""}${props.sheet ? " on-sheets" : ""}${lively && !props.sheet ? " lively-room" : ""}`}
      style={{
        ...(props.sheet && ({ "--sheet-wide": `${props.sheet.wide}px` } as CSSProperties)),
        ...(lively && framed() ? { overflow: "hidden" } : {}),
      }}
    >
      {above}
      <div
        ref={kept.ref}
        style={{
          ...(props.sheet && { maxWidth: props.sheet.wide }),
          // Until it is drawn the page holds the room it last took, so nothing under it jumps.
          ...(!ready && { minHeight: kept.height ?? 240 }),
        }}
        className={`tisty-sheet relative mx-auto overflow-hidden rounded-[10px] bg-sheet ${lively ? "flex min-h-0 flex-1 flex-col" : "shrink-0"} ${widthOf}`}
      >
        {!ready && <Skeleton inset={props.inline ? "46px 32px" : undefined} />}
        {writable && !fixed && ready && !props.view?.beside && (
          <ViewSwitch markdown={markdown} onTurn={setMarkdown} />
        )}
        <Page
          bar={BAR}
          sheet={props.sheet}
          body={drawn}
          label={label}
          tall={!lively}
          untitled={props.inline}
          shaping={props.shaping}
          onDoc={(file) => said.current.onDoc?.(file)}
          onOpen={(reference) => said.current.onOpen?.(reference)}
          onTag={(tag) => said.current.onTag?.(tag)}
          draw={writable ? docPlaced : undefined}
          already={alreadyShown.current}
          onDresser={(dress) => {
            dresser.current = dress;
          }}
          onBlock={reading || onPage ? undefined : block}
          onBegin={
            onPage ? (at, seed, near) => pageWriter.current?.hands.begin(at, seed, near) : undefined
          }
          onSource={onPage ? (at) => markdownOn(fenceInside(latest.current, at)) : undefined}
          onHintWords={
            onPage && fixed
              ? (at, hint) => pageWriter.current?.hands.hintWords(at, hint)
              : undefined
          }
          stepDates={props.stepDates}
          onTick={reading || fixed ? undefined : ticked}
          onStick={reading || fixed ? undefined : restuck}
          onPart={reading || fixed || !props.onPart ? undefined : parted}
          onCut={reading || !fixed ? undefined : cut}
          onWiden={reading || fixed ? undefined : widen}
          onTable={reading || fixed ? undefined : tables.onTable}
          onFormat={reading || fixed ? undefined : onPage ? pageFormat : format}
          onChoose={reading || fixed || onPage ? undefined : chose}
          onMend={reading || fixed ? undefined : mended}
          onMenu={fixed ? undefined : menued}
          onMore={reading || fixed ? undefined : asked}
          onPicture={pictures.view}
          onAway={left}
          card={props.card}
          part={props.part}
          onTask={(id) => said.current.onTask?.(id)}
          blurb={props.blurb}
          again={props.again}
          cards={props.cards}
          onDiagram={reading || fixed || !props.wholeable ? undefined : diagram}
          onDiagramExport={fixed ? undefined : diagramOut}
          onApart={(source) => {
            const named = said.current.file;
            // A task page not written yet has no file the widget could lead back to.
            const from = named?.startsWith("unwritten:") ? undefined : named;
            viewWidget(
              source,
              from ? { id: "", file: from, title: said.current.label, place: "" } : undefined,
            ).catch(() => {});
          }}
          onOutline={(heads) => said.current.onOutline?.(heads)}
          onShown={(page, body) => {
            shown.current = page;
            shownBody.current = body ?? null;
            pageWritten(page?.document ?? null);
            said.current.onPage?.(page);
            if (page) setReady(true);
            setTurned((many) => many + 1);
          }}
        />
      </div>
      {below && lively && framed() ? (
        <LivelyBelow folds={Boolean(props.belowFolds)} width={widthOf}>
          {below}
        </LivelyBelow>
      ) : (
        below
      )}
      {!below && !props.inline && !lively && (
        <div aria-hidden="true" className="h-[40vh] shrink-0" />
      )}
      {menu && (
        <Menu
          at={{ x: menu.x, y: menu.y }}
          label={menu.label ?? t("formatting")}
          choices={menu.choices}
          find={menu.find}
          foot={menu.foot}
          keeps
          onClose={() => {
            menu.unpicked?.();
            setMenu(null);
          }}
        />
      )}
      {pictures.viewer}
      {tables.menu}
      {more && (
        <Menu
          at={{ x: more.x, y: more.y }}
          label={t("cardMore")}
          choices={offered(more)}
          onClose={() => setMore(null)}
        />
      )}
    </div>
  );
}
