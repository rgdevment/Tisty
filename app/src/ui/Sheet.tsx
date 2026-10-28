import { Transaction } from "@codemirror/state";
import { save as intoFile } from "@tauri-apps/plugin-dialog";
import {
  type CSSProperties,
  type ReactNode,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { copied, copyAttached, type Mend, opened, viewWidget } from "../core";
import { viewDiagram } from "../drawing";
import { CATCHES, takesFiles } from "../dropped";
import { t } from "../locales";
import { DOC, partCard } from "../markdown";
import type { Moved } from "../paging";
import { named } from "../previews";
import { inEditor, reanchored, spanOf, touches } from "./anchoring";
import { FORMATS, flipped } from "./barring";
import { caretAt, type Editing, edited } from "./blocking";
import { cardedAt, movedCard, putCard } from "./carded";
import type { Carded } from "./carding";
import { exportChoices, fileName } from "./drawing/exporting";
import { wrapped } from "./formatting";
import { useKeptHeight } from "./heights";
import { MARKED_FOR, markLately } from "./lately";
import Menu, { type Choice } from "./Menu";
import { mended as mendedText } from "./mending";
import Page from "./Page";
import { charsAt, type Head, spanned, type Went } from "./paging";
import type { PartCarded } from "./partcarding";
import { closed, laid, parting } from "./parting";
import type { Chosen, Formatting, Menued } from "./quickly";
import { filedAs, type Reshape, reshaped } from "./recarding";
import Skeleton from "./Skeleton";
import { ASIDE_MARGIN, settles } from "./seeking";
import type { Sheet as SheetSize } from "./sheeting";
import { type Asking, pressed, putBlock, stagesIn } from "./slashing";
import { stepLine, steppedTitle } from "./staging";
import { useTables } from "./tableSheet";
import { widened } from "./tabling";
import { usePictures } from "./usePictures";
import { darkNow } from "./viewer/acting";

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

export default function Sheet(props: Readonly<Props>) {
  const { file, value, label, reading = false, above, below } = props;
  const fixed = Boolean(props.inline && props.shaping);
  const room = useRef<HTMLDivElement>(null);
  const said = useRef(props);
  said.current = props;
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
  // Follows the open block through words written around it from outside; false when they reach it.
  const follows = useRef<((was: string, now: string) => boolean) | null>(null);
  // The blocks the whole-document editor covers, shown again as it is left.
  const veiled = useRef<(() => void) | null>(null);
  // Asked while the page was still being drawn again, so it opens once the new one is shown.
  const pending = useRef<{ caret: number; all: boolean; turn: number } | null>(null);
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
  const latest = useRef(value);
  const echoes = useRef<string[]>([]);
  const shown = useRef<Window | null>(null);
  const putting = useRef(-1);
  const hanging = useRef(false);
  const [ready, setReady] = useState(false);
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
  } | null>(null);
  const askedOn = useRef("");
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

  const whole = useCallback(
    (next: string) => {
      const before = latest.current;
      written(next);
      setDrawn(next);
      settled(before, next);
    },
    [written, settled],
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
    whole?: { caret: number; all?: boolean },
  ) => {
    if (open.current) return left();
    // A click on the page still on screen while the next one is drawn names bytes of an older text.
    if (reading || showing.current !== latest.current) return;
    let [from, end] = at;
    const piece = latest.current.slice(from, end);
    const tail = /\r?\n$/.exec(piece)?.[0] ?? "";
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
        follows.current = null;
        veiled.current?.();
        veiled.current = null;
        // Only a block left by the person closes its gap: text from outside or another file is not its own.
        const ours = home === onFile.current && latest.current === mine;
        if (ours && !quiet.current && !fresh && !latest.current.slice(from, end).trim()) {
          written(closed(latest.current, from, end));
        }
        setDrawn(latest.current);
        if (ours) settled(before, latest.current, renamed);
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
          ask: (kind, at, put, first) => asking?.ask(kind, at, put, first && before),
          steps: () => stagesIn(latest.current),
        };
      },
      Boolean(said.current.inline && said.current.shaping),
      Boolean(whole),
      fixed || !said.current.wholeable
        ? undefined
        : (all) => queueMicrotask(() => (whole && !all ? left() : wholly(undefined, all))),
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
    open.current.kept(said.current.kept ?? true);
    const editing = open.current.view;
    const caret = whole
      ? inEditor(latest.current, whole.caret)
      : seed
        ? seed.length
        : inEditor(editing.state.sliceDoc(), caretAt(editing.state.sliceDoc(), read));
    const length = editing.state.doc.length;
    editing.dispatch({
      selection: whole?.all ? { anchor: 0, head: length } : { anchor: Math.min(caret, length) },
      scrollIntoView: true,
    });
    if (asked) formatted(editing, read, asked);
  };

  const { wholeAsked, allAsked } = props;
  // biome-ignore lint/correctness/useExhaustiveDependencies: `wholly` and `left` read only refs, so the count alone decides.
  useEffect(() => {
    if (!wholeAsked) return;
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
    wholly(waiting.caret, waiting.all);
  }, [turned, drawn]);

  // One editor over every word, laid where the blocks were, which are only hidden meanwhile.
  const wholly = (asked?: number, all = false) => {
    if (reading || fixed || !said.current.wholeable) return;
    const caret = asked ?? (open.current ? caretNow.current?.() : undefined) ?? 0;
    if (open.current) left();
    const sheet = shown.current?.document.querySelector<HTMLElement>(".page");
    // A block just left draws the page again, and an editor laid in the old one would go with it.
    if (!sheet || showing.current !== latest.current) {
      pending.current = { caret, all, turn: turnedNow.current };
      return;
    }
    const fresh = sheet.ownerDocument.createElement("div");
    sheet.prepend(fresh);
    sheet.classList.add("whole");
    veiled.current = () => {
      sheet.classList.remove("whole");
      fresh.remove();
    };
    block([0, latest.current.length], fresh, "", "", undefined, { caret, all });
    if (open.current) return;
    veiled.current();
    veiled.current = null;
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

  const tables = useTables({
    latest,
    drawn: showing,
    page: () => shown.current?.document ?? null,
    frame: () => shown.current?.frameElement?.getBoundingClientRect(),
    whole,
    writing: () => Boolean(open.current),
    leave: left,
    markdown: (at, box) =>
      block([charsAt(latest.current, at[0]), charsAt(latest.current, at[1])], box),
    fail: (problem) => said.current.onFail?.(problem),
    turned,
  });

  const widen = (at: [number, number], column: number, dashes: number) => {
    if (open.current) return left();
    if (reading || showing.current !== latest.current) return;
    const next = widened(latest.current, at, column, dashes);
    if (next === null) return;
    whole(next);
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
    const filed = filedAs(latest.current, one.at, one.target);
    return [
      { key: "keep", label: t("cardKeepCopy"), icon: "↧", onPick: () => void keepCopy(one.target) },
      {
        key: "live",
        label: filed ? t("cardAsWidget") : t("cardAsFile"),
        icon: "▣",
        off: !/\.html?$/i.test(one.target),
        onPick: () => reshape(one, filed ? "live" : "file"),
      },
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
      className={`scroller gutter flex min-h-0 flex-1 flex-col outline-none${above || below ? " leafed" : ""}${stuck ? " stuck" : ""}${props.sheet ? " on-sheets" : ""}`}
      style={props.sheet && ({ "--sheet-wide": `${props.sheet.wide}px` } as CSSProperties)}
    >
      {above}
      <div
        ref={kept.ref}
        style={{
          ...(props.sheet && { maxWidth: props.sheet.wide }),
          // Until it is drawn the page holds the room it last took, so nothing under it jumps.
          ...(!ready && { minHeight: kept.height ?? 240 }),
        }}
        className="tisty-sheet relative mx-auto w-full shrink-0 overflow-hidden rounded-[10px] bg-sheet"
      >
        {!ready && <Skeleton inset={props.inline ? "46px 32px" : undefined} />}
        <Page
          sheet={props.sheet}
          body={drawn}
          label={label}
          stranger={false}
          tall
          untitled={props.inline}
          shaping={props.shaping}
          onDoc={(file) => said.current.onDoc?.(file)}
          onOpen={(reference) => said.current.onOpen?.(reference)}
          onTag={(tag) => said.current.onTag?.(tag)}
          onBlock={reading ? undefined : block}
          stepDates={props.stepDates}
          onTick={reading || fixed ? undefined : ticked}
          onPart={reading || fixed || !props.onPart ? undefined : parted}
          onCut={reading || !fixed ? undefined : cut}
          onWiden={reading || fixed ? undefined : widen}
          onTable={reading || fixed ? undefined : tables.onTable}
          onFormat={reading || fixed ? undefined : format}
          onChoose={reading || fixed ? undefined : chose}
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
          onApart={(lent) => {
            const named = said.current.file;
            // A task page not written yet has no file the widget could lead back to.
            const from = named?.startsWith("unwritten:") ? undefined : named;
            viewWidget(
              lent,
              from ? { id: "", file: from, title: said.current.label, place: "" } : undefined,
            ).catch(() => {});
          }}
          onOutline={(heads) => said.current.onOutline?.(heads)}
          onShown={(page) => {
            shown.current = page;
            said.current.onPage?.(page);
            if (page) setReady(true);
            setTurned((many) => many + 1);
          }}
        />
      </div>
      {below}
      {!below && !props.inline && <div aria-hidden="true" className="h-[40vh] shrink-0" />}
      {menu && (
        <Menu
          at={{ x: menu.x, y: menu.y }}
          label={menu.label ?? t("formatting")}
          choices={menu.choices}
          keeps
          onClose={() => setMenu(null)}
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
