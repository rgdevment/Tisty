import { convertFileSrc } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Editor as Writing } from "@tiptap/core";
import type { Node as Written } from "@tiptap/pm/model";
import { type EditorState, NodeSelection } from "@tiptap/pm/state";
import { CellSelection } from "@tiptap/pm/tables";
import { EditorContent, useEditor } from "@tiptap/react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  docRead,
  type Filed,
  type Glimpse,
  glimpseFetch,
  glimpseKept,
  noteTrouble,
  served,
  weighs,
} from "../core";
import { CATCHES, takesFiles } from "../dropped";
import { t } from "../locales";
import { spawned } from "../making";
import { DOC, docOf } from "../markdown";
import { card, filed, paged, pagesOf } from "../paging";
import { named, pictured } from "../previews";
import Asking from "./Asking";
import Floats from "./Floats";
import Glyphs from "./Glyphs";
import Menu from "./Menu";
import Papers from "./Papers";
import { previewing, type Reach } from "./previewing";
import Shot from "./Shot";
import Slash, { asked, type Block, narrowed } from "./Slash";
import Tabled from "./Tabled";
import { tagging } from "./tagging";
import { asMarkdown, type Head, headed, loosened, written } from "./writing";

export const stripped = (html: string): string =>
  html.replace(/<img\b[^>]*>/gi, (tag) =>
    /\bsrc\s*=\s*["'](?!https?:|attachments\/)/i.test(tag) ? "" : tag,
  );

export const clicking =
  (onOpen?: (reference: string) => void, onDoc?: (id: string) => void) =>
  (
    view: { state: { doc: Written } },
    pos: number,
    event: { metaKey?: boolean; ctrlKey?: boolean },
  ): boolean => {
    if (!event.metaKey && !event.ctrlKey) return false;
    const at = view.state.doc.resolve(pos);
    const picture = view.state.doc.nodeAt(pos);
    const src = picture?.type.name === "image" ? String(picture.attrs.src ?? "") : "";
    const link = at.marks().find((one) => one.type.name === "link")?.attrs.href;
    const target = src || String(link ?? "");
    if (!target) return false;
    const paper = docOf(target);
    if (paper) {
      onDoc?.(paper);
      return true;
    }
    if (/^(https?|mailto|tel):/i.test(target)) return false;
    onOpen?.(target);
    return true;
  };

const spanless = (line: string): string =>
  line.replace(/<span[^>]*data-ico="([^"]*)"[^>]*>[\s\S]*?<\/span>/g, ":$1:");

export const glimpsed = (body: string): string =>
  body
    .split("\n")
    .slice(1)
    .map((line) =>
      spanless(line)
        .replace(/^[#>\s*+-]+/, "")
        .trim(),
    )
    .filter(Boolean)
    .join(" · ")
    .slice(0, 160);

export const perched = (empty: boolean, code: boolean, hushed: boolean, whole: boolean): boolean =>
  !hushed && !empty && !code && !whole;

export const stale = (value: string, mine: string, shown: () => string | null): boolean =>
  value !== mine && shown() !== value;

const GLYPHS_TALL = 330;
const WAITS = 10_000;
const ASIDE_MARGIN = 80;

export interface Settling {
  at: () => HTMLElement | null;
  want: number | (() => number | null);
  done: () => void;
  put: (down: number) => void;
  watch?: () => HTMLElement | null;
}

export const settles = ({ at, want, done, put, watch }: Settling) => {
  let frame = 0;
  let mine = -1;
  let over = false;
  let seeing: ResizeObserver | null = null;
  let timer = 0;
  let writing = false;
  let tallness = -1;

  const sheet = at();

  function nudged() {
    if (writing) return;
    const room = at();
    if (!room || mine < 0) return;
    if (room.scrollHeight !== tallness) return;
    if (room.scrollTop !== mine) stop();
  }

  function stop() {
    if (over) return;
    over = true;
    seeing?.disconnect();
    sheet?.removeEventListener("scroll", nudged);
    window.clearTimeout(timer);
    cancelAnimationFrame(frame);
    done();
  }

  const reach = () => {
    const room = at();
    if (!room) return stop();
    const asked = typeof want === "function" ? want() : want;
    if (asked === null) return;
    writing = true;
    room.scrollTop = asked;
    mine = room.scrollTop;
    writing = false;
    put(mine);
    tallness = room.scrollHeight;
    if (mine >= asked) stop();
  };

  frame = requestAnimationFrame(reach);
  sheet?.addEventListener("scroll", nudged);
  timer = window.setTimeout(stop, WAITS);

  const grows = watch?.();
  if (grows && typeof ResizeObserver !== "undefined") {
    seeing = new ResizeObserver(() => {
      if (!over) reach();
    });
    seeing.observe(grows);
  }

  return stop;
};

const middle = (editor: Writing, from: number, to: number) => {
  const a = caret(editor, from);
  const b = caret(editor, to);
  return { x: (a.x + b.x) / 2, y: Math.min(a.y, b.y) };
};

const topOf = (editor: Writing, at: number) => {
  try {
    const held = editor.view.domAtPos(at).node as HTMLElement | null;
    const table = (held?.nodeType === 1 ? held : held?.parentElement)?.closest("table");
    if (!table) return caret(editor, at);
    const box = table.getBoundingClientRect();
    return { x: box.left + box.width / 2, y: box.top - 6 };
  } catch {
    return caret(editor, at);
  }
};

const caret = (editor: Writing, at: number) => {
  try {
    const spot = editor.view.coordsAtPos(at);
    return { x: spot.left, y: spot.bottom };
  } catch {
    return { x: 0, y: 0 };
  }
};

export const shotNode = (state: EditorState): string | null => {
  const held = state.selection;
  if (!(held instanceof NodeSelection) || held.node.type.name !== "image") return null;
  const src = String(held.node.attrs.src ?? "");
  return pictured(src) ? src : null;
};

export const shotAt = (
  editor: Writing,
): { at: { x: number; y: number }; src: string; name: string } | null => {
  const src = shotNode(editor.state);
  if (!src || /^(https?|data):/i.test(src)) return null;
  const held = editor.state.selection;
  const seen = editor.view.nodeDOM(held.from) as HTMLElement | null;
  const box = seen?.getBoundingClientRect?.();
  if (!box) return null;
  const called = held instanceof NodeSelection ? String(held.node.attrs.alt ?? "") : "";
  return { at: { x: box.left, y: Math.max(8, box.top - 40) }, src, name: called || named(src) };
};

const aimed = (editor: Writing) => {
  editor.commands.focus();
  return caret(editor, editor.state.selection.from);
};

interface Props {
  value: string;
  reading?: boolean;
  folder?: string | null;
  paper?: string;
  onMade?: (id: string, name: string) => void;
  label?: string;
  papers?: Filed[];
  onAttach?: () => Promise<string | null>;
  onOpen?: (reference: string) => void;
  onKeep?: (reference: string, name: string) => void;
  onDoc?: (id: string) => void;
  onTag?: (tag: string) => void;
  onOwn?: (id: string) => void;
  onWrite: (text: string) => void;
  onShaped?: (text: string) => void;
  onBlocks?: (blocks: Block[]) => void;
  onOutline?: (heads: Head[]) => void;
  onLaid?: (root: HTMLElement) => void;
  onReady?: (read: () => unknown) => void;
  onInsert?: (put: (file: string, title: string) => void) => void;
  seek?: number;
  anchor?: string;
  onSeen?: (at: number) => void;
  above?: React.ReactNode;
  below?: React.ReactNode;
}

export default function Editor({
  value,
  reading,
  folder,
  paper,
  onMade,
  label,
  papers,
  onAttach,
  onOpen,
  onKeep,
  onDoc,
  onTag,
  onOwn,
  onWrite,
  onShaped,
  onBlocks,
  onOutline,
  onLaid,
  onReady,
  onInsert,
  seek,
  anchor,
  onSeen,
  above,
  below,
}: Props) {
  const [asking, setAsking] = useState<{ at: { x: number; y: number }; word: string } | null>(null);
  const [active, setActive] = useState(0);
  const [picked, setPicked] = useState<{ at: { x: number; y: number } } | null>(null);
  const [tabled, setTabled] = useState<{ at: { x: number; y: number } } | null>(null);
  const [shot, setShot] = useState<{
    at: { x: number; y: number };
    src: string;
    name: string;
  } | null>(null);
  const [tying, setTying] = useState<{ x: number; y: number } | null>(null);
  const [choosing, setChoosing] = useState<{ x: number; y: number; leaf: boolean } | null>(null);
  const [swapping, setSwapping] = useState<{
    at: { x: number; y: number };
    untie?: () => void;
    drop: () => void;
    keep?: () => void;
    own?: () => void;
  } | null>(null);
  const [glyphing, setGlyphing] = useState<{ x: number; y: number } | null>(null);
  const [naming, setNaming] = useState<{ x: number; y: number; leaf: boolean } | null>(null);
  const mine = useRef(value);
  const sheet = useRef<HTMLDivElement | null>(null);
  const urls = useRef(new Map<string, string>());
  const weights = useRef(new Map<string, number>());
  const blurbs = useRef(new Map<string, string>());
  const glimpses = useRef(new Map<string, Glimpse | null>());
  const pending = useRef(new Set<string>());
  const missing = useRef(new Set<string>());
  const nudge = useRef(() => {});
  const waiting = useRef(false);
  const reach = useRef<Reach>({
    url: () => null,
    weight: () => null,
    title: () => null,
  });
  const now = useRef<{ open: boolean; count: number; take: () => void }>({
    open: false,
    count: 0,
    take: () => {},
  });

  const looked = useRef((_: Writing) => {});
  const hushed = useRef(false);
  const look = (editor: Writing) => {
    const { $from, $to, empty } = editor.state.selection;
    const code = editor.isActive("codeBlock");
    const whole =
      editor.state.selection instanceof NodeSelection && editor.state.selection.node.isAtom;
    setPicked(
      perched(empty, code, hushed.current, whole)
        ? { at: middle(editor, $from.pos, $to.pos) }
        : null,
    );
    setShot(shotAt(editor));
    const cells = editor.state.selection instanceof CellSelection;
    setTabled(
      !hushed.current && (empty || cells) && editor.isActive("table")
        ? { at: topOf(editor, $from.pos) }
        : null,
    );
    if (hushed.current || !empty || code || editor.isActive("code")) {
      return setAsking(null);
    }
    const word = asked($from.parent.textBetween(0, $from.parentOffset, undefined, " "));
    if (word === null) return setAsking(null);
    setAsking({ at: caret(editor, $from.pos), word });
    setActive(0);
  };

  const hands = useRef({
    onWrite,
    onOpen,
    onKeep,
    onDoc,
    onOwn,
    onShaped,
    onOutline,
    onLaid,
    onReady,
    onInsert,
  });
  hands.current = {
    onWrite,
    onOpen,
    onKeep,
    onDoc,
    onOwn,
    onShaped,
    onOutline,
    onLaid,
    onReady,
    onInsert,
  };
  looked.current = look;

  const picking = useRef(onTag);
  picking.current = onTag;
  const shapes = useMemo(
    () => [...written(), previewing(() => reach.current), tagging((tag) => picking.current?.(tag))],
    [],
  );

  const own = filed(papers, paper);
  const leaves = useMemo(() => paged(papers, paper), [papers, paper]);

  const props = useMemo(
    () => ({
      attributes: (state: EditorState) => ({
        class: shotNode(state) ? "tisty-doc shot-picked" : "tisty-doc",
        // Saying it takes files is what has one copied in before anyone asks whether it can.
        ...(reading ? {} : { [CATCHES]: "" }),
        role: "textbox",
        "aria-multiline": "true",
        spellcheck: "true",
        ...(label ? { "aria-label": label } : {}),
      }),
      transformPastedHTML: stripped,
      handleDOMEvents: {
        mousedown: () => {
          hushed.current = false;
          return false;
        },
        contextmenu: () => {
          hushed.current = true;
          setPicked(null);
          setAsking(null);
          return false;
        },
      },
      handleClick: clicking(
        (reference) => hands.current.onOpen?.(reference),
        (paper) => hands.current.onDoc?.(paper),
      ),
      handleKeyDown: (_view: unknown, e: KeyboardEvent) => {
        hushed.current = false;
        if (!now.current.open) return false;
        const count = now.current.count;
        if (e.key === "ArrowDown") {
          setActive((was) => (was + 1) % count);
          return true;
        }
        if (e.key === "ArrowUp") {
          setActive((was) => (was - 1 + count) % count);
          return true;
        }
        if (e.key === "Enter" || e.key === "Tab") {
          now.current.take();
          return true;
        }
        if (e.key === "Escape") {
          setAsking(null);
          return true;
        }
        return false;
      },
    }),
    [label, reading],
  );

  const wrote = useCallback(({ editor }: { editor: Writing }) => {
    const text = asMarkdown(editor);
    if (text !== null) {
      mine.current = text;
      hands.current.onWrite(text);
    }
    looked.current(editor);
    outlined.current(editor);
    hands.current.onLaid?.(editor.view.dom as HTMLElement);
  }, []);

  const moved = useCallback(({ editor }: { editor: Writing }) => looked.current(editor), []);

  const shaped = useCallback(({ editor }: { editor: Writing }) => {
    const text = asMarkdown(editor);
    if (text !== null) hands.current.onShaped?.(text);
    outlined.current(editor);
    hands.current.onLaid?.(editor.view.dom as HTMLElement);
    hands.current.onReady?.(() => editor.getJSON());
    hands.current.onInsert?.((file, title) =>
      editor.chain().focus("end").insertContent(card(file, title)).run(),
    );
  }, []);

  const listed = useRef("");
  const outlined = useRef((editor: Writing) => {
    const heads = headed(editor);
    const now = heads.map((one) => `${one.level}:${one.text}`).join("\n");
    if (now === listed.current) return;
    listed.current = now;
    hands.current.onOutline?.(heads);
  });

  const editor = useEditor({
    autofocus: false,
    editable: !reading,
    extensions: shapes,
    content: loosened(value),
    editorProps: props,
    onUpdate: wrote,
    onSelectionUpdate: moved,
    onCreate: shaped,
  });

  const blocks: Block[] = editor
    ? [
        {
          key: "h1",
          label: t("bigTitle"),
          hint: "#",
          icon: "heading1",
          run: () => editor.chain().focus().toggleHeading({ level: 1 }).run(),
        },
        {
          key: "h2",
          label: t("midTitle"),
          hint: "##",
          icon: "heading2",
          run: () => editor.chain().focus().toggleHeading({ level: 2 }).run(),
        },
        {
          key: "bullets",
          label: t("bullets"),
          hint: "-",
          icon: "bullets",
          run: () => editor.chain().focus().toggleBulletList().run(),
        },
        {
          key: "numbers",
          label: t("numbers"),
          hint: "1.",
          icon: "numbers",
          run: () => editor.chain().focus().toggleOrderedList().run(),
        },
        {
          key: "todo",
          label: t("checks"),
          hint: "[ ]",
          icon: "checks",
          run: () => editor.chain().focus().toggleTaskList().run(),
        },
        {
          key: "quote",
          label: t("quote"),
          hint: ">",
          icon: "quote",
          run: () => editor.chain().focus().toggleBlockquote().run(),
        },
        ...(
          [
            ["note", "info"],
            ["tip", "star"],
            ["important", "bell"],
            ["warning", "warning"],
            ["caution", "siren"],
          ] as const
        ).map(([kind, icon]) => ({
          key: `callout-${kind}`,
          label: t(`said${kind}` as Parameters<typeof t>[0]),
          hint: `[!${kind.toUpperCase()}]`,
          icon,
          run: () =>
            editor.isActive("callout")
              ? editor.chain().focus().updateAttributes("callout", { kind }).run()
              : editor.chain().focus().toggleWrap("callout", { kind }).run(),
        })),
        {
          key: "mermaid",
          label: t("diagramIt"),
          hint: "```mmd",
          icon: "graph",
          run: () => editor.chain().focus().toggleCodeBlock({ language: "mermaid" }).run(),
        },
        {
          key: "math",
          label: t("formulaIt"),
          hint: "```math",
          icon: "maths",
          run: () => editor.chain().focus().toggleCodeBlock({ language: "math" }).run(),
        },
        {
          key: "code",
          label: t("codeBlock"),
          hint: "```",
          icon: "code",
          run: () => editor.chain().focus().toggleCodeBlock().run(),
        },
        {
          key: "table",
          label: t("table"),
          hint: "|",
          icon: "table",
          run: () =>
            editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(),
        },
        {
          key: "link",
          label: t("linkIt"),
          hint: "[ ]( )",
          icon: "link",
          run: () => setTying(aimed(editor)),
        },
        ...(own && !own.pageOf
          ? [
              {
                key: "newpage",
                label: t("insertNewPage"),
                hint: "\u271a",
                icon: "pages",
                run: () => setNaming({ ...aimed(editor), leaf: true }),
              },
              {
                key: "page",
                label: t("insertPage"),
                hint: "[[ ]]",
                icon: "pages",
                run: () => setChoosing({ ...aimed(editor), leaf: true }),
              },
            ]
          : []),
        {
          key: "paper",
          label: t("insertDoc"),
          hint: "[[ ]]",
          icon: "page",
          run: () => setChoosing({ ...aimed(editor), leaf: false }),
        },
        {
          key: "newpaper",
          label: t("insertNewDoc"),
          hint: "\u271a",
          icon: "plus",
          run: () => setNaming({ ...aimed(editor), leaf: false }),
        },
        {
          key: "icon",
          label: t("insertIcon"),
          hint: "\u{1f600}",
          icon: "emoji",
          run: () => setGlyphing(aimed(editor)),
        },
        {
          key: "pen",
          label: t("penIt"),
          hint: "==",
          icon: "highlight",
          run: () => editor.chain().focus().extendMarkRange("highlight").toggleHighlight().run(),
        },
        {
          key: "rule",
          label: t("leafBreak"),
          hint: "---",
          icon: "rule",
          run: () => editor.chain().focus().setHorizontalRule().run(),
        },
        ...(onAttach
          ? [
              {
                key: "attach",
                label: t("attachment"),
                hint: "↧",
                icon: "clip",
                run: () => {
                  onAttach().then((markdown) => {
                    if (markdown) editor.chain().focus().insertContent(markdown).run();
                  });
                },
              },
            ]
          : []),
      ]
    : [];

  const handed = useRef<{ writing: unknown; keys: string } | null>(null);
  useEffect(() => {
    // A document that cannot be written must not be offered blocks that write to it.
    const offered = reading ? [] : blocks;
    const keys = offered.map((one) => one.key).join(",");
    if (handed.current?.writing === editor && handed.current.keys === keys) return;
    handed.current = { writing: editor, keys };
    onBlocks?.(offered);
  }, [blocks, editor, onBlocks, reading]);

  const shown = asking && !reading ? narrowed(blocks, asking.word) : [];

  const take = (block: Block) => {
    if (!editor || !asking) return;
    const to = editor.state.selection.$from.pos;
    editor
      .chain()
      .focus()
      .deleteRange({ from: to - asking.word.length - 1, to })
      .run();
    setAsking(null);
    block.run();
  };

  now.current = {
    open: Boolean(asking) && shown.length > 0,
    count: shown.length,
    take: () => take(shown[Math.min(active, shown.length - 1)]),
  };

  useEffect(() => {
    if (!editor || editor.isDestroyed) return;
    if (!stale(value, mine.current, () => asMarkdown(editor))) return;
    mine.current = value;
    editor.commands.setContent(loosened(value), { emitUpdate: false });
  }, [editor, value]);

  /// The reader gets the keyboard without a caret: focus on the sheet moves nothing, while focus
  /// in the text drags the view to wherever the caret sits.
  useEffect(() => {
    sheet.current?.focus({ preventScroll: true });
  }, []);

  nudge.current = () => {
    if (waiting.current) return;
    waiting.current = true;
    queueMicrotask(() => {
      waiting.current = false;
      if (!editor || editor.isDestroyed) return;
      editor.view.dispatch(editor.state.tr.setMeta("preview", true));
    });
  };

  const fetches = (reference: string, get: () => Promise<void>) => {
    if (pending.current.has(reference)) return;
    pending.current.add(reference);
    get()
      .catch((problem: unknown) => {
        missing.current.add(reference.slice(reference.indexOf(":") + 1));
        const code = (problem as { code?: string } | null)?.code;
        if (code) void noteTrouble(code).catch(() => {});
      })
      .then(() => nudge.current());
  };

  reach.current = {
    here: paper,
    onDoc,
    onOpen,
    onMenu: reading
      ? undefined
      : (at, untie, drop, kept, leaf) =>
          setSwapping({
            at,
            untie,
            drop,
            keep: kept && onKeep ? () => hands.current.onKeep?.(kept.at, kept.name) : undefined,
            own: leaf && onOwn ? () => hands.current.onOwn?.(leaf) : undefined,
          }),
    onWorld: (at) => {
      void openUrl(at).catch(() => {});
    },
    gone: (reference) => missing.current.has(reference),
    onAgain: (reference) => {
      missing.current.delete(reference);
      pending.current.delete(`url:${reference}`);
      pending.current.delete(`weight:${reference}`);
      nudge.current();
    },
    url: (reference) => {
      const held = urls.current.get(reference);
      if (held) return held;
      fetches(`url:${reference}`, () =>
        served(reference).then((real) => {
          urls.current.set(reference, convertFileSrc(real));
        }),
      );
      return null;
    },
    weight: (reference) => {
      const held = weights.current.get(reference);
      if (held !== undefined) return held;
      fetches(`weight:${reference}`, () =>
        weighs(reference).then((bytes) => {
          weights.current.set(reference, bytes);
        }),
      );
      return null;
    },
    title: (id) => {
      if (!papers) return undefined;
      const one = papers.find((paper) => paper.file === id);
      return one ? one.title.trim() || t("untitledDoc") : null;
    },
    page: (id) => {
      const at = leaves.indexOf(id);
      return at < 0 ? null : at + 1;
    },
    glimpse: (at) => {
      const held = glimpses.current.get(at);
      if (held !== undefined) return held;
      fetches(`glimpse:${at}`, () =>
        glimpseKept(at).then((one) => {
          glimpses.current.set(at, one);
        }),
      );
      return null;
    },
    onGlimpse: (at) => {
      glimpses.current.set(at, null);
      fetches(`glimpsing:${at}`, () =>
        glimpseFetch(at).then((one) => {
          glimpses.current.set(at, one);
        }),
      );
    },
    blurb: (id) => {
      const held = blurbs.current.get(id);
      if (held !== undefined) return held;
      fetches(`blurb:${id}`, () =>
        docRead(id).then((body) => {
          blurbs.current.set(id, glimpsed(body));
        }),
      );
      return null;
    },
  };

  const opened = Boolean(asking) && shown.length > 0;
  const current = Math.min(active, shown.length - 1);

  const wanted = useRef(seek);
  const marked = useRef(anchor);
  const putting = useRef(-1);
  const seen = useRef(onSeen);
  seen.current = onSeen;
  useEffect(() => {
    const want = wanted.current;
    const mark = marked.current;
    if (!sheet.current || !editor || editor.isDestroyed || (!want && !mark)) return;
    const spot = () => {
      const room = sheet.current;
      if (!room) return null;
      const found = mark ? room.querySelector<HTMLElement>(`[data-doc="${mark}"]`) : null;
      const box = found?.getBoundingClientRect();
      if (!box || (!box.height && !box.top)) return want ?? null;
      const above = box.top - room.getBoundingClientRect().top;
      return Math.max(0, Math.round(above + room.scrollTop - ASIDE_MARGIN));
    };
    putting.current = 0;
    return settles({
      at: () => sheet.current,
      want: spot,
      done: () => {
        putting.current = -1;
        const room = sheet.current;
        if (!room || room.scrollHeight <= room.clientHeight) return;
        seen.current?.(room.scrollTop);
      },
      put: (down) => {
        putting.current = down;
      },
      watch: () => (editor && !editor.isDestroyed ? editor.view.dom : null),
    });
  }, [editor]);

  useEffect(() => {
    const dom = editor && !editor.isDestroyed ? editor.view.dom : null;
    if (!dom) return;
    dom.setAttribute("aria-expanded", String(opened));
    if (!opened) {
      dom.removeAttribute("aria-controls");
      return dom.removeAttribute("aria-activedescendant");
    }
    dom.setAttribute("aria-controls", "slash-menu");
    dom.setAttribute("aria-activedescendant", `slash-${current}`);
  }, [editor, opened, current]);

  useEffect(() => {
    if (!editor || editor.isDestroyed || reading) return;
    return takesFiles(editor.view.dom, (put, at) => {
      const landed = at ? editor.view.posAtCoords({ left: at.left, top: at.top })?.pos : undefined;
      const chain = editor.chain().focus(landed ?? undefined);
      chain.insertContent(put).run();
    });
  }, [editor, reading]);

  useEffect(() => {
    if (!editor || editor.isDestroyed) return;
    let live = true;
    const dress = () =>
      requestAnimationFrame(() => {
        if (editor.isDestroyed) return;
        editor.view.dom.querySelectorAll<HTMLImageElement>("img[src]").forEach((img) => {
          const at = img.getAttribute("src") ?? "";
          if (!at || /^(https?|data|asset|blob|file):/i.test(at)) return;
          if (!pictured(at)) return;
          const cached = urls.current.get(at);
          if (cached) {
            img.setAttribute("src", cached);
            return;
          }
          served(at)
            .then((real) => {
              const url = convertFileSrc(real);
              urls.current.set(at, url);
              if (live) img.setAttribute("src", url);
            })
            .catch(() => undefined);
        });
      });

    dress();
    editor.on("transaction", dress);
    return () => {
      live = false;
      editor.off("transaction", dress);
    };
  }, [editor]);

  return (
    <>
      <div
        ref={sheet}
        tabIndex={-1}
        onScroll={(one) => {
          const room = one.currentTarget as HTMLElement;
          const down = room.scrollTop;
          if (putting.current >= 0) return;
          if (room.scrollHeight <= room.clientHeight) return;
          onSeen?.(down);
        }}
        className={`scroller gutter flex min-h-0 flex-1 flex-col${above || below ? " leafed" : ""}`}
      >
        {above}
        <EditorContent editor={editor} className={above || below ? undefined : "flex-1"} />
        {below}
      </div>
      {asking && shown.length > 0 && (
        <Slash at={asking.at} blocks={shown} active={active} onPick={take} />
      )}
      {picked && editor && !asking && !tying && !reading && (
        <Floats editor={editor} at={picked.at} />
      )}
      {tabled && editor && !reading && <Tabled editor={editor} at={tabled.at} />}
      {shot && editor && (
        <Shot
          at={shot.at}
          onOpen={() => onOpen?.(shot.src)}
          onKeep={onKeep ? () => onKeep(shot.src, shot.name) : undefined}
          onDrop={
            reading
              ? undefined
              : () => {
                  editor.chain().focus().deleteSelection().run();
                  setShot(null);
                }
          }
        />
      )}
      {tying && editor && !reading && (
        <Floats editor={editor} at={tying} asking onDone={() => setTying(null)} />
      )}
      {naming && editor && (
        <>
          <span
            className="fixed inset-0 z-30"
            onMouseDown={() => {
              setNaming(null);
              editor.commands.focus();
            }}
          />
          <div
            style={{
              left: Math.max(8, Math.min(naming.x, window.innerWidth - 290)),
              top: Math.max(8, naming.y + 6),
            }}
            onKeyDown={(e) => {
              if (e.key !== "Escape") return;
              e.preventDefault();
              e.stopPropagation();
              setNaming(null);
              editor.commands.focus();
            }}
            className="fixed z-40 w-[272px] rounded-[10px] border border-hair bg-rail p-1.5 shadow-xl"
          >
            <Asking
              leaf={naming.leaf}
              onName={(name) => {
                const leaf = naming.leaf;
                setNaming(null);
                spawned(name, folder ?? undefined, leaf ? own?.id : undefined)
                  .then((born) => {
                    onMade?.(born.id, name);
                    editor
                      .chain()
                      .focus()
                      .insertContent({
                        type: "image",
                        attrs: { src: `${DOC}${born.id}`, alt: name },
                      })
                      .run();
                  })
                  .catch(() => editor.commands.focus());
              }}
            />
          </div>
        </>
      )}
      {glyphing && editor && (
        <>
          <span
            className="fixed inset-0 z-30"
            onMouseDown={() => {
              setGlyphing(null);
              editor.commands.focus();
            }}
          />
          <div
            style={{
              left: Math.max(8, Math.min(glyphing.x, window.innerWidth - 290)),
              ...(glyphing.y + 6 + GLYPHS_TALL > window.innerHeight
                ? { bottom: Math.max(8, window.innerHeight - glyphing.y + 6) }
                : { top: Math.max(8, glyphing.y + 6) }),
              maxHeight: `${GLYPHS_TALL}px`,
            }}
            onKeyDown={(e) => {
              if (e.key !== "Escape") return;
              e.preventDefault();
              e.stopPropagation();
              setGlyphing(null);
              editor.commands.focus();
            }}
            className="fixed z-40 flex w-[272px] flex-col overflow-hidden rounded-[10px] border border-hair bg-rail p-1.5 shadow-xl"
          >
            <Glyphs
              onPick={(key, hue) => {
                setGlyphing(null);
                editor
                  .chain()
                  .focus()
                  .insertContent({ type: "ico", attrs: { name: key, hue: hue ?? null } })
                  .run();
              }}
            />
          </div>
        </>
      )}
      {swapping && (
        <Menu
          at={swapping.at}
          label={t("moreOnIt")}
          choices={[
            {
              key: "keep",
              label: t("keepACopy"),
              icon: "↧",
              off: !swapping.keep,
              onPick: swapping.keep,
            },
            {
              key: "own",
              label: t("ownDoc"),
              icon: "⇤",
              off: !swapping.own,
              onPick: swapping.own,
            },
            {
              key: "link",
              label: t("showAsLink"),
              icon: "↩",
              off: !swapping.untie,
              onPick: swapping.untie,
            },
            { key: "drop", label: t("remove"), icon: "✕", danger: true, onPick: swapping.drop },
          ]}
          onClose={() => setSwapping(null)}
        />
      )}

      {choosing && editor && (
        <>
          <span
            className="fixed inset-0 z-30"
            onMouseDown={() => {
              setChoosing(null);
              editor.commands.focus();
            }}
          />
          <div
            style={{
              left: Math.max(8, Math.min(choosing.x, window.innerWidth - 290)),
              top: Math.max(8, choosing.y + 6),
            }}
            onKeyDown={(e) => {
              if (e.key !== "Escape") return;
              e.preventDefault();
              e.stopPropagation();
              setChoosing(null);
              editor.commands.focus();
            }}
            className="fixed z-40 w-[272px] rounded-[10px] border border-hair bg-rail p-1.5 shadow-xl"
          >
            <Papers
              all={
                choosing.leaf ? pagesOf(papers, paper) : papers?.filter((one) => one.file !== paper)
              }
              onPick={(picked) => {
                setChoosing(null);
                editor
                  .chain()
                  .focus()
                  .insertContent({
                    type: "image",
                    attrs: { src: DOC + picked.file, alt: picked.title },
                  })
                  .run();
              }}
            />
          </div>
        </>
      )}
    </>
  );
}
