import { useLayoutEffect, useMemo, useRef, useState } from "react";
import { known as drawn } from "../glyphs";
import { t, type Word } from "../locales";
import { isMark } from "../marks";
import Glyph from "./Glyph";
import Hue from "./Hue";
import { type Family, sifted, useCatalogue } from "./Icons";

const known = (key: string) => drawn(key) || isMark(key);

const ROW = 36;
const SPARE = 2;

type Row = { title: string } | { keys: string[] };

const laid = (families: Family[], columns: number, titled: boolean): Row[] => {
  const rows: Row[] = [];
  for (const family of families) {
    if (titled) rows.push({ title: family.name });
    for (let at = 0; at < family.icons.length; at += columns) {
      rows.push({ keys: family.icons.slice(at, at + columns) });
    }
  }
  return rows;
};

interface Props {
  icon?: string;
  colour?: string | null;
  onIcon: (key: string | undefined) => void;
  onColour?: (hue: string | undefined) => void;
  autoFocus?: boolean;
  keepFocus?: boolean;
  clears?: boolean;
  tall?: string;
  opens?: "marks" | "icons";
}

export default function Pick({
  icon,
  colour,
  onIcon,
  onColour,
  autoFocus,
  keepFocus,
  clears = true,
  tall = "max-h-52",
  opens,
}: Props) {
  const { all, families } = useCatalogue();
  const [word, setWord] = useState("");
  const [marks, setMarks] = useState(opens ? opens === "marks" : !icon || isMark(icon));
  const [only, setOnly] = useState<string>();
  const box = useRef<HTMLFieldSetElement>(null);
  const [sized, setSized] = useState(false);
  const [wide, setWide] = useState(0);
  const [high, setHigh] = useState(0);
  const [down, setDown] = useState(0);
  const hold = keepFocus ? (e: React.MouseEvent) => e.preventDefault() : undefined;
  const said = word.trim();

  const marking = useMemo(
    () => families.filter((one) => one.icons.length > 0 && one.icons.every(isMark)),
    [families],
  );
  const split = marking.length > 0 && marking.length < families.length;
  const mine = useMemo(() => {
    if (!split) return families;
    return marks ? marking : families.filter((one) => !marking.includes(one));
  }, [families, marking, marks, split]);
  const ours = useMemo(() => (mine.length ? mine.flatMap((one) => one.icons) : all), [mine, all]);

  const shown = useMemo(() => {
    if (said) return [{ name: "", icons: sifted(ours, said).filter(known) }];
    const kept = only ? mine.filter((family) => family.name === only) : mine;
    if (!kept.length) return [{ name: "", icons: ours.filter(known) }];
    return kept.map((family) => ({ ...family, icons: family.icons.filter(known) }));
  }, [ours, mine, only, said]);

  const titled = shown.length > 1;
  const columns = wide ? Math.max(1, Math.floor(wide / ROW)) : 0;
  const rows = useMemo(
    () => (columns ? laid(shown, columns, titled) : []),
    [shown, columns, titled],
  );
  const many = shown.reduce((sofar, family) => sofar + family.icons.length, 0);

  useLayoutEffect(() => {
    const held = box.current;
    if (!held) return;
    const measure = () => {
      setWide(held.clientWidth);
      setHigh(held.clientHeight);
    };
    measure();
    setSized(true);
    if (typeof ResizeObserver === "undefined") return;
    const watcher = new ResizeObserver(measure);
    watcher.observe(held);
    return () => watcher.disconnect();
  }, []);

  const restart = () => {
    setDown(0);
    if (box.current) box.current.scrollTop = 0;
  };

  const room = high || ROW * 6;
  // Clamps a scrollTop that outran the rows (a stale value after the list shrank) to the tail.
  const settled = Math.min(down, Math.max(0, rows.length * ROW - room));
  const first = Math.max(0, Math.floor(settled / ROW) - SPARE);
  const last = Math.min(rows.length, Math.ceil((settled + room) / ROW) + SPARE);

  const button = (key: string) => (
    <button
      key={key}
      type="button"
      onMouseDown={hold}
      onClick={() => onIcon(key)}
      aria-pressed={icon === key}
      aria-label={key}
      title={key}
      className={`grid h-8 w-8 shrink-0 place-items-center rounded-lg ${
        icon === key ? "bg-accent-soft" : "hover:bg-hover"
      }`}
    >
      <Glyph name={key} />
    </button>
  );

  return (
    <div className="flex min-h-0 flex-col gap-1.5">
      <div className="flex shrink-0 gap-1.5">
        <input
          autoFocus={autoFocus}
          value={word}
          onChange={(e) => {
            setWord(e.target.value);
            restart();
          }}
          placeholder={t("siftIcons")}
          aria-label={t("siftIcons")}
          className="min-w-0 flex-1 rounded-lg bg-hover px-2.5 py-1 text-[12px] outline-none placeholder:text-faint"
        />
        {clears && (
          <button
            type="button"
            onMouseDown={hold}
            onClick={() => onIcon(undefined)}
            aria-pressed={!icon}
            aria-label={t("noIcon")}
            title={t("noIcon")}
            className={`grid w-8 shrink-0 self-stretch place-items-center rounded-lg text-[13px] ${
              icon ? "text-faint hover:bg-hover" : "bg-accent-soft text-accent"
            }`}
          >
            ○
          </button>
        )}
      </div>
      {split && (
        <div role="tablist" aria-label={t("iconKinds")} className="flex shrink-0 gap-1">
          {[true, false].map((mine) => (
            <button
              key={String(mine)}
              type="button"
              role="tab"
              onMouseDown={hold}
              onClick={() => {
                setMarks(mine);
                setOnly(undefined);
                restart();
              }}
              aria-selected={marks === mine}
              className={`flex-1 rounded-lg px-2 py-1 text-[11.5px] ${
                marks === mine ? "bg-accent-soft text-accent" : "text-faint hover:bg-hover"
              }`}
            >
              {mine ? t("marksTab") : t("iconsTab")}
            </button>
          ))}
        </div>
      )}
      {mine.length > 0 && !said && (
        <div
          role="group"
          aria-label={t("iconFamilies")}
          className="scroller flex shrink-0 gap-1 overflow-x-auto pb-0.5"
        >
          {[undefined, ...mine.map((family) => family.name)].map((name) => (
            <button
              key={name ?? "all"}
              type="button"
              onMouseDown={hold}
              onClick={() => {
                setOnly(name);
                restart();
              }}
              aria-pressed={only === name}
              className={`shrink-0 rounded-full px-2 py-0.5 text-[11px] whitespace-nowrap ${
                only === name ? "bg-accent-soft text-accent" : "text-faint hover:bg-hover"
              }`}
            >
              {name ? (t(`family_${name}` as Word) ?? name) : t("allIcons")}
            </button>
          ))}
        </div>
      )}
      <fieldset
        ref={box}
        onScroll={(e) => setDown(e.currentTarget.scrollTop)}
        className={`scroller min-h-0 overflow-y-auto ${tall}`}
      >
        <legend className="sr-only">{t("pickAnIcon")}</legend>
        {many === 0 && sized && (
          <p className="px-2.5 py-1.5 text-[12px] text-faint">{t("noneHere")}</p>
        )}
        {columns === 0 ? (
          sized && (
            <div className="flex flex-wrap content-start gap-1">
              {shown.flatMap((family) => family.icons).map(button)}
            </div>
          )
        ) : (
          <div style={{ height: rows.length * ROW }} className="relative">
            <div
              className="absolute inset-x-0 top-0"
              style={{ transform: `translateY(${first * ROW}px)` }}
            >
              {rows.slice(first, last).map((row, at) => {
                // The row number keys these: a family and its first icon share a name often enough.
                const mark = first + at;
                return "title" in row ? (
                  <p
                    key={mark}
                    className="flex h-9 items-end px-0.5 pb-1 text-[10px] font-medium tracking-wide text-faint uppercase"
                  >
                    {t(`family_${row.title}` as Word) ?? row.title}
                  </p>
                ) : (
                  <div key={mark} className="flex h-9 gap-1">
                    {row.keys.map(button)}
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </fieldset>
      {onColour && (!split || !marks) && (
        <div className="shrink-0 border-t border-hair pt-2">
          <Hue chosen={colour} onPick={onColour} onHold={hold} />
        </div>
      )}
    </div>
  );
}
