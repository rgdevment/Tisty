import { useEffect, useId, useRef, useState } from "react";
import { flipFold, railed, useBreadth } from "../breadth";
import { ACTIONS, actions, sifted } from "../commanding";
import { type Found, sow, type Task } from "../core";
import { seeing, useDesk } from "../desk";
import { jumps } from "../jumping";
import { t } from "../locales";
import { findNow } from "./finding";
import { useHeldFocus } from "./heldFocus";
import { optionOf, PickList, usePicker } from "./Picker";
import { useSearch } from "./Search";
import { onMac } from "./WindowChrome";

export const PALETTE_KEY = onMac ? "⌘P" : "Ctrl+P";
export const ACTIONS_KEY = onMac ? "⇧⌘P" : "Ctrl+Shift+P";

const covered = (): boolean => document.querySelector('[aria-modal="true"]') !== null;

// Ctrl+P never prints the window: it opens the palette, and with Shift its actions.
export function usePaletteKey(open: boolean, onOpen: (asked: string) => void) {
  const now = useRef(onOpen);
  now.current = onOpen;
  useEffect(() => {
    const keyed = (event: KeyboardEvent) => {
      if (event.repeat || event.altKey || !(event.ctrlKey || event.metaKey)) return;
      if (event.key.toLowerCase() !== "p") return;
      event.preventDefault();
      if (!open && covered()) return;
      now.current(event.shiftKey ? ACTIONS : "");
    };
    window.addEventListener("keydown", keyed);
    return () => window.removeEventListener("keydown", keyed);
  }, [open]);
}

export default function Palette() {
  const [asked, setAsked] = useState<string | null>(null);
  usePaletteKey(asked !== null, setAsked);
  if (asked === null) return null;
  return <Opened asked={asked} onAsked={setAsked} onClose={() => setAsked(null)} />;
}

function Opened({
  asked,
  onAsked,
  onClose,
}: {
  asked: string;
  onAsked: (asked: string) => void;
  onClose: () => void;
}) {
  const desk = useDesk();
  const { chosen, choose, papers, data, hands, here, setHere, opening, openDoc, board } = desk;
  const folded = railed(useBreadth(), chosen.named === "keeping").folded;
  const commanding = asked.startsWith(ACTIONS);
  const words = commanding ? asked.slice(ACTIONS.length) : asked;
  const [found, setFound] = useState<Found | null>(null);
  useSearch(commanding ? "" : words, "either", setFound, desk.setError);

  // The palette goes before the act, so whatever the act opens takes the focus after it.
  const after =
    <A extends unknown[]>(act: (...given: A) => unknown) =>
    (...given: A) => {
      onClose();
      act(...given);
    };
  const rows = commanding
    ? sifted(
        actions({
          chosen,
          here,
          hands,
          board,
          folded,
          choose,
          library: () => {
            choose({ named: "docs" });
            setHere(null);
          },
          diary: desk.toDiary,
          capture: () => void sow().catch(() => {}),
          find: () => {
            if (chosen.named === "keeping") choose({ named: "tasks" });
            findNow(folded);
          },
          fold: flipFold,
        }).map((one) => ({ ...one, onPick: after(one.onPick) })),
        words,
      )
    : jumps(papers, data.every ?? data.lists, words, found, {
        doc: after(openDoc),
        folder: after((id: string) => {
          choose({ named: "docs" });
          setHere(id);
        }),
        task: after((one: Task) => {
          choose(seeing(one));
          opening(one);
        }),
      });

  const list = useId();
  const { on, setAt, keyed } = usePicker(rows, asked, onClose);
  const box = useRef<HTMLDivElement>(null);
  const { wrapped, held } = useHeldFocus(box);
  const empty = commanding ? t("paletteNoAction") : words.trim() ? t("paletteNothing") : null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={t("paletteLabel")}
      className="fixed inset-0 z-50 flex justify-center bg-veil px-4 pt-[12vh]"
      onMouseDown={(event) => event.target === event.currentTarget && onClose()}
      onKeyDown={wrapped}
      onBlur={held}
    >
      <div
        ref={box}
        className="flex max-h-[min(520px,76vh)] w-full max-w-[560px] flex-col self-start rounded-[10px] border border-hair bg-bg p-1.5 shadow-lift-tall"
      >
        <input
          value={asked}
          role="combobox"
          aria-expanded="true"
          aria-controls={list}
          aria-activedescendant={optionOf(list, on)}
          aria-label={t("paletteLabel")}
          placeholder={t(commanding ? "paletteActionsHint" : "paletteHint")}
          spellCheck={false}
          onFocus={(event) => event.currentTarget.setSelectionRange(asked.length, asked.length)}
          onChange={(event) => onAsked(event.target.value)}
          onKeyDown={(event) => {
            if (!event.nativeEvent.isComposing) keyed(event);
          }}
          className="w-full rounded-md bg-hover px-3 py-2 text-[13px] text-ink outline-none placeholder:text-faint"
        />
        <div className="scroller mt-1 min-h-0 flex-1">
          {rows.length > 0 ? (
            <PickList id={list} label={t("paletteLabel")} rows={rows} on={on} onHover={setAt} />
          ) : (
            empty && <p className="px-2.5 py-2 text-[12.5px] text-faint">{empty}</p>
          )}
        </div>
        <p className="mt-1 border-t border-hair px-2.5 pt-1.5 pb-0.5 text-[11.5px] text-faint">
          {t("paletteFoot")}
        </p>
      </div>
    </div>
  );
}
