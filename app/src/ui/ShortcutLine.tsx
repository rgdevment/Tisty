import { useEffect, useState } from "react";
import { type Door, keepShortcut, shortcut, shortcutsOffered } from "../core";
import { t } from "../locales";
import type { Run, Which, Word } from "./Card";
import { Line, off } from "./Rows";

interface Props {
  door: Door;
  which: Which;
  title: string;
  why: string;
  label: string;
  ready: boolean;
  keys: string | null;
  chosen?: string;
  held: boolean;
  said?: Word;
  trouble?: Word;
  run: Run;
  onKeys: (keys: string | null) => void;
  onChosen: (wanted: string) => void;
}

export default function ShortcutLine({
  door,
  which,
  title,
  why,
  label,
  ready,
  keys,
  chosen,
  held,
  said,
  trouble,
  run,
  onKeys,
  onChosen,
}: Props) {
  const [offered, setOffered] = useState<string[]>([]);
  useEffect(() => {
    shortcutsOffered(door)
      .then((all) => setOffered(all ?? []))
      .catch(() => {});
  }, [door]);
  const chosenOff = chosen === "off";

  return (
    <Line title={title} why={why} which={which} said={said} trouble={trouble}>
      {ready && (
        <select
          aria-label={label}
          value={keys ?? (chosenOff ? "off" : "")}
          disabled={held}
          onChange={(e) => {
            const wanted = e.target.value;
            run(
              which,
              keepShortcut(wanted, door).finally(() => {
                shortcut(door)
                  .then(onKeys)
                  .catch(() => {});
              }),
              () => onChosen(wanted),
            );
          }}
          className={`rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
        >
          {offered.map((one) => (
            <option key={one} value={one}>
              {one}
            </option>
          ))}
          <option value="off">{t("quickKeysOff")}</option>
          {!keys && !chosenOff && <option value="" disabled hidden />}
        </select>
      )}
    </Line>
  );
}
