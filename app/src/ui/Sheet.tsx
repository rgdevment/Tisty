import { save as intoFile } from "@tauri-apps/plugin-dialog";
import { type ReactNode, useCallback, useEffect, useRef, useState } from "react";
import { copied, copyAttached, type Mend, viewWidget } from "../core";
import { CATCHES, takesFiles } from "../dropped";
import { t } from "../locales";
import { DOC } from "../markdown";
import type { Moved } from "../paging";
import { named } from "../previews";
import { FORMATS, flipped } from "./barring";
import { caretAt, type Editing, edited } from "./blocking";
import { movedCard, putCard } from "./carded";
import type { Carded } from "./carding";
import { wrapped } from "./formatting";
import Menu, { type Choice } from "./Menu";
import { mended as mendedText } from "./mending";
import Page from "./Page";
import { charsAt, type Head, spanned, type Went } from "./paging";
import { closed, laid, parting } from "./parting";
import type { Formatting, Menued } from "./quickly";
import { filedAs, type Reshape, reshaped } from "./recarding";
import { ASIDE_MARGIN, settles } from "./seeking";
import type { Sheet as SheetSize } from "./sheeting";
import { type Asking, putBlock } from "./slashing";
import { widened } from "./tabling";

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
  readonly onOwn?: (file: string) => void;
  readonly onFail?: (problem: unknown) => void;
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
  readonly sheet?: SheetSize;
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
  const [more, setMore] = useState<Extract<Went, { kind: "more" }> | null>(null);
  const [menu, setMenu] = useState<{ x: number; y: number; choices: Choice[] } | null>(null);
  const askedOn = useRef("");
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

  const block = (
    at: [number, number],
    element: HTMLElement,
    seed = "",
    read = "",
    asked?: Pick<Formatting, "said" | "open" | "close">,
  ) => {
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
    if (asked) formatted(editing, read, asked);
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
    const { picked, block: under } = asked;
    const choices: Choice[] = [];
    if (picked)
      choices.push({ key: "copy", label: t("copyIt"), onPick: () => void copied(picked.said) });
    const free = !reading && !editing;
    if (free && picked) {
      for (const [said, , mark, close] of FORMATS) {
        choices.push({
          key: `format-${said}`,
          label: t(said),
          apart: choices.length === 1,
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
    if (choices.length > 0) setMenu({ ...at, choices });
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

  // A block the translator would say otherwise is mended only on the page it was found on.
  const mended = (one: Mend, body: string) => {
    if (open.current) return left();
    if (reading || latest.current !== body || showing.current !== body) return;
    const next = mendedText(body, [one]);
    written(next);
    setDrawn(next);
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
    written(next);
    setDrawn(next);
  };

  const widen = (at: [number, number], column: number, dashes: number) => {
    if (open.current) return left();
    if (reading || showing.current !== latest.current) return;
    const next = widened(latest.current, at, column, dashes);
    if (next === null) return;
    written(next);
    setDrawn(next);
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
    written(next);
    setDrawn(next);
  };

  const keepCopy = (reference: string) =>
    intoFile({ defaultPath: named(reference) })
      .then((where) => (where ? copyAttached(reference, where) : undefined))
      .catch((error_) => said.current.onFail?.(error_));

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
      <div
        style={props.sheet && { maxWidth: props.sheet.wide }}
        className="tisty-sheet mx-auto w-full shrink-0 overflow-hidden rounded-[10px] bg-sheet"
      >
        <Page
          sheet={props.sheet}
          body={drawn}
          label={label}
          stranger={false}
          tall
          onDoc={(file) => said.current.onDoc?.(file)}
          onOpen={(reference) => said.current.onOpen?.(reference)}
          onTag={(tag) => said.current.onTag?.(tag)}
          onBlock={reading ? undefined : block}
          onTick={reading ? undefined : ticked}
          onWiden={reading ? undefined : widen}
          onFormat={reading ? undefined : format}
          onMend={reading ? undefined : mended}
          onMenu={menued}
          onMore={reading ? undefined : asked}
          onAway={left}
          card={props.card}
          blurb={props.blurb}
          again={props.again}
          onApart={(lent) => {
            const from = said.current.file;
            viewWidget(
              lent,
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
      {menu && (
        <Menu
          at={{ x: menu.x, y: menu.y }}
          label={t("formatting")}
          choices={menu.choices}
          keeps
          onClose={() => setMenu(null)}
        />
      )}
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
