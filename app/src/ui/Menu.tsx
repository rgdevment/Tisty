import {
  type CSSProperties,
  type KeyboardEvent,
  type ReactNode,
  useEffect,
  useId,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { createPortal } from "react-dom";
import { folded } from "../folding";
import Glyph from "./Glyph";
import {
  type Anchor,
  beside,
  EDGE,
  found,
  leadingOf,
  type Placed,
  placement,
  stepped,
  typedTo,
} from "./menuKeys";

export interface Choice {
  key: string;
  label: string;
  icon?: string;
  glyph?: string;
  mark?: string;
  danger?: boolean;
  apart?: boolean;
  off?: boolean;
  hint?: string;
  keys?: string;
  code?: string;
  on?: boolean;
  /** The heading over a run of rows: a row with the same group as the one above it stays under that heading. */
  group?: string;
  /** More words it is found by, beside its label and those of the rows it opens. */
  words?: string;
  /** A list of its own, opened beside this row. */
  into?: { label: string; choices: Choice[] };
  onPick?: () => void;
}

interface Props {
  /** `top` is where the line the menu answers begins, so a menu put above it never covers it; `right` is its right edge. */
  at: Anchor;
  choices: Choice[];
  label: string;
  /** Opened from the bottom of the screen: `at.y` is the edge the menu grows up from. */
  up?: boolean;
  /** A row's own actions: shown whole above the row when they fit there, rather than cut off below it. */
  whole?: boolean;
  keeps?: boolean;
  find?: string;
  /** A line under the choices on how the keys walk them. */
  foot?: string;
  /** Shortcuts in one row over the list, the choices most often wanted; they step aside while something is sought. */
  row?: Choice[];
  /** What is said when nothing answers what was sought. */
  none?: (sought: string) => string;
  /** How wide the menu stands, when its rows need more than the narrowest. */
  wide?: number;
  /** The most it grows to, so a long list scrolls under its search rather than taking the window. */
  tallest?: number;
  /** Put away without a choice, it hands back what was typed in its search, so those words are not lost. */
  onClose: (sought?: string) => void;
}

const HOVER_MS = 160;

// The bar above a page holds its title and its menu, so a menu flipped up stops under it.
const ceilingOver = (top: number) =>
  Array.from(document.querySelectorAll(".ribbon"), (one) => one.getBoundingClientRect().bottom)
    .filter((bottom) => bottom <= top)
    .reduce((most, bottom) => Math.max(most, bottom), EDGE);

const wordsOf = (one: Choice): string =>
  [one.label, one.words, one.code, ...(one.into?.choices.map(wordsOf) ?? [])]
    .filter(Boolean)
    .join(" ");

const viewNow = () => ({ wide: window.innerWidth, tall: window.innerHeight });

const styled = ({ x, top, bottom, tall }: Placed): CSSProperties => ({
  left: x,
  top,
  bottom,
  maxHeight: tall,
});

const CARD =
  "scroller arrive fixed z-[70] max-h-[80vh] [scrollbar-gutter:stable] rounded-[10px] border border-hair bg-rail p-1 shadow-lg";

type Shared = {
  /** Every card the menu has open, so a press in any of them is no press outside. */
  cards: Set<HTMLElement>;
  close: (sought?: string) => void;
  keeps?: boolean;
};

type Listed = {
  choices: Choice[];
  active: number;
  opened: number | null;
  named: string;
  /** The keys walk the rows themselves, as nothing else holds them. */
  roving: boolean;
  onHover: (index: number) => void;
  onPick: (index: number, row: HTMLElement) => void;
};

// Rows with a group heading are held together under it, so a reader hears where each one belongs.
const runsOf = (choices: Choice[]) =>
  choices.reduce<{ group?: string; at: number[] }[]>((runs, one, at) => {
    const last = runs[runs.length - 1];
    if (last && last.group === one.group) last.at.push(at);
    else runs.push({ group: one.group, at: [at] });
    return runs;
  }, []);

function Rows({ choices, active, opened, named, roving, onHover, onPick }: Listed) {
  const row = (at: number) => {
    const one = choices[at];
    return (
      <div key={one.key}>
        {one.apart && <div className="my-1 h-px bg-hair" />}
        <button
          type="button"
          role="menuitem"
          id={`${named}-${at}`}
          data-active={at === active ? "" : undefined}
          aria-haspopup={one.into ? "menu" : undefined}
          aria-expanded={one.into ? opened === at : undefined}
          tabIndex={roving && at === active ? 0 : -1}
          title={one.hint}
          onClick={(e) => onPick(at, e.currentTarget)}
          onMouseMove={() => onHover(at)}
          className={`group flex w-full items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-[12.5px] focus:outline-none data-[active]:bg-hover ${
            one.danger ? "text-urgent" : "text-ink"
          }${one.on ? " font-semibold" : ""}`}
        >
          {one.glyph ? (
            <Glyph
              name={one.glyph}
              className="h-[15px] w-[15px] text-faint group-data-[active]:text-accent"
            />
          ) : one.code ? (
            <span aria-hidden className="w-9 shrink-0 text-[10.5px] tracking-[0.04em] text-faint">
              {one.code}
            </span>
          ) : (
            <span aria-hidden className="w-3.5 shrink-0 text-center text-[11.5px] text-faint">
              {one.icon ?? ""}
            </span>
          )}
          <span className="truncate">{one.label}</span>
          {one.mark && (
            <span aria-hidden className="ml-auto pl-4 font-mono text-[10.5px] text-faint">
              {one.mark}
            </span>
          )}
          {one.keys && (
            <span aria-hidden className="ml-auto pl-4 text-[11.5px] text-faint">
              {one.keys}
            </span>
          )}
          {one.on && (
            <span aria-hidden className="ml-auto pl-3 text-[11.5px] text-faint">
              ✓
            </span>
          )}
          {one.into && (
            <span
              aria-hidden
              className={`${one.mark || one.keys ? "pl-2" : "ml-auto pl-3"} text-[11.5px] text-faint`}
            >
              ›
            </span>
          )}
        </button>
      </div>
    );
  };
  return runsOf(choices).map((run) =>
    run.group ? (
      <fieldset
        key={`group-${run.at[0]}`}
        aria-label={run.group}
        className="m-0 min-w-0 border-0 p-0"
      >
        <div
          aria-hidden
          className="px-2.5 pt-2 pb-1 text-[10.5px] uppercase tracking-[0.06em] text-faint"
        >
          {run.group}
        </div>
        {run.at.map(row)}
      </fieldset>
    ) : (
      run.at.map(row)
    ),
  );
}

type Walking = {
  choices: Choice[];
  active: number;
  setActive: (at: number) => void;
  opened: number | null;
  open: (at: number, row: HTMLElement | null, focused: boolean) => void;
  /** The search holds the keys: the arrows move the mark while the letters still go to the words. */
  sifting: boolean;
};

const rowOf = (card: HTMLElement | null, at: number) =>
  card?.querySelectorAll<HTMLElement>("[role=menuitem]")[at] ?? null;

const walked = (e: KeyboardEvent, card: HTMLElement | null, w: Walking): boolean => {
  const count = w.choices.length;
  if (e.key === "ArrowDown" || e.key === "ArrowUp") {
    w.setActive(stepped(count, w.active, e.key === "ArrowDown" ? 1 : -1));
    return true;
  }
  if (e.key === "Home" || e.key === "End") {
    if (w.sifting) return false;
    w.setActive(e.key === "Home" ? 0 : count - 1);
    return true;
  }
  const into = w.choices[w.active]?.into;
  if (e.key === "ArrowRight" && into) {
    const field = e.target instanceof HTMLInputElement ? e.target : null;
    if (field && field.selectionStart !== field.value.length) return false;
    w.open(w.active, rowOf(card, w.active), true);
    return true;
  }
  if (w.sifting || e.key.length !== 1 || e.ctrlKey || e.metaKey || e.altKey) return false;
  const to = typedTo(
    w.choices.map((one) => one.label),
    w.active,
    e.key,
  );
  if (to >= 0) w.setActive(to);
  return to >= 0;
};

// A list of its own opens only once the pointer rests on its row, so passing over it opens nothing.
const useHovering = (open: Walking["open"], shut: () => void) => {
  const waiting = useRef(0);
  const over = useRef(-1);
  useEffect(() => () => window.clearTimeout(waiting.current), []);
  const reset = () => {
    window.clearTimeout(waiting.current);
    over.current = -1;
  };
  const hovered = (one: Choice | undefined, at: number, row: HTMLElement | null) => {
    if (over.current === at) return;
    over.current = at;
    window.clearTimeout(waiting.current);
    waiting.current = window.setTimeout(
      () => (one?.into ? open(at, row, false) : shut()),
      HOVER_MS,
    );
  };
  return { hovered, reset };
};

// A row found by the words of a row inside it opens on that row, so Enter takes what was sought.
const soughtIn = (one: Choice | undefined, sought: string): number => {
  const inside = one?.into?.choices.filter((row) => !row.off) ?? [];
  const hits = inside.flatMap((row, at) => (sought && found(wordsOf(row), sought) ? [at] : []));
  if (hits.length === 0) return 0;
  const words = hits.map((at) => ({ words: wordsOf(inside[at]), opens: Boolean(inside[at].into) }));
  return hits[leadingOf(words, sought)] ?? 0;
};

type FlyoutProps = Shared & {
  label: string;
  choices: Choice[];
  from: HTMLElement;
  focused: boolean;
  /** The row the keys start on. */
  first?: number;
  back: () => void;
  /** The pointer reached it, so a row it crossed on the way no longer asks to close it. */
  onEnter: () => void;
};

function Flyout({
  label,
  choices,
  from,
  focused,
  first = 0,
  back,
  onEnter,
  ...shared
}: FlyoutProps) {
  const card = useRef<HTMLDivElement>(null);
  const named = useId();
  const live = choices.filter((one) => !one.off);
  // Unseen until it is laid, yet still able to take the focus, which a hidden card refuses.
  const [where, setWhere] = useState<CSSProperties>({ opacity: 0 });
  const [active, setActive] = useState(focused ? first : -1);
  const [opened, setOpened] = useState<{ at: number; row: HTMLElement; focused: boolean } | null>(
    null,
  );
  const open = (at: number, row: HTMLElement | null, keyed: boolean) =>
    row && setOpened({ at, row, focused: keyed });
  const shut = () => setOpened(null);
  const { hovered, reset } = useHovering(open, shut);

  useLayoutEffect(() => {
    const own = card.current;
    if (!own) return;
    shared.cards.add(own);
    const edges = own.offsetHeight - own.clientHeight;
    const size = { wide: own.offsetWidth, tall: own.scrollHeight + edges };
    setWhere(styled(beside(from.getBoundingClientRect(), size, viewNow())));
    return () => {
      shared.cards.delete(own);
    };
  }, [from, shared.cards]);

  useEffect(() => {
    if (active >= 0) rowOf(card.current, active)?.focus();
  }, [active]);

  // Opened by the pointer and then asked for by a key, it takes the keys from then on.
  useEffect(() => {
    if (!focused) return;
    setActive(first);
    rowOf(card.current, first)?.focus();
  }, [focused, first]);

  const pick = (at: number, row: HTMLElement) => {
    const one = live[at];
    if (!one) return;
    if (one.into) return open(at, row, true);
    shared.close();
    one.onPick?.();
  };

  return (
    <>
      {createPortal(
        <div
          ref={card}
          role="menu"
          aria-label={label}
          data-keeps-writing={shared.keeps ? "" : undefined}
          style={where}
          onPointerEnter={onEnter}
          onKeyDown={(e) => {
            if (!card.current?.contains(e.target as Node)) return;
            e.stopPropagation();
            if (e.key === "Escape" || e.key === "ArrowLeft") {
              e.preventDefault();
              return back();
            }
            const w = { choices: live, active, setActive, opened: opened?.at ?? null, open };
            if (walked(e, card.current, { ...w, sifting: false })) e.preventDefault();
          }}
          className={`${CARD} min-w-[190px]`}
        >
          <Rows
            choices={live}
            active={active}
            opened={opened?.at ?? null}
            named={named}
            roving
            onHover={(at) => {
              setActive(at);
              hovered(live[at], at, rowOf(card.current, at));
            }}
            onPick={pick}
          />
        </div>,
        document.body,
      )}
      {opened && live[opened.at]?.into && (
        <Flyout
          {...shared}
          key={opened.at}
          label={live[opened.at].into?.label ?? ""}
          choices={live[opened.at].into?.choices ?? []}
          from={opened.row}
          focused={opened.focused}
          back={() => {
            const at = opened.at;
            shut();
            reset();
            rowOf(card.current, at)?.focus();
          }}
          onEnter={reset}
        />
      )}
    </>
  );
}

export default function Menu({
  at,
  choices,
  label,
  up,
  whole,
  keeps,
  find,
  foot,
  row,
  none,
  wide,
  tallest = Number.POSITIVE_INFINITY,
  onClose,
}: Props) {
  const card = useRef<HTMLDivElement>(null);
  const field = useRef<HTMLInputElement>(null);
  const header = useRef<HTMLDivElement>(null);
  const footer = useRef<HTMLDivElement>(null);
  const named = useId();
  const cards = useRef(new Set<HTMLElement>()).current;
  const [where, setWhere] = useState<CSSProperties>({ left: at.x, top: at.y });
  const [asked, setAsked] = useState("");
  const typed = useRef("");
  const sifting = Boolean(find);
  const sought = folded(asked.trim());
  const live = choices.filter((one) => !one.off && found(wordsOf(one), sought));
  const leading = leadingOf(
    live.map((one) => ({ words: wordsOf(one), opens: Boolean(one.into) })),
    sought,
  );
  const [active, setActive] = useState(sifting ? leading : 0);
  // A row marked by the pointer is already in view; scrolling to it would slide the next one under the pointer.
  const byPointer = useRef(false);
  const mark = (at: number, pointed = false) => {
    byPointer.current = pointed;
    setActive(at);
  };
  const [opened, setOpened] = useState<{ at: number; row: HTMLElement; focused: boolean } | null>(
    null,
  );
  const shown = Math.min(active, live.length - 1);
  const close = (sought?: string) => onClose(sought);
  const shared: Shared = { cards, close, keeps };
  const tiles = row?.filter((one) => !one.off) ?? [];

  const open = (at: number, from: HTMLElement | null, focused: boolean) =>
    from && setOpened({ at, row: from, focused });
  const shut = () => setOpened(null);
  const { hovered, reset } = useHovering(open, shut);

  // The menu is laid once from where it was asked: what is sought then only shortens it toward the line it answers.
  useLayoutEffect(() => {
    const own = card.current;
    if (!own) return;
    const lay = () => {
      const edges = own.offsetHeight - own.clientHeight;
      const tall = Math.min(own.scrollHeight + edges, tallest);
      // The card arrives scaled down a little, so its width is read untransformed.
      const size = { wide: Math.max(own.offsetWidth, own.getBoundingClientRect().width), tall };
      const anchor = { x: at.x, y: at.y, top: at.top, right: at.right };
      setWhere(styled(placement(anchor, size, viewNow(), ceilingOver(at.top ?? at.y), up, whole)));
    };
    lay();
    window.addEventListener("resize", lay);
    return () => window.removeEventListener("resize", lay);
  }, [at.x, at.y, at.top, at.right, up, whole, tallest, choices.length]);

  useLayoutEffect(() => {
    const own = card.current;
    if (!own) return;
    cards.add(own);
    return () => {
      cards.delete(own);
    };
  }, [cards]);

  useEffect(() => {
    const came = document.activeElement as HTMLElement | null;
    const own = card.current;
    // A choice that put the focus somewhere of its own, a field to type in, keeps it there.
    return () => {
      const now = document.activeElement;
      const ours = [...cards].some((one) => one.contains(now));
      if (!now || now === document.body || own?.contains(now) || ours) came?.focus?.();
    };
  }, [cards]);

  // What is sought puts the mark on the row Enter would take.
  // biome-ignore lint/correctness/useExhaustiveDependencies: only new words move the mark there.
  useEffect(() => {
    if (sifting) mark(leading);
    setOpened(null);
    reset();
  }, [sought]);

  // The heading and the foot stay put over the rows, so a row is brought into view clear of both.
  useEffect(() => {
    const own = card.current;
    const row = rowOf(own, shown);
    if (own) {
      const [over, under] = [header.current?.offsetHeight ?? 0, footer.current?.offsetHeight ?? 0];
      own.style.scrollPaddingTop = `${over}px`;
      own.style.scrollPaddingBottom = `${under}px`;
    }
    if (sifting) field.current?.focus();
    else row?.focus({ preventScroll: true });
    if (!byPointer.current) row?.scrollIntoView?.({ block: "nearest" });
  }, [shown, sifting]);

  useEffect(() => {
    const away = (e: Event) => {
      const inside = [...cards].some((one) => one.contains(e.target as Node));
      if (!inside) onClose(typed.current);
    };
    // A click in a page's frame never reaches this document; it takes the focus from the window instead.
    const left = () => onClose(typed.current);
    const listen = setTimeout(() => {
      document.addEventListener("mousedown", away);
      document.addEventListener("scroll", away, true);
      window.addEventListener("blur", left);
    });
    return () => {
      clearTimeout(listen);
      document.removeEventListener("mousedown", away);
      document.removeEventListener("scroll", away, true);
      window.removeEventListener("blur", left);
    };
  }, [onClose, cards]);

  const pick = (at: number, from: HTMLElement | null) => {
    const one = live[at];
    if (!one) return;
    if (one.into) return open(at, from, true);
    onClose();
    one.onPick?.();
  };

  const regain = (at: number) => {
    shut();
    reset();
    if (sifting) field.current?.focus();
    else rowOf(card.current, at)?.focus();
  };

  const head: ReactNode = sifting && (
    <input
      ref={field}
      type="search"
      value={asked}
      placeholder={find}
      aria-label={find}
      aria-controls={named}
      aria-activedescendant={shown >= 0 ? `${named}-${shown}` : undefined}
      onChange={(e) => {
        typed.current = e.target.value;
        // A space right after the slash is a sentence going on, as in «10 / 2», not a block being sought.
        if (e.target.value.startsWith(" ")) return onClose(e.target.value);
        setAsked(e.target.value);
      }}
      className="mx-1 mb-1 w-[calc(100%-0.5rem)] rounded-md bg-hover px-2.5 py-1.5 text-[12.5px] text-ink outline-none placeholder:text-faint"
    />
  );

  return (
    <>
      {createPortal(
        <div
          ref={card}
          id={named}
          role="menu"
          aria-label={label}
          data-keeps-writing={keeps ? "" : undefined}
          style={{ ...where, width: wide }}
          onKeyDown={(e) => {
            if (!card.current?.contains(e.target as Node)) return;
            if (e.key === "Escape") {
              e.stopPropagation();
              return onClose(typed.current);
            }
            if (e.key === "Enter" && e.target === field.current) {
              e.preventDefault();
              return pick(shown, rowOf(card.current, shown));
            }
            const w = {
              choices: live,
              active: shown,
              setActive: mark,
              opened: opened?.at ?? null,
              open,
            };
            if (walked(e, card.current, { ...w, sifting })) e.preventDefault();
          }}
          className={`${CARD} min-w-[210px]`}
        >
          {(head || tiles.length > 0) && (
            <div ref={header} className="sticky -top-1 z-10 -mx-1 -mt-1 bg-rail px-1 pt-1">
              {head}
              {tiles.length > 0 && !sought && (
                <div
                  className="grid gap-1 px-1 pb-1"
                  style={{ gridTemplateColumns: `repeat(${tiles.length}, minmax(0, 1fr))` }}
                >
                  {tiles.map((one) => (
                    <button
                      key={one.key}
                      type="button"
                      tabIndex={-1}
                      aria-label={one.label}
                      title={one.hint ?? one.label}
                      onClick={() => {
                        onClose();
                        one.onPick?.();
                      }}
                      className="flex min-w-0 flex-col items-center gap-0.5 rounded-md border border-hair px-0.5 py-1.5 text-faint hover:bg-hover hover:text-ink"
                    >
                      {one.glyph ? (
                        <Glyph name={one.glyph} className="h-[15px] w-[15px]" />
                      ) : (
                        <span aria-hidden className="text-[12.5px]">
                          {one.icon}
                        </span>
                      )}
                      <span className="w-full truncate text-center text-[10.5px]">{one.label}</span>
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}
          <Rows
            choices={live}
            active={shown}
            opened={opened?.at ?? null}
            named={named}
            roving={!sifting}
            onHover={(at) => {
              mark(at, true);
              hovered(live[at], at, rowOf(card.current, at));
            }}
            onPick={pick}
          />
          {live.length === 0 && none && (
            <p role="status" className="px-2.5 py-3 text-[12.5px] text-faint">
              {none(asked.trim())}
            </p>
          )}
          {foot && (
            <div ref={footer} className="sticky -bottom-1 -mx-1 -mb-1 bg-rail px-1 pb-1">
              <p className="mx-1 mt-1 border-hair border-t px-2 pt-1.5 pb-0.5 text-[10.5px] leading-snug text-faint">
                {foot}
              </p>
            </div>
          )}
        </div>,
        document.body,
      )}
      {opened && live[opened.at]?.into && (
        <Flyout
          {...shared}
          key={opened.at}
          label={live[opened.at].into?.label ?? ""}
          choices={live[opened.at].into?.choices ?? []}
          from={opened.row}
          focused={opened.focused}
          first={soughtIn(live[opened.at], sought)}
          back={() => regain(opened.at)}
          onEnter={reset}
        />
      )}
    </>
  );
}
